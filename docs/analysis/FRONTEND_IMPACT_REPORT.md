# Frontend Impact Report

> Generated: 2026-06-01
> Baseline: `d2f44248` | Head: `aca0aad9` | Refactoring: `36e55f23`
> Change database: `docs/analysis/CHANGE_INTERFACE_DB.json`

## Summary

- **Files scanned**: All `.ts` and `.tsx` files under `src/`
- **Total changed commands in DB**: 8 removed, 54 renamed, 5 added, ~27 signature-changed
- **Critical frontend breakage**: 0
- **Missing new command bindings**: 5
- **Potentially broken code paths**: 1 (deprecated fallback)
- **Cosmetic/stale references (comments/docs)**: ~10 files with non-functional references

---

## 1. Removed Commands -- No Impact

All 8 removed `resource_*` commands have no remaining `invoke()` calls in the frontend.

| Removed Command | Frontend Status |
|---|---|
| `resource_create_or_reuse` | Replaced by `vfs_create_or_reuse` in `src/features/chat/resources/api.ts:148` |
| `resource_get` | Replaced by `vfs_get_resource` in `src/features/chat/resources/api.ts:185,207` |
| `resource_get_latest` | Replaced by `vfs_get_resource` (same call) in `src/features/chat/resources/api.ts:208` |
| `resource_exists` | Replaced by `vfs_resource_exists` in `src/features/chat/resources/api.ts:229` |
| `resource_increment_ref` | Replaced by `vfs_increment_ref` in `src/features/chat/resources/api.ts:242` |
| `resource_decrement_ref` | Replaced by `vfs_decrement_ref` in `src/features/chat/resources/api.ts:255` |
| `resource_get_versions_by_source` | Stubbed to return empty in `src/features/chat/resources/api.ts:269-272` |
| `resource_get_content_from_vfs` | Not referenced anywhere |

**Verdict: No action needed.** The frontend already migrated to all VFS equivalents.

---

## 2. Renamed Commands -- No Functional Breakage

### 2.1 Workspace (17 commands: `workspace_*` -> `chat_v2_workspace_*`)

All `invoke()` calls use the **new names**. The API wrapper at `src/features/chat/workspace/api.ts` already calls:
- `chat_v2_workspace_create` (line 116)
- `chat_v2_workspace_get` (line 145)
- `chat_v2_workspace_close` (line 155)
- `chat_v2_workspace_delete` (line 165)
- `chat_v2_workspace_create_agent` (line 177)
- `chat_v2_workspace_list_agents` (line 186)
- `chat_v2_workspace_send_message` (line 199)
- `chat_v2_workspace_list_messages` (line 213)
- `chat_v2_workspace_set_context` (line 229)
- `chat_v2_workspace_get_context` (line 245)
- `chat_v2_workspace_list_documents` (line 259)
- `chat_v2_workspace_get_document` (line 273)
- `chat_v2_workspace_list_all` (line 284)
- `chat_v2_workspace_run_agent` (line 434)
- `chat_v2_workspace_cancel_agent` (line 463)
- `chat_v2_workspace_manual_wake` (line 590)
- `chat_v2_workspace_cancel_sleep` (line 613)

Old names appear only in non-functional contexts:
- **MCP tool display names** in `src/mcp/builtinMcpServer.ts` (lines 1247-1400) -- these are user-facing MCP tool names with `BUILTIN_NAMESPACE` prefix. They are NOT Tauri invoke calls and need to keep their existing names for MCP protocol compatibility.
- **Skill documentation** in `src/features/chat/skills/builtin-tools/workspace-tools.ts` (lines 29-68) -- user-facing documentation prompting the LLM to call MCP tools by name. These are MCP tool names, not Tauri commands.
- **Console.log messages** in `src/features/chat/plugins/events/toolCall.ts` (lines 320-378) -- display strings only.
- **Test labels** in `src/features/chat/debug/workspaceOrchestrationTestPlugin.ts` (lines 802-865) -- event/log labels only.
- **JSDoc** in `src/features/chat/skills/builtin-tools/subagent-worker.ts` (line 127) -- documentation only.

**Verdict: No action needed.** MCP tool names are intentionally distinct from Tauri command names.

### 2.2 Anki Connect (10 commands)

All `invoke()` calls use the **new names**:
- `anki_connect_batch_export_cards` -- `src/services/ankiApiAdapter.ts:64`
- `anki_connect_export_apkg_with_template` -- `src/services/ankiApiAdapter.ts:68`
- `anki_connect_save_cards` -- `src/services/ankiApiAdapter.ts:103`
- `anki_connect_add_cards` -- `src/features/chat/anki/index.tsx:230`
- `anki_connect_export_multi_apkg` -- `src/features/chat/anki/index.tsx:168`

Old names appear only in **JSDoc comments**:
- `src/features/chat/anki/index.tsx:190,228` (documentation)
- `src/components/anki/cardforge/engines/CardAgent.ts:549` (comment)

**Verdict: No action needed.**

### 2.3 Enhanced Anki (23 commands)

All `invoke()` calls use the **new names**:
- `enhanced_anki_pause_document_processing` -- `src/components/anki/TaskDashboardPage.tsx:402`, `src/components/anki/cardforge/engines/CardAgent.ts:449`, `src/components/anki/cardforge/engines/TaskController.ts:100`
- `enhanced_anki_resume_document_processing` -- `src/components/anki/TaskDashboardPage.tsx:405`, `src/components/anki/cardforge/engines/CardAgent.ts:456`, `src/components/anki/cardforge/engines/TaskController.ts:156`
- `enhanced_anki_trigger_task_processing` -- `src/components/anki/TaskDashboardPage.tsx:421`, `src/components/anki/cardforge/engines/CardAgent.ts:471`
- `enhanced_anki_delete_document_session` -- `src/components/anki/TaskDashboardPage.tsx:432`, `src/services/ankiApiAdapter.ts:262`, `src/components/anki/cardforge/engines/CardAgent.ts:480`
- `enhanced_anki_delete_card` -- `src/services/ankiApiAdapter.ts:140`
- `enhanced_anki_start_document_processing` -- `src/services/ankiApiAdapter.ts:236`

Old names appear only in **JSDoc and inline comments**:
- `src/components/anki/cardforge/engines/TaskController.ts:23-27,74,130,186,250,305,397` (JSDoc)
- `src/components/anki/cardforge/engines/CardAgent.ts:394` (comment)
- `src/services/ankiApiAdapter.ts:135` (comment)
- `src/debug-panel/plugins/ChatAnkiWorkflowDebugPlugin.ts:43` (comment)

**Verdict: No action needed.**

### 2.4 Todo (19 commands: `todo_*` -> `vfs_todo_*`)

All `invoke()` calls in `src/features/todo/api.ts` (lines 21-105) already use **new names**:
- `vfs_todo_create_list`, `vfs_todo_get_list`, `vfs_todo_list_lists`, etc.

**Verdict: No action needed.**

### 2.5 Pomodoro (5 commands: `pomodoro_*` -> `vfs_pomodoro_*`)

All `invoke()` calls in `src/features/pomodoro/api.ts` (lines 44-60) already use **new names**:
- `vfs_pomodoro_create_record`, `vfs_pomodoro_get_record`, etc.

**Verdict: No action needed.**

---

## 3. Added Commands -- Missing Frontend Bindings

### 3.1 OCR Storage Commands

5 new Tauri commands have **zero frontend bindings**:

| Command | Backend Module | Returns | Frontend Binding |
|---|---|---|---|
| `vfs_ocr_store_result` | `vfs::ocr_storage_handlers` | `VfsResult<String>` | NONE |
| `vfs_ocr_list_results` | `vfs::ocr_storage_handlers` | `VfsResult<Vec<OcrStorageEntry>>` | NONE |
| `vfs_ocr_delete_result` | `vfs::ocr_storage_handlers` | `VfsResult<()>` | NONE |
| `vfs_ocr_mark_exported` | `vfs::ocr_storage_handlers` | `VfsResult<()>` | NONE |
| `vfs_ocr_list_for_export` | `vfs::ocr_storage_handlers` | `VfsResult<Vec<OcrStorageEntry>>` | NONE |

### 3.2 Missing TypeScript Type

The `OcrStorageEntry` struct has no TypeScript counterpart. Based on the backend definition, the needed interface is:

```typescript
export interface OcrStorageEntry {
  id: string;
  resource_id: string;
  text: string;
  confidence: number;
  source: string;
  exported: boolean;
  created_at: string;
}
```

### 3.3 Recommended Action

Create a new API file `src/api/vfsOcrStorageApi.ts` with wrapper functions and the `OcrStorageEntry` interface, modeled after the existing VFS API files (`src/api/vfsRagApi.ts`). Suggested structure:

```typescript
export interface OcrStorageEntry {
  id: string;
  resource_id: string;
  text: string;
  confidence: number;
  source: string;
  exported: boolean;
  created_at: string;
}

export async function vfsOcrStoreResult(
  resourceId: string,
  text: string,
  confidence: number,
  source: string
): Promise<string> {
  return invoke<string>('vfs_ocr_store_result', { resourceId, text, confidence, source });
}

export async function vfsOcrListResults(
  resourceId: string
): Promise<OcrStorageEntry[]> {
  return invoke<OcrStorageEntry[]>('vfs_ocr_list_results', { resourceId });
}

export async function vfsOcrDeleteResult(id: string): Promise<void> {
  return invoke<void>('vfs_ocr_delete_result', { id });
}

export async function vfsOcrMarkExported(id: string): Promise<void> {
  return invoke<void>('vfs_ocr_mark_exported', { id });
}

export async function vfsOcrListForExport(): Promise<OcrStorageEntry[]> {
  return invoke<OcrStorageEntry[]>('vfs_ocr_list_for_export');
}
```

---

## 4. Signature Changes -- No Frontend Impact

### 4.1 Error Type Upgrades (~27 commands)

All commands whose return type changed from `Result<T, String>` to `Result<T, TypedError>` do **not** require frontend changes. In Tauri v2, the `invoke()` API serializes errors as strings regardless of the Rust error type. The error message format may differ slightly (typed errors add context) but the API contract is unchanged.

Commands in this category:
- `check_anki_connect_availability`, `get_deck_names`, `get_model_names`, `get_model_field_names`, `create_deck_if_not_exists`, `import_apkg`
- `vfs_get_dimension_range`, `vfs_get_preset_dimensions`
- `dstu_parse_path`, `dstu_unwatch`, `dstu_watch`
- All `data_governance_*` commands with signature changes
- All `vfs_unified_index_*` and `vfs_resource_*` commands

### 4.2 Internal Helpers (4 commands, no frontend exposure)

`validate_id_format`, `validate_id_format_any`, `ToolExecutor::execute`, `ExecutionContext::save_tool_block`, `delete_persisted_todo_list`, and `parse_params` are backend-internal and have no frontend impact.

**Verdict: No action needed.**

---

## 5. Potentially Broken Code Path

### 5.1 `generate_anki_cards_for_segment` -- Missing Backend Command

- **File**: `src/services/ankiApiAdapter.ts:173`
- **Code**: `return await invoke('generate_anki_cards_for_segment', params);`
- **Status**: This command **does not exist** in the Rust backend (no match in `src-tauri/`).
- **Method annotation**: The method `generateAnkiCardsForSegment` is marked `@deprecated` and is a fallback code path inside `ankiApiAdapter`.
- **Risk**: If any caller actually triggers this deprecated path, it will fail at runtime with a "command not found" error from Tauri.
- **Recommendation**:
  1. Either remove the deprecated method entirely, or
  2. Replace the invoke call with the equivalent new command (`enhanced_anki_start_document_processing`), or
  3. Add the command back to the Rust backend if still needed.

---

## 6. Moved/Deleted Modules -- No Frontend Impact

Six Rust modules were deleted (adapters, resource_handlers, resource_repo), but these had no direct frontend-facing API surface. The frontend only called their Tauri commands (which were migrated or removed as covered above). No frontend code imports these modules.

New modules added (`paddleocr_api.rs`, `dstu/error.rs`, `essay_grading/error.rs`, `memory/error.rs`, `review_plan_error.rs`, `shared.rs`, `ocr_storage.rs`, `ocr_storage_handlers.rs`) are backend-only. Only the OCR storage handlers expose new Tauri commands that need frontend bindings (see Section 3).

---

## 7. Full File-by-File Impact List

| File | Change Needed | Priority |
|------|--------------|----------|
| `src/api/vfsOcrStorageApi.ts` | **CREATE** -- New API file with TypeScript bindings for 5 OCR commands | HIGH |
| `src/services/ankiApiAdapter.ts:173` | Investigate `generate_anki_cards_for_segment` dead code | MEDIUM |
| `src/features/chat/skills/builtin-tools/subagent-worker.ts:142` | Update inline comment referencing `builtin-workspace_get_context` (cosmetic) | LOW |
| `src/features/chat/plugins/events/toolCall.ts:320-378` | Console.log messages still say `workspace_create` (cosmetic) | LOW |
| `src/features/chat/debug/workspaceOrchestrationTestPlugin.ts:802-865` | Test labels use old name `workspace_create` / `workspace_create_agent` (cosmetic) | LOW |
| `src/features/chat/skills/builtin-tools/workspace-tools.ts` | Documentation text uses old names (cosmetic -- these are MCP tool names, should stay) | NONE |
| `src/mcp/builtinMcpServer.ts:1247-1400` | MCP tool display names use old names (these are MCP names, should stay) | NONE |
| `src/components/anki/cardforge/engines/TaskController.ts` | JSDoc references old command names (cosmetic) | LOW |
| `src/components/anki/cardforge/engines/CardAgent.ts` | Comments reference old command names (cosmetic) | LOW |
| `src/features/chat/anki/index.tsx` | Comments reference old command names (cosmetic) | LOW |
| `src/services/ankiApiAdapter.ts` | Comments reference old command names (cosmetic) | LOW |
| `src/debug-panel/plugins/ChatAnkiWorkflowDebugPlugin.ts:43` | Comment references old name (cosmetic) | LOW |

---

## 8. Recommendations

### Critical (before deployment)
1. Create `src/api/vfsOcrStorageApi.ts` with bindings for the 5 new OCR storage commands.

### Moderate
2. Fix or remove the `generateAnkiCardsForSegment` deprecated method in `src/services/ankiApiAdapter.ts` -- it calls a backend command that no longer exists.

### Nice-to-have (cosmetic)
3. Update console.log strings in `src/features/chat/plugins/events/toolCall.ts` to use `chat_v2_workspace_create` instead of `workspace_create`.
4. Update JSDoc in `src/components/anki/cardforge/engines/TaskController.ts` and `src/components/anki/cardforge/engines/CardAgent.ts` to reference new command names.
5. Update comments in `src/features/chat/anki/index.tsx` and `src/services/ankiApiAdapter.ts` to use new command names.

---

## Appendix A: Search Methodology

- Searched all `.ts` and `.tsx` files under `src/` using `grep` for each old command name
- Searched with regex boundaries (`\bword\b`) to avoid substring false positives
- Separately searched for `invoke()` calls to distinguish functional calls from comments/docs
- Verified all found `invoke()` calls against the change database to confirm they use the correct (new) name
- Cross-referenced with Rust backend source to verify existence of commands

## Appendix B: Files Scanned

- **Total `.ts`/`.tsx` files in `src/`**: 1575
- **Files with `invoke()` calls to involved commands**: ~15 files (all verified using new names)
- **Files with cosmetic old-name references only**: ~10 files
- **Files needing immediate action**: 0 existing files (1 new file needed)
