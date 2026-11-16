# 发布签名

Android 不允许用不同签名覆盖安装同一个包名的应用。签名一旦变化，用户必须先卸载
（`INSTALL_FAILED_UPDATE_INCOMPATIBLE`），本地设置全部丢失。所以「固定签名」是
这个项目从「能装的包」变成「能升级的产品」的前提。

## 两种模式

`apk.yml` 的 `Prepare signing key` 步骤按 secret 是否存在自动选择：

| 模式 | 触发条件 | 行为 | 能否覆盖升级 |
|---|---|---|---|
| `stable` | 存在 `SIGNING_KEYSTORE_B64` | 解码仓库 secret 里的 keystore 用于签名 | ✅ 可以 |
| `ephemeral` | 没有 secret（如全新 fork） | runner 上临时 `keytool` 生成一次性密钥，用完删除 | ❌ 必须先卸载 |

构建日志会打印 `SIGNING_MODE`，`apksigner verify --print-certs` 输出证书 SHA-256，
Release 说明里也会写入同一份指纹，便于核对。

> **当前状态（2026-11-16）**：仓库 secrets 已配置 4 项，下一次打 `v*` 标签构建即为
> `stable` 模式，证书 SHA-256 应为 `C7:25:9B:E7:…:83:CD`。
> 由于 `v0.1.0` ~ `v0.3.0` 都是一次性密钥签的，升级到第一个 stable 包时**仍需卸载一次**，
> 之后才能一路覆盖升级。

## 启用 stable 模式

一次性配置，共 4 个 repository secret（**Settings → Secrets and variables → Actions →
New repository secret**）；已配置后可跳过本节。

| 名称 | 内容 |
|---|---|
| `SIGNING_KEYSTORE_B64` | release keystore 的 base64（单行，约 5.9 KB） |
| `SIGNING_STORE_PASSWORD` | keystore 密码 |
| `SIGNING_KEY_ALIAS` | 密钥别名，默认 `yunlou` |
| `SIGNING_KEY_PASSWORD` | 密钥密码（PKCS12 下与 store 密码相同） |

密钥材料**不进仓库**。仓库是公开的，公开的签名私钥意味着任何人都能签一个
「同名同签名、版本号更高」的包覆盖到用户电视上。

配置完成后，下一次打 `v*` 标签即生效，无需改代码。

## 本地保管

签名材料放在仓库之外的本地目录（示例）：

```
~/Documents/airplay-tv/.signing/
  yunlou-release.jks      # PKCS12，RSA 4096，有效期 10000 天
  CREDENTIALS.txt         # storePassword / keyAlias / keyPassword
```

**这个目录必须离线备份**（U 盘 / 密码管理器）。丢失后无法再发布「可覆盖升级」的版本，
只能让所有用户卸载重装。

## 指纹

```
keytool -list -v -keystore yunlou-release.jks -alias yunlou -storepass <密码> | grep SHA256
apksigner verify --print-certs app-release.apk
```

App 内「设置 → 关于 → 签名指纹」显示同一份 SHA-256 的前 8 字节，可用来确认手里的
APK 与已安装的版本出自同一把钥匙。

## 本地构建（可选）

仓库根目录放 `local.properties`（已在 `.gitignore` 中）：

```properties
storeFile=/absolute/path/to/yunlou-release.jks
storePassword=...
keyAlias=yunlou
keyPassword=...
```

`app/build.gradle.kts` 检测到 `storeFile` 就会创建 `release` signingConfig；
没有该文件的构建输出未签名 release APK（不可安装）。
