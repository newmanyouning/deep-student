# Event System — Frontend-Backend Event Flow

> This document maps every Tauri event emitted by the Rust backend and the corresponding frontend `listen()` consuming it.

## Event Categories

| Category | Channel Pattern | Backend Source | Frontend Consumer |
|---|---|---|---|
| Chat Streaming | `chat_v2_event_{session_id}` | `chat_v2/events.rs` | `TauriAdapter.ts` |
| Chat Session | `chat_v2_session_{session_id}` | `chat_v2/events.rs` | `TauriAdapter.ts` |
| Chat Request Body | `chat_v2_llm_request_body` | `llm_manager/model2_pipeline.rs` | `TauriAdapter.ts` |
| Media Processing | `media-processing-*` | `vfs/pdf_processing_service.rs` | `usePdfProcessingProgress.ts` |
| Media Processing (legacy) | `pdf-processing-*` | `vfs/pdf_processing_service.rs` | `usePdfProcessingProgress.ts` |
| OCR Progress (legacy) | `pdf_ocr_progress` | `pdf_ocr_service.rs` | `usePdfProcessingProgress.ts` |
| Anki Generation | `anki_generation_event` | `enhanced_anki_service.rs`, `streaming_anki_service.rs` | `TauriAdapter.ts` |
| Workspace | `workspace_*` | `chat_v2/workspace/emitter.rs` | `workspace/events.ts` |
| Data Governance | `data-governance-migration-status` | `lib.rs` (setup) | `useMigrationStatusListener.ts` |
| Data Governance Sync | `data-governance-sync-progress` | `data_governance/sync/emitter.rs` | `DataGovernanceDashboard.tsx` |
| Backup | `backup-job-progress` | `backup_job_manager.rs` | `useBackupJobListener.ts` |
| Cloud Sync | `cloud-sync-progress` | `cloud_storage/mod.rs` | `CloudSyncManager` |
| MCP | `mcp_tools_changed` | `lib.rs` (MCP init) | `McpService` |
| MCP Test | `mcp-test-progress` | `cmd/mcp.rs` | MCP Debug Panel |
| Menu | `menu-event-*` | `menu.rs` | `menuEventBridge.ts` |
| DSTU | `dstu:change:{path}` | `dstu/handler_utils/node_converters.rs` | `LearningHubSidebar.tsx` |
| Canvas | `canvas:ai-edit-request` | `chat_v2/tools/canvas_executor.rs` | `useCanvasAIEditHandler.ts` |
| Import Progress | `question_import_progress`, `csv_import_progress`, `textbook-import-progress`, `notes-import-progress` | `commands.rs`, `cmd/textbooks.rs`, `cmd/notes.rs` | `DataImportExport.tsx` |
| Legacy Migration | `chat_v2_migration_event` | `chat_v2/migration/legacy_migration.rs` | `ChatMigrationSection.tsx` |
| Anki Tool | `anki_tool_call` | `chat_v2/tools/anki_executor.rs` | `CardEngine.ts` |

---

## a) Event Emission Map — Backend Sources

```mermaid
flowchart TB
    subgraph Backend["Rust Backend — Event Emitters"]
        CHAT_EVTS["chat_v2/events.rs</br>ChatV2EventEmitter"]
        LLM_STR["llm_manager/streaming.rs</br>citations & sources events"]
        MODEL2["llm_manager/model2_pipeline.rs</br>chat_v2_llm_request_body"]
        
        PDF_SVC["vfs/pdf_processing_service.rs</br>PdfProcessingService"]
        OCR_SVC["pdf_ocr_service.rs</br>PdfOcrService"]
        
        ESSAY_EVTS["essay_grading/events.rs</br>EssayGradingEventEmitter"]
        QBANK_EVTS["qbank_grading/events.rs</br>QbankGradingEventEmitter"]
        TRANS_EVTS["translation/events.rs</br>TranslationEventEmitter"]
        
        ANKI_SVC["enhanced_anki_service.rs</br>streaming_anki_service.rs"]
        
        WORK_EMIT["chat_v2/workspace/emitter.rs</br>WorkspaceEventEmitter"]
        
        BACKUP_MGR["backup_job_manager.rs"]
        CLOUD_SYNC["cloud_storage/mod.rs"]
        DG_SYNC["data_governance/sync/emitter.rs</br>SyncProgressEmitter"]
        
        DSTU_NODE["dstu/handler_utils/node_converters.rs"]
        CANVAS["chat_v2/tools/canvas_executor.rs"]
        
        MCP_CLIENT["mcp/client.rs</br>McpClient"]
    end

    subgraph Channels["Event Channels"]
        C1["chat_v2_event_{sessionId}</br>Block-level (content/thinking/tool_call/rag/memory/web_search)"]
        C2["chat_v2_session_{sessionId}</br>Session-level (stream_start/complete/error/cancelled)"]
        C3["chat_v2_event_{sessionId}_web_search</br>Source citations"]
        C4["chat_v2_event_{sessionId}_rag_sources</br>RAG citations"]
        C5["chat_v2_event_{sessionId}_memory_sources</br>Memory citations"]
        
        C6["media-processing-progress</br>media-processing-completed</br>media-processing-error"]
        C7["pdf-processing-progress</br>pdf-processing-completed</br>pdf-processing-error"]
        C8["pdf_ocr_progress"]
        
        C9["essay_grading_event_{sessionId}"]
        C10["qbank_grading_event_{sessionId}"]
        C11["translation_event_{sessionId}"]
        
        C12["anki_generation_event"]
        
        C13["workspace_message_received</br>workspace_agent_joined</br>workspace_agent_status_changed</br>workspace_closed</br>workspace_coordinator_awakened</br>workspace_subagent_retry</br>workspace_warning"]
        
        C14["backup-job-progress</br>backup-jobs-resumable"]
        C15["cloud-sync-progress"]
        C16["data-governance-sync-progress"]
        
        C17["dstu:change:{path}</br>dstu:change"]
        C18["canvas:ai-edit-request"]
        
        C19["mcp_tools_changed</br>mcp-test-progress"]
        
        C20["question_import_progress</br>csv_import_progress</br>textbook-import-progress</br>notes-import-progress"]
        C21["chat_v2_llm_request_body"]
    end

    CHAT_EVTS --> C1
    CHAT_EVTS --> C2
    LLM_STR --> C3
    LLM_STR --> C4
    LLM_STR --> C5
    MODEL2 --> C21
    
    PDF_SVC --> C6
    PDF_SVC --> C7
    OCR_SVC --> C8
    
    ANKI_SVC --> C12
    
    WORK_EMIT --> C13
    CANVAS --> C18
    
    ESSAY_EVTS --> C9
    QBANK_EVTS --> C10
    TRANS_EVTS --> C11
    
    BACKUP_MGR --> C14
    CLOUD_SYNC --> C15
    DG_SYNC --> C16
    
    DSTU_NODE --> C17
    
    MCP_CLIENT --> C19
    
    %% Commands emitting progress
    CMDS_RS["commands.rs</br>(import progress)"] --> C20
```

### How Events Are Emitted (Rust Pattern)

The backend uses `tauri::Emitter` via either:
1. **Window**: `window.emit("event_name", payload)` — scoped to the emitting window
2. **AppHandle**: `app_handle.emit("event_name", payload)` — global, all windows receive it

The global `AppHandle` is stored at module level in `src-tauri/src/lib.rs:124`:
```rust
static GLOBAL_APP_HANDLE: OnceLock<AppHandle> = OnceLock::new();
```

---

## b) Event Subscription Map — Frontend Listeners

```mermaid
flowchart TB
    subgraph Frontend["Frontend (React/TypeScript) — Event Listeners"]
        TAURI_ADAPTER["src/features/chat/adapters/TauriAdapter.ts</br>ChatV2TauriAdapter"]
        
        MEDIA_HOOK["src/hooks/usePdfProcessingProgress.ts</br>usePdfProcessingProgress()"]
        
        WORKSPACE["src/features/chat/workspace/events.ts</br>initWorkspaceEventListeners()"]
        
        BACKUP["src/hooks/useBackupJobListener.ts"]
        MIGRATION["src/hooks/useMigrationStatusListener.ts"]
        
        DATA_GOV["src/components/DataGovernanceDashboard.tsx"]
        DATA_IMPORT["src/components/DataImportExport.tsx"]
        
        ANKI["src/components/anki/cardforge/engines/CardEngine.ts</br>src/services/ankiApiAdapter.ts"]
        
        DND["src/hooks/useTauriDragAndDrop.ts"]
        
        MENU_BRIDGE["src/menu/menuEventBridge.ts"]
        MCP["src/mcp/tauriStdioTransport.ts"]
        
        DEBUG["src/debug-panel/plugins/*"]
        
        CANVAS_HOOK["src/features/notes/hooks/useCanvasAIEditHandler.ts"]
        NOTES_CTX["src/features/notes/NotesContext.tsx"]
        SIDEBAR["src/features/learning-hub/LearningHubSidebar.tsx"]
        SYNC_SECTION["src/features/settings/components/SyncSettingsSection.tsx"]
    end

    subgraph Events["Event Channels"]
        E1["chat_v2_event_{sessionId}"]
        E2["chat_v2_session_{sessionId}"]
        E3["chat_v2_llm_request_body"]
        E4["anki_generation_event"]
        
        E5["media-processing-progress</br>media-processing-completed</br>media-processing-error"]
        E6["pdf-processing-progress</br>pdf-processing-completed</br>pdf-processing-error"]
        E7["pdf_ocr_progress"]
        
        E8["workspace_message_received"]
        E9["workspace_agent_joined"]
        E10["workspace_agent_status_changed"]
        E11["workspace_closed"]
        E12["workspace_coordinator_awakened"]
        E13["workspace_subagent_retry"]
        E14["workspace_warning"]
        E15["workspace_worker_ready"]
        
        E16["backup-job-progress"]
        E17["backup-jobs-resumable"]
        
        E18["data-governance-migration-status"]
        E19["data-governance-sync-progress"]
        
        E20["question_import_progress"]
        E21["csv_import_progress"]
        E22["textbook-import-progress"]
        E23["notes-import-progress"]
        
        E24["dstu:change"]
        E25["canvas:ai-edit-request"]
        
        E26["tauri://drag-drop</br>tauri://drag-enter</br>tauri://drag-leave</br>tauri://drag-over"]
        E27["tauri://close-requested"]
    end

    TAURI_ADAPTER --> E1
    TAURI_ADAPTER --> E2
    TAURI_ADAPTER --> E3
    TAURI_ADAPTER --> E4
    
    MEDIA_HOOK --> E5
    MEDIA_HOOK --> E6
    MEDIA_HOOK --> E7
    
    WORKSPACE --> E8
    WORKSPACE --> E9
    WORKSPACE --> E10
    WORKSPACE --> E11
    WORKSPACE --> E12
    WORKSPACE --> E13
    WORKSPACE --> E14
    WORKSPACE --> E15
    
    BACKUP --> E16
    BACKUP --> E17
    MIGRATION --> E18
    DATA_GOV --> E19
    SYNC_SECTION --> E19
    DATA_IMPORT --> E20
    DATA_IMPORT --> E21
    DATA_IMPORT --> E22
    DATA_IMPORT --> E23
    
    SIDEBAR --> E24
    CANVAS_HOOK --> E25
    
    DND --> E26
    NOTES_CTX --> E27
    
    ANKI --> E4
    MENU_BRIDGE -->|"menu-event-*"| EVT_MENU
    MCP -->|"mcp-stdio-*"| EVT_MCP
    DEBUG --> E1
    DEBUG --> E8
```

### Key Frontend Event Handlers

**ChatV2TauriAdapter** (`src/features/chat/adapters/TauriAdapter.ts:500-514`)
```typescript
// Event listeners registered in setup()
listen<BackendEvent>(`chat_v2_event_${sessionId}`, (event) => {
  this.handleBlockEvent(event.payload);  // content/thinking/tool_call/rag chunks
});
listen<SessionEventPayload>(`chat_v2_session_${sessionId}`, (event) => {
  this.handleSessionEvent(event.payload);  // stream_start/complete/error/cancelled
});
listen('anki_generation_event', (event) => {
  this.handleAnkiGenerationEvent(event.payload);
});
listen('chat_v2_llm_request_body', (event) => {
  this.handleLlmRequestBody(event.payload);
});
```

**usePdfProcessingProgress** (`src/hooks/usePdfProcessingProgress.ts:181-252`)
```typescript
// Unified events
listen('media-processing-progress', handler);
listen('media-processing-completed', handler);
listen('media-processing-error', handler);
// Legacy fallback
listen('pdf-processing-progress', handler);
listen('pdf-processing-completed', handler);
listen('pdf-processing-error', handler);
// Legacy OCR
listen('pdf_ocr_progress', handler);
```

---

## c) Event Lifecycle Sequence Diagrams

### 1. Media Processing Events (PDF Upload → Processing → Completion)

```mermaid
sequenceDiagram
    actor User
    participant FE as Frontend<br/>(React)
    participant Store as pdfProcessingStore
    participant Hook as usePdfProcessingProgress
    participant Rust as Rust Backend
    participant PDFSvc as PdfProcessingService
    participant Lance as Lance Vector Store

    User->>FE: Upload PDF file
    FE->>Rust: invoke('vfs_upload_file')
    Rust-->>FE: UploadFileResult (fileId)
    FE->>Rust: invoke('vfs_start_pdf_processing', { fileId })

    Note over Rust,PDFSvc: === Processing Pipeline ===
    PDFSvc->>PDFSvc: Page compression (concurrent)
    PDFSvc->>PDFSvc: OCR extraction (concurrent)
    PDFSvc->>PDFSvc: Text chunking & indexing

    PDFSvc->>Hook: emit('media-processing-progress', { fileId, status: { stage: "compressing", percent: 30 } })
    Hook->>Store: update(fileId, { stage: "compressing", percent: 30 })
    Store-->>FE: Re-render progress bar

    PDFSvc->>Hook: emit('media-processing-progress', { fileId, status: { stage: "ocr_processing", percent: 65, readyModes: ["text"] } })
    Hook->>Store: update(fileId, { stage: "ocr_processing", percent: 65, readyModes: ["text"] })
    Hook->>Hook: invalidateResourceCache(fileId) ← new mode available
    Store-->>FE: Re-render with text mode ready

    PDFSvc->>Hook: emit('media-processing-progress', { fileId, status: { stage: "indexing", percent: 90, readyModes: ["text", "ocr"] } })
    Hook->>Store: update(fileId, { stage: "indexing", percent: 90, readyModes: ["text", "ocr"] })

    PDFSvc->>Lance: Store vector embeddings

    alt Success
        PDFSvc->>Hook: emit('media-processing-completed', { fileId, stage: "completed", readyModes: ["text", "ocr"] })
        Hook->>Store: setCompleted(fileId, ["text", "ocr"])
        Hook->>Hook: invalidateResourceCache(fileId)
        Store-->>FE: Show "Processing Complete" ✓
    else Error
        PDFSvc->>Hook: emit('media-processing-error', { fileId, stage: "ocr_processing", error: "OCR failed: ..." })
        Hook->>Store: setError(fileId, error, "ocr_processing")
        Store-->>FE: Show error notification
    end
```

### 2. OCR Progress Events (Legacy `pdf_ocr_progress`)

```mermaid
sequenceDiagram
    participant FE as Frontend
    participant Hook as usePdfProcessingProgress
    participant Store as pdfProcessingStore
    participant Rust as Rust Backend
    participant OCR as PdfOcrService

    FE->>Rust: invoke('start_pdf_ocr_backend', { fileId })
    Rust-->>FE: { session_id: "ocr_sess_xxx" }
    FE->>FE: registerOcrSession(fileId, "ocr_sess_xxx")

    Note over OCR: === PDF Rendering Phase ===
    OCR->>Hook: emit('pdf_ocr_progress', { type: "RenderStarted", session_id: "ocr_sess_xxx", total_pages: 150 })
    Hook->>Store: update(fileId, { stage: "ocr_processing", currentPage: 0, totalPages: 150 })

    loop For each page (0..149)
        OCR->>Hook: emit('pdf_ocr_progress', { type: "PageRendered", session_id, page_index: i, rendered: i+1, total: 150 })
        Hook->>Store: update(fileId, { currentPage: i, percent: ((i+1)/150)*100 })
    end

    Note over OCR: === OCR Processing Phase ===
    OCR->>OCR: Process rendered images through<br/>PaddleOCR / LLM vision models

    OCR->>Hook: emit('pdf_ocr_progress', { type: "Completed", session_id, has_failures: false })
    Hook->>Store: setCompleted(fileId, ["ocr"], "completed")

    alt One or more pages failed
        OCR->>Hook: emit('pdf_ocr_progress', { type: "PageFailed", session_id, page_index: 42, error: "..." })
        Hook->>Store: UPDATE: setError for that page
        OCR->>Hook: emit('pdf_ocr_progress', { type: "Completed", session_id, has_failures: true })
        Hook->>Store: setCompleted(fileId, ["ocr"], "completed_with_issues")
    end
```

### 3. Chat V2 Streaming Events

```mermaid
sequenceDiagram
    participant User
    participant FE as Frontend (React)
    participant Adapter as TauriAdapter
    participant EventBridge as eventBridge.ts
    participant Store as ChatStore
    participant Rust as Rust Backend
    participant Pipeline as ChatV2Pipeline
    participant LLM as LLMManager

    User->>FE: Type message + Send
    FE->>FE: Optimistically create user message in Store
    FE->>Rust: invoke('chat_v2_send_message', { sessionId, content, ... })
    
    Note over Rust: === Stream Start ===
    Pipeline->>Adapter: emit('chat_v2_session_{sessionId}', { eventType: "stream_start", messageId, modelId })
    Adapter->>EventBridge: handleStreamStart(payload)
    EventBridge->>Store: CREATE assistant message placeholder (pending state)

    Note over Rust: === Variant Start (multi-model) ===
    Pipeline->>Adapter: emit('chat_v2_event_{sessionId}', { type: "variant_start", phase: "start", variantId, modelId })

    Note over Rust: === Block Events (may interleave) ===

    par Thinking Block
        Pipeline->>Adapter: emit('chat_v2_event_{sessionId}', { type: "thinking", phase: "start", messageId, blockId: "blk_1" })
        EventBridge->>Store: CREATE thinking block
        
        loop Streaming chunks
            Pipeline->>Adapter: emit('chat_v2_event_{sessionId}', { type: "thinking", phase: "chunk", blockId: "blk_1", chunk: "思考中..." })
            EventBridge->>Store: APPEND chunk to block
        end
        
        Pipeline->>Adapter: emit('chat_v2_event_{sessionId}', { type: "thinking", phase: "end", blockId: "blk_1" })
        Store-->>FE: Render complete thinking block
    and Content Block
        Pipeline->>Adapter: emit('chat_v2_event_{sessionId}', { type: "content", phase: "start", messageId })
        EventBridge->>Store: CREATE content block (chunk buffering begins)
        
        loop Streaming chunks
            Pipeline->>Adapter: emit('chat_v2_event_{sessionId}', { type: "content", phase: "chunk", blockId: "...", chunk: "Hello..." })
            EventBridge->>chunkBuffer: Buffer chunk
            chunkBuffer->>Store: Flush to Store periodically
            Store-->>FE: Progressive content rendering
        end
        
        Pipeline->>Adapter: emit('chat_v2_event_{sessionId}', { type: "content", phase: "end", blockId: "...", result: { ... } })
    and Tool Call Block
        Pipeline->>Adapter: emit('chat_v2_event_{sessionId}', { type: "tool_call_preparing", phase: "start", payload: { toolName, toolCallId } })
        EventBridge->>Store: CREATE "preparing" tool placeholder
        
        Pipeline->>Adapter: emit('chat_v2_event_{sessionId}', { type: "tool_call", phase: "start", blockId: "blk_3", payload: { toolName, toolInput } })
        EventBridge->>Store: CREATE tool_call block (mcp_tool type)
        Store-->>FE: Show tool invocation card
        
        alt Tool execution on backend
            LLM->>LLM: Execute tool (web_search, rag, etc.)
            LLM->>Adapter: emit('chat_v2_event_{sessionId}_web_search', { sources: [...] })
            EventBridge->>Store: UPDATE tool block with sources
            LLM->>Adapter: emit('chat_v2_event_{sessionId}', { type: "tool_call", phase: "chunk", blockId: "blk_3", chunk: "...result..." })
        end
        
        Pipeline->>Adapter: emit('chat_v2_event_{sessionId}', { type: "tool_call", phase: "end", blockId: "blk_3", result: { ... } })
    and RAG Sources
        LLM->>Adapter: emit('chat_v2_event_{sessionId}_rag_sources', { sources: [...] })
        EventBridge->>Store: UPDATE message context with RAG sources
    end

    Note over Rust: === Variant End ===
    Pipeline->>Adapter: emit('chat_v2_event_{sessionId}', { type: "variant_end", phase: "end", variantId, status: "success" })

    Note over Rust: === Stream Complete ===
    Pipeline->>Adapter: emit('chat_v2_session_{sessionId}', { eventType: "stream_complete", messageId, durationMs: 5432, usage: { ... } })
    Adapter->>EventBridge: handleStreamComplete(payload)
    EventBridge->>Store: FINALIZE message (remove pending status, save completed)
    
    alt Error
        Pipeline->>Adapter: emit('chat_v2_session_{sessionId}', { eventType: "stream_error", messageId, error: "..." })
        Adapter->>EventBridge: handleStreamAbort(payload)
        EventBridge->>Store: MARK message as failed
        Store-->>FE: Show error state with retry button
    end

    Store-->>FE: Render complete message
```

### Event Channel Summary Table

| Channel Pattern | Payload Type | Purpose | Serialization |
|---|---|---|---|
| `chat_v2_event_{sessionId}` | `BackendEvent` (camelCase) | Block-level lifecycle (start/chunk/end/error) | `#[serde(rename_all = "camelCase")]` |
| `chat_v2_session_{sessionId}` | `SessionEvent` (camelCase) | Session-level flow control | `#[serde(rename_all = "camelCase")]` |
| `{streamEvent}_web_search` | `{sources: [...], tool_name, timestamp}` | Web search citation sources | Manual `json!()` |
| `{streamEvent}_rag_sources` | `{sources: [...], tool_name, timestamp}` | RAG citation sources | Manual `json!()` |
| `{streamEvent}_memory_sources` | `{sources: [...], tool_name, timestamp}` | Memory citation sources | Manual `json!()` |
| `chat_v2_llm_request_body` | `{streamEvent, model, url, requestBody, ...}` | LLM request body for debug | Manual `json!()` |
| `anki_generation_event` | `{type, sessionId, ...}` | Anki card generation progress | Manual `json!()` |
| `media-processing-*` | `{fileId, status/readyModes/error, mediaType}` | PDF/image processing pipeline | `#[serde(rename_all = "camelCase")]` |
| `pdf_ocr_progress` | `{type, session_id, page_index, ...}` | Legacy OCR per-page events | Manual `json!()` |
| `workspace_*` | Varies by event type | Multi-agent workspace events | Manual `json!()` |
| `backup-job-progress` | `BackupJobSnapshot` | Backup job progress | `#[serde(rename_all = "camelCase")]` |
| `data-governance-sync-progress` | `SyncProgress` | Data governance sync phases | `#[serde(rename_all = "camelCase")]` |
| `dstu:change:{path}` | `{action, resourceId, ...}` | DSTU resource change notifications | Manual `json!()` |

---

> **Note on Sequence IDs**: The `ChatV2EventEmitter` maintains per-session atomic sequence counters (`SESSION_SEQUENCE_COUNTERS` in `chat_v2/events.rs:713`) that produce strictly increasing `sequenceId` values. The frontend `eventBridge.ts` uses these to detect out-of-order or dropped events.
