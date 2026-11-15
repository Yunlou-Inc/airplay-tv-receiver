# NOTICE

云楼投屏（Yunlou AirPlay TV）是一个衍生作品（derivative work），基于以下上游开源项目构建。
本文件依据 GNU General Public License v3.0 第 4、5、6 条的要求，声明来源、版权与许可。

## 上游项目与版权

| 组件 | 上游来源 | 许可 | 版权 |
|---|---|---|---|
| android-airplay-server（主体工程、JNI 桥接、Android 渲染/音频管线、UxPlay 补丁集） | https://github.com/jqssun/android-airplay-server | GPL-3.0 | Copyright (c) jqssun and contributors |
| UxPlay（AirPlay/RAOP 协议服务端 C 库） | https://github.com/FDH2/UxPlay | GPL-3.0-or-later | Copyright (c) FDH2/UxPlay contributors |
| FFmpeg（无损音频解码，源码构建） | https://ffmpeg.org | LGPL/GPL（依构建配置） | Copyright (c) FFmpeg contributors |
| libplist | https://github.com/libimobiledevice/libplist | LGPL-2.1+ | Copyright (c) libimobiledevice contributors |
| openssl-cmake / OpenSSL | https://github.com/viaduck/openssl-cmake ；https://www.openssl.org | Apache-2.0（OpenSSL 3.x） | Copyright (c) OpenSSL contributors |
| oboe（低延迟音频，prefab AAR） | https://github.com/google/oboe | Apache-2.0 | Copyright (c) Google LLC |
| Next Player（视频播放器代码参考） | https://github.com/anilbeesetti/nextplayer | GPL-3.0 | Copyright (c) Anil Beesetti |

完整许可文本见本仓库 `LICENSE`（GPL-3.0）。各子模块保留其自身的 LICENSE 文件。

## 本项目所做的修改

相对上游 `jqssun/android-airplay-server`（main 分支），本 fork 做了如下品牌化与工程化改动：

1. `applicationId` 由 `io.github.jqssun.airplay` 改为 `cn.net.yunlou.airplaytv`（安装身份）。
2. 应用显示名改为「云楼投屏」；Gradle 工程名改为 `YunlouAirPlayTV`。
3. 版本信息改为本 fork 自己的版本号（`0.1.0-yunlou`，versionCode=1）。
4. 新增云端出包工作流 `.github/workflows/build-debug-apk.yml`（GitHub Actions），
   产出可安装的 debug APK 供零环境验收；保留上游 `apk.yml` 不动。
5. `app/build.gradle.kts` 新增可选 `-Pabis=` 属性，允许 CI 只编译单一 ABI
   （arm64-v8a）以缩短从源码编译 OpenSSL/FFmpeg 的时间；省略该参数时与上游默认一致。
6. 新增本 NOTICE、项目 README 与 `docs/CI_APK_DELIVERY.md`（零环境验收手册），
   明确衍生来源与 GPL 合规义务。
7. 安全加固（详见 `docs/SECURITY_REVIEW.md`）：
   - `Prefs.kt` 将 `boot_auto_start` 默认值由 `true` 改为 `false`，
     不在每次开机后静默开启监听；用户可在设置中自行启用。
   - `app/src/main/cpp/CMakeLists.txt` 新增 `OPENSSL_BUILD_HASH`，
     为构建期从第三方镜像下载的 OpenSSL 源码 tarball 启用 SHA256 校验。
     该哈希取自 OpenSSL 官方发布（openssl.org 与 GitHub release 双来源一致），
     并已通过完整下载镜像文件本地计算比对确认一致。
8. 新增 `docs/SECURITY_REVIEW.md`（静态安全审查报告：证据、加固与审查边界）。

**未改动**：Kotlin 源码包名（`io.github.jqssun.airplay`）与 JNI 原生符号
（`Java_io_github_jqssun_airplay_*`）。这是有意保留的内部技术标识——它们与 native 代码强绑定，
修改需同步重编译原生库，且与最终用户/安装身份无关。保留它们是 GPL 衍生作品诚实标注来源的一部分。

## 仓库历史与许可边界

本仓库 `master` 分支当前**整体为 GPL-3.0 衍生作品**。
早期自研的 Rust 协议层 + Kotlin 骨架（`airplay-protocol` crate 为 MIT OR Apache-2.0，
根目录 LICENSE 当时为占位符）不能真投屏，已于 2026-11-15 放弃，
完整历史归档于 `legacy-prototype` 分支。

该归档分支仅作历史留存，**不属于当前主线的分发内容**，其许可也不与本项目混合：
本 NOTICE 及 `LICENSE`（GPL-3.0 全文）只约束 `master` 当前树所包含的代码。
如需分发，请以 `master` 当前树为准提供完整对应源代码。

## GPL-3.0 合规义务提醒

- 本作品整体以 **GPL-3.0** 许可发布；任何对外分发（APK、AAB、源码）都必须附带完整对应源代码、
  本 LICENSE 与 NOTICE，并保留所有上游版权声明。
- 若仅个人/家庭内部使用、不分发，则 GPL 的分发义务不触发。
- 分发时须保留各子模块（UxPlay / FFmpeg / libplist / OpenSSL / oboe）各自的许可与版权声明。
- 本声明不构成法律意见；商业分发前请自行复核各依赖许可（尤其 FFmpeg 的 LGPL/GPL 构建选项）。

## 免责声明

本项目与上游一样，**不隶属于 Apple Inc.**。AirPlay 由 Apple Inc. 持有商标。
