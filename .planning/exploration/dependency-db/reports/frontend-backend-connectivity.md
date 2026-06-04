# 前端→后端数据连通性分析报告

**日期**: 2026-05-29
**数据来源**: deps.db (imports + exports + calls)

---

## 总体概览

| 指标 | 数值 |
|------|------|
| 直接调用 `invoke()` 的前端文件 | **167 个** |
| 通过 `src/api/` 层调用的文件 | **47 个** |
| 直接调用 / API层调用 比率 | **3.6 : 1** |
| 检测到的唯一 invoke 命令 | **~200 个** |

---

## 各功能域连通性

### Chat V2 — 46 文件, 29 命令 ⭐ 最复杂

```
前端: features/chat/ (46 文件直接 invoke)
  ├── TauriAdapter.ts (核心适配器, 4105行)
  ├── contextHelper.ts
  ├── MessageItem.tsx
  └── plugins/blocks/* (卡片/工具渲染)

后端命令:
  chat_v2_* (发送/取消/分支/删除/归档/变体...)
  cancel_stream, TauriAPI.readFileAsBytes

后端模块:
  src-tauri/src/chat_v2/ (102文件, 87000行)
```

**问题**: Chat V2 有 TauriAdapter 作为适配层，但 46 个文件中有许多直接 import `@tauri-apps/api/core`，没有全部通过 TauriAdapter。适配器应该是单点入口，但目前只是"另一个调用方"。

### Settings — 18 文件, 22 命令

```
前端: features/settings/components/ (18文件)
  ├── McpToolsSection.tsx (2247行)
  ├── McpEditorSection.tsx (1878行)
  ├── ShadApiEditModal.tsx (1827行)
  └── ApisTab / GeneralTab / ...

后端命令:
  get/save/delete_setting (设置读写)
  get/save_api_configurations (API 配置)
  get/save_model_assignments (模型分配)
  add/remove_ocr_engine (OCR 引擎)
  chat_v2_delete/restore_session (会话管理 ← 跨界!)

后端模块:
  src-tauri/src/cmd/ + commands.rs + llm_manager + ocr_adapters
```

**问题**: 设置模块调用了 `chat_v2_delete_session` / `chat_v2_restore_session` — **跨界调用**。会话管理应该通过 Chat V2 模块的 API 封装。

### Learning Hub — 7 文件, 8 命令

```
前端: features/learning-hub/ (7文件)
后端命令: qbank_* (题目操作) + TauriAPI.* (文件读写)
后端模块: question_bank_service + file_manager
```

**问题**: 命令数量少(8个)，但 LearningHub 是一个完整的文件浏览器。说明大量操作通过 DSTU 协议代理，invoke 只用于特殊操作。

### Anki/Template — 4 文件, 5 命令

```
前端: components/anki/cardforge/ (4引擎文件)
后端命令: chat_v2_anki_cards_result, delete/pause/resume/trigger document processing
后端模块: streaming_anki_service + enhanced_anki_service
```

### Notes — 7 文件直接 invoke（但检测到0个后端命令？）

```
前端文件: NotesContext, NotesSidebar, NotesSidebarV2, PreviewPanel...
```

**分析**: Notes 的文件操作可能通过 DSTU 协议（`dstu/api.ts`），因此直接的 invoke 调用可能是通过 DSTU 的封装层进行的。命令名称不匹配 `note_` 前缀，说明 Notes 的 CRUD 实际上走的是 VFS/DSTU 通道。

### Stores — 2 文件, 4 命令

```
questionBankStore (1630行): qbank_delete/set_sync/update_sync
reviewPlanStore: review_plan_delete
```

**确认**: 题库 Store 是唯一绕过 API 层的 Store。

---

## 后端模块被调用情况

| 后端模块 | 被调用量 | 调用方 | 架构评价 |
|---------|---------|--------|---------|
| Chat V2 | ⭐⭐⭐⭐⭐ | chat, settings, notes, anki, sidebar | 最复杂的调用图 |
| VFS | ⭐⭐⭐⭐ | api, chat, dstu | 通过 DSTU 协议代理 |
| Settings | ⭐⭐⭐⭐ | 几乎所有模块 | 全局基础设施 |
| LLM/Model | ⭐⭐ | settings | 仅设置页调用 |
| Anki | ⭐⭐ | comp/anki, services | 独立功能 |
| Question Bank | ⭐⭐ | learning-hub, hooks, stores | 通过 Store 和 Hook |
| **Notes** | ⭐ (0检测) | — | 实际走 DSTU/VFS 通道 |
| **Exam Sheet** | ⭐ (0检测) | — | 可能已废弃或走 VFS |
| Translation | ⭐ | translation | 独立 |
| Essay | ⭐ | essay-grading | 独立 |
| Memory | ⭐ | api | 通过 API 层 |
| MCP | ⭐⭐ | mcp, settings | 独立协议 |

---

## 架构问题

### P1: 命令命名混乱

```
chat_v2_*         — 前缀统一 ✅
qbank_*           — 前缀统一 ✅
dstu_*            — 前缀统一 ✅
memory_*          — 前缀统一 ✅

get_setting / save_setting / delete_setting  — 无前缀 ⚠️
cancel_stream                                  — 无模块前缀 ⚠️
generate_anki_cards_from_document              — 冗长 ⚠️
TauriAPI.readFileAsBytes                       — 通过 TauriAPI 包装 ⚠️
```

**问题**: 大约 60% 的命令有统一前缀，40% 没有。`cancel_stream` 无法判断属于哪个模块。

### P2: 跨界调用

| 调用方 | 调用了 | 问题 |
|--------|--------|------|
| Settings | `chat_v2_delete_session` | 设置页直接操作 Chat 数据 |
| Settings | `chat_v2_restore_session` | 同上 |
| Chat | `vfs_*` / `dstu_*` | Chat 直接操作 VFS（应通过 DSTU 适配器） |

### P3: API 层未充分利用

```
直接 invoke: 167 文件
通过 API 层:  47 文件
────────────────────
API 层覆盖率: 22%
```

API 层 `src/api/` 有 10 个封装良好的模块（vfsRagApi, vfsUnifiedIndexApi, memoryApi...），但仅 22% 的 invoke 调用方通过它。

### P4: 数据流路径不统一

```
文件上传路径:
  Chat → TauriAdapter → invoke('chat_v2_send_message')
  LearningHub → invoke('TauriAPI.readFileAsBytes') ← 不同的入口!
  Settings → invoke('get_setting') ← 直接读写设置

笔记 CRUD:
  Notes → DSTU → invoke('dstu_*')
  但 NotesSidebar 也直接 invoke ← 绕过 DSTU
```

---

## 改进方案

### 方案 A: 分层 API 网关 (推荐)

建立清晰的三层调用架构：

```
┌──────────────────────────────────────────┐
│  前端 Feature 层                          │
│  chat / learning-hub / notes / settings   │
└────────────┬─────────────────────────────┘
             │ 只调用
┌────────────▼─────────────────────────────┐
│  API 网关层 (src/api/)                    │
│  chatApi / vfsApi / settingsApi / ...    │
│  每个网关封装一个后端模块的所有命令        │
└────────────┬─────────────────────────────┘
             │ invoke
┌────────────▼─────────────────────────────┐
│  后端 Tauri 命令                          │
│  src-tauri/src/cmd/ + commands.rs         │
└──────────────────────────────────────────┘
```

**实施步骤**:
1. 为每个后端模块创建对应的 API 网关（chatApi, qbankApi, settingsApi, notesApi...）
2. 将现有的直接 `invoke()` 调用迁移到 API 网关
3. API 网关负责: 类型转换(snake_case↔camelCase)、错误处理、重试逻辑

### 方案 B: 命令命名规范化

统一所有 Tauri 命令为 `模块_操作_目标` 格式:

```
现有 → 建议
cancel_stream → chat_v2_cancel_stream
get_setting → settings_get
TauriAPI.readFileAsBytes → vfs_read_file_bytes
generate_anki_cards_from_document → anki_generate_cards
```

### 方案 C: 消除跨界调用

- Settings 中删除 `chat_v2_delete_session`，改为调用 Chat V2 的 API 封装
- Chat 中删除直接 VFS 操作，改为通过 `src/dstu/api.ts` 封装
- 所有 Store 禁止直接 import `@tauri-apps/api/core`

---

## 建议优先级

| 优先级 | 措施 | 影响范围 |
|--------|------|---------|
| P0 | 补全 API 网关层 (chatApi, settingsApi, notesApi) | 167文件 → 迁移 |
| P1 | 规范化命令命名 | 后端 ~200 命令 |
| P1 | 消除 Store 的 invoke 调用 (2个Store) | questionBankStore, reviewPlanStore |
| P2 | 消除 Settings→Chat 跨界调用 | Settings 组件 |
| P2 | 统一文件操作入口 (全部走 DSTU) | LearningHub + Chat |
