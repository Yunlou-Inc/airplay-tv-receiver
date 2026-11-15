# 第三方依赖说明

## 当前直接依赖（来自已提交构建清单）

| 名称 | 声明版本 | 用途 | 来源 | 许可证核验 |
|---|---:|---|---|---|
| Android Gradle Plugin | 8.7.3 | Android 构建插件 | Gradle 插件仓库/Google | 待核验 |
| Kotlin Android / Compose 插件 | 2.0.21 | Kotlin 与 Compose 编译 | Gradle 插件仓库 | 待核验 |
| `androidx.core:core-ktx` | 1.15.0 | Android Kotlin 扩展 | Google Maven | 待核验 |
| `androidx.activity:activity-compose` | 1.10.0 | Compose Activity | Google Maven | 待核验 |
| Compose BOM | 2025.01.00 | Compose 版本约束 | Google Maven | 待核验 |
| Compose UI / tooling preview / Material 3 | 由 BOM 解析 | 界面 | Google Maven | 待解析、待核验 |
| Lifecycle ViewModel Compose / KTX | 2.8.7 | 状态生命周期 | Google Maven | 待核验 |
| Compose UI tooling | 由 BOM 解析；仅调试 | 预览/调试 | Google Maven | 待解析、待核验 |
| JUnit | 4.13.2；仅测试 | JVM 单元测试 | Maven Central | 待核验 |

Rust `airplay-protocol` 当前清单没有外部 crate 依赖，并声明 `MIT OR Apache-2.0`。`cargo-fuzz` 仅在文档中作为可选工具提及，不是当前工作区依赖。Android SDK、JDK、Gradle、Rust/Cargo 和传递依赖仍属于工具链/供应链范围，尚未形成完整清单。

## 重要限制

仓库没有锁文件、完整传递依赖解析结果、许可证文本或软件物料清单，故上表不能证明许可证兼容或供应链安全。Compose BOM 管理的实际版本必须从成功解析后的依赖图记录。

## 新增依赖准入

新增前记录名称、精确版本、用途、来源、许可证、直接/传递属性、校验摘要、维护状态、已知漏洞、替代方案和批准人。固定版本并验证完整性；禁止未经批准的安装脚本。密码学只使用经维护、审查的库，禁止自制算法。

发布候选必须生成机器可读的软件物料清单，保留许可证文本和修改说明，并执行漏洞与许可证策略检查；扫描不能替代对网络、媒体和密码学依赖的人工审查。
