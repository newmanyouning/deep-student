# Batch 5: TypeScript API 层 — 重新扫描报告

> 扫描时间: 2026-05-30 15:55 CST | 474+80+81+... files | 状态: ✅ 完成

## 5.1 Feature 模块结构

| Feature | 文件数 | 结构 |
|---------|--------|------|
| `chat/` | **474** | 最大模块: adapters/hooks/plugins/skills/pages/workspace/debug/dev |
| `settings/` | 81 | 组件驱动 (47 .tsx) |
| `mindmap/` | 97 | 独立架构 |
| `learning-hub/` | 80 | 组件 + hooks |
| `notes/` | 59 | Milkdown 编辑器 |
| `todo/` | 9 | 本地 api.ts (22 invoke) |
| `pomodoro/` | 7 | 本地 api.ts (5 invoke) |
| `sandbox/` | 8 | 实验区 |
| 其他 6 个 | 1-5 | 薄入口 |

## 5.2 API 层分析

### 三层共存 (核心问题)

| 层 | 位置 | 模式 |
|----|------|------|
| **新 API** | `src/api/` (10 files) | `{domain}Api = { method() }` 命名空间 |
| **旧 API** | `src/utils/tauriApi.ts` + 6 子模块 | Barrel re-export + `TauriAPI.method()` |
| **本地 API** | Feature 内的 `api.ts` | Feature 独立封装 |

### API 命名不一致

| 命名空间 | 示例 | 风格 |
|----------|------|------|
| `LlmUsageApi` | PascalCase | ✅ |
| `DataGovernanceApi` | PascalCase | ✅ |
| `attachmentConfigApi` | camelCase | ❌ 不一致 |
| `vfsFileApi` | camelCase | ❌ 不一致 |
| `memoryApi.ts` | 扁平函数 | ❌ 无容器 |

## 5.3 绕过统计 (P1-07 诊断)

**200+ `invoke()` 调用分布在 82+ 文件中直接绕过 API 层。**

| 模块 | 绕过数 | 严重度 |
|------|--------|--------|
| settings | ~80 | **HIGH** |
| chat (非 TauriAdapter) | ~50 | **HIGH** |
| hooks/ | ~15 | MEDIUM |
| services/ | ~20 | MEDIUM |
| learning-hub | ~10 | LOW |

## 5.4 前端 API 冲突

| ID | 严重度 | 问题 | 位置 |
|----|--------|------|------|
| N5-01 | **HIGH** | 200+ 处直接 invoke 绕过 API 层 | 82 文件 |
| N5-02 | **HIGH** | 3 种 API 层模式并存 | `api/` vs `tauriApi.ts` vs 本地 api.ts |
| N5-03 | MEDIUM | 命名风格不统一 | `src/api/` 内混用 PascalCase/camelCase |

---

*Batch 5 完成。文件: 700+ | 冲突: 3 (N5-01..N5-03)*
