# M1–2 Android 集成设计：授权门禁 + 显式绑定的 RTSP 监听

> 状态：设计稿（沙盒起草，未经 cargo-ndk / Android 构建或真机验证）。
> 前置：见 `docs/ADR-001-ffi-integration.md`（FFI/JNI 集成决策，接线必须先评审）。
> 目标里程碑：安全网络骨架 & 发现（M1–2）。

## 1. 目标

把 Rust 协议引擎在 Android TV 上**上线到网络**，但必须满足两项硬约束（来自 `THREAT_MODEL.md` 与 `ARCHITECTURE.md`）：

1. **显式用户授权前不得监听任何端口**（consent gate，默认关闭）。
2. **只绑定单一、显式的本地接口**（初始 127.0.0.1 做本地验证，之后是用户在设置里选定的 LAN 接口 IP），**绝不** `0.0.0.0` / 通配。

复用 `frontend.rs` 里已有的防御性限制（连接上限、读写超时、503 限流、逐连接 Router）。**不做**私有密码学、不做媒体——只跑"有界 RTSP 解析 + 会话状态机 + 路由"，任何需要密钥的操作在路由层返回 `Unsupported`。

## 2. 当前真实状态（基于已读源码）

- `app/.../service/ReceiverService.kt`：当前是**私有服务边界**，只通过 `NsdDiscoveryController` 做 DNS-SD 发现；没有持有任何 Rust 上下文句柄。
- 授权门禁**已存在**于发现侧：`NsdDiscoveryController.start(config)` 仅当 `config.enabled == true` 才发布；`ReceiverUiState` 里已有 `discoveryEnabled: Boolean = false`。监听侧需镜像这一模式。
- Rust `protocol-rust/.../ffi.rs` 暴露 `airplay_context_create/destroy/reset/state/handle` —— **没有 listen/accept 接口**。
- Rust `protocol-rust/.../frontend.rs` 已实现完整、有界的阻塞式 TCP RTSP 服务端（`bind/run/serve_stream`、连接计数、读写超时、503 限流、逐连接 Router），但**当前不可从 C-ABI 触达**。→ **这是 M1–2 必须在 Rust 侧关闭的缺口**。
- `app/.../ffi/AirPlayProtocol.kt`：包了上述 5 个 FFI 调用；需补 `listen` / `stopListen` 两个门面。

## 3. 组件职责

1. **`ProtocolEngine`（新增，由 `ReceiverService` 持有）**
   - 生命周期：`ReceiverService.onCreate()` 时 `AirPlayProtocol.createContext()`（失败则 `null`，不崩溃）；`onDestroy()` 时 `destroyContext()`。
   - 对外：`startListening(config)` / `stopListening()`。
   - 持有 Rust 上下文 `Long` 句柄（**只存 Long，绝不裸指针**——遵循 ADR-001）。

2. **授权门禁（consent gate）**
   - `ReceiverUiState` 新增 `listeningEnabled: Boolean = false`（镜像 `discoveryEnabled`）。
   - `ProtocolEngine.startListening` 在 `listeningEnabled != true` 时**拒绝起监听**，直接返回，不打开任何 socket。
   - UI 开关必须是**显式**的（默认关闭，用户主动打开），不在任何路径上默认开启。

3. **绑定策略（bind policy）**
   - 监听地址来自显式选择：先在设置里用 127.0.0.1 做本地验证，再开放"选定 LAN 接口 IP"。
   - 以字节形式传给 Rust，**禁止 0.0.0.0 / 通配**；Rust 侧若收到通配/非法地址应拒绝（-1）。

4. **Rust 侧需新增的 FFI 表面**（在 `ffi.rs` 补齐，复用 `frontend.rs`）
   - `airplay_context_listen(ctx: *mut c_void, addr_ptr: *const u8, addr_len: usize, port: u16) -> i32`
   - `airplay_context_stop_listen(ctx: *mut c_void) -> i32`
   - 两者在 Rust 内部**启停一个独立 accept 线程**，该线程跑现有的 `frontend` accept 循环；逐连接处理器复用与 `airplay_context_handle` 相同的解析/状态逻辑。
   - 实时路径上必须强制 `frontend.rs` 的限制：连接上限、读写超时、503 限流。

5. **线程模型**
   - accept 循环运行在**专属 Rust 线程**（非 ADR-001 的 FFI 单线程）。
   - FFI 控制调用（listen/stop/state）仍走 ADR-001 的专用单线程 + 全局 `Mutex`。
   - `stop` 必须**协作式**（原子 stop 标志）以避免拆除时阻塞；accept 循环周期性检查该标志后优雅退出。

## 4. 明确不在 M1–2 范围

- 配对 / Apple 私有密码学 / 密钥交换 → 路由层返回 `UnsupportedAirPlayProtocol`。
- 音视频解码/渲染、遥控、持久化。
- 任何"Apple 认证"或"完整互操作"的宣称。

## 5. 完成 M1–2 所需的验证证据

- [ ] `android-rust/airplay-jni` 经 cargo-ndk 构建出 `armeabi-v7a / arm64-v8a / x86_64` 三 ABI 的 `libairplay_jni.so`，落入 `app/src/main/jniLibs`。
- [ ] `ReceiverService` 启动、`createContext()` 成功、`AirPlayProtocol.isAvailable == true`（真机）。
- [ ] 授权 OFF → **不开任何 socket**（设备上 `ss`/`netstat` 验证零绑定端口）。
- [ ] 授权 ON + 绑定 127.0.0.1 → 端口**仅**在回环上；外部扫描显示关闭。
- [ ] 模糊/溢出输入（超大 RTSP、重复头、尾随字节）→ 连接关闭、进程不崩。
- [ ] ADR-001 §验证清单 对应项关闭。

## 6. 需在可信环境执行（沙盒无法构建）

```bash
# 构建 JNI 库（需 cargo + cargo-ndk + Android NDK，沙盒被网络/工具链拦）
cd android-rust/airplay-jni
cargo ndk -t armeabi-v7a -t arm64-v8a -t x86_64 \
  -o ../app/src/main/jniLibs build --release

# app/build.gradle.kts 接入：加载 airplay_jni、引入 jni 依赖、把产物当 jniLibs
# 生成根 Cargo.lock 与 jni crate 的 Cargo.lock（需 cargo）
cd protocol-rust && cargo generate-lockfile
cd ../android-rust/airplay-jni && cargo generate-lockfile
```

## 7. 一致性提醒（接线上线前必须处理）

- `ffi.rs` 的 `profile()` 当前 `device_id = [0;6]` 占位符；**监听/发布前必须换成真实、非零、每设备唯一标识**（从 Android 侧注入），否则会公告冲突/被忽略。
- 当前 `features = 0`（保守默认，遵循 ADR-001）；未实现的能力不得 advertised。
