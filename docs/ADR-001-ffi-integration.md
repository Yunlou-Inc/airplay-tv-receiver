# ADR-001: Android(Kotlin) ↔ Rust 协议层 FFI/JNI 集成

- **状态**：Proposed（已在沙盒中起草，未经构建/真机验证）
- **日期**：2026-11-15
- **决策人**：待评审（Rust/系统工程师 + Android(TV) 工程师）
- **关联**：风险矩阵「集成时 ABI/所有权/线程模型出错」要求本 ADR 先于接线；Milestone 1–2（安全网络骨架 & 发现）

---

## 1. 背景（Context）

`airplay-protocol` crate 已提供一个**干净、契约明确的 C-ABI**（`protocol-rust/crates/airplay-protocol/src/ffi.rs`）：

| 符号 | 语义 |
|---|---|
| `airplay_context_create() -> *mut c_void` | 创建上下文；失败返回 null |
| `airplay_context_destroy(handle)` | 销毁；重复销毁安全 |
| `airplay_context_reset(handle) -> i32` | 复位路由器；0 成功，负值错误 |
| `airplay_context_state(handle) -> i32` | 会话状态：0 Idle / 1 Announced / 2 Configured / 3 Streaming / 4 Paused / 5 Closed；-1 句柄缺失 |
| `airplay_context_handle(handle, input, in_len, output, out_cap) -> isize` | 处理一帧 RTSP；正数=写入字节数；若返回值 > out_cap 表示需更大缓冲（调用方扩容重试）；负值为错误码 |
| `airplay_listen_start(handle, addr, addr_len, authorized) -> i32` | 启动有界 TCP 监听；`authorized` 必须为 1（用户已显式授权）；正数=绑定端口；负值为错误码 |
| `airplay_listen_stop(handle) -> i32` | 停止监听；0 成功，-1 未在监听/无效句柄 |
| `airplay_listen_state(handle) -> i32` | 0 未监听，>0 绑定端口，-1 无效句柄 |
| `airplay_listen_bound_addr(handle, out, cap) -> isize` | 回读实际绑定地址串；正数=字节数，若 > cap 需扩容重试 |

`handle()` 错误码：`−1` 参数非法/空指针/输入溢出，`−2` 解析失败，`−3` 锁/状态错误，`−4` 路由错误，`−127` panic 被捕获（**绝不跨 FFI 边界 unwind**）。`listen_*` 错误码：`−1` 参数非法，`−2` 锁中毒，`−3` 非法/通配地址，`−8` 未授权，`−9` 已在监听，`−10` 绑定/IO 失败，`−127` panic。输入上限 `MAX_FFI_INPUT = 64 KiB`，绑定地址上限 128 字节。

**现状缺口**：Kotlin 侧（`app/`）完全没有调用这个 FFI，也没有 JNI/NDK 构建链路；`frontend.rs` 的纯 Rust TCP 服务端也没有在任意 Android 组件里启动。这是当前最大的真实集成缺口，也是 Milestone 1–2 的前置。

本 ADR 在「不依赖法律资料、不依赖联网」的前提下，定下接线机制、所有权、线程、错误模型、构建接线与 ABI 稳定性，作为实际接线（修改 `build.gradle.kts`、取 `jni` 依赖、cargo-ndk 构建）之前的评审基线。

---

## 2. 决策（Decision）

### 2.1 机制：独立 JNI 桥接 crate 包裹既有 C-ABI
- 新建 **`android-rust/airplay-jni`**（位于 `protocol-rust/` 工作区**之外**，避免把 `jni` 外部依赖带进核心 crate、保持其离线可构建）。
- 该 crate 以 `cdylib` 产出 `libairplay_jni.so`，`#[no_mangle] pub extern "system" fn Java_com_example_airplaytv_ffi_AirPlayProtocol_*` 函数**薄封装**既有 `airplay_context_*`（仅做 JNI 类型 ↔ 字节缓冲转换 + panic 二次捕获）。
- 核心 crate `airplay-protocol` 保持**零外部依赖、离线可构建**；C-ABI 符号名与签名是二者之间唯一契约。

### 2.2 构建接线（在可信环境执行，本 ADR 不修改构建文件）
- 用 **`cargo-ndk`** 为 `arm64-v8a`、`armeabi-v7a`、`x86_64` 产出 `libairplay_jni.so`，放入 `app/src/main/jniLibs/<abi>/`。
- Kotlin 侧 `System.loadLibrary("airplay_jni")`。
- 选 `cargo-ndk` + 手动放置（或 `rust-android-gradle` 插件）二选一；倾向 `cargo-ndk` 以最小侵入 Gradle。

### 2.3 所有权
- Rust 拥有全部协议状态，句柄为不透明 id（C 侧是 `*mut c_void`，Kotlin 侧用 `Long` 持有，**绝不**暴露裸指针）。
- 句柄生命周期绑定到 Android `ReceiverService` 生命周期：Service `onCreate` 调 `createContext()`，`onDestroy` 调 `destroyContext()`。

### 2.4 线程
- FFI 内部用全局 `Mutex<HashMap<usize, Context>>` 串行化访问；同一时刻多调用会串行而非并发。
- Kotlin 侧在**专用单线程**（`HandlerThread`/Looper）上同步调用 native 方法，避免 JNI `JNIEnv` 跨线程失效与重入。原生方法本身不持有 `JNIEnv`/`jobject` 越过 Rust 调用。

### 2.5 错误模型
- Kotlin 侧把 C-ABI 返回码映射为 `ProtocolResult` 密封类：`Ok(value)` / `Err(code, msg)` / `Unavailable`（native 库未加载）。
- `handle()` 返回值 > `outputCap` 时，Kotlin 自动扩容缓冲并重试（最多 4 次）。
- `create` 返回 `0` ⇒ `null`（失败）。

### 2.6 输入边界
- Kotlin 传入的 `request` 超过 `64 KiB` 直接在 Kotlin 侧拒绝（`MAX_FFI_INPUT`），不进 native。
- native 侧再次强制 `MAX_FFI_INPUT`，双保险。

### 2.7 安全约束（继承 SECURE_CODING / THREAT_MODEL）
- **绝不**让 Rust panic 跨 FFI 边界（native 方法外层 `catch_unwind`，返回 −127）；JNI 桥接层再包一层 `catch_unwind` 兜底。
- **绝不**在日志中输出配对秘密、设备密钥或请求/响应明文里的敏感字段；日志仅脱敏计数与状态迁移。
- 设备画像维持保守默认（`features=0`，见已提交的 `ffi.rs` 修正）；**在真实、非零、每设备 `device_id` 就位前，不对外广播任何能力**。

### 2.8 监听接口与授权门禁（2026-11-15 增补）
- `frontend.rs` 已有完整有界 TCP 服务端（连接上限/超时/503 限流/逐连接 Router），本轮通过 `airplay_listen_*` 暴露到 C-ABI，供 M1-2 使用。
- **默认静默**：不调用 `listen_start` 就没有任何 socket。且 `listen_start` 双重门禁：
  ① `authorized != 1` 直接返回 `−8`（Kotlin 侧只有拿到用户显式同意才传 true）；
  ② 核心 `validate_bind_addr` 只接受显式 IP 字面量（回环或具体网卡），拒绝 `0.0.0.0`/`[::]` 通配与非数字主机名，防止无意中对全网暴露。
- 端口 0 允许（OS 分配临时端口），但实际绑定地址通过 `local_addr()` 回读，`bound_addr`/`state` 返回真实端口。
- 停止/销毁时置 `stop` 原子标志唤醒 accept 循环退出，`context_destroy` 亦会停止监听，避免端口泄漏。
- `listen_*` 与 `handle()` 的数值错误码有语义重叠，Kotlin 门面用**独立的** `mapListenCode` 映射，保证错误消息真实。

### 2.9 ABI 稳定性
- C-ABI 符号名与签名是唯一契约；JNI crate 仅是薄适配。
- 任何 C-ABI 变更必须同步本 ADR 并递增版本；Kotlin 门面与 Rust 桥接必须成对更新。

---

## 3. 后果（Consequences）

- **正面**：补上最大集成缺口；Milestone 1–2（把 `frontend.rs` TCP 服务端经 Android 启动、显式本地接口绑定、用户授权后才监听）得以推进；核心 crate 保持离线纯净。
- **负面/成本**：`airplay-jni` 引入 `jni` 外部依赖（可信环境需联网取）；集成在未于可信环境构建+真机验证前**不成立**。
- **纪律**：本 ADR 是设计基线；实际接线（改 `build.gradle.kts`、取依赖、cargo-ndk）须在可信环境完成，并回填下述证据后才标记「已接线」。

---

## 4. 验证清单（接线完成、标记「已接线」前必须具备的证据）

- [ ] `cargo-ndk` 对 3 个 ABI 构建全绿，产出 `libairplay_jni.so`
- [ ] `app` 模块 `assembleDebug` 含 `jniLibs` 且 `loadLibrary` 在模拟器/真机成功
- [ ] Kotlin 门面单测（mock/不依赖真机）通过：create→state=Idle、非法 handle→Err
- [ ] 端到端：真机发送 `ANNOUNCE` → `state()` 返回 `Announced`
- [ ] 负向测试：畸形/超长输入 → 返回错误码而非崩溃、无 panic 跨边界
- [ ] 监听门禁：`authorized=0` → `−8` 且无 socket 打开；`0.0.0.0`/`[::]` → `−3` 拒绝
- [ ] 回环监听：`listen_start("127.0.0.1:0", authorized=1)` → 返回临时端口，`state()`>0，`bound_addr` 回读一致
- [ ] 停止：`listen_stop` 后 `state()`=0；`context_destroy` 后端口释放（无泄漏）

---

## 5. 范围外（Out of Scope）

- Milestone 3 配对/认证密码学（**硬法律门禁**，本 ADR 不涉及）
- 音视频解码/渲染/同步（Milestone 4–6）
- 任何 Apple 私有算法猜测或逆向

---

## 6. 相关文件（本次起草，未构建验证）

- `docs/ADR-001-ffi-integration.md`（本文件）
- `app/src/main/java/com/example/airplaytv/ffi/AirPlayProtocol.kt`（Kotlin JNI 门面，additive，编译安全）
- `android-rust/airplay-jni/Cargo.toml` + `src/lib.rs`（Rust JNI 桥接骨架，位于工作区外，未编译）
