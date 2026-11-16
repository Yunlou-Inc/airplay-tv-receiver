# 云楼投屏（Yunlou AirPlay TV）

[![license](https://img.shields.io/badge/License-GPLv3-blue.svg)](LICENSE)

一个开源的 AirPlay 接收端，可把 Android 设备 / Android TV 变成可被 iPhone、iPad、Mac 投屏的显示与扬声器。
基于成熟上游项目 [`jqssun/android-airplay-server`](https://github.com/jqssun/android-airplay-server)
（其底层为 [`UxPlay`](https://github.com/FDH2/UxPlay)）二次开发，**本 fork 只做品牌化与工程化，不重复自研协议**。

> 本项目不隶属于 Apple Inc。AirPlay 是 Apple Inc. 的商标。

## 为什么用这个方案

自研 AirPlay 协议栈成本极高（私有协议、FairPlay 认证、H.264/H.265 实时解码），且卡在合法协议资料门禁。
本 fork 直接采用社区已验证的成熟实现，把精力放在"能用、能验收"上——装到电视上即可被 iOS/macOS 投屏。

## 项目沿革

本仓库早期有一套自研的 Rust 协议层 + Kotlin 骨架（MIT OR Apache-2.0），仅能发现设备、不能真正投屏。
该路线已**正式放弃**。`master` 自 2026-11-15 起即为本 GPL-3.0 衍生作品的主线；
自研历史作为 `master` 合并提交 `801a05f0` 的第一个父提交 `5fc97e18` 保留在仓库对象中，
**不属于当前分发内容**，两者许可不同，请勿混用。临时分支 `android` / `legacy-prototype` 已删除。
云端出包已实测通过（首次完整构建约 4～5 分钟，产出 armeabi-v7a 可安装 APK）。

## 能力

- 屏幕镜像：H.264 / H.265 (HEVC) 硬件解码
- 音频：AAC-ELD、AAC-LC、ALAC
- 视频：HLS 播放、下载、遥控器控制
- 音乐：曲目信息、封面、遥控
- Android TV：方向键导航、快进快退
- 画中画、自动分辨率/刷新率切换
- 可选 PIN 认证
- 调试浮层（FPS / 码率 / 编解码 / 分辨率等）
- Android 媒体会话集成（通知栏控制）

## 面向使用者的界面

- **中英双语**，跟随电视系统语言；一级导航只有「首页 / 设置」，日志收在设置里
- **首次启动引导**：三步投屏说明 + 可选的开机自启，之后可随时在「设置 → 关于」重看
- **启动失败会说人话**：区分「本地网络权限被拒 / 原生引擎加载失败 / 端口被占用」，
  每种都给出可操作的下一步（重试、看日志、换端口）
- **关于页**：版本号、签名指纹（可核对安装包来源）、下载页、源码、隐私与许可
- 遥控器：焦点环动画、错误卡片默认聚焦「重试」、日志页默认聚焦返回

> 不支持 DRM 内容（如 Apple TV App 的受保护内容）。

## 兼容性

- Android 6.0+（`minSdk 23`；含 Android TV / Google TV，上游主要在 7.0+ 上验证）
- 与发送端处于同一局域网的 iOS / iPadOS / macOS 设备

## 构建（需要 Android SDK/NDK）

本项目含 4 个 native 子模块（UxPlay / libplist / openssl-cmake / FFmpeg），构建前必须先初始化：

```bash
git clone --recursive <本仓库地址>
# 或 clone 后执行：git submodule update --init --recursive
cd airplay-tv-receiver-fork
./gradlew assembleDebug
```

产物：`app/build/outputs/apk/debug/app-debug.apk`

构建环境（与上游一致）：
- JDK 21
- Android SDK Platform 37 / Build Tools / NDK `27.0.12077973`
- CMake 3.22.1+
- Gradle 9.8.1（使用仓库自带 `./gradlew`，无需本机安装 Gradle）

## 零环境验收（推荐）

验收机器**不需要** Android Studio / JDK / SDK / Gradle，也不需要升级 macOS。
用 GitHub Actions 在云端出包，本机只下载 APK 安装即可。见 [`docs/CI_APK_DELIVERY.md`](docs/CI_APK_DELIVERY.md)。

## Release 包

正式交付包在仓库 **Releases** 页面：`YunlouAirPlayTV-<版本>-release.apk`（已签名，
可直接 `adb install -r`），并附 `.sha256` 校验文件。推送 `v*` 标签会自动构建并发布；
也可在 Actions 里手动运行 **Release APK** 工作流单独出包。

> 签名说明：`SIGNING_*` 仓库 secret 已配置，正式包使用**固定发布密钥**（证书 SHA-256
> `C7:25:9B:E7:…:83:CD`），可直接覆盖升级；历史 `v0.1.0`~`v0.3.0` 由一次性密钥签名，
> 升级到第一个固定签名包时需卸载一次。见 [`docs/SIGNING.md`](docs/SIGNING.md)。
> 云端只保证能编译、能签名、能安装，**真机投屏尚未验收**。

## 下载页

**在线地址：<https://yunlou-inc.github.io/airplay-tv-receiver/>**（GitHub Pages，由
`.github/workflows/pages.yml` 在 `site/` 变更时自动发布）

`site/index.html` 是给普通用户看的落地页（单文件、无外部依赖）：下载按钮、三步图示、
安装说明、SHA256 校验方法、常见问题、许可声明。品牌资源同目录：
`icon-512.png` / `icon-192.png` / `apple-touch-icon.png` / `og.png`（社交分享卡）。

App 内的品牌标识与页面同源：`drawable/ic_launcher_foreground.xml`（启动器图标）、
`drawable/ic_brand_mark.xml`（引导页）、`drawable/banner.xml`（TV 横幅），均为「云 + 负形播放键」。

## 通过 ADB 配置（无屏幕设备）

```bash
export KEY="server_name" VALUE="Speaker"
am broadcast -n cn.net.yunlou.airplaytv/.SettingsReceiver \
  -a io.github.jqssun.airplay.SET_SETTING --es key "$KEY" --es value "$VALUE"

am start -n cn.net.yunlou.airplaytv/.MainActivity
am broadcast -n cn.net.yunlou.airplaytv/.SettingsReceiver -a io.github.jqssun.airplay.LIST_SETTINGS
```

> 注：自定义 broadcast action 字符串保留上游命名空间（`io.github.jqssun.airplay.*`），
> 与安装身份 `applicationId` 相互独立，改动无收益，故保留。

## 许可与来源

本项目整体以 **GPL-3.0** 发布，是上游 `jqssun/android-airplay-server`（GPL-3.0）的衍生作品。
上游又基于 UxPlay（GPL-3.0-or-later）并集成 FFmpeg / libplist / OpenSSL / oboe。
完整来源、版权与修改清单见 [`NOTICE.md`](NOTICE.md)，许可全文见 [`LICENSE`](LICENSE)。

分发 APK/AAB 时须随附完整对应源代码并保留全部版权声明；仅自用不分发则不触发 GPL 分发义务。
本声明不构成法律意见。

## 安全

本项目做过一次静态源码安全审查，结论、逐项证据（文件 + 行号）、已落地加固、
剩余风险与审查边界均记录在 [`docs/SECURITY_REVIEW.md`](docs/SECURITY_REVIEW.md)。
摘要：未发现后门特征（无硬编码外联地址、无遥测 SDK、无动态代码加载、无硬编码凭据）；
依赖锁定到精确 commit；默认关闭开机自动监听。但静态审查不能证明"绝对安全"，
且底层 4 个开源库源码不在本仓库内、未做运行时抓包——请阅读该报告了解边界。

## 致谢

- [jqssun/android-airplay-server](https://github.com/jqssun/android-airplay-server) —— 主体实现
- [UxPlay](https://github.com/FDH2/UxPlay) —— AirPlay/RAOP 协议服务端
- [FFmpeg](https://ffmpeg.org) —— 无损音频解码
- [Next Player](https://github.com/anilbeesetti/nextplayer) —— 视频播放器参考
