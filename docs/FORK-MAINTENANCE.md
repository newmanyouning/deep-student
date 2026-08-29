# 独立维护版：断开点位与重连指南

> 建立时间：2026-08-29 | 适用版本：0.9.40-fork.1 起
> 原则：**暂时断开上游更新通道，但保留全部配置与代码路径，重连 = 改一个开关**

---

## 1. 版本号方案

全链路统一使用 semver 前缀式（构建、显示、发布 tag 一致，不做转换）：

```
<上游版本>-fork.<修订号>
  0.9.40-fork.1   ← 当前
  0.9.40-fork.2   ← 同基线修订
  0.9.41-fork.1   ← 合并上游 0.9.41 后重新从 .1 计数
```

**升级修订号需要同步改 4 处**（`scripts/generate-version.mjs` 会把 package.json 的值
同步到 `src/version.ts` 和 Android `tauri.properties`）：

| 文件 | 字段 |
|------|------|
| `package.json` | `"version"` |
| `src-tauri/Cargo.toml` | `version`（cargo 会自动同步 Cargo.lock） |
| `src-tauri/tauri.conf.json` | `version` |
| `package.json` 改完后运行 | `npm run version:generate` |

发布 tag：`v0.9.40-fork.2`（与版本号一致）。

版本比较逻辑（`src/hooks/useAppUpdater.ts` 的 `isNewerVersion`）已支持 `-fork.N`
段：fork.1 → fork.2 判定为更新；同版本号下上游正式版 > fork 前缀版。

---

## 2. 断开点位清单

### 点位 ① 桌面端 Tauri updater（`src-tauri/tauri.conf.json`）

**现状（已断开上游）**：
```json
"updater": {
  "endpoints": [
    "https://github.com/newmanyouning/deep-student/releases/latest/download/latest.json"
  ],
  "pubkey": "<本仓库自己的 minisign 公钥>"
}
```

**上游原配置（重连时替换回去）**：
```json
"endpoints": [
  "https://download.deepstudent.cn/releases/latest.json",
  "https://github.com/helixnow/deep-student/releases/latest/download/latest.json"
],
"pubkey": "dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk6IDkxREY0RkI0Qjk5NEQ5MjQKUldRazJaUzV0RS9ma2JqbmNwNGNKT1FGc1BWcjRXa2J6K0psVDZFT05YUkc3OTIxYXNPR3FMcEsK"
```

> ⚠️ 原 pubkey 与上游私钥配对 —— 断开前上游发的更新包能通过本应用签名校验，
> 这是本次隔离的最关键点位。

### 点位 ② 前端检查更新开关（`src/hooks/useAppUpdater.ts`）

```ts
const UPSTREAM_UPDATES_ENABLED = false;  // ← 断开开关
```

- `false`：启动/手动"检查更新"不发起任何网络请求，直接视为最新版
- 上游 URL 常量（`R2_LATEST_URL`、`GH_LATEST_URL`）原样保留在开关下方，**重连 = 改回 `true`**
- 本仓库的更新源常量：`FORK_GH_LATEST_URL` / `FORK_RELEASES_URL` / `FORK_REPO_API_URL`

**重连上游**：`UPSTREAM_UPDATES_ENABLED = true` + 恢复点位 ① 的原配置。
**仅用本仓库更新**：开关保持 `false`（当前状态，前端不查）；
或改为 `true` 且保留本仓库 endpoint（走本仓库 latest.json）。

### 点位 ③ 界面链接

| 文件 | 位置 | 现状 |
|------|------|------|
| `src/features/settings/components/AboutTab.tsx` | 更新弹窗"GitHub 下载" | 指向本仓库 Releases |
| 同上 | "项目链接"分组 | 本仓库 GitHub/Issues + 原项目官网 + 上游仓库（归属） |
| `src/features/settings/components/UpdateNotificationDialog.tsx` | "GitHub 下载" | 指向本仓库 Releases |

### 点位 ④ CI 工作流

已移入 `.github/workflows/disabled/`（GitHub 不执行子目录）：

| 文件 | 用途 | 重连方式 |
|------|------|---------|
| `cla.yml` | 上游 CLA 签名机器人 | `git mv` 回 workflows/ 根目录 |
| `upload-r2.yml` | 手动同步 Release 到上游 R2 CDN (download.deepstudent.cn) | 同上 + 配置 `CLOUDFLARE_*` secrets |

保留的 `release.yml` / `rebuild-release.yml` / `rebuild-android.yml` / `purge-cache.yml`
中仍包含 R2/CDN 相关步骤（引用上游 `download.deepstudent.cn`）——
这些仅在 CI 里执行、需 `CLOUDFLARE_*` secrets，本仓库未配置时该步骤跳过/失败，
不影响 GitHub Release 本身，也不影响应用运行时。

### 点位 ⑤ HTTP 能力白名单（保留未动）

`src-tauri/capabilities/default.json`、`mobile.json` 中的
`https://www.deepstudent.cn/*` 仅为放行规则，**不产生任何网络请求**，保留以便重连。

---

## 3. 签名方案（全部免费）

| 平台 | 方案 | 位置/说明 |
|------|------|-----------|
| iOS / iPad | Apple 免费开发者签名（7 天有效） | `renew-ipad.sh` 每周续签；文档 `docs/iPad-Weekly-Renewal-Guide.md`、`docs/iPad-Installation-Guide.md`（Bundle ID: `com.lanxia.deepstudent`） |
| Android | 开发密钥库（自签名） | `scripts/build_android.sh` 模式 2/3 |
| 桌面更新器 | minisign 密钥对（免费生成） | 私钥：`~/.tauri/deep-student-fork.key`（空密码，**勿提交仓库**），公钥已写入 tauri.conf.json |

**私钥备份提醒**：`C:\Users\1\.tauri\deep-student-fork.key` 丢失后已发更新无法签名。
如需 CI 签更新包：GitHub 仓库 Settings → Secrets 添加
`TAURI_SIGNING_PRIVATE_KEY`（私钥全文）与 `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`（空）。

---

## 4. 发布流程（独立版）

1. 改 4 处版本号（见 §1）→ `npm run version:generate`
2. 提交 + 打 tag：`git tag v0.9.40-fork.2 && git push origin v0.9.40-fork.2`
3. GitHub Actions：`rebuild-release.yml`（workflow_dispatch 填 tag）或推 tag 触发 `release.yml`
4. Release 说明模板（三段式：本版变化 / 归属 / 签名隔离提醒）参考 CHANGELOG.md 的
   `0.9.40-fork.1` 条目
5. 如需启用应用内更新：在 Release assets 附 `latest.json`，格式：
   ```json
   {
     "version": "0.9.40-fork.2",
     "pub_date": "2026-09-05T00:00:00Z",
     "platforms": {
       "windows-x86_64": { "signature": "<.sig 文件内容>", "url": "https://github.com/newmanyouning/deep-student/releases/download/v0.9.40-fork.2/DeepStudent_x64-setup.exe" }
     }
   }
   ```
   `signature` 来自构建时生成的 `.sig` 文件（`createUpdaterArtifacts: true` 时产出）。

---

## 5. 与上游同步（合并时）

```bash
git fetch upstream                      # upstream remote 保留未删
git checkout -b sync/upstream-0.9.41
git merge upstream/main                 # 或 cherry-pick 需要的提交
# 解决冲突后：版本号改为 0.9.41-fork.1，更新 CHANGELOG
```

注意：
- merge 后检查本文件 §2 的各点位是否被上游改动覆盖（重点：tauri.conf.json updater、
  useAppUpdater.ts 开关、workflows/disabled）
- fork 关系保留中，向上游提 PR：正常从 main 拉分支 push 到 origin 后开 PR（默认基准上游）
