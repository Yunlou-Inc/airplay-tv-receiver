# 开发机环境搭建与运行手册

> 面向：在你自己的开发机上，把 `airplay-tv-receiver` 从 clone 跑到 App 启动。
> 适用：macOS / Linux（Windows 用 WSL2 后按 Linux 段执行）。
> 前置：一台能联网的机器、管理员权限（装 SDK）。本手册的命令均可直接复制执行。

> **只需验收、不想/不能搭环境？**（例如 macOS 10.15 无法升级、装不了新版 Android Studio）
> 走云端出包方案：验收方零环境要求，见 [CI_APK_DELIVERY.md](CI_APK_DELIVERY.md)。
> 本手册适用于要在本地从零搭建开发环境的场景。

## 0. 先建立正确预期（重要）

这个项目目前是**安全骨架**，不是能投屏的接收端。按本手册跑通后：

- ✅ 能构建 Debug APK、装到设备/模拟器、看到主界面；
- ✅ 发现开关默认关闭；打开后（授权 `NEARBY_WIFI_DEVICES`）发布 `_airplay._tcp.` 与 `_raop._tcp.`，5 分钟倒计时自动停；
- ❌ **不能**真的接收 AirPlay 投屏——没有真实监听/配对/媒体管线；
- ❌ Rust 协议引擎的 JNI 库 `libairplay_jni.so` 尚未接入 Gradle 构建，因此 `AirPlayProtocol.isAvailable == false`，协议代码优雅降级、不生效。这是**当前阶段的预期行为，不是构建错误**。

要让协议引擎真正参与运行，需先完成 ADR-001 / M1-2 的接线（接 cargo-ndk、补 `ffi.rs` 的 listen/accept），那是后续实现工作。本手册解决的是"把现有骨架跑起来"。

---

## 1. 克隆与定位

```bash
git clone https://github.com/Yunlou-Inc/airplay-tv-receiver.git
cd airplay-tv-receiver
REPO=$PWD
echo $REPO
```

## 2. JDK 17

```bash
# macOS（Homebrew 装过 openjdk@17 的情况）
brew install openjdk@17 2>/dev/null || true
export JAVA_HOME=$(/usr/libexec/java_home -v 17)

# Linux（Debian/Ubuntu）
# sudo apt install -y openjdk-17-jdk
# export JAVA_HOME=/usr/lib/jvm/java-17-openjdk-amd64

java -version     # 必须显示 17.x
echo $JAVA_HOME   # 必须非空
```

把 `export JAVA_HOME=...` 写进 `~/.zshrc` / `~/.bashrc` 以免每次重设。

## 3. Android SDK（platform 35 + build-tools）

**方式 A（推荐，图形界面）**：装 Android Studio → SDK Manager → 勾选
- `Platforms → Android 15 (API 35)`
- `SDK Tools → Android SDK Build-Tools 35.0.0`
- `SDK Tools → Android SDK Platform-Tools`（含 adb）

记下 SDK 路径，通常是 `~/Library/Android/sdk`（macOS）或 `~/Android/Sdk`（Linux）。

**方式 B（命令行）**：先装 cmdline-tools，然后

```bash
export ANDROID_HOME=$HOME/Library/Android/sdk   # Linux 改成 $HOME/Android/Sdk
export PATH=$ANDROID_HOME/cmdline-tools/latest/bin:$ANDROID_HOME/platform-tools:$PATH
sdkmanager --licenses                       # 全部 y 接受
sdkmanager "platforms;android-35" "build-tools;35.0.0" "platform-tools"
adb version                                  # 验证 platform-tools
```

**写 `local.properties`**（在仓库根目录，此文件不进版本库）：

```bash
cat > local.properties <<EOF
sdk.dir=$ANDROID_HOME
EOF
# Windows 路径需写成 sdk.dir=C\:\\Users\\you\\AppData\\Local\\Android\\Sdk
```

## 4. 恢复 `gradle-wrapper.jar`（当前唯一缺失的构建文件）

仓库里有 `gradlew` 和 `gradle-wrapper.properties`，只缺 `gradle-wrapper.jar`。**不要从随便的网上地址拷**——用下面带 SHA 校验的官方方式生成。

`gradle-wrapper.properties` 里固定了 Gradle 8.9 的下载地址与 SHA-256：

```
distributionUrl=.../gradle-8.9-bin.zip
distributionSha256Sum=d5d70e0bd28f7d6c71e4a11227081679e29e165b59dfe802cc6d5db6f4f93e16
```

```bash
# 4.1 下载并校验官方发行包
cd /tmp
curl -fL -o gradle-8.9-bin.zip https://services.gradle.org/distributions/gradle-8.9-bin.zip
shasum -a 256 gradle-8.9-bin.zip      # Linux 用 sha256sum
# 必须输出：d5d70e0bd28f7d6c71e4a11227081679e29e165b59dfe802cc6d5db6f4f93e16
# 不一致 → 停止，网络/源被污染，不要继续。

# 4.2 解压并用它生成 wrapper（会产出可信的 gradle-wrapper.jar）
unzip -q gradle-8.9-bin.zip -d /tmp/gradle-dist
cd $REPO
/tmp/gradle-dist/gradle-8.9/bin/gradle wrapper --gradle-version 8.9 --distribution-type bin

# 4.3 确认 jar 已生成，且 SHA 固定值仍在 properties 中（生成可能覆盖）
ls -l gradle/wrapper/gradle-wrapper.jar
grep distributionSha256Sum gradle/wrapper/gradle-wrapper.properties
```

若 4.3 的 `distributionSha256Sum` 丢失或变了，写回固定值：

```bash
cd $REPO
grep -q '^distributionSha256Sum=' gradle/wrapper/gradle-wrapper.properties || \
  echo 'distributionSha256Sum=d5d70e0bd28f7d6c71e4a11227081679e29e165b59dfe802cc6d5db6f4f93e16' >> gradle/wrapper/gradle-wrapper.properties
# macOS:
sed -i '' 's/^distributionSha256Sum=.*/distributionSha256Sum=d5d70e0bd28f7d6c71e4a11227081679e29e165b59dfe802cc6d5db6f4f93e16/' gradle/wrapper/gradle-wrapper.properties
# Linux 用: sed -i 's/^distributionSha256Sum=.*/distributionSha256Sum=d5d70e.../' ...
```

```bash
./gradlew --version    # 现在应正常输出 Gradle 8.9（会自动下载/校验发行包）
```

## 5. Rust 工具链与 Cargo.lock

仓库根有 `rust-toolchain.toml`，固定 `1.85.1`（含 rustfmt/clippy）。rustup 会自动安装它：

```bash
cd $REPO/protocol-rust
cargo --version        # 首次会下载 1.85.1，耐心等
cargo generate-lockfile
cargo test             # 跑 Rust 单测（无需网络、无外部依赖）
cargo fmt --check && cargo clippy --all-targets -- -D warnings
```

`cargo generate-lockfile` 生成 `Cargo.lock`（本 crate 无外部依赖，内容很小，可入库）。

## 6. 依赖校验元数据（首次联网，人工审核）

```bash
cd $REPO
./gradlew --no-daemon --write-verification-metadata sha256 help
# 产出 gradle/verification-metadata.xml —— 必须人工扫一遍来源是否可信后再入库。
```

## 7. 构建四件套（按顺序，全绿才算构建通过）

```bash
cd $REPO
./gradlew --no-daemon spotlessCheck
# 若仅报格式差异：./gradlew --no-daemon spotlessApply 后重跑 spotlessCheck
./gradlew --no-daemon :app:lintDebug
./gradlew --no-daemon :app:testDebugUnitTest
./gradlew --no-daemon :app:assembleDebug

# 产物校验
sha256sum app/build/outputs/apk/debug/app-debug.apk   # macOS 用 shasum -a 256
```

> 注：新增的 `app/.../ffi/AirPlayProtocol.kt` 若被 ktlint 判格式不符，先跑 `spotlessApply`（会规范化该文件），这是正常的。

## 8. 安装与运行

```bash
adb devices                       # 设备/模拟器需已连接并授权调试
adb install -r app/build/outputs/apk/debug/app-debug.apk
adb logcat | grep -i airplaytv    # 观察日志
```

或在 Android Studio 里直接 Run `app`。

## 9. 运行验收清单

- [ ] 冷启动看到主界面，状态 `WAITING`。
- [ ] 发现开关**默认关闭**；进程重启/ViewModel 重建后仍是关闭（不会自动后台开放）。
- [ ] 打开开关：Android 13+ 弹 `NEARBY_WIFI_DEVICES` 权限；拒绝 → 开关回退关闭并显示拒绝态、不发布服务。
- [ ] 授权后：用同网段另一台设备 `dns-sd -B _airplay._tcp.`（macOS）或 DNS-SD 浏览器 App，能看到 `_airplay._tcp.` 与 `_raop._tcp.` 两条记录。
- [ ] 5 分钟倒计时归零自动停止；手动关闭立即撤销注册；重复开关幂等。
- [ ] 断网 → 状态变 `NetworkUnavailable`，记录撤销；恢复 → 重新发布。
- [ ] 确认**没有**真正的 AirPlay 端口在监听（符合当前阶段）——这是预期，不是缺陷。

## 10. 常见坑

| 现象 | 原因 | 解决 |
|---|---|---|
| `gradle-wrapper.jar is missing` | 没做第 4 步 | 按第 4 步带 SHA 校验生成 |
| `SDK location not found` | 缺 `local.properties` | 按第 3 步写入 `sdk.dir` |
| `JAVA_HOME is not set` | 没设 JDK 17 | 按第 2 步 `export JAVA_HOME` |
| `Could not resolve com.android.tools.build:gradle:8.7.3` | 网络/代理 | 确认 `google()` / `mavenCentral()` 可达；`settings.gradle.kts` 用 `FAIL_ON_PROJECT_REPOS`，不要在项目里另加仓库 |
| `spotlessCheck` 失败 | 新文件格式 | `./gradlew spotlessApply` 后重跑 |
| Rust 组件下载卡住 | rustup 源慢 | 配 `RUSTUP_DIST_SERVER` / `RUSTUP_UPDATE_ROOT` 镜像 |
| App 装上了但"没功能" | 见第 0 节 | 当前阶段预期；协议引擎未接线 |

## 11. 跑通之后

把第 4–6 步产生的 `gradle-wrapper.jar`、`Cargo.lock`、`gradle/verification-metadata.xml` 纳入版本控制并提交（提交前确认来源已审核）。需要我帮你把这些产物回填/提交/推送时，明确说一声即可——推送需你单独批准。
