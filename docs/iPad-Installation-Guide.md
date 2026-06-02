# Deep Student iPad 安装说明

> Bundle ID: `com.deepstudent.app`
> 最低要求: iPadOS 16.0+

---

## ⚠️ 重要提示

iPad 应用**无法像桌面应用那样双击 IPA 安装**。iPadOS 限制所有应用必须通过签名验证。需要根据你的 Apple Developer 账号类型选择安装方式。

---

## 方案对比

| 方式 | 需要 | 难度 | 适用场景 |
|------|------|:--:|------|
| **Apple Configurator 2** | Mac + USB 线 | ⭐ | 个人测试 |
| **Xcode Devices** | Mac + USB 线 | ⭐ | 开发者测试 |
| **TestFlight** | Apple Developer 账号 ($99/年) | ⭐⭐ | 小范围分发 |
| **OTA 分发** | HTTPS 服务器 + 企业证书 | ⭐⭐⭐ | 企业内部 |
| **AltStore** | 免费 Apple ID | ⭐⭐ | 个人（7天重签） |

---

## 方法 1: Apple Configurator 2（推荐）

**前提**: 一台 Mac，USB 数据线

1. 在 Mac App Store 安装 [Apple Configurator](https://apps.apple.com/app/id1037126344)
2. 用 USB 线连接 iPad 到 Mac
3. 打开 Apple Configurator 2
4. 在左侧选择你的 iPad
5. 将 `DeepStudent-iPadOS.ipa` 拖入设备主屏幕区域
6. 等待进度条完成
7. 在 iPad 上：**设置 → 通用 → VPN 与设备管理** → 点击信任开发者证书

---

## 方法 2: Xcode（开发者）

**前提**: Mac + Xcode 15+

1. 用 USB 线连接 iPad
2. 打开 Xcode → 菜单栏 **Window → Devices and Simulators**
3. 左侧选择你的 iPad
4. 点击已安装应用列表下方的 **"+"** 按钮
5. 选择 `DeepStudent-iPadOS.ipa`
6. 等待安装完成，iPad 桌面出现应用图标

---

## 方法 3: TestFlight（需要 Developer 账号）

**前提**: Apple Developer Program ($99/年)

1. 在 Mac 上安装 [Transporter](https://apps.apple.com/app/id1450874784)
2. 打开 Transporter，将 `DeepStudent-iPadOS.ipa` 拖入
3. 点击"交付"上传到 App Store Connect
4. 在 [App Store Connect](https://appstoreconnect.apple.com) → TestFlight → 内部测试
5. 添加测试员（Apple ID 邮箱）
6. 测试员在 iPad 安装 TestFlight app
7. 在 TestFlight 中接受邀请 → 安装

**优点**: 无线安装，自动更新，最多 100 内部测试员

---

## 方法 4: OTA 无线分发（企业/Ad-Hoc）

需要 HTTPS Web 服务器。在服务器上放置 IPA 和以下 `manifest.plist`：

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
 "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>items</key>
  <array>
    <dict>
      <key>assets</key>
      <array>
        <dict>
          <key>kind</key>
          <string>software-package</string>
          <key>url</key>
          <string>https://你的服务器.com/DeepStudent-iPadOS.ipa</string>
        </dict>
      </array>
      <key>metadata</key>
      <dict>
        <key>bundle-identifier</key>
        <string>com.deepstudent.app</string>
        <key>bundle-version</key>
        <string>0.9.40</string>
        <key>kind</key>
        <string>software</string>
        <key>title</key>
        <string>Deep Student</string>
      </dict>
    </dict>
  </array>
</dict>
</plist>
```

在 iPad Safari 中打开：
```
itms-services://?action=download-manifest&url=https://你的服务器.com/manifest.plist
```

**要求**: 服务器必须 HTTPS，plist MIME type = `application/xml`

---

## 方法 5: AltStore（免费 Apple ID）

**前提**: Mac/PC + 免费 Apple ID

1. 在 Mac 上安装 [AltServer](https://altstore.io)
2. 用 USB 连接 iPad
3. AltServer → Install AltStore → 选择 iPad
4. 在 iPad 上登录 AltStore（用同一个 Apple ID）
5. 在 AltStore → My Apps → "+" → 选择 `DeepStudent-iPadOS.ipa`

**限制**: 免费 Apple ID 每 **7 天**需重签一次（AltStore 可在同一 WiFi 下自动续签）

---

## Ad-Hoc 构建：设备 UDID 注册

如果使用 **Ad-Hoc 签名**，必须先将 iPad 的 UDID 注册到 Provisioning Profile：

1. Mac 连接 iPad → Finder → 点击 iPad 名称 → 点击序列号位置切换到 **UDID**
2. 复制 UDID（40 位十六进制）
3. [Apple Developer Portal → Devices](https://developer.apple.com/account/resources/devices/list) → 添加设备
4. [Profiles](https://developer.apple.com/account/resources/profiles/list) → 重新生成包含该设备的 Provisioning Profile
5. 将新 Profile 的 Base64 编码更新到 GitHub Secrets 中

---

## 首次安装后

无论哪种方法，首次打开应用可能提示"未受信任的开发者"：

1. iPad: **设置 → 通用 → VPN 与设备管理**
2. 找到对应开发者证书
3. 点击 **信任 "<证书名称>"**
4. 确认信任
5. 返回桌面，正常打开 Deep Student

---

## GitHub Secrets 配置

CI 构建需要以下 Secrets：

| Secret | 说明 |
|--------|------|
| `APPLE_CERTIFICATE_BASE64` | p12 证书的 Base64 |
| `APPLE_CERTIFICATE_PASSWORD` | p12 证书密码 |
| `APPLE_PROVISIONING_PROFILE_BASE64` | .mobileprovision 的 Base64 |
| `APPLE_TEAM_ID` | Apple Developer Team ID |

生成方式：
```bash
# 证书
base64 -i developer_certificate.p12

# Provisioning Profile
base64 -i deepstudent.mobileprovision
```

**如果未配置这些 Secrets**，CI 将使用 Xcode 自动签名（仅限 development 模式）。
