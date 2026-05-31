# Batch 4: TypeScript 前端基础设施 — 重新扫描报告

> 扫描时间: 2026-05-30 15:50 CST | 16+11+75+40 = 142 文件 | 状态: ✅ 完成

## 4.1 类型文件 (`src/types/` — 16 文件)

### 核心类型清单

| TS 接口/类型 | Rust 对应结构体 | 状态 |
|-------------|----------------|------|
| `ChatMessage` | `models::ChatMessage` | ✅ 一致 |
| `RagSourceInfo` | `models::RagSourceInfo` | ✅ 一致 |
| `DocumentAttachment` | `models::DocumentAttachment` | ✅ 一致 |
| `ApiConfig` | `llm_manager::ApiConfig` | ✅ 一致 |
| `VendorConfig` | `llm_manager::VendorConfig` | ✅ 一致 |
| `AnkiCard` | `models::AnkiCard` | ✅ 一致 |
| `AppError` | `models::AppError` | ⚠️ 字段名不一致 (TS: `type`, Rust: `error_type` → serde rename 已修复) |
| `MistakeItem` | `models::MistakeItem` | ⚠️ 已 @deprecated, 仍存在 9 处引用 |

### 文件状态

| 文件 | 行数 | 状态 | 说明 |
|------|------|------|------|
| `types/index.ts` | ~900 | 活跃 | 核心类型 God File (API/Chat/Anki/Template 混合) |
| `types/api.ts` | — | ⚠️ 存在 | 纯 re-export (REF-001 曾删除, git revert 恢复) |
| `types/ui.ts` | — | ⚠️ 存在 | 同上 |
| `types/hooks.ts` | — | ⚠️ 存在 | 同上 |
| `types/dataGovernance.ts` | — | 活跃 | 数据治理类型 |
| `types/enhanced-field-types.ts` | — | 活跃 | 增强字段类型 |
| `types/dragDrop.ts` | — | 活跃 | 拖拽类型 |

### 命名冲突

| ID | 文件 | 问题 | 建议 |
|----|------|------|------|
| N4-01 | `types/api.ts` `types/ui.ts` `types/hooks.ts` | 纯 re-export 无意义层 (P2-11 已修复后 revert 恢复) | 重新删除 |
| N4-02 | `types/index.ts` | God File 900+ 行 | 按域拆分 |

## 4.2 Stores (`src/stores/` — 14 文件)

| Store 文件 | 行数 | 模式 | 持久化 |
|-----------|------|------|--------|
| `questionBankStore.ts` | 1629 | God Store | 无 |
| `ankiQueueStore.ts` | 248→110 (REF-007) | Zustand | 自定义 TauriAPI |
| `reviewPlanStore.ts` | 746 | Zustand | 无 |
| `researchStore.ts` | 506 | Zustand | 无 |
| `uiStore.ts` | 22 | Zustand + persist | Zustand persist ✅ |
| `anki/types.ts` | 419 | 类型定义 | — |
| `anki/useAnkiUIStore.ts` | 485 | Zustand | 无 |
| 其余 7 个 | <200 | Zustand | 无 |

### 冲突发现

| ID | 位置 | 问题 | 状态 |
|----|------|------|------|
| N4-03 | `store/` vs `stores/` | 双目录并存 (P2-07) | REF-006 修复后 revert 恢复 |
| N4-04 | `ankiQueueStore.ts` | 自定义持久化 vs `uiStore` Zustand persist | P2-08, REF-007 修复后 revert 恢复 |

## 4.3 工具函数 (`src/utils/` — 75 文件)

| 类别 | 文件数 | 关键文件 |
|------|--------|----------|
| API 封装 | 12 | `chatApi.ts`, `configApi.ts`, `systemApi.ts` |
| 数据格式 | 8 | `formatUtils.ts`, `common.ts`, `cn.ts` |
| 业务工具 | 30 | `ankiSourceBuilder.ts`, `graphApi.ts` |
| 测试/调试 | 10 | `testApi.ts`, `debugLogger.ts` |
| 新文件 | 1 | `tauriPersistStorage.ts` (REF-007, 新增) |

### 冲突发现

| ID | 位置 | 问题 |
|----|------|------|
| N4-05 | `cn.ts` vs `lib/utils.ts` | cn() 双实现 (P2-10) — REF-004 修复后 revert 恢复 |
| N4-06 | `shared.ts:121` | 仍导入已废弃的 `MistakeItem` |

## 4.4 依赖数据库更新

| 表 | 新增 | 说明 |
|----|------|------|
| `module_definitions` | 16 types + 14 stores + 75 utils = 105 | TypeScript 前端模块 |
| `dependency_edges` | — | 待 Batch 5 API 层补充 |
| `conflict_tracker` | 6 (N4-01..N4-06) | 已记录 |

---

*Batch 4 完成。文件: 142 | 模块: 105 | 冲突: 6*
