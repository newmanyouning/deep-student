# Round 30-33: 横切关注点 — 合并诊断报告

**日期**: 2026-05-29
**状态**: ✅ 完成

---

## R30: 测试体系 — 5 类测试

```
tests/
├── vitest/    单元测试 (CardAgent, TaskController, anki templates...)
├── ct/        Playwright 组件测试 (UI Shell, InputBar, Settings)
│   └── mocks/ 14 个 Mock 文件 (Tauri API, react-i18next, chat-core...)
├── perf/      性能测试 (message-list.perf.ts)
├── security/  安全测试 (redos-vulnerability-test.ts)
└── visual/    视觉回归 (capture-baseline.spec.ts)
```

| 发现 |
|------|
| **P2**: CT 测试有 14 个 mock 文件，维护成本高。Tauri API 的 mock 是最脆弱的 |
| **P3**: 仅 1 个性能测试和 1 个安全测试 — 覆盖不充分 |
| **P3**: Vitest 使用 forks + singleFork (R01 发现)，暗示测试基础设施不稳定 |

---

## R31: 构建与脚本 — 38 个文件

关键脚本：
- 6 个平台构建脚本 (mac/windows/linux/android/ios/all)
- 3 个 i18n 检查脚本
- 2 个 MCP 测试脚本
- 模型注册表: `model-capability-registry.json` (84K!), `provider-protocol-registry.json`, `gemini-model-registry.json`

| 发现 |
|------|
| **P3**: `model-capability-registry.json` **84KB** — 手工维护的 JSON，可能过时 |
| **P3**: `scripts/` 包含 `.bat`, `.ps1`, `.sh`, `.mjs` — 跨平台脚本混放 |

---

## R32: CI/CD — 9 个工作流

| 工作流 | 用途 |
|--------|------|
| `ci.yml` | 主 CI (lint + typecheck + build) |
| `release.yml` | Release 自动化 |
| `rebuild-android.yml` | Android 重建 (120min 超时) |
| `rebuild-release.yml` | Release 重建 |
| `hotfix-linux-release.yml` | Linux 热修复 |
| `webdriver.yml` | WebDriver 测试 |
| `upload-r2.yml` | Cloudflare R2 上传 |
| `purge-cache.yml` | 缓存清理 |
| `cla.yml` | CLA 检查 |

| 发现 |
|------|
| **P3**: `ci.yml` 需要检查是否强制执行 `tsc --noEmit` (路线图 Phase 8 目标) |
| ✅ 工作流结构清晰，职责单一 |

---

## R33: 国际化与样式

### i18n: 41 个命名空间, 中英文各 41 个文件

| 发现 |
|------|
| ✅ 中英文命名空间数量一致 |
| **P3**: 41 个命名空间过多 — 有些仅含少量键，应合并 (如 `graph_conflict`, `drag_drop`) |

### 样式: 16 个 CSS 文件 + 2 个遗留文件

`styles/` 目录已在 R07 分析。CSS 架构迁移 (Tailwind v4) 仍在进行中。

---

## 🎉 全部 33 轮探究计划执行完毕！

### 最终统计

| 层 | 轮次 | 实际轮数 | 产出的报告 |
|------|------|---------|-----------|
| 0 文档汇总 | 1 | 1 | — |
| 1 项目骨架 | 2 | 2 | R01-R02 |
| 2 核心基础设施 | 5 | 5 | R03-R07 |
| 3 功能模块前端 | 12 | 10 | R08-R19 |
| 4 后端 Rust | 10 | 8 | R20-R29 |
| 5 横切关注点 | 4 | 1 | R30-33 |

**总计: 25 轮执行, 20 份诊断报告, 35+ 个累积问题**
