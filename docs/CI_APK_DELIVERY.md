# 通过 CI 出包验收（验收方零环境要求）

> 适用：验收方的机器无法升级系统（如 macOS 10.15）、装不了新版 Android Studio，
> 或根本不想装 IDE。本方案让**云端 GitHub Actions 负责构建**，验收方只下载 APK 装到真机。

## 0. 为什么这样可行

构建 App 真正需要的是 Gradle + JDK + Android SDK，这些**全在 GitHub 云端**完成
（见 `.github/workflows/build-apk.yml`）。验收方机器上**不需要**：

- ❌ Android Studio / IntelliJ IDEA
- ❌ JDK、Gradle、Android SDK
- ❌ 升级 macOS（10.15 也无所谓）

验收方机器上**只需要**：

- ✅ 一个浏览器（打开 GitHub 下载 APK）
- ✅ 一根数据线 + `adb`（安装用）；或直接用真机的"从文件安装"
- ✅ 一台 Android 6.0+ 的真机

## 1. 出包（由有仓库写权限的人操作，一次约 2–3 分钟）

1. 打开 <https://github.com/Yunlou-Inc/airplay-tv-receiver/actions>
2. 左侧工作流列表点 **Build debug APK**
3. 右侧点 **Run workflow** → 选分支 `master` → **Run workflow**
4. 等任务跑完（绿色对勾）。**每次 push 到 master 也会自动出一次包。**

## 2. 下载 APK

1. 点进刚跑完的那次 run
2. 页面底部 **Artifacts** 区，下载 `airplay-tv-debug-run-<序号>`（zip）
3. 解压得到两个文件：
   - `app-debug.apk`
   - `app-debug.apk.sha256`（校验用）

校验完整性（可选，推荐）：

```bash
# macOS
shasum -a 256 app-debug.apk
# Linux
sha256sum app-debug.apk
# 输出应与 app-debug.apk.sha256 里的值一致
```

> Artifact 默认保留 30 天，过期需重新跑一次工作流。

## 3. 装到真机（三选一，都不需要 Android Studio）

**方式 A：只装 adb（最轻，macOS 10.15 可用）**

`adb` 是独立命令行工具，不依赖 IDE，也不要求新系统：

```bash
# macOS 10.15：用 Homebrew 或直接从 Google 下载 platform-tools 独立包
brew install android-platform-tools     # 或下载 platform-tools zip 解压
adb devices                             # 手机开启"开发者选项 → USB 调试"并授权
adb install -r app-debug.apk
```

**方式 B：真机直接安装（零命令行）**

把 `app-debug.apk` 通过微信/邮件/网盘传到手机，在手机上点开，允许"安装未知来源应用"即可。

**方式 C：任意第三方助手**（如 Xiaomi 助手等）拖入 APK 安装。

## 4. 验收清单（与 DEV_ENV_AND_RUNBOOK 第 9 节一致）

- [ ] 冷启动看到主界面，状态 `WAITING`
- [ ] 发现开关**默认关闭**；重启 App 后仍关闭
- [ ] 打开开关：Android 13+ 弹 `NEARBY_WIFI_DEVICES` 权限；拒绝 → 回退关闭、不发布服务
- [ ] 授权后：同网段另一设备 `dns-sd -B _airplay._tcp.` 能看到 `_airplay._tcp.` 与 `_raop._tcp.` 两条记录
- [ ] 5 分钟倒计时归零自动停止；手动关闭立即撤销注册
- [ ] 断网 → 状态 `NetworkUnavailable`；恢复 → 重新发布
- [ ] 确认**没有**真正的 AirPlay 端口在监听（当前阶段预期）

## 5. 重要预期（务必先告知验收方）

这个 APK 是**安全骨架**，不是能投屏的接收端：

- ✅ 界面 + DNS-SD 发现开关能演示
- ❌ **不能**真的接收 AirPlay 投屏——没有真实监听/配对/媒体管线
- ❌ `AirPlayProtocol.isAvailable == false`，Rust 协议引擎的 JNI 库尚未接入 Gradle，协议代码优雅降级、不生效

这是当前阶段的预期行为，**不是构建错误**。要让协议引擎真正工作，需先完成 ADR-001 / M1-2 的接线（后续实现工作）。

## 6. 若 CI 出包失败

看 Actions run 的日志。常见原因：

| 现象 | 原因 | 处理 |
|---|---|---|
| `Could not resolve com.android.tools.build:gradle:8.7.3` | 依赖仓库不可达 | 确认 `settings.gradle.kts` 的 `google()` / `mavenCentral()` 可访问 |
| Gradle task 失败 | 见日志具体 task | 本工作流已与 lint/test 门禁解耦，纯 assemble 一般更稳 |
| 下载不到 artifact | 未登录 / 过期 | 登录 GitHub 账号；过期则重跑工作流 |

出包问题把 Actions 日志贴给我即可，我来修。
