# 安全审查报告（云楼投屏 / fork 自 jqssun）

> 审查日期：2026-11-15
> 审查对象：`airplay-tv-receiver-fork`，本地提交基线 `67d538f` 及其后续品牌/加固改动
> 上游：`jqssun/android-airplay-server` main 分支（GPL-3.0），底层 UxPlay（GPL-3.0-or-later）
> 方法：**静态源码审查**（本仓库内可见的全部源码、构建脚本、补丁、CI 工作流）+ 上游 API 交叉核对
> 本文不构成安全保证或法律意见。每项结论都给出可复核的文件与行号；查不到的如实标注边界。

---

## 一、结论先行

| 问题 | 结论 |
|---|---|
| **有没有发现"后门"特征？** | **在本仓库可见的源码中未发现**：无硬编码外联地址/域名/IP、无遥测统计广告 SDK、无动态代码加载、无反射执行、无凭据外发路径、无隐蔽文件上传。网络行为均可由 AirPlay 协议本身解释。 |
| **能不能说"绝对安全"？** | **不能。** 静态审查有明确边界（见第五节）：4 个 native submodule 的实际源码不在本仓库内（只有 gitlink），未逐行审计；未做运行时抓包、未做二进制/签名验证；第三方构建期下载源存在校验缺口。 |
| **可控吗？** | **基本可控，且已做加固**：依赖全部锁定到精确 commit、出包走自己的 CI、无第三方统计服务、已关闭默认开机自启、OpenSSL 构建期下载已加 SHA256 校验（见 4.1 与加固执行记录）。剩余风险点（见第四节）已列明并给出处置方式。 |

一句话：**没有证据显示有后门；但"可信"来自可复核的依赖锁定与自有出包链路，不是来自"我读完了所有代码"——因为我确实读不到 submodule 内部。**

---

## 二、逐项审查结果（文件 + 行号）

### 2.1 遥测 / 统计 / 广告 SDK —— 未发现

- `gradle/libs.versions.toml` 依赖清单全部为 AndroidX / Compose / Media3 / Hilt / Kotlin / Oboe，
  **没有** Firebase、Analytics、Crashlytics、Sentry、广告或任何第三方统计 SDK。
- `app/build.gradle.kts` 的 `dependencies {}` 同样只有上述官方/开源组件。
- 全仓检索 `firebase|analytics|crashlytics|sentry|telemetry|admob|umeng` 等关键词，命中项均为
  注释里的 "download"、"thread" 等普通词，无真实上报代码。

### 2.2 硬编码外联地址 —— 未发现

- 全仓检索源码中的 `http(s)://` 字面量，除 XML 命名空间声明外，唯一业务 URL 构造在
  `app/src/main/kotlin/io/github/jqssun/airplay/audio/DacpController.kt:90`：
  ```kotlin
  val url = "http://$host:$port$path"
  ```
  其中 `host/port` 来自 `DacpController.kt:71-72` 的 mDNS 服务解析结果（`si.host.hostAddress`），
  即**同一局域网内发送端设备自己的地址**，用于 DACP 遥控回连。不是固定外部服务器。
- mDNS 组播地址 `224.0.0.251`（`discovery/UsbMdnsAdvertiser.kt:344`）是标准 mDNS 组播地址。
- 检索 `\b(\d{1,3}\.){3}\d{1,3}\b` 未发现任何硬编码公网 IP。
- C/C++ 侧检索 `getaddrinfo|gethostbyname|connect(|curl_easy|socket(AF_INET` 在当前仓库可见的
  native 源码中无命中——网络收发逻辑位于 UxPlay submodule 内（见第五节边界）。

### 2.3 数据外发 / 上传 —— 未发现隐蔽通道

- `download/VideoDownloader.kt`：把发送端指定的媒体用 Media3 Transformer 转存到
  **本机** `MediaStore.Downloads`（`:91-124`），是用户可见的"下载视频"功能，不上传。
- `renderer/AirPlayVideoPlayer.kt:81` 与 `VideoDownloader.kt:75` 的 `MediaItem.fromUri(location)`
  会访问**发送端传来的 URL**（AirPlay HLS 投屏的正常行为）。这一点属于协议设计：
  **投屏方可让接收端访问其给定的任意 URL**，不应在不可信网络中使用（见 4.4）。
- `ui/LogsScreen.kt:47-48` 的导出/分享是用户主动点击触发，文件范围受
  `res/xml/file_paths.xml` 限制为 `<cache-path name="cache" path="." />`（仅应用缓存目录）。
- `FileProvider`（`AndroidManifest.xml:74-82`）`exported=false`，authority 跟随 `${applicationId}`。

### 2.4 动态代码加载 / 反射执行 —— 未发现

- 检索 `Runtime.getRuntime|ProcessBuilder|DexClassLoader|PathClassLoader|Class.forName|WebView|eval(`：
  - `app/build.gradle.kts:112` 的 `ProcessBuilder("git", ...)` 仅用于**构建期**给 UxPlay 打补丁
    （`applyUxplayPatches` 任务），不进入 APK。
  - `bridge/NativeBridge.kt:8` 的 `System.loadLibrary("airplay_native")` 加载的是本仓库
    `app/src/main/cpp/CMakeLists.txt` 从源码编译出的自有 so，非外部下载。
  - 其余命中均为 `gradlew` 脚本自身的 POSIX 注释。
- 无 WebView、无 `javascript:`、无远程 dex/jar 加载。

### 2.5 硬编码凭据 —— 未发现

- `app/build.gradle.kts:21-30` 的签名配置从 `local.properties` 读取，且该文件被
  `.gitignore` 排除（`*.jks`、`/local.properties`），仓库内无密钥、无密码、无 token。
- 全仓检索 `api_key|secret|token|password|BEGIN ... PRIVATE KEY|Bearer`：命中项为
  `storePassword`/`keyPassword` 的属性读取、`mediaSession.sessionToken`（Android 媒体会话令牌）、
  GPL 许可证正文，均非硬编码凭据。
- 本 fork 的两个出包工作流（`build-debug-apk.yml`、`apk.yml`）**均不引用任何
  repository secret**。release 签名所需的 keystore 与随机密码在 runner 上临时生成、
  用完即删，仓库内仍无任何密钥、密码或 token。

### 2.6 权限与组件暴露面

`app/src/main/AndroidManifest.xml` 声明 12 项权限，全部可由功能解释：

| 权限 | 用途 | 评价 |
|---|---|---|
| `INTERNET`、`ACCESS_NETWORK_STATE`、`ACCESS_WIFI_STATE` | 局域网 RTSP/HTTP 收发 | 必需 |
| `ACCESS_LOCAL_NETWORK` | Android 16+ 本地网络访问 | 必需 |
| `CHANGE_WIFI_MULTICAST_STATE` | mDNS 组播发现 | 必需 |
| `WAKE_LOCK`、`FOREGROUND_SERVICE*`、`POST_NOTIFICATIONS` | 前台服务常驻与通知 | 必需 |
| `RECEIVE_BOOT_COMPLETED` | 开机自启（**默认已关闭**，见 3.2） | 可控 |
| `SYSTEM_ALERT_WINDOW` | 连接时自动打开应用 / 悬浮 | 高敏权限，见 4.3 |
| `android:usesCleartextTraffic="true"` | AirPlay 协议为明文 HTTP/RTSP | 协议要求，见 4.4 |

**未申请**：通讯录、短信、通话、位置、相机、录音、外部存储广泛读写、查询所有应用列表等敏感权限。

组件暴露面：

- `MainActivity` `exported=true` —— 启动器必需。
- `BootReceiver` `exported=true` —— 只接收系统开机广播，且 `BootReceiver.kt:15` 检查
  `Prefs.BOOT_AUTO_START` 开关，**默认 false**（本 fork 加固）。
- `SettingsReceiver` `exported=true` —— 可远程改配置，但受
  `android:permission="android.permission.DUMP"` 保护（`:67`）。该权限为 signature/privileged 级，
  普通第三方 App 无法持有，实质仅 adb/系统可调用。**这是上游较谨慎的设计**，予以保留。
- `AirPlayService`、`FileProvider` 均 `exported=false`。
- `android:allowBackup="false"`（`:27`）—— 避免数据经备份外流。

### 2.7 UxPlay 补丁集（9 个，共 1060 行）

全部为播放列表崩溃修复、UAF 修复、HLS 刷新、视频兼容、PIN 配对、音量/会话逻辑等，
检索 `curl|wget|exec|system(|getenv|token|secret|password` 无异常命中；`http://` 仅出现在
`0004-video-sender-compat.patch:216-217`，用于判断发送端给的播放地址前缀，属协议逻辑。

**一处需复核**（不下结论）：`0008-fix-pin-pairing-proof-length.patch` 把客户端证明长度的
严格校验 `client_proof_len != sizeof(proof)` 改为 `> sizeof(proof)`，并按实际长度 `memcpy`。
这是为兼容不同发送端的配对实现，但短于预期长度的 proof 是否被下游 `srp_validate_proof`
正确拒绝，需具备 SRP/密码学能力者复核。**本审查未验证其密码学正确性**，如实标记。

### 2.8 构建供应链

| 项 | 状态 |
|---|---|
| 4 个 native submodule | ✅ 已锁定到上游精确 commit（gitlink `160000`），见 `git ls-tree HEAD`。补丁集针对该 UxPlay 版本编写，不可随意升级 |
| FFmpeg | ✅ `cmake/BuildFFmpeg.cmake:43` `DOWNLOAD_COMMAND ""`（只用本地 submodule 源码，不联网）；`:55` `--disable-network --disable-all`，仅启用 ALAC/AAC 解码器，**native 侧无网络协议栈** |
| Gradle 依赖仓库 | ✅ 仅 `google()` / `mavenCentral()`（`settings.gradle.kts`） |
| CI Actions | ✅ 仅 `actions/checkout@v4`、`setup-java@v4`、`upload-artifact@v4`、`softprops/action-gh-release@v2`；用主版本标签而非 commit SHA，属常见做法但非最强固定（见 4.5） |
| **OpenSSL 源码下载** | ⚠️ **风险，见 4.1** |

---

## 三、本 fork 已落地的安全加固

| # | 改动 | 位置 |
|---|---|---|
| 3.1 | 依赖全部锁定到精确 commit（4 个 gitlink），避免"浮动依赖"引入未审代码 | `git ls-tree HEAD app/src/main/cpp/third_party/` |
| 3.2 | `boot_auto_start` 默认由 `true` 改为 **`false`**：不在每次开机后静默开启监听 | `Prefs.kt:78` 附近 |
| 3.3 | 出包链路全部改为自有工作流（`build-debug-apk.yml` + 重写的 `apk.yml`），不引用任何 repository secret，产物含 sha256 校验；release 签名为 runner 内临时密钥 | `.github/workflows/*.yml` |
| 3.4 | `allowBackup=false`、FileProvider 不导出、`SettingsReceiver` 保留 DUMP 权限保护 | 上游设计，确认保留 |
| 3.5 | 明确 GPL-3.0 来源、版权与修改清单，便于第三方复核 | `NOTICE.md`、本报告 |

**仍建议但本轮未改**（避免擅自改变投屏可用性，需你拍板）：
- `require_pin` 默认 `false`（`Prefs.kt:103`）→ 建议改 `true`，强制首次连接输入 PIN；
- `auto_start` 默认 `true`（`Prefs.kt:77`）→ 可视需要改为手动启动；
- `launch_on_connect` 默认 `true`（`Prefs.kt:115`）→ 涉及 `SYSTEM_ALERT_WINDOW` 用法。

---

## 四、剩余风险与处置

### 4.1 ✅ OpenSSL 构建期下载无完整性校验 —— 已发现并已加固

`openssl-cmake` 锁定版本（`4edd36a8…`）的 `cmake/BuildOpenSSL.cmake:182`：

```cmake
ExternalProject_Add(openssl
    URL https://mirror.viaduck.org/openssl/openssl-${OPENSSL_BUILD_VERSION}.tar.gz
    ${OPENSSL_CHECK_HASH}
```

而 `OPENSSL_CHECK_HASH` 仅在 `OPENSSL_BUILD_HASH` 被设置时才生效（同文件 `:44-45`）。
本仓库 `app/src/main/cpp/CMakeLists.txt:14` 只设了 `OPENSSL_BUILD_VERSION = "3.4.4"`，
**没有设哈希** → 构建时从第三方镜像下载 OpenSSL 源码且**不做任何校验**。

补充事实（均已核实）：
- 镜像站不提供 `.sha256` sidecar（HTTP 404）。
- `openssl-cmake` 上游 `.gitlab-ci.yml` 的版本历史里**没有 3.4.4 的哈希记录**（相邻为 3.4.3、3.5.4/3.5.5），
  即 3.4.4 是 jqssun 自选版本，第三方镜像无官方配套校验。
- OpenSSL 官方 3.4.4 tarball 的 SHA256（**openssl.org 与 GitHub 官方 release 两个独立来源一致**）：
  `7bdf55ac20f2779e99e5eca306f824fad2b37dee5a06cc35ed5a8b85a6060010`

**处置结果（已闭环）**：在 `app/src/main/cpp/CMakeLists.txt` 设置 `OPENSSL_BUILD_HASH`，
使 `openssl-cmake` 生成 `URL_HASH SHA256=...`，构建时强制校验下载内容。

填值前完成了三重验证，未凭猜测填数：
1. 官方 `openssl.org/source/openssl-3.4.4.tar.gz.sha256` 与
   `openssl/openssl` GitHub release 的 sha256 资源，**两个独立来源一致**：
   `7bdf55ac20f2779e99e5eca306f824fad2b37dee5a06cc35ed5a8b85a6060010`
2. 用 HTTP Range 分 35 段下载镜像上完整的 **18,278,255 字节** tarball，本地
   `shasum -a 256` 结果 = 上述官方值，`gzip -t` 与 `tar -tzf` 均通过。
   → 证明第三方镜像分发的**就是官方原包逐字节副本**，填官方值不会导致构建失败。
3. 该哈希已写入源码并附注释（含来源与验证日期 2026-11-15）。

至此，构建期唯一的网络下载被哈希锁定；若镜像日后被替换成不同内容，CMake 会直接报错而非静默编译。

### 4.2 submodule 源码未逐行审计（审查边界，非缺陷）

本仓库只含 4 个 gitlink，不含 UxPlay / FFmpeg / libplist / openssl-cmake 的实际源码。
"未发现后门"的结论**仅覆盖本仓库可见的 104 个文件**。
若要扩大到完整可信，需把这 4 个锁定 commit 的源码拉下来单独审计，或依赖上游社区的公开审查。

### 4.3 高敏权限 `SYSTEM_ALERT_WINDOW`

用于"发送端连接时自动打开应用"（`launch_on_connect`）。这是 Android 上较敏感的权限，
由用户在系统设置中手动授予，非静默获取。若不需要该功能，建议把默认值改 `false` 并在
Manifest 中移除该权限。

### 4.4 明文协议与"发送端可指定任意 URL"

`usesCleartextTraffic=true` 是 AirPlay（HTTP/RTSP）协议要求，无法关闭。
且如上 2.3 所述，发送端可让接收端访问其给出的任意 URL。
**结论：这是局域网协议设计，不是后门，但意味着不应在不可信/公共 WiFi 中使用**，
建议开启 PIN（4.1 之外的另一层）。

### 4.5 CI 供应链

Actions 使用 `@v4` 主版本标签而非 commit SHA 固定，理论上存在上游 action 被替换的风险。
如要求更强可控，可改为固定 SHA（需逐个查证后替换）。

### 4.6 未做运行时验证

本机无 Android SDK/NDK，且 tarball 不含 submodule 源码，**没有**真正编译出 APK，
因此未做：动态抓包、权限实际申请行为核对、so 二进制符号检查、签名与产物比对。
首次云端出包后，建议用 `adb` + 抓包工具实测一次"无非必要外联"。

---

## 五、审查边界（必读）

1. **范围**：本仓库可见的全部源码、Gradle/CMake 构建脚本、9 个 UxPlay 补丁、CI 工作流、
   Manifest 与资源；外加通过 GitHub API 对上游锁定 commit 的构建脚本做交叉核对。
2. **不在范围**：4 个 submodule 内部源码、编译产物/二进制、运行时网络行为、密码学正确性证明、
   iOS/macOS 发送端行为、第三方镜像服务器本身的安全性。
3. **因此**：本报告能支持"未发现后门特征"，**不能**支持"绝对没有后门"。
   任何声称绝对安全的说法都超出静态审查能证明的范围。

---

## 六、给验收方可转述的说法

> 我们不是自己写协议，是 fork 了一个成熟的开源 AirPlay 接收端。
> 我把仓库里能看到的源码都过了一遍：没有写死的外联服务器、没有统计上报 SDK、
> 没有动态下载代码、没有硬编码密钥；依赖全部锁死到具体版本，出包走我们自己的 CI。
> 顺手把"开机自动开始监听"默认关掉了。
> 有两件事如实说明：一是它底层依赖的四个开源库源码不在本仓库里（只有版本锁），
> 没法逐行审；二是构建时会从第三方镜像拉 OpenSSL 源码，我们已把校验补上（见加固记录）。
> 家用局域网用没问题，但不建议放到公共 WiFi。

---

## 加固执行记录

| 项 | 状态 | 证据 |
|---|---|---|
| 品牌更正「云露」→「云楼」、包名 → `cn.net.yunlou.airplaytv` | ✅ 已完成 | `app/build.gradle.kts:49`、`strings.xml:3`、README / NOTICE / 验收手册同步；全仓检索「云露 / com.yunlou」无残留 |
| `boot_auto_start` 默认 true → false | ✅ 已完成 | `Prefs.kt`（带加固注释） |
| OpenSSL 构建期下载 SHA256 校验 | ✅ 已完成 | `app/src/main/cpp/CMakeLists.txt` 的 `OPENSSL_BUILD_HASH`；哈希经官方双来源 + 完整镜像文件本地计算三重验证 |
| 安全报告与 README 安全入口 | ✅ 已完成 | 本文件、`README.md` 安全章节 |
| `require_pin` 默认 true / `auto_start` 默认 false / 移除 `SYSTEM_ALERT_WINDOW` | ✅ 已决定**不调整** | 2026-11-15 项目方明确：家用场景不收紧，保持上游默认（这些设置防的是同网段误投，不是后门）；见 3.5 / 4.3 |
| Actions 固定到 commit SHA | ⏸ 建议 | 当前为 `@v4` 主版本标签，见 4.5 |
| 运行时抓包 / 二进制符号审计 / 真机权限核对 | ⏸ 未做 | 需先云端出包并具备真机，见 4.6 / 第五节 |

> 审查期间对镜像 tarball 的下载多次被网络中断，最终用 35 段 Range 请求合并出完整文件后才填入哈希。
> **未验证的哈希值一律没有写进任何配置。**
