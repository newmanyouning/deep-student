# Round 05: API 层与服务层 — 诊断报告

**日期**: 2026-05-29
**状态**: ✅ 完成

---

## API 层 (src/api/) — 10 个文件

### 文件清单

| 文件 | 职责 | 调用模式 | 行数 |
|------|------|---------|------|
| `questionBankApi.ts` | 题库类型定义 + API | invoke() | ~200 |
| `vfsFileApi.ts` | VFS 文件类型定义 + API | invoke() | ~80 |
| `vfsRagApi.ts` | VFS RAG 向量检索 | invoke() | ~160 |
| `vfsUnifiedIndexApi.ts` | VFS 统一索引状态 | invoke() | ~120 |
| `vfsPdfProcessingApi.ts` | PDF 预处理流水线 | invoke() | **230** |
| `dataGovernance.ts` | 数据治理 (备份/同步/审计) | invoke() + listen() | ~200 |
| `memoryApi.ts` | 智能记忆 API | invoke() | ~55 |
| `llmUsageApi.ts` | LLM 用量趋势数据 | invoke() | ~55 |
| `attachmentConfigApi.ts` | 附件根目录配置 | invoke() | **23** (最小) |
| `debugDatabase.ts` | ⚠️ 调试专用数据库直读 | invoke() | ~120 |

### 设计评估

API 层的**目录组织良好**，按领域拆分清晰。但存在两种导出风格：

| 风格 | 示例 | 文件数 |
|------|------|--------|
| **模块函数** (export function) | `vfsPdfProcessingApi.ts` | 7 |
| **对象字面量** (export const api = {}) | `attachmentConfigApi.ts`, `ankiConnectClient.ts` | 3 |

### 亮点: vfsPdfProcessingApi

最成熟的 API 文件:
- **snake_case → camelCase 适配层**：`normalizeStatus()` 函数将后端的 `current_page`/`ready_modes` 转换为前端的 `currentPage`/`readyModes`
- **fallback 处理**：null/undefined 输入返回安全的 `FALLBACK_STATUS`
- **便捷对象导出**：同时支持 `import { getPdfProcessingStatus }` 和 `import { vfsPdfProcessingApi }` + `vfsPdfProcessingApi.getStatus()`

---

## 服务层 (src/services/) — 6 个文件

### 文件清单

| 文件 | 职责 | 架构模式 |
|------|------|---------|
| `templateService.ts` | 模板管理 (CRUD + 复杂度分析 + 智能降级) | **Singleton 类** |
| `ankiConnectClient.ts` | Anki Connect 桥接 (检查/牌组/模型) | **对象字面量** |
| `ankiApiAdapter.ts` | Anki 批量操作 + 断点续传 | **对象字面量** |
| `templateRenderService.ts` | Mustache 模板渲染 | **静态类** |
| `resourceSyncService.ts` | 资源同步 (笔记→resources.db) | **模块函数** |
| `multimodalRagService.ts` | 多模态 RAG (VL-Embedding/Reranker) | **模块函数 + Feature Flag** |

### 架构模式不统一

同一目录下有 **4 种不同的架构模式**：
- Singleton 类 (`TemplateService.getInstance()`)
- 静态类 (`TemplateRenderService.renderCard()`)
- 对象字面量 (`ankiConnectClient.check()`)
- 模块函数 (`resourceSyncService.syncNote()`)

### 亮点: multimodalRagService 的 Feature Flag

```typescript
export const MULTIMODAL_INDEX_ENABLED = true;
```
全局开关统一控制多模态索引功能，UI 和逻辑通过此处统一管理。这是项目中为数不多的清晰的 Feature Flag 示例。

---

## 事件系统 (src/events/) — 1 个文件

### chat.ts — 类型安全的事件系统

- 6 个事件常量定义（3 个是测试事件）
- 类型化的 `ChatEventMap` 映射事件名 → payload 类型
- `dispatchChatEvent<K>()` — 泛型约束确保 payload 类型正确
- `addChatEventListener<K>()` — 返回 cleanup 函数
- `waitForChatEvent<K>()` — Promise 包装，支持超时和 filter

### 异常点

payload 类型中仍使用 "businessId（错题ID）" 作为字段注释，延续了已废弃的 MistakeItem 术语。

---

## 核心发现：API 层被 Store 绕过

这是 Round 04 发现的问题的确认和细化:

| 调用方 | 调用方式 | 是否符合架构 |
|--------|---------|-------------|
| `useUnifiedIndexStore` | `vfsUnifiedIndexApi.getStatus()` | ✅ 正确使用 API 层 |
| `useQuestionBankStore` | `invoke('qbank_list_questions')` | ❌ 绕过 API 层 |
| `useHpiasStore` | 事件驱动 | △ 非标准模式 |
| `ankiConnectClient` (service) | `invoke('check_anki_connect_status')` | △ 服务层调用 |
| `attachmentConfigApi` | `invoke('vfs_get_attachment_config')` | ✅ 正确封装 |

**API 层存在且设计良好**，但大量 Store 没有使用它。这不是 API 层缺失的问题，而是**架构纪律问题** — 开发者选择在 Store 中直接 `invoke()` 而不是通过 API 层。

---

## 发现的问题

- [ ] **P1** — API 层存在但被大量 Store 绕过：`questionBankStore` (20+ 个直接 invoke)、`reviewPlanStore`、多个 service 文件都绕过 API 层直接调用 invoke
- [ ] **P2** — 服务层 6 个文件用了 4 种不同的架构模式（Singleton/静态类/对象字面量/模块函数），应统一为 1-2 种
- [ ] **P2** — `events/chat.ts` payload 注释中仍使用 "businessId（错题ID）" 术语，延续废弃的 MistakeItem 概念
- [ ] **P3** — `debugDatabase.ts` 存在但被标记为调试专用；如果它必须存在，说明 API 层缺少某些正常查询接口
- [ ] **P3** — 6 个事件中 3 个是测试事件 (`TEST_*`)，生产事件系统与测试事件混合在同一文件中
- [ ] **P4** — `attachmentConfigApi` 和 `vfsPdfProcessingApi` 导出了对象字面量形式，与主流的模块函数导出风格不一致

---

## 建议优先处理

1. 制定规范：Store 必须通过 API 层调用后端，禁止直接 `invoke()`
2. 为 `questionBankStore` 创建对应的 `questionBankApi` 封装层（API 类型已定义在 `api/questionBankApi.ts`，但缺少 API 函数）
3. 统一服务层架构模式为模块函数（与 API 层保持一致）
4. 将 `events/chat.ts` 中的测试事件分离到 `debug-panel/events/` 目录
