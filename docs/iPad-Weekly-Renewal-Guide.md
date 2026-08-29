# Deep Student iPad 版每周续签指南

> 适用: 免费个人 Apple ID 签名装机 | Bundle ID: `com.lanxia.deepstudent`
> 脚本位置: `~/deep-student/renew-ipad.sh`（2026-08-27 实测通过）
> 相关文档: [iPad-Installation-Guide.md](iPad-Installation-Guide.md)

---

## 背景：为什么需要每周续签

本机使用**免费 Apple ID（个人团队 PHHX9WCMB7）**签名安装。苹果限制免费签名的描述文件**有效期只有 7 天**：

| 阶段 | 表现 |
|------|------|
| 7 天内 | 正常使用，无需任何操作，断网也能打开 |
| 第 7 天后 | App 图标还在，但点击提示「不再可用」→ 需要续签 |
| 联网验证 | 只在安装/续签那一刻需要一次网络，之后不用反复验证 |

> 付费开发者账号（99 美元/年）签名一年有效，无此烦恼。

---

## 每周续签操作（三步）

> 时机：哪天 App 点不开了，就续一次

1. **iPad 用数据线连上 Mac**，解锁屏幕
2. 打开「终端」(Terminal)，执行：
   ```bash
   ~/deep-student/renew-ipad.sh
   ```
3. 等待显示 **`✅ 续签完成`**（约 3-6 分钟），iPad 保持联网，点开 App 即用

---

## 故障排查

| 报错提示 | 原因与解决办法 |
|----------|----------------|
| 构建失败 | Xcode 账号登录过期 → 打开 Xcode → Settings → Accounts 重新登录，再跑脚本 |
| 安装失败 | iPad 未连接/未解锁/线缆问题 → 重插数据线，解锁后重试 |
| App 提示不受信任 | iPad 设置 → 通用 → VPN与设备管理 → 信任 Apple Development 证书 |
| 要求联网验证开发者 | iPad 连 Wi-Fi 等几秒自动通过 |
| 服务启动失败 | 查看日志 `/tmp/tauri-open.log`；确认网络正常 |

---

## 脚本做了什么

```
[1/4] 启动 Tauri 配置服务（tauri ios build --open 常驻进程，供 Xcode 脚本阶段取配置）
[2/4] xcodebuild -allowProvisioningUpdates → 向 Apple 续签 7 天描述文件 + 编译签名
      （Rust 已缓存，只重跑打包签名部分）
[3/4] xcrun devicectl 把签名好的 app 安装到 iPad
[4/4] 收尾清理后台进程
```

### 关键参数（换设备/账号时需同步修改脚本）

| 参数 | 值 |
|------|-----|
| iPad UDID | `00008112-0015295C0A99A01E` |
| 开发团队 ID | `PHHX9WCMB7` |
| 签名证书 SHA-1 | `8437DC9C4C1F8BDCA10A18E84137073E1C87B0B3` |
| 产物路径 | `~/Library/Developer/Xcode/DerivedData/deep-student-cwjhdczzrfkpzseprfgjvtkrxqmf/Build/Products/debug-iphoneos/Deep Student.app` |

### ⚠️ 技术坑备忘：Tauri WebSocket 配置服务

Tauri 的 Xcode 工程 "Build Rust Code" 脚本阶段必须连接一个由 **`tauri ios build --open` 常驻进程**提供的 WebSocket 配置服务（地址写在 `$TMPDIR/com.deepstudent.app-server-addr`）。直接 ⌘R 或单独跑 xcodebuild 会报：

```
failed to read CLI options: Context("failed to build WebSocket client", Io(Os { code: 61, ConnectionRefused }))
```

续签脚本第 1 步就是为此先把这个服务拉起来。手动构建时同理：先起服务，再构建。

---

## 手动补签名（脚本失效时的兜底）

```bash
APP="$HOME/Library/Developer/Xcode/DerivedData/deep-student-cwjhdczzrfkpzseprfgjvtkrxqmf/Build/Products/debug-iphoneos/Deep Student.app"
codesign --force --sign 8437DC9C4C1F8BDCA10A18E84137073E1C87B0B3 \
  --entitlements "$HOME/Library/Developer/Xcode/DerivedData/deep-student-cwjhdczzrfkpzseprfgjvtkrxqmf/Build/Intermediates.noindex/deep-student.build/debug-iphoneos/deep-student_iOS.build/Deep Student.app.xcent" \
  --timestamp=none "$APP"
xcrun devicectl device install app --device 00008112-0015295C0A99A01E "$APP"
```
