# 路线 2：改用现成开源方案（决策与集成方案）

> 背景：验收方反馈"家用开源小东西，费什么劲，不如直接用开源项目"。
> 用户选择路线 2 —— 转向集成现成开源实现，用成熟代码达成"能投屏"（L3），
> 同时化解一直卡着的"合法协议资料"门禁（借力已有实现，无需自研私有算法）。
>
> 调研日期：2026-11-15。信息来源：各项目 GitHub 仓库、F-Droid、AUR、XDA、
> 以及社区技术调研文档（见文末"信息来源"）。许可结论请以各项目 LICENSE 原文为准，
> 本文不构成法律意见。

---

## 一、TL;DR（一句话结论）

**已经存在"开箱即用"的开源 AirPlay 2 接收端，且已适配 Android TV、能真投屏出画面**：
`jqssun/android-airplay-server`（基于 UxPlay，GPL-3.0，已上架 Google Play / F-Droid）。
对"家用开源小项目"，最省事的做法是**直接采用它**（或 fork 后换成自己的名字/UI），
而不是继续自研协议栈。代价只有一个：**必须接受 GPL-3.0 的强 copyleft**（分发即开源）。

---

## 二、候选项目对比矩阵

| 项目 | 协议/能力 | 平台 | 许可 | 活跃度 | 对本项目的可用性 |
|---|---|---|---|---|---|
| **jqssun/android-airplay-server** | AirPlay 2 接收端；屏幕镜像 H.264/H.265、AAC-ELD/LC/ALAC 音频、HLS 视频、PIN 认证、遥控器、PiP | **Android 7.0+ / Android TV** | **GPL-3.0** | 高（约 854★，近期持续提交，已上架 F-Droid + Google Play） | ⭐ **直接可用**：就是本项目想做的东西，已实现 L3；可 fork 改名换 UI |
| **bertanbakir/airplay-screen-receiver** | AirPlay 屏幕镜像；H.264/H.265、横竖屏、JmDNS 发现、Leanback 启动器 | **Android TV / Google TV，API 26+** | **GPLv3** | 新项目，活跃开发中（含 NOTICE.md 许可说明） | 可用，TV 体验更纯粹；成熟度略低于 jqssun |
| **UxPlay（FDH2/UxPlay）** | AirPlay Mirror + Audio 服务端；H.264/H.265 镜像、ALAC 无损、HLS；仅实现 AirPlay 2 "Legacy Protocol" | Linux / macOS / Windows（C + GStreamer） | **GPL-3.0** | 很高（约 2986★，v1.74，持续维护） | 上游核心库；上述两个 Android 项目都基于它。非 Android 原生，需 JNI 移植（已有项目做过） |
| **shairport-sync** | 仅**音频** AirPlay 1/2 接收（无视频/照片）；多房间同步 | Linux / BSD / macOS | **MIT（宽松）** | 高，成熟 | ⚠️ 只能做"AirPlay 音箱"，**不能投屏出画面**，不满足 L3；但许可最宽松 |
| **openairplay/airplay2-receiver** | AirPlay 2 接收端参考实现（Python），逆向研究成果 | Python | 无明确许可（视作保留权利/仅研究参考） | 中（约 2382★，最后一次提交 2026-06） | 仅作协议学习资料，**不可作为代码来源** |
| **pyatv-rs / prior-art 调研** | 纯 Rust 重实现的先验调研文档，梳理各实现许可与协议面 | 文档 | 文档本身供参考 | 2026-08 | 有价值的技术背景资料，佐证"视频解码应交给成熟框架" |

### 关键洞察

1. **能投屏出画面（视频镜像）的开源接收端，全部是 GPL 系列**，无一例外——因为核心都来自 UxPlay（GPL-3.0）。
   想规避 GPL，就只能退回"仅音频"（shairport-sync, MIT），那就达不到 L3。
2. UxPlay 自己都**只实现了 AirPlay 2 的 "Legacy Protocol"**，未达成完整现代 AirPlay 2 兼容——
   这印证了"自研到完整兼容"成本极高，也说明现成方案已是社区可达的务实上限。
3. 视频解码的业界共识是**交给成熟框架**（GStreamer / MediaCodec），而不是手写解码器。
   jqssun 项目正是用 Android MediaCodec 解码 + SurfaceView 渲染。

---

## 三、许可合规：必须正面回答的问题

### 3.1 本项目当前许可状态（重要）

- 根目录 `LICENSE` 是**占位符（NOT A LICENSE GRANT）**，项目级许可**尚未确定**。
- Rust crate 元数据写的是 `MIT OR Apache-2.0`（与根占位符不一致，属待办）。

### 3.2 GPL-3.0 意味着什么（大白话）

| 场景 | 是否触发 GPL 义务 | 说明 |
|---|---|---|
| 自己家里用、不分发 | **不触发** | GPL 只约束"分发"，个人/内部使用不受限 |
| fork 后自己改、只在自己设备跑 | **不触发** | 同上 |
| 把 APK 发给别人 / 上架应用商店 | **触发** | 必须**整个衍生作品以 GPL-3.0 开源**，提供对应源码、构建脚本、依赖版本、许可声明 |
| 想做成闭源/商业授权产品 | **不可行（走 GPL 路线）** | 需另寻宽松许可方案（目前只有音频级 MIT），或获 UxPlay 作者授权 |

### 3.3 单向兼容（好消息）

若最终选 GPL 路线：现有 Rust crate 的 `MIT OR Apache-2.0` 代码**可以被 GPL-3.0 作品包含**
（宽松许可 → GPL 是单向兼容的），不冲突。但**整个对外分发的作品要按 GPL-3.0 整体开源**。

### 3.4 协议合法性旁注

UxPlay / jqssun 均声明"不隶属于 Apple"，协议由公开资料与行为观察实现（非 Apple 公开规范）。
采用现成实现，等于沿用社区既有的逆向成果；本项目原红线"不猜测/不逆向私有算法"在**引用成熟开源实现**的路径下不再构成自研阻塞，
但需知悉底层协议实现本身仍是逆向产物——这是所有第三方 AirPlay 接收端的共同现状。

---

## 四、三条可执行路线（按推荐度排序）

### 路线 A —— 直接用现成 App（零开发，最快）⭐ 最贴合对方诉求

- 在目标 Android TV 上直接安装 `jqssun/android-airplay-server`（F-Droid / Google Play / GitHub Release）或 `bertanbakir/airplay-screen-receiver`。
- 同一局域网，iOS/macOS 控制中心即可看到并投屏。
- **工作量：0**；**满足 L3**。
- 适合：只要"家里能用"、不需要自有品牌。

### 路线 B —— Fork 现成项目二次开发（推荐给"要有自己项目"的场景）⭐

1. fork `jqssun/android-airplay-server`（功能最全、成熟度最高、有 CI 与 F-Droid 分发）。
2. 换包名（`applicationId`）、应用名、图标、主题；精简不需要的设置项。
3. 复用其 GitHub Actions 出 APK（本机 macOS 10.15 零环境即可验收，与既有 `CI_APK_DELIVERY.md` 方案一致）。
4. **合规**：保留 UxPlay 等上游版权与许可声明；按 GPL-3.0 开源整个衍生作品；补 `NOTICE.md`。
5. **工作量**：小（UI/品牌层改动为主，协议与媒体管线无需自研）。
- 适合：既要能投屏，又要"这是我们自己的开源项目"。

### 路线 C —— 在本仓库内集成 UxPlay（不推荐）

- 保留现有 Kotlin/Rust 骨架，把 UxPlay 作为 native 库 + JNI 集成进来。
- 等于把 jqssun 已做过的 JNI/MMediaCodec 工作**重做一遍**，且同样受 GPL 约束。
- **工作量**：大；**收益**：几乎没有（不如直接走 B）。
- 仅当"必须保留现有工程结构/品牌"时才考虑。

### 明确不推荐

- 继续自研协议栈冲 L2/L3：成本最高，且仍卡合法资料门禁，被现成方案完全覆盖。
- shairport-sync 路线：仅音频，达不到"投屏出画面"。

---

## 五、对现有工程的处理建议

| 资产 | 建议 |
|---|---|
| `protocol-rust/`（自研协议骨架，MIT/Apache） | 路线 A/B 下不再进入产品；可保留为"协议学习/研究"资产，或从主分支归档 |
| Android UI + DNS-SD 发现（L0/L1） | 路线 B 下由 fork 项目的成熟实现替代 |
| CI 出包方案（`build-apk.yml`、`CI_APK_DELIVERY.md`） | ✅ 保留思路，B 路线可直接复用其云端出 APK 模式 |
| 已完成文档（DEFINITION_OF_DONE / ADR-001 / 本方案） | 保留，作为决策留痕 |
| 根 `LICENSE` 占位符 | 若走 GPL 路线，必须替换为 GPL-3.0 全文并统一 crate 元数据；若纯自用可不分发 |

---

## 六、给验收方/需求方可直接转述的结论

> 你说得对，这是个家用小东西，自研协议栈确实费力不讨好。我调研过了：
> 已经有一个成熟的开源 AirPlay 2 接收端（`jqssun/android-airplay-server`，基于 UxPlay），
> 支持 Android TV、屏幕镜像（H.264/H.265）、音频、PIN 认证，已经上架 F-Droid 和 Google Play，
> 装到电视上就能被 iPhone 投屏。我们直接用它（或 fork 一份换成我们的名字），
> 不用再自研协议，几天就能出能投屏的版本。唯一要注意的是它是 GPL-3.0，
> 我们要发 APK 就得按 GPL 开源——但本来这就是开源项目，正好一致。
> 要不要我直接按这个方案做？

---

## 七、下一步行动清单（待用户确认路线后执行）

- [ ] 用户确认走 **A**（直接用现成 App）还是 **B**（fork 二次开发）。
- [ ] 若 B：确定目标包名/应用名/品牌、目标 Android TV 机型与 iOS 版本（首个互操作组合）。
- [ ] 若 B：fork → 品牌化改动 → 接 CI 出 debug/release APK → 云端出包验收（本机零环境）。
- [ ] 决定项目许可：自用不分发可不处理；分发则落实 GPL-3.0 全文 + NOTICE.md + crate 元数据对齐。
- [ ] 明确：是否保留 `protocol-rust/` 作为研究资产。

---

## 八、信息来源

- jqssun/android-airplay-server —— https://github.com/jqssun/android-airplay-server （README / LICENSE / F-Droid 页面）
- bertanbakir/airplay-screen-receiver —— https://github.com/bertanbakir/airplay-screen-receiver
- FDH2/UxPlay —— https://github.com/FDH2/UxPlay （README / uxplay.spec / AUR 包许可字段 GPL-3.0-or-later）
- mikebrady/shairport-sync —— MIT（Homebrew Formulae / Debian copyright）
- openairplay/airplay2-receiver —— https://github.com/openairplay/airplay2-receiver
- SkrOYC/pyatv-rs prior-art 调研 —— https://github.com/SkrOYC/pyatv-rs/blob/main/docs/research/prior-art.md
- XDA 论坛 jqssun 发布帖 —— https://xdaforums.com/t/app-guide-open-source-airplay-receiver-for-android-mirroring-video-audio-support.4785938/

> 注：许可与活跃度数据会随时间变化，落地前请以各仓库当时的 LICENSE 与提交记录复核。
