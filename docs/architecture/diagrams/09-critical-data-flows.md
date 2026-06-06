# Critical Data Flows — Frontend-Backend Interaction Sequences

> This document traces three critical end-to-end operations through the full frontend-backend stack, referencing source files and line numbers.

---

## a) PDF Upload → OCR → Searchable Flow

### Source Files

| Component | File | Key Lines |
|---|---|---|
| Learning Hub FileContentView | `src/features/learning-hub/views/FileContentView.tsx` | Drop handler |
| VFS Ref API | `src/features/chat/context/vfsRefApiEnhancements.ts` | `upload()` wrapper |
| VFS File API | `src/api/vfsFileApi.ts` | Lines 91-93: `vfsFileApi.upload()` |
| VFS Upload Handler | `src-tauri/src/vfs/handlers/file_handlers.rs` | `vfs_upload_file` |
| PDF Processing Service | `src-tauri/src/vfs/pdf_processing_service.rs` | Event emission |
| PdfOcrService (legacy) | `src-tauri/src/pdf_ocr_service.rs` | Lines 360-480: render events |
| PdfProcessing Hook | `src/hooks/usePdfProcessingProgress.ts` | Lines 181-252: event handlers |
| Media Processing Store | `src/features/pdf/stores/pdfProcessingStore.ts` | State management |
| VFS Index Handlers | `src-tauri/src/vfs/handlers/index_handlers.rs` | `vfs_rag_search` |

### Full Sequence Diagram

```mermaid
sequenceDiagram
    box Frontend
        actor User
        participant FCV as FileContentView
        participant API as vfsFileApi
        participant REF as vfsRefApiEnhancements
        participant HOOK as usePdfProcessingProgress
        participant STORE as pdfProcessingStore
    end
    box Backend
        participant RUST as Tauri IPC
        participant VFS as vfs::handlers::file_handlers
        participant BLOB as Blob Storage
        participant PPS as PdfProcessingService
        participant INDEX as Index Handlers
        participant LANCE as LanceVectorStore
    end

    User->>FCV: Drag & drop PDF file
    FCV->>FCV: Read file as base64
    
    FCV->>API: vfsFileApi.upload({ name, mimeType, base64Content })
    
    API->>RUST: invoke('vfs_upload_file', { params })
    
    Note over RUST,VFS: === Step 1: File Upload ===
    VFS->>VFS: Compute SHA256 hash
    VFS->>BLOB: Store blob (dedup by hash)
    VFS->>VFS: Insert file record in vfs_db.files
    
    alt File already exists (dedup)
        VFS-->>API: Return existing file + isNew: false
    else New file
        VFS-->>API: { file, sourceId, isNew: true }
    end
    
    API-->>FCV: UploadFileResult
    
    Note over FCV,STORE: === Step 2: Preview Rendering ===
    FCV->>FCV: Generate preview (first page render)
    FCV->>REF: resolveVfsRefs(sourceId)
    REF->>RUST: invoke('vfs_get_attachment_content', ...)
    RUST-->>REF: base64 content
    REF->>FCV: Update with resolved refs
    
    Note over PPS,STORE: === Step 3: Processing Pipeline Start ===
    FCV->>RUST: invoke('vfs_start_pdf_processing', { fileId: sourceId })
    RUST->>PPS: start_processing(sourceId)
    
    par Concurrent: Page Compression
        PPS->>PPS: Compress pages (parallel)
        PPS->>HOOK: emit('media-processing-progress', { fileId, status: { stage: "compressing", percent: 30 } })
        HOOK->>STORE: update(fileId, { stage: "compressing", percent: 30 })
    and Concurrent: OCR Processing
        PPS->>PPS: OCR each page (via PaddleOCR/LLM)
        PPS->>HOOK: emit('media-processing-progress', { fileId, status: { stage: "ocr_processing", percent: 65, readyModes: ["text"] } })
        HOOK->>STORE: update(fileId, { stage: "ocr_processing", percent: 65, readyModes: ["text"] })
        HOOK->>REF: invalidateResourceCache(fileId)
    end

    Note over INDEX,LANCE: === Step 4: Vector Indexing ===
    PPS->>PPS: Generate text chunks
    PPS->>INDEX: Vector index callback (injected via VfsIndexCoordinator)
    INDEX->>LANCE: Embed & store chunks
    
    PPS->>HOOK: emit('media-processing-completed', { fileId, stage: "completed", readyModes: ["text", "ocr", "image"] })
    HOOK->>STORE: setCompleted(fileId, ["text", "ocr", "image"])
    HOOK->>REF: invalidateResourceCache(fileId)
    STORE-->>FCV: Update UI: show processed indicator

    Note over User,LANCE: === Step 5: Search (post-processing) ===
    User->>FCV: Search for content within PDF
    FCV->>API: invoke('vfs_rag_search', { query, sourceId })
    API->>RUST: invoke('vfs_rag_search', { query, sourceId })
    RUST->>INDEX: rag_search()
    INDEX->>LANCE: Vector similarity search
    LANCE-->>INDEX: Matching chunks with scores
    INDEX-->>API: SearchResult[]
    API-->>FCV: Display search results with highlights
```

### Key Observations

- File upload deduplication happens by SHA256 hash inside `vfs_upload_file`
- The media processing pipeline runs asynchronously and reports progress via events
- `invalidateResourceCache()` is called both when new readyModes appear and on completion, ensuring `resolveVfsRefs` returns fresh data
- Legacy OCR (`pdf_ocr_progress`) coexists with the unified `media-processing-*` events; both update the same `pdfProcessingStore`
- The `PdfProcessingService` has `VfsIndexCoordinator` callback injected at app startup (`lib.rs:1875-1889`)

---

## b) Chat Message Flow

### Source Files

| Component | File | Key Lines |
|---|---|---|
| InputBar | `src/features/chat/InputBar.tsx` | Send handler |
| ChatStore | `src/features/chat/core/store/chatStore.ts` | State management |
| TauriAdapter | `src/features/chat/adapters/TauriAdapter.ts` | Lines 453-569: setup, Listeners 500-514 |
| EventBridge | `src/features/chat/core/middleware/eventBridge.ts` | `handleBackendEventWithSequence()` |
| Chat V2 Send | `src-tauri/src/chat_v2/handlers/send_message.rs` | `chat_v2_send_message` |
| Chat V2 Pipeline | `src-tauri/src/chat_v2/pipeline/` | Message processing |
| LLM Manager | `src-tauri/src/llm_manager/` | Provider routing + streaming |
| Chat V2 Events | `src-tauri/src/chat_v2/events.rs` | Lines 688-1349: EventEmitter |
| Streaming (LLM) | `src-tauri/src/llm_manager/streaming.rs` | Lines 485-590: citations |
| Chunk Buffer | `src/features/chat/core/middleware/chunkBuffer.ts` | `chunkBuffer` module |

### Full Sequence Diagram

```mermaid
sequenceDiagram
    box Frontend
        actor User
        participant IB as InputBar
        participant CS as ChatStore (Zustand)
        participant TA as TauriAdapter
        participant EB as eventBridge.ts
        participant CB as chunkBuffer.ts
        participant UI as React UI
    end
    box Backend
        participant IPC as Tauri IPC
        participant SEND as send_message.rs
        participant PL as ChatV2Pipeline
        participant LLM as LLMManager
        participant EVT as chat_v2/events.rs
        participant DB as ChatV2Database
    end

    User->>IB: Type message + Press Enter/Send
    IB->>CS: sendMessage(content, contextRefs)
    CS->>CS: ADD user message (optimistic)
    CS-->>UI: Re-render with user message
    
    CS->>TA: sendMessage(sessionId, content, ...)
    
    TA->>TA: buildSendOptionsSnapshot()
    TA->>TA: collectContextRefs()
    TA->>TA: buildSendContextRefs()  ← context refs for attachments
    
    TA->>IPC: invoke('chat_v2_send_message', { sessionId, content, contextRefs, ... })
    
    Note over SEND,DB: === Backend Processing ===
    SEND->>DB: Create/append message record
    SEND->>PL: process_message(message)
    
    PL->>EVT: emit('chat_v2_session_{sessionId}', SessionEvent::stream_start(messageId, modelId))
    
    Note over EVT,UI: === Frontend: Stream Start ===
    EVT-->>TA: Event received (session channel)
    TA->>EB: handleStreamStart({ messageId, modelId })
    EB->>CS: CREATE assistant message (pending)
    CS-->>UI: Show "AI is typing..." placeholder, model badge

    Note over PL,LLM: === LLM Invocation ===
    PL->>LLM: stream_response(prompt, tools, ...)
    
    loop Per tool round (max N rounds)
        Note over LLM: === Tool Call (if triggered) ===
        LLM->>EVT: emit('chat_v2_event_{sessionId}', BackendEvent::start("tool_call_preparing", payload: { toolName, toolCallId }))
        EVT-->>TA: Event received
        TA->>EB: handleBackendEventWithSequence(event)
        EB->>CS: CREATE "preparing" tool placeholder
        CS-->>UI: Show tool placeholder
        
        LLM->>EVT: emit('chat_v2_event_{sessionId}', BackendEvent::start("tool_call", blockId, payload: { toolName, toolInput }))
        EVT-->>TA: Event received
        TA->>EB: handleBackendEventWithSequence(event)
        EB->>CS: CREATE tool_call block (mcp_tool type)
        CS-->>UI: Show tool card with name + input
        
        LLM->>LLM: Execute tool (web_search, rag, anki, ...)
        
        alt Tool has streaming output
            loop Tool chunk events
                LLM->>EVT: emit_chunk("tool_call", blockId, chunk)
                EVT-->>TA: chunk event
                TA->>EB: handleBackendEventWithSequence
                EB->>CS: APPEND chunk to tool block
            end
        end
        
        LLM->>EVT: emit end("tool_call", blockId, result)
        EVT-->>TA: end event
        TA->>EB: handleBackendEventWithSequence
        EB->>CS: FINALIZE tool block with result
        
        LLM->>EVT: emit citations ({streamEvent}_web_search / _rag_sources)
        EVT-->>TA: citation event
        TA->>CS: UPDATE tool block with source citations
        CS-->>UI: Show citation sources
    end

    Note over LLM,UI: === Content Streaming ===
    LLM->>EVT: emit('chat_v2_event_{sessionId}', BackendEvent::start("thinking", messageId, blockId))
    EVT-->>TA
    TA->>EB: handleBackendEventWithSequence
    EB->>CS: CREATE thinking block
    
    loop Thinking chunks
        LLM->>EVT: emit_chunk("thinking", blockId, "Analyzing...")
        EVT-->>TA
        TA->>EB: handleBackendEventWithSequence
        EB->>CS: APPEND to thinking block
        CS-->>UI: Show reasoning in progress
    end
    
    LLM->>EVT: emit end("thinking", blockId)
    EVT-->>TA
    TA->>EB: handleBackendEventWithSequence
    EB->>CS: FINALIZE thinking block

    LLM->>EVT: emit('chat_v2_event_{sessionId}', BackendEvent::start("content", messageId, blockId))
    EVT-->>TA
    TA->>EB: handleBackendEventWithSequence
    EB->>CS: CREATE content block
    CB->>CB: Initialize chunk buffer for this block

    loop Content chunks (streamed text)
        LLM->>EVT: emit_chunk("content", blockId, "Hello world...")
        EVT-->>TA
        TA->>EB: handleBackendEventWithSequence(event)
        EB->>CB: Buffer chunk
        CB->>CS: Flush (debounced, ~50ms interval)
        CS-->>UI: Progressive text rendering (token-by-token)
    end
    
    LLM->>EVT: emit end("content", blockId)
    EVT-->>TA
    TA->>EB: handleBackendEventWithSequence
    EB->>CB: Flush remaining chunks
    CB->>CS: FINALIZE block
    CS-->>UI: Show complete response

    Note over SEND,UI: === Stream Complete ===
    SEND->>EVT: emit('chat_v2_session_{sessionId}', SessionEvent::stream_complete_with_usage(messageId, durationMs, usage))
    EVT-->>TA
    TA->>EB: handleStreamComplete({ messageId, durationMs, usage })
    EB->>autoSave: Auto-save session
    EB->>CS: UPDATE message status → "completed"
    EB->>CS: UPDATE token usage
    CS-->>UI: Show completion indicator, token stats, model info

    Note over TA,EB: Cleanup
    TA->>TA: clearEventContext()
    TA->>TA: resetBridgeState()

    alt Error during stream
        PL->>EVT: emit('chat_v2_session_{sessionId}', SessionEvent::stream_error(messageId, error))
        EVT-->>TA
        TA->>EB: handleStreamAbort({ messageId, error })
        EB->>CS: UPDATE message status → "error"
        CS-->>UI: Show error state with retry button
        
        TA->>TA: Attempt reconnection (retrySetupListeners)
    end

    alt User cancels
        User->>IB: Click Cancel
        CS->>TA: abortStream(sessionId, messageId)
        TA->>IPC: invoke('chat_v2_cancel_stream', { sessionId, messageId })
        PL->>LLM: Cancel LLM stream
        PL->>EVT: emit('chat_v2_session_{sessionId}', SessionEvent::stream_cancelled(messageId))
        EVT-->>TA
        TA->>EB: handleStreamAbort({ messageId })
        EB->>CS: UPDATE message status → "cancelled"
        CS-->>UI: Show "Generation cancelled"
    end
```

### Key Observations

- The frontend optimistically creates a user message before the `invoke()` call, enabling instant UI response
- Event sequence IDs are tracked per session to detect out-of-order or dropped events (`chat_v2/events.rs:713`)
- Chunk buffering (`chunkBuffer.ts`) debounces rapid content chunks into periodic store updates (typically ~50ms intervals)
- The `autoSave` middleware saves the session after stream completion
- Tool calls can span multiple rounds (tool loops), each round emitting block events independently
- In multi-model (variant) mode, `variant_start`/`variant_end` events bracket each model's block events

---

## c) Learning Hub Resource Open Flow

### Source Files

| Component | File | Key Lines |
|---|---|---|
| Learning Hub | `src/features/learning-hub/LearningHubSidebar.tsx` | Resource click handler |
| FileContentView | `src/features/learning-hub/views/FileContentView.tsx` | PDF viewer container |
| usePdfLoader | `src/features/learning-hub/hooks/usePdfLoader.ts` | PDF loading logic |
| VFS File API | `src/api/vfsFileApi.ts` | Lines 91-97 |
| pdfstream protocol | `src-tauri/src/pdf_protocol.rs` | HTTP Range Request handler |
| VFS Attachment | `src-tauri/src/vfs/handlers/attachment_handlers.rs` | `vfs_get_attachment_content` |
| VFS PDF handlers | `src-tauri/src/vfs/handlers/pdf_handlers.rs` | `vfs_get_blob_pdfstream_url` |
| EnhancedPdfViewer | `src/features/learning-hub/components/EnhancedPdfViewer.tsx` | react-pdf integration |

### Full Sequence Diagram

```mermaid
sequenceDiagram
    box Frontend
        actor User
        participant LH as LearningHubSidebar
        participant FCV as FileContentView
        participant PDFL as usePdfLoader
        participant EPV as EnhancedPdfViewer
        participant HOOK as usePdfProcessingProgress
        participant REF as vfsRefApiEnhancements
    end
    box Backend
        participant RUST as Tauri IPC
        participant VFS as vfs::handlers
        participant BLOB as Blob Storage
        participant PPS as PdfProcessingService
    end

    User->>LH: Click on textbook/resource
 
    LH->>LH: Determine resource type (PDF/image)
    LH->>FCV: Navigate to FileContentView(resourceId)

    Note over FCV,EPV: === Step 1: Load File Metadata ===
    FCV->>RUST: invoke('vfs_get_file', { fileId: resourceId })
    RUST->>VFS: vfs_get_file(resourceId)
    VFS-->>FCV: VfsFile { id, fileName, pageCount, blobHash, ... }

    FCV->>FCV: Check processing status from store

    alt Not processed yet
        FCV->>RUST: invoke('vfs_start_pdf_processing', { fileId: resourceId })
        RUST->>PPS: start_processing(resourceId)
        Note over PPS,HOOK: Processing pipeline (async, events via media-processing-*)
        PPS->>HOOK: emit('media-processing-progress', { fileId, status: { ... } })
        HOOK->>HOOK: Update pdfProcessingStore
    end

    Note over FCV,EPV: === Step 2: Load PDF Content ===
    FCV->>RUST: invoke('vfs_get_blob_pdfstream_url', { fileId: resourceId })
    RUST->>VFS: vfs_get_blob_pdfstream_url(resourceId)
    VFS->>BLOB: Look up blob storage path
    VFS-->>FCV: "pdfstream://blobs/{blobHash}/{fileName}.pdf"

    FCV->>PDFL: init(url = pdfstream://...)
    
    Note over PDFL,EPV: === Step 3: Render PDF (page-by-page) ===
    PDFL->>EPV: Provide pdfstream URL
    EPV->>EPV: react-pdf reads via pdfstream:// protocol
    
    EPV->>RUST: HTTP GET pdfstream://blobs/{hash}/file.pdf
    Note over RUST: Request handled by pdf_protocol.rs handle_asset_protocol()
    RUST->>BLOB: Read blob file
    RUST-->>EPV: 200 OK (full PDF bytes for initial load)
    
    EPV->>RUST: HTTP Range Request (bytes=0-8191) — Range header
    RUST->>BLOB: Seek and read partial content
    RUST-->>EPV: 206 Partial Content (first 8KB)

    loop For each page as user scrolls
        EPV->>RUST: HTTP Range Request for specific byte range
        RUST->>BLOB: Read range from blob
        RUST-->>EPV: 206 Partial Content with Content-Range header
        EPV->>EPV: Render page using react-pdf
    end

    Note over FCV,REF: === Step 4: Resolve OCR/Text Content ===
    alt OCR text requested (e.g., for RAG context)
        FCV->>REF: resolveVfsRefs(resourceId, mode: "ocr")
        REF->>STORE: Check pdfProcessingStore for readyModes
        STORE-->>REF: readyModes includes "ocr"
        
        REF->>RUST: invoke('vfs_get_resource_ocr_info', { resourceId })
        RUST->>VFS: vfs_get_resource_ocr_info(resourceId)
        VFS-->>REF: ResourceOcrInfo { ocrText, extractedText, activeSource }
        REF-->>FCV: OCR text content
    end

    Note over FCV,REF: === Step 5: PDF Page Image for RAG ===
    alt Page image needed (e.g., for LLM context)
        FCV->>RUST: invoke('vfs_get_pdf_page_image', { resourceId, pageIndex: 3 })
        RUST->>VFS: vfs_get_pdf_page_image(resourceId, 3)
        VFS->>BLOB: Read page_image_{3}.jpg
        RUST-->>FCV: base64 encoded JPEG
        FCV->>FCV: Use in LLM context / display
    end
```

### Key Observations

- The `pdfstream://` protocol (`lib.rs:1729-1756`, `pdf_protocol.rs`) is a custom Tauri URI scheme that handles HTTP Range Requests, enabling react-pdf to efficiently load PDFs page-by-page without loading the entire file into memory
- Processing pipeline runs asynchronously; the UI displays processing status from the store while waiting
- `resolveVfsRefs` checks both the in-memory store (for cached resolved refs) and the backend (for fresh OCR/text results)
- OCR text chunks are stored in VFS and indexed in Lance for vector search; the `vfs_get_pdf_page_image` command provides page-level images for multimodal RAG
- The flow supports both initial full PDF load (first HTTP GET returns full bytes) and efficient partial-range loading (subsequent Range requests return 206 Partial Content)

---

## Flow Dependency Graph

```mermaid
flowchart LR
    subgraph FlowA["Flow A: PDF Upload→OCR→Search"]
        UPLOAD["vfs_upload_file"] -->|"triggers"| PROCESS["vfs_start_pdf_processing"]
        PROCESS -->|"progress via"| EVENTS["media-processing-* events"]
        PROCESS -->|"triggers"| INDEX["VFS Vector Indexing"]
        INDEX -->|"enables"| SEARCH["vfs_rag_search"]
    end

    subgraph FlowB["Flow B: Chat Message"]
        SEND["chat_v2_send_message"] -->|"invokes"| PIPELINE["ChatV2Pipeline"]
        PIPELINE -->|"streams via"| CHAT_EVTS["chat_v2_event_*"]
        PIPELINE -->|"calls"| LLM_API["LLM Provider API"]
        LLM_API -->|"triggers"| TOOLS["Tool Execution"]
        TOOLS -->|"may call"| RAG["vfs_rag_search"]
        TOOLS -->|"may call"| ANKI["Enhanced Anki"]
    end

    subgraph FlowC["Flow C: Resource Open"]
        OPEN["vfs_get_file"] -->|"provides"| META["File metadata"]
        META -->|"URL via"| PDFSTREAM["pdfstream:// protocol"]
        PDFSTREAM -->|"Range requests"| RENDER["Page rendering"]
        OPEN -->|"triggers"| PROCESS2["Processing check"]
        PROCESS2 -->|"may start"| PROCESS
    end

    FLOW_A -.->|"provides data for"| FLOW_B
    FLOW_C -.->|"provides context for"| FLOW_B
```

---

## Event-Driven Architecture Summary

The application uses a **hybrid invoke + event-driven** architecture:

1. **Command Pattern**: Frontend calls `invoke('command_name', args)` for request-response operations (CRUD, configuration)
2. **Event Pattern**: Backend emits events via `window.emit()` for streaming and state changes (chat tokens, processing progress)
3. **Custom Protocol**: `pdfstream://` for efficient binary data streaming (PDF file serving)

```
Invoke (request-response)       Event (streaming/async)
┌──────────┐                    ┌──────────┐
│ Frontend │──invoke('cmd')──▶  │ Backend  │
│          │◀───Result────────  │          │
└──────────┘                    └──────────┘

                                ┌──────────┐
                                │ Backend  │──emit('event', payload)──▶  ┌──────────┐
                                │          │                              │ Frontend │
                                └──────────┘                              │(listen)  │
                                                                          └──────────┘
```
