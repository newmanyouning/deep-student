# 根目录完整扫描 — 诊断报告

**日期**: 2026-05-29
**状态**: ✅ 完成

---

## 扫描范围

覆盖项目根目录下所有**非 src/ src-tauri/ 的文件夹和单个文件**，补充此前以代码为中心的扫描盲区。

---

## 一、根目录单文件 (已扫描 + 新增)

### 已在 R01/R00 覆盖

| 文件 | 行数 | 状态 |
|------|------|------|
| `package.json` | 206 | ✅ R00 |
| `eslint.config.js` | 237 | ✅ R01 |
| `vite.config.ts` | 264 | ✅ R01 |
| `tailwind.config.js` | 203 | ✅ R01 |
| `tsconfig.json` | 32 | ✅ R01 |
| `vitest.config.ts` | 47 | ✅ R01 |
| `vitest.setup.ts` | 58 | ✅ R01 |
| `playwright-ct.config.ts` | — | ✅ R01 |
| `postcss.config.js` | 7 | ✅ R01 |
| `index.html` | 37 | ✅ R01 |
| `.env.example` | 117 | ✅ R01 |
| `.gitignore` | 283 | ✅ R01 |
| `.gitattributes` | 21 | ✅ R01 |
| `.stylelintrc.json` | 36 | ✅ R01 |
| `.stylelintignore` | 6 | ✅ R01 |
| `.release-channel` | 1 | ✅ R01 |
| `release-please-config.json` | 23 | ✅ R01 |
| `.vsconfig` | 10 | ✅ R01 |

### 新增

| 文件 | 行数 | 用途 | 发现 |
|------|------|------|------|
| `ACCESSIBILITY-REVIEW.md` | 353 | 无障碍审查报告 | ⚠️ .gitignore 中应被排除的 "内部文档" 风格文件但保留在仓库 |
| `AUDIT_REPORT_v0.9.35.md` | 187 | v0.9.35 审计报告 | 同上，但 .gitignore 只排除了 `docs/*audit*.md` |
| `CHANGELOG.md` | — | 版本变更记录 | ✅ 正常 |
| `LICENSE` | — | AGPL-3.0 | ✅ 正常 |
| `fix-bluetooth-mt7921.ps1` | 95 | MT7921 蓝牙修复脚本 | ⚠️ 个人环境工具，不应在仓库 |
| `verify-bluetooth.ps1` | 59 | 蓝牙验证脚本 | ⚠️ 同上 |
| `package-lock.json` | — | npm lock | ✅ 正常 |

### 问题

- **P3**: `ACCESSIBILITY-REVIEW.md` 和 `AUDIT_REPORT_v0.9.35.md` — 审查/审计文档，与 `.gitignore` 中排除的 `docs/*audit*.md` 和 `docs/*review*.md` 同类型，但位于根目录未被排除
- **P3**: `fix-bluetooth-mt7921.ps1` + `verify-bluetooth.ps1` — 个人开发环境脚本 (`MT7921` 是特定网卡型号)，不应在公共仓库

---

## 二、隐藏目录

### .cargo/ — Rust 编译配置

**文件**: `config.toml` (~82 行)

| 配置项 | 值 | 说明 |
|--------|----|------|
| `build.jobs` | 14 | 并行编译任务数 |
| `target.x86_64-pc-windows-msvc` | `target-cpu=x86-64` | 交叉编译兼容性 |
| `net.git-fetch-with-cli` | true | 使用 git CLI |
| `env.PROTOC_INCLUDE` | vendor-protobuf-include | Protobuf include 路径 |

RAM disk 加速、sccache、zld 链接器均为**注释状态**（未启用）。

**发现**: 配置质量良好，注释清晰，无异常。

### .claude/ — Claude Code 配置

**文件**: `settings.local.json` (8 行)

仅包含 4 条 Bash 命令的 allow 权限 — 标准的 Claude Code 本地设置。

### .kiro/ — Kiro AI 工具配置

**文件**:
- `settings/mcp.json` — 4 个 MCP 服务器定义 (context7, searxng, sqlite, memory)
- `specs/release-pipeline-hardening/.config.kiro` — 发布管线加固 spec
- `specs/release-pipeline-normalization/.config.kiro` — 发布管线规范化 spec

**发现**: Kiro 是另一个 AI 编码代理工具。项目中同时使用 Claude Code (`.claude/`) 和 Kiro (`.kiro/`) 两个 AI 工具，形成**多 AI 代理并存**的开发环境。

### .roundtable/ — 圆桌协议

**文件**: `GUIDE.md` (97 行)

定义 "基于文件读写与 JSON 状态机的多代理讨论系统"。任何代理只需阅读本指南即可加入讨论。这是一个**AI 代理协作协议**。

### .skills/ — 自定义 AI 技能

**文件**:
- `spss-paper-analysis/SKILL.md` — SPSS 统计分析工作流
- `statistics-tools/SKILL.md` — 统计执行工具组

两个自定义 Skill，与项目内置的 12 个 Chat V2 skills 不同。这些是**项目级别的全局技能**（三级加载中的 "项目级"）。

### .study-ui/ — UI 研究实验室 (190 文件)

```
.study-ui/
├── package.json             — 独立 npm 项目
├── components.json          — UI 组件配置
├── eslint.config.mjs        — 独立 ESLint 配置
├── kumo.json                — Kumo UI 配置
├── index.html               — 独立入口
├── docs/plans/ (30 个文件)  — UI 设计计划 (2026-03 ~ 2026-07)
└── docs/research/tmp/ (5 文件) — 研究临时文件
```

**.study-ui 是一个独立的前端实验项目**，有自己的 package.json 和配置。30 个设计计划文件记录了大量 UI/UX 改造（侧边栏半透明、Apple UI 对齐、移动端适配等）。

**问题**:
- **P2**: `.study-ui/` 190 文件 — 独立项目/实验代码保留在主仓库中。应该独立为单独仓库或彻底合并到主项目

### .vscode/ — 已扫描 (R01)

`extensions.json` — Tauri + rust-analyzer 推荐。无 `settings.json`。

---

## 三、其他目录

### docs/ — 文档 (22 文件, ~3,900 行)

| 子目录/文件 | 用途 |
|------------|------|
| `BUILD-CONFIG.md` | 构建配置指南 |
| `CODE_STYLE.md` | 代码风格规范 |
| `README-BUILD.md` | 构建说明 |
| `THIRD_PARTY_LICENSES.md` | 第三方许可 |
| `DEEPSEEK-V4-V32-RELEASE-NOTES.md` | 版本发布说明 |
| `cloud-sync-compatibility-analysis-2026-05-23.md` | 云同步兼容性分析 |
| `翻译键缺失详细报告.md` | i18n 缺失键报告 |
| `plans/` (2 文件) | CSS 架构迁移计划 |
| `.deepseek/` (13 文件, ~1,048 行) | DeepSeek AI 代理指令 + 10 个 Skill |

**关键发现**: `docs/.deepseek/` 是**第四套** AI 代理配置：
- `.claude/` — Claude Code
- `.kiro/` — Kiro
- `.skills/` — DeepStudent 全局技能
- `docs/.deepseek/` — DeepSeek AI 指令

这意味着项目使用过 **至少 3 种不同的 AI 编码工具**，解释了 `.gitignore` 中排除了多种 AI 工具目录（`.windsurf/`, `.cursor/`, `.serena/`, `.codex/`）。

### eslint-rules/ — 1 文件

`no-native-button.js` — 自定义 ESLint 规则，强制使用 `NotionButton`，禁止原生 `<button>`。支持 JSX 检测。

### example/ — 57 个截图

全是 PNG 截图文件，用于 README 展示。与 README.md 中的图片引用一致。✅ 正常。

### mcp-servers/ — 几乎为空

`tauri-plugin-mcp/src/tools/index.ts` — 仅一个空导出文件。MCP 服务器实现可能在 `src-tauri/` 中。

### playwright/ — Playwright CT 入口

`index.html` + `index.tsx` — Playwright 组件测试的独立入口页面。✅ 正常（配合 R01 中的 `playwright-ct.config.ts`）。

### test-results/ — 空

✅ 正常 (测试产物目录)。

### public/ — 已扫描 (R01)

静态资源: Logo、PDF.js worker、Tauri SVG。

---

## 问题汇总

### 新增 P2

| ID | 问题 |
|----|------|
| P2-S1 | `.study-ui/` 190 文件独立项目，应独立仓库或合并 |
| P2-S2 | `.kiro/` + `.claude/` + `docs/.deepseek/` — **3 种 AI 工具并存**，配置可能冲突 |

### 新增 P3

| ID | 问题 |
|----|------|
| P3-S1 | `fix-bluetooth-mt7921.ps1` + `verify-bluetooth.ps1` — 个人环境脚本不应在公共仓库 |
| P3-S2 | `ACCESSIBILITY-REVIEW.md` + `AUDIT_REPORT_v0.9.35.md` — 与 .gitignore 规则不一致 |
| P3-S3 | `mcp-servers/` 几乎为空，仅一个占位文件 |
| P3-S4 | `.roundtable/` — 多代理协议目录，用途不明确 |

---

## .gitignore 与仓库清洁度评估

`.gitignore` (283 行) 排除了大量目录和文件模式，但存在不一致：
- 排除了 `docs/plans/` 但 `docs/plans/` 中 2 个 CSS 迁移计划仍在仓库
- 排除了 `docs/*audit*.md` 但根目录的 `AUDIT_REPORT_v0.9.35.md` 未被覆盖
- 排除了多种 AI 工具目录但 `.claude/`, `.kiro/`, `docs/.deepseek/` 仍在仓库
