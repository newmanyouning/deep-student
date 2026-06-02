# DATA TRACE: Memory Operations Pipeline

> Generated: 2026-06-01
> Files examined: 6 Rust + 3 TypeScript
> Purpose: Trace LLM response -> memory storage -> frontend retrieval

---

## 1. Overview: Two Memory Ingestion Paths

```
                       LLM Response
                      /            \
          Path A (Explicit Tool)    Path B (Auto-Extract)
          memory_write_smart        save_results_post_commit
          memory_write_batch        -> trigger_auto_memory_extraction
          memory_write              -> extract_and_store
               \                         /
                \                       /
            write_smart_with_source()
                       |
              LLM Decision (reuse/add/update/append)
                       |
                  VFS Note (SQLite + LanceDB vector index)
```

---

## 2. Path A: Explicit Tool Call

### Entry Point
**File**: `C:/deep-student/src-tauri/src/chat_v2/tools/memory_executor.rs`

The LLM during a chat can call these tools:
- `builtin-memory_write` -> `execute_write()` (line 243)
- `builtin-memory_write_smart` -> `execute_write_smart()` (line 609)
- `builtin-memory_write_batch` -> `execute_write_batch()` (line 769)

Each constructs a `MemoryService` from `ExecutionContext` (line 66-89):
```rust
fn get_service(&self, ctx: &ExecutionContext) -> ToolResult<MemoryService> {
    let vfs_db = ctx.vfs_db.as_ref().ok_or_else(...)?;
    let llm_manager = ctx.llm_manager.as_ref().ok_or_else(...)?;
    let lance_store = ctx.vfs_lance_store.clone()...;
    let mem_storage = Arc::new(VfsMemoryStorage::new(vdb, lance_store, llm_manager));
    Ok(MemoryService::new_with_storage(mem_storage, llm_manager))
}
```

### Smart Write Flow (`execute_write_smart`, lines 609-767)
1. Extract `title`, `content`, `folder`, `memory_type`, `memory_purpose` from tool arguments.
2. **Sensitive data filter** -> returns `FILTERED` if PII detected.
3. **Content length check** -> returns `FILTERED` if exceeds `memory_type.max_content_chars()`.
4. Resolve idempotency key (SHA-256 of normalized content + session + message, line 914-948).
5. Call `service.write_smart_with_source(...)` which delegates to `MemoryService`.

### Smart Write Inner Flow (`write_smart_with_source`)
**File**: `C:/deep-student/src-tauri/src/memory/service.rs` (line 1026-1600+)

1. **Idempotency check** -> returns cached result if same key already processed.
2. **Sensitive data & length filters** (repeat for safety).
3. **Memory type routing**:
   - `Note` type -> `write_explicit_memory` -> `write_typed(Note)` + immediate indexing.
   - `Study` type -> `write_explicit_memory` -> `upsert_study_memory` (title-dedup) + immediate indexing.
   - `Fact` type (default) -> proceeds to LLM decision.
4. **Fact hard-reject** (line 1174): checks if content looks like knowledge/vocabulary instead of user fact.
5. **Privacy mode** -> local title-dedup only, skip LLM calls.
6. **Search similar memories** (line 1235-1249): vector search top-15 with `SearchPurpose::InternalDedup` (no hit recording).
7. **LLM Decision** (line 1264-1278):
   - Calls `MemoryLLMDecision::decide()` with new content + similar memories.
   - LLM returns `MemoryDecisionResponse` with `event`, `target_note_id`, `confidence`, `reason`.
   - If LLM call fails -> safe fallback to ADD with 0.6 confidence.
8. **Confidence threshold** (line 1287): if UPDATE/APPEND/DELETE confidence < 0.65 -> downgrade to NONE (returned as `downgraded: true`).
9. **Execute decision**:
   - `ADD` -> `write_typed(Create)` + immediate indexing.
   - `UPDATE` -> `update_by_id_with_source()` + sync system tags + immediate indexing.
   - `APPEND` -> read existing + append content + `update_by_id_with_source()` + sync tags + indexing.
   - Safety: if `target_note_id` not in candidate set -> downgrade to ADD.

### Write Output Format
```rust
// SmartWriteOutput (service.rs line 274-287)
{
  note_id: String,
  event: "ADD" | "UPDATE" | "APPEND" | "DELETE" | "NONE" | "FILTERED",
  is_new: bool,
  confidence: f32,
  reason: String,
  resource_id: Option<String>,
  downgraded: bool,
}
```

---

## 3. Path B: Automatic Post-Chat Extraction

### Entry Point
**File**: `C:/deep-student/src-tauri/src/chat_v2/pipeline/persistence.rs` (line 935-1080)

After each message transaction commit (`commit_tx` -> `save_results_post_commit` -> `trigger_auto_memory_extraction`):

```rust
fn trigger_auto_memory_extraction(&self, ctx: &PipelineContext) {
```
**Gate checks** (all sync, before spawning):
1. VFS DB exists
2. Auto-extract frequency != `Off`
3. Privacy mode disabled
4. Content length >= frequency threshold
5. **Race guard**: if LLM already wrote fact memories via tools this turn -> skip (line 1008-1029)

If all pass -> spawn async task calling `MemoryAutoExtractor::extract_and_store()`.

### Auto Extractor
**File**: `C:/deep-student/src-tauri/src/memory/auto_extractor.rs`

1. `extract_candidates(user_content, assistant_content, existing_profile)` (line 39-69):
   - Truncates user/assistant content to 1500 chars each.
   - Builds an extraction prompt with rules: extract user facts only, skip knowledge, skip existing profile.
   - Calls `llm_manager.call_memory_decision_raw_prompt(&prompt)`.
   - Parses JSON array response into `Vec<CandidateMemory>`.

2. `extract_and_store()` (line 72-176):
   - For each candidate, calls `memory_service.write_smart_with_source(..., MemoryOpSource::AutoExtract, ...)`.
   - Deduplicates within the same batch by content hash.
   - After all stored, calls `memory_service.refresh_profile_summary()`.

### Extraction Prompt (line 178-233)
```
You are a user memory extractor. Extract atomic facts about the **user** from this conversation.

Rules:
1. Each memory: short statement about the user (<=50 chars)
2. Only user facts: identity, learning status, preferences, time constraints, goals
3. NEVER extract: subject knowledge, problem content, solution processes, document summaries
4. Max 5 items, skip already recorded facts
5. Return [] if nothing new

Output format:
[{"title": "keyword", "content": "short sentence", "folder": "category"}]
```

---

## 4. Memory Data Model

### Memory Types and Constraints
**File**: `C:/deep-student/src-tauri/src/memory/service.rs` (line 47-99)

| MemoryType | Content Limit | Intended Use |
|------------|---------------|--------------|
| `Fact` | 200 chars | Atomic user facts (default) |
| `Study` | 4000 chars | Vocabulary, knowledge points, errors |
| `Note` | 2000 chars | Methodology, experience, tips |

### Memory Purposes and Search Weights (line 101-159)

| Purpose | Search Weight | Meaning |
|---------|---------------|---------|
| `Internalized` | 1.4 | Core content user must understand |
| `Memorized` | 1.0 | Standalone facts (default) |
| `Supplementary` | 0.8 | Supporting knowledge |
| `Systemic` | 0.65 | System meta-info, not user-visible |

### Tag System
Tags are stored as VFS note tags with special prefixes:
- `_type:fact|study|note` - memory type
- `_purpose:internalized|memorized|supplementary|systemic` - purpose
- `_ref:<id>` - lightweight cross-reference
- `_hits:<N>` - search hit count (for evolution)
- `_last_hit:<ISO>` - last hit timestamp
- `_important` / `_stale` - flags

### Storage Backend
- **Primary**: VFS SQLite notes (via `VfsMemoryStorage` -> `VfsDatabase`)
- **Vector Index**: LanceDB (via `VfsLanceStore`) for hybrid search (keyword + embedding)
- **Immediate indexing**: After write, `index_resource_immediately` triggers embedding generation + LanceDB write.

---

## 5. Memory Retrieval (Search)

### Entry Points
1. **Tool call**: `MemoryToolExecutor::execute_search()` -> `service.search_with_rerank(query, top_k, false)`
2. **Tauri command**: `memory_search` handler -> `service.search_with_rerank(&query, k, false)`

### Search Pipeline (`search_with_rerank`, service.rs line 1675)
```
search_with_rerank(query, top_k, use_query_rewrite)
  |
  |-- (optional) MemoryQueryRewriter.rewrite_simple() -- another LLM call to expand query
  |
  |-- MemoryReranker.new() -- check if reranker API model configured
  |     |-- has_reranker_api? -> retrieval_k = top_k * 2
  |     |-- no reranker -> retrieval_k = top_k
  |
  |-- search(query, retrieval_k)  [line 612]
  |     |-- generate_embedding(query) via LLMManager
  |     |-- get_memory_folder_ids (cached folder tree)
  |     |-- storage.hybrid_search("text", query, embedding, k, folder_ids, ["note"])
  |     |-- for each result: get_note_by_resource_id, filter to memory root
  |     |-- compute_tag_weight, apply time_decay (60-day half-life)
  |     |-- record_search_hits (async, fire-and-forget)
  |
  |-- (optional) MemoryReranker.rerank(query, results) -- API reranker call
  |
  v
  Vec<MemorySearchResult>
```

### Search Result Format
```rust
// MemorySearchResult (service.rs line 187-196)
{
  note_id: String,
  note_title: String,
  folder_path: String,
  chunk_text: String,
  score: f32,
  updated_at: Option<String>,  // ISO 8601
}
```

### Tool Executor Search Output (memory_executor.rs line 162-184)
```
{
  "sources": [
    { "title": "...", "snippet": "...", "score": 0.95,
      "metadata": { "document_id": "...", "memory_id": "...", "note_id": "...", "folder_path": "...", "source_type": "memory" } }
  ],
  "results": [ ... MemorySearchResult ... ],
  "count": N
}
```

---

## 6. Frontend Retrieval

### API Layer
**File**: `C:/deep-student/src/api/memoryApi.ts`

| Function | Tauri Command | Return Type |
|----------|---------------|-------------|
| `searchMemory(query, topK?)` | `memory_search` | `MemorySearchResult[]` |
| `readMemory(noteId)` | `memory_read` | `MemoryReadOutput \| null` |
| `listMemory(folderPath?, limit?, offset?)` | `memory_list` | `MemoryListItem[]` |
| `writeMemory(title, content, folderPath?, mode?)` | `memory_write` | `MemoryWriteOutput` |
| `writeMemorySmart(title, content, ...)` | `memory_write_smart` | `SmartWriteOutput` |
| `writeMemoryBatch(items, ...)` | `memory_write_batch` | `MemoryBatchWriteOutput` |
| `deleteMemory(noteId)` | `memory_delete` | `void` |
| `getMemoryTree()` | `memory_get_tree` | `FolderTreeNode \| null` |

### Frontend Data Types (memoryApi.ts)

```typescript
interface MemorySearchResult {
  noteId: string; noteTitle: string; folderPath: string;
  chunkText: string; score: number;
}

interface MemoryReadOutput {
  noteId: string; title: string; content: string;
  folderPath: string; updatedAt: string;
}

interface SmartWriteOutput {
  noteId: string; event: 'ADD' | 'UPDATE' | 'APPEND' | 'DELETE' | 'NONE' | 'FILTERED';
  isNew: boolean; confidence: number; reason: string;
  resourceId?: string; downgraded: boolean;
}
```

### Display Component
**File**: `C:/deep-student/src/features/chat/plugins/blocks/memory.tsx`

- Renders a `<MemoryBlock>` inside the chat when `block.type === 'memory'`.
- Takes raw `toolOutput` from block, normalizes `sources` and `results` fields.
- Memory type icons: `conversation` -> Chat, `long_term` -> Clock, `user_profile` -> User.
- Shows streaming spinner, error state, or `SourceList` with up to 3 visible results.
- Compatible with both new `sources[]` format and legacy `results[]` format.

### Source Conversion Chain
**File**: `C:/deep-student/src/features/chat/plugins/blocks/components/types.ts`

```
BackendSourceInfo ---> RetrievalSource (frontend)
{ title?, snippet?,  |  { id, type, title, snippet,
  score?, metadata? }    url?, score?, metadata }
```
The `convertBackendSource()` function adds `id` (`{blockId}-source-{index}`) and `type` (`'memory'`).

---

## 7. Data Flow Diagram (Complete Trace)

```
[Chat UI] user types message
    |
    v
[LLM Manager] generates response + tools
    |
    |--- Path A: Tool Call ---\
    |                          |
    |   builtin-memory_write_smart {title, content, memory_type, ...}
    |                          |
    |                          v
    |   [MemoryToolExecutor::execute_write_smart()]
    |       - sensitive filter
    |       - resolve idempotency key
    |                          |
    |                          v
    |   [MemoryService::write_smart_with_source()]
    |       - search similar memories (vector)
    |       - LLM decide: add/update/append/none
    |       - execute decision via write_typed()
    |       - immediate indexing (embedding -> LanceDB)
    |       - spawn post-write maintenance (profile refresh + category refresh + evolution)
    |                          |
    |                          v
    |   VFS SQLite Note + LanceDB vector index
    |
    |--- Path B: Auto-Extract ---\
    |
    |   [Pipeline::save_results_post_commit()]
    |       - fire-and-forget trigger_auto_memory_extraction()
    |                          |
    |                          v
    |   [MemoryAutoExtractor::extract_and_store()]
    |       - LLM extract candidates from user + assistant content
    |       - for each: write_smart_with_source(MemoryOpSource::AutoExtract)
    |       - refresh_profile_summary()
    |
    v
[Memory stored as VFS notes under __system__/ + LanceDB vectors]

Later:
[Search Query]
    |
    v
[Search Pipeline]
    |-- (opt) query rewrite (LLM)
    |-- generate embedding (LLM)
    |-- hybrid search (LanceDB)
    |-- retrieve VFS notes
    |-- (opt) rerank (API)
    |-- apply time decay
    |-- record hits
    |
    v
[Frontend] MemoryBlock displays sources in chat
```

---

## 8. Key LLM Calls in the Pipeline

| Stage | LLM Call | Model Used | Frequency |
|-------|----------|------------|-----------|
| Auto-extract | Extract user facts from conversation | `memory_decision_model` | Per message (gated by frequency config) |
| Smart write decision | Decide ADD/UPDATE/APPEND vs similar memories | `memory_decision_model` | Per write_smart call |
| Query rewrite | Expand search query for better recall | LLM (provider default) | Per search with `use_query_rewrite=true` |
| Rerank | Reorder search results by relevance | Reranker API model | Per search (if configured) |
| Profile refresh | Aggregate user profile summary | LLM | On post-write maintenance |
| Category refresh | Classify memories into categories | LLM | On post-write maintenance (frequency-gated) |

---

## 9. Files Referenced

| File | Role |
|------|------|
| `src-tauri/src/chat_v2/tools/memory_executor.rs` | Tool executor: routes LLM tool calls to memory service |
| `src-tauri/src/memory/service.rs` | Core memory service: write/read/search with smart LLM decision |
| `src-tauri/src/memory/auto_extractor.rs` | Post-chat automatic memory extraction from conversations |
| `src-tauri/src/memory/llm_decision.rs` | LLM module for smart write decision (add/update/append) |
| `src-tauri/src/memory/reranker.rs` | Optional API reranker for search results |
| `src-tauri/src/chat_v2/pipeline/persistence.rs` | Pipeline hook: triggers auto-extraction after message commit |
| `src/api/memoryApi.ts` | Frontend API layer: Tauri command wrappers |
| `src/features/chat/plugins/blocks/memory.tsx` | Frontend display component for memory search results |
| `src/features/chat/plugins/blocks/components/types.ts` | Type conversion: backend -> frontend source format |
