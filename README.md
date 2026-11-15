# AirPlay TV Receiver — Android TV / Rust 工程骨架

> **禁止用于生产。** 本仓库仅是界面、进程边界与防御性协议原语骨架；不具备可用的 AirPlay 接收能力，也未声明 Apple 认证或完整兼容。

## 截至 2026-11-15 的状态

| 类别 | 已核验状态 |
|---|---|
| 已实现 | Android TV Compose 界面状态模型；等待、配对确认、播放中和已配对设备等界面流程；私有、不可导出的服务声明及本地 Binder 接缝；默认关闭、可配置的 Android DNS-SD 发布及发现状态接缝；Rust 的有界 RTSP 请求解析、发现/设备模型、显式会话状态机；Kotlin 状态单元测试与 Rust 测试/模糊目标骨架 |
| 未实现 | Android 与 Rust 集成；协议端口网络监听（DNS-SD 发布已实现但尚未设备验证）；真实配对、认证、密钥交换/安全存储；AirPlay 私有密码学与解密；音视频接收、解码、同步、渲染；遥控；持久化；完整互操作测试 |
| 未核验 | 本环境中没有构建成功证据；没有 Gradle Wrapper、`Cargo.lock`、持续集成报告、设备测试报告或完整软件物料清单 |
| 明确禁止 | 生产部署、公网暴露、处理真实长期凭据、把界面演示当作安全配对、宣称完整 AirPlay 兼容或 Apple 认证、规避访问控制/数字版权管理 |

## 目标

在合法互操作资料、授权设备与媒体的前提下，逐步实现电视端接收器：发送端可发现电视，经屏幕确认完成安全配对与会话建立，将授权音视频切换到电视，并可靠停止、撤销和清理。首阶段只承诺一个明确记录的发送端/系统版本与接收硬件组合。

## 工程布局

- `app/`：Android TV 界面、状态模型及隔离服务接缝。
- `protocol-rust/`：独立 Rust 协议原语骨架；当前没有接入 Android 应用。
- `docs/`：架构、威胁模型、安全编码、测试计划和里程碑。
- `BUILDING.md`、`CONTRIBUTING.md`、`THIRD_PARTY.md`：构建、贡献和依赖治理。

## 构建

已声明工具链为 JDK 17、Android SDK 35、Gradle 8.9+，Rust 使用 Cargo。仓库没有 Gradle Wrapper，且未在本环境验证构建。候选命令、前置条件和证据要求见 [BUILDING.md](BUILDING.md)。

## 文档

- [架构](docs/ARCHITECTURE.md)
- [威胁模型](docs/THREAT_MODEL.md)
- [安全编码规范](docs/SECURE_CODING.md)
- [测试计划与接管验收](docs/TEST_PLAN.md)
- [里程碑与后续顺序](docs/MILESTONES.md)
- [贡献规范](CONTRIBUTING.md)
- [第三方依赖](THIRD_PARTY.md)
- [质量验收台账、发布门禁与真机记录模板](docs/ACCEPTANCE_LEDGER.md)

## 法律与商标

AirPlay 是 Apple Inc. 的商标。本项目不表示 Apple 认可、认证或关联。仅可使用有权使用的资料、设备、媒体和密钥；不得提交设备唯一私钥、受限规范或规避措施。仓库整体许可证尚未提供；在明确之前不要假定全部内容可再分发。Rust crate 中声明的双许可证仅适用于其自身清单所涵盖内容，不能代替仓库级许可证。
