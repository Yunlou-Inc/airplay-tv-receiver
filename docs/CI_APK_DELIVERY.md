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

- 一个浏览器（打开 Actions 下载 APK）
- 一台 Android 7.0+ 电视 / 盒子（arm64-v8a，覆盖绝大多数设备）
- 一个 iPhone / iPad / Mac，与目标设备同一局域网
- 安装方式二选一：`adb`（数据线或网络 ADB），或真机上"从文件安装"

## 1. 出包（由有仓库写权限的人操作）

1. 打开本仓库的 **Actions** 页。
2. 左侧工作流列表点 **Build debug APK (acceptance)**。
3. 右侧 **Run workflow** → 选分支 `master`（默认分支）→ **Run workflow**。
4. 等待运行完成（绿色对勾）。**每次 push 到 master 也会自动出一次包。**

> 构建需从源码编译 OpenSSL 与 FFmpeg，默认只编译 `arm64-v8a` 一个 ABI 以控制
> 时长与磁盘占用。**已实测：云端首次完整构建约 4 分钟**（run 43723492706，
> 产出 33 MB 可安装 debug APK）。若目标设备不是 arm64，见第 6 节调整 ABI。

## 2. 下载与校验

1. 点进刚跑完的那次 run，页面底部 **Artifacts** 区下载
   `yunlou-airplaytv-debug-run-<序号>`（zip）。
2. 解压得到：
   - `app-debug.apk`
   - `app-debug.apk.sha256`

校验（可选）：

```bash
# macOS
shasum -a 256 app-debug.apk
# Linux
sha256sum app-debug.apk
```

## 3. 安装到电视 / 盒子

```bash
# 数据线或网络 ADB 连接后
adb connect <设备IP>:5555
adb install -r app-debug.apk
```

或把 APK 拷到 U 盘 / 用文件管理器，在设备上选择"安装未知来源应用"。

> 这是 **debug 签名** 的 APK，可正常安装使用，但不用于上架应用商店。
> 卸载命令：`adb uninstall cn.net.yunlou.airplaytv`

## 4. 投屏验收（L3：真出画面）

1. 打开 App「云楼投屏」，授予"附近的设备 / 本地网络"权限。
2. 点 **Start** 启动接收服务（若开启"自动启动"则跳过）。
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

## 6. 调整 ABI（非 arm64 设备）

工作流里构建命令含 `-Pabis=arm64-v8a`。如需多 ABI，编辑
`.github/workflows/build-debug-apk.yml` 里 Assemble 那一步：

```bash
./gradlew --no-daemon :app:assembleDebug -Pabis=arm64-v8a,armeabi-v7a,x86_64 --stacktrace
```

或直接去掉 `-Pabis=...`（恢复上游默认：全部 3 个 ABI）。ABI 越多，云端编译越慢。

## 7. 已知限制与注意

- **未经真机验证**：本工作流尚未在云端实跑过。首次运行如遇 SDK/依赖问题，
  请看该次 run 的日志定位（多数是 `sdkmanager` 版本或子模块拉取问题），
  按报错微调 `.github/workflows/build-debug-apk.yml`。
- **API 37 预览态**：`compileSdk = 37` 较新；工作流已用 `sdkmanager --channel=3`
  允许解析预览版 SDK。若 Google 尚未发布对应平台，需要把 `compileSdk/targetSdk`
  临时降到已发布版本后重跑。
- **submodule 必须 recursive**：`applyUxplayPatches` 任务会对 UxPlay 子模块执行
  `git apply`，因此 checkout 用了 `submodules: recursive` + `fetch-depth: 0`，请勿删减。
- **不支持 DRM 内容**（如 Apple TV App 的受保护影片）；这是上游共同的协议限制。
- 上游 `apk.yml`（签名 release / 发 GitHub Release）仍然保留，但它需要
  `STORE`、`LOCAL` 两个 secrets 且只在 `release/**` 分支或 `v*.*.*` 标签触发；
  日常验收请用本工作流，不要依赖它。

## 8. 合规提醒

本 App 是 GPL-3.0 衍生作品。把 APK 分发给第三方（含发给验收方、上传应用商店）时，
须同时提供完整对应源代码并保留 `LICENSE` 与 `NOTICE.md`。详见根目录 `NOTICE.md`。
