# 构建与测试

## 固定基线

- JDK 17
- Gradle 8.9（`gradle/wrapper/gradle-wrapper.properties` 固定下载地址与 SHA-256）
- Android Gradle Plugin 8.7.3、Kotlin/Compose 插件 2.0.21
- compileSdk/targetSdk 35，minSdk 23
- Spotless 7.0.2、ktlint 1.5.0

仓库包含 Wrapper 启动脚本和属性文件。受当前离线执行环境限制，可信的 `gradle-wrapper.jar` 无法取得，且不得伪造二进制文件；请从经审核的 Gradle 8.9 发布包生成/恢复 `gradle/wrapper/gradle-wrapper.jar`，提交前核验来源。当前脚本在 JAR 缺失时会给出确定性错误。

## Android 构建

设置 `JAVA_HOME` 为 JDK 17，并确保 Android SDK 35 已安装：

```sh
# 首次联网解析依赖后，生成并人工审核严格依赖校验元数据
./gradlew --write-verification-metadata sha256 help
# 将生成的 gradle/verification-metadata.xml 纳入版本控制，再启用严格校验。

./gradlew --no-daemon spotlessCheck
# 如仅报告格式差异，先执行：
# ./gradlew --no-daemon spotlessApply
# 然后重新执行 spotlessCheck。仓库通过 settings.gradle.kts 集中管理依赖仓库，
# 并为 Spotless 的格式化工具配置提供 Maven Central，禁止根项目另行添加仓库。
./gradlew --no-daemon :app:lintDebug
./gradlew --no-daemon :app:testDebugUnitTest
./gradlew --no-daemon :app:assembleDebug
```

Debug APK 输出为 `app/build/outputs/apk/debug/app-debug.apk`。建议记录：

```sh
sha256sum app/build/outputs/apk/debug/app-debug.apk
```

不应凭空手写依赖摘要；必须由 Gradle 对实际解析的传递依赖生成，再进行人工来源审核。审核完成后可在 `gradle.properties` 设置 `org.gradle.dependency.verification=strict`。

## 发现服务验收

发现默认关闭，应用启动或控制器绑定不会发布服务。仅当用户显式打开“允许局域网设备发现”后：

1. Android 13+ 请求 `NEARBY_WIFI_DEVICES`；拒绝时开关恢复关闭并显示拒绝状态，不启动发布。
2. 权限已授予时发布 `_airplay._tcp.` 与 `_raop._tcp.`，界面展示启动、已发布、网络不可用或错误状态。
3. 5 分钟倒计时归零后自动停止；用户手动关闭立即停止。
4. 重复打开或关闭为幂等操作，不重复注册。
5. 进程/ViewModel 清理时停止；不会自动后台开放。

设备测试应在同一局域网使用 DNS-SD 浏览器确认打开后出现两条记录，关闭、倒计时结束或失去网络后撤销；权限拒绝路径也必须实测。服务记录出现不代表 AirPlay 协议互操作完成。

## 当前环境的可复现阻塞

本沙盒没有 `java`、`gradle`、Android SDK、缓存依赖或 `gradle-wrapper.jar`，且任务禁止联网获取二进制发布包。因此无法在此处执行 Lint、单元测试、构建 Debug APK或生成真实依赖校验元数据。当前可复现命令及预期结果：

```sh
./gradlew --version
# ERROR: .../gradle/wrapper/gradle-wrapper.jar is missing.
java -version
# java: command not found
```

在具备上述受信工具链的环境恢复 Wrapper JAR后，按“Android 构建”四条命令执行；仅在全部成功后宣称构建通过。
