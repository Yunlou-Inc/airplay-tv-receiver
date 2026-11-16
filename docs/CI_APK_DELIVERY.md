# 云端出包与验收（零环境）

> 适用：验收方机器无法升级系统（如 macOS 10.15）、装不了新版 Android Studio，
> 或不想装任何 IDE。本方案让 **GitHub Actions 在云端完成全部构建**，
> 验收方只需一个浏览器 + 一台 Android TV/盒子 + 一个 iOS/macOS 发送端。

## 0. 为什么本机可以零环境

构建这个 App 真正需要的是 JDK 21 + Android SDK 37 + NDK 27 + CMake + Gradle 9.8.1，
以及 4 个 native 子模块（UxPlay / libplist / openssl-cmake / FFmpeg，从源码编译 OpenSSL 与 FFmpeg）。
这些**全部在 GitHub 云端 runner 上完成**。

验收方机器上**不需要**：

- Android Studio / IntelliJ IDEA
- JDK、Android SDK、NDK、CMake、Gradle
- 升级 macOS

验收方机器上**只需要**：

- 一个浏览器（打开 Releases / Actions 下载 APK）
- 一台 Android 6.0+ 电视 / 盒子（按 CPU 架构选对应包，见下面的表格）
- 一个 iPhone / iPad / Mac，与目标设备同一局域网
- 安装方式二选一：`adb`（数据线或网络 ADB），或真机上"从文件安装"

## 1. 出包（由有仓库写权限的人操作）

仓库有两个出包工作流，**都不需要额外配置就能跑**：

| 工作流 | 产物 | 用途 |
|---|---|---|
| **Build debug APK (acceptance)** | debug 签名 APK | 日常验收；每次 push 到 `master` 自动出包 |
| **Release APK** | 4 个架构包（arm64-v8a / armeabi-v7a / x86_64 / universal）+ `.sha256` | 正式交付；`v*` 标签自动发布到 GitHub Releases |

可选：配置 4 个 `SIGNING_*` 仓库 secret 后，release 改用固定发布密钥签名，不同版本可以
覆盖升级；未配置时回退到 runner 上临时生成的密钥。见 [`SIGNING.md`](SIGNING.md)。

手动运行：打开本仓库 **Actions** → 左侧选工作流 → 右侧 **Run workflow** → 选分支 `master` → **Run workflow**。

发布正式包：推送 `v*` 标签（如 `v0.5.0-yunlou`），**Release APK** 会并行构建四个包，
再合成一个 GitHub Release。

> 构建需从源码编译 OpenSSL 与 FFmpeg。四个包在矩阵作业里**并行**编译，单包约 4～6 分钟；
> `universal` 一个包要编三种 ABI，耗时最长（约 10 分钟）。

## 2. 下载与校验

正式包：仓库 **Releases** 页面按架构下载 `YunlouAirPlayTV-<版本>-<架构>.apk` 及 `.sha256`
（只需装一个，四个包装的是同一个 App）。
日常包：点进 Actions 里刚跑完的 run，页面底部 **Artifacts** 下载 zip 并解压得到
`app-debug.apk` / `app-debug.apk.sha256`。

校验（可选）：

```bash
# macOS
shasum -a 256 YunlouAirPlayTV-0.5.0-yunlou-arm64-v8a.apk
# Linux
sha256sum YunlouAirPlayTV-0.5.0-yunlou-arm64-v8a.apk
```

## 3. 安装到电视 / 盒子

```bash
# 数据线或网络 ADB 连接后
adb connect <设备IP>:5555
adb install -r YunlouAirPlayTV-0.3.0-yunlou-release.apk
```

或把 APK 拷到 U 盘 / 用文件管理器，在设备上选择"安装未知来源应用"。

> debug 包与 release 包都可正常安装使用，均不用于上架应用商店。
> **签名注意**：构建日志与 Release 说明里都有 `SIGNING_MODE`。
> `stable` = 固定发布密钥，可直接覆盖升级；`ephemeral` = 一次性密钥，
> 不同批次 APK **不能覆盖安装**（需先 `adb uninstall`）。详见 [`SIGNING.md`](SIGNING.md)。
> 卸载命令：`adb uninstall cn.net.yunlou.airplaytv`

## 4. 投屏验收（L3：真出画面）

1. 打开 App「云楼投屏」，授予"附近的设备 / 本地网络"权限。
2. 点 **启动 / Start** 开启接收服务（若开启"自动启动"则跳过）。
3. 让 iPhone / iPad / Mac 连接到**同一个局域网**（同一 WiFi、同一网段）。
4. iOS：控制中心 → **屏幕镜像** → 选择显示的设备名（默认 server name 可在设置里改）。
5. 首次连接若开启了 PIN，输入电视上显示的 4 位 PIN。
6. **验收通过标准**：电视上实时显示 iPhone 画面，并有声音；
   可旋转、可播放视频、可用遥控器/方向键操作。

## 5. 无屏幕设备：用 ADB 改配置

```bash
export KEY="server_name" VALUE="LivingRoomTV"
am broadcast -n cn.net.yunlou.airplaytv/.SettingsReceiver \
  -a io.github.jqssun.airplay.SET_SETTING --es key "$KEY" --es value "$VALUE"

# 列出全部设置项（JSON）
am start -n cn.net.yunlou.airplaytv/.MainActivity
am broadcast -n cn.net.yunlou.airplaytv/.SettingsReceiver -a io.github.jqssun.airplay.LIST_SETTINGS
```

## 6. 架构与包的选择

`Release APK` 工作流固定产出四个包（矩阵作业，`matrix.abis` 决定每个包编哪些 ABI）：

| 包 | ABI | 适用 |
|---|---|---|
| `-arm64-v8a.apk` | `arm64-v8a` | 2019 年后的电视 / 盒子，**默认发这个** |
| `-armeabi-v7a.apk` | `armeabi-v7a` | 老款 32 位电视（Android 6/7/8 时代芯片） |
| `-x86_64.apk` | `x86_64` | 安卓模拟器 / Intel 芯片盒子 |
| `-universal.apk` | 三者全含 | 不确定架构时兜底，体积最大 |

本地单独出某个包：

```bash
./gradlew :app:assembleRelease -Pabis=arm64-v8a -Pminify=false
./gradlew :app:assembleRelease -Pabis=arm64-v8a,armeabi-v7a,x86_64 -Pminify=false   # 通用包
```

ABI 越多，云端编译越慢、包越大。

## 7. 已知限制与注意

- **云端构建已实测通过**：debug run `43723492706`、`43724780228`、`43724782111`，
  `BUILD SUCCESSFUL in 4m 11s`，产出 33 MB 可安装 APK。若后续遇到 SDK/依赖问题，
  请看该次 run 的日志定位（多数是 `sdkmanager` 版本或子模块拉取问题）。
- **未经真机投屏验证**：云端只保证"能编译、能出包、签名有效、能安装"，
  实际投屏画面/声音仍需在目标电视上验收。
- **API 37 预览态**：`compileSdk = 37` 较新；工作流已用 `sdkmanager --channel=3`
  允许解析预览版 SDK。若 Google 尚未发布对应平台，需要把 `compileSdk/targetSdk`
  临时降到已发布版本后重跑。
- **submodule 必须 recursive**：`applyUxplayPatches` 任务会对 UxPlay 子模块执行
  `git apply`，因此 checkout 用了 `submodules: recursive` + `fetch-depth: 0`，请勿删减。
- **release 签名有两种模式**：配置 `SIGNING_*` 仓库 secret 时使用固定发布密钥
  （`SIGNING_MODE=stable`，可覆盖升级）；未配置时在 runner 上生成一次性 keystore
  （`SIGNING_MODE=ephemeral`，必须先卸载）。两条路径都不把私钥写进仓库。
  密钥如何生成、备份与轮换见 [`SIGNING.md`](SIGNING.md)。
- **release 默认不混淆**：`-Pminify=false`。native 层通过 JNI/反射访问 Java 类，
  keep 规则未经真机验证前不启用 R8，避免投屏功能在运行时被裁掉。
- **不支持 DRM 内容**（如 Apple TV App 的受保护影片）；这是上游共同的协议限制。

## 8. 合规提醒

本 App 是 GPL-3.0 衍生作品。把 APK 分发给第三方（含发给验收方、上传应用商店）时，
须同时提供完整对应源代码并保留 `LICENSE` 与 `NOTICE.md`。详见根目录 `NOTICE.md`。
