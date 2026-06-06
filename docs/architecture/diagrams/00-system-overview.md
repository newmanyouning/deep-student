# System Overview -- C4 Architecture Diagrams

> This document describes the Deep Student system at the C4 Model's System Context and Container levels.
> Generated from source code analysis of `src-tauri/` (Rust backend) and `src/` (React frontend).

---

## System Context Diagram

The diagram below shows Deep Student as a single system interacting with external actors and services.

```mermaid
C4Context
  title System Context Diagram -- Deep Student

  Person(user, "Student / User", "Learner using the desktop application for study, review, and Anki card creation")

  System_Boundary(deep_student, "Deep Student") {
    System(app, "Deep Student Desktop App", "Tauri v2 desktop application for intelligent learning support")
  }

  System_Ext(anki_connect, "AnkiConnect", "Anki desktop add-on exposing HTTP API for card/deck management")
  System_Ext(paddle_ocr, "PaddleOCR API", "OCR engine accessed via REST API for text extraction from images")
  System_Ext(llm_apis, "LLM API Providers", "DeepSeek, OpenAI, Anthropic, SiliconFlow, and other LLM APIs")
  System_Ext(web_search, "Web Search Engines", "Bing, Google, or configurable search providers for RAG")
  System_Ext(cloud_sync, "Cloud Sync (WebDAV/S3)", "Remote storage for backup synchronization across devices")
  System_Ext(secure_store, "OS Secure Store", "Platform credential storage (Keychain, Credential Manager, libsecret)")

  Rel(user, app, "Uses", "GUI interaction")
  Rel(app, anki_connect, "HTTP requests", "Create decks, add cards, check status")
  Rel(app, paddle_ocr, "REST API", "OCR text extraction from PDFs/images")
  Rel(app, llm_apis, "HTTP/streaming", "Chat completions, embeddings, translations")
  Rel(app, web_search, "HTTP requests", "Web search for current information")
  Rel(app, cloud_sync, "WebDAV/S3", "Backup upload/download/versioning")
  Rel(app, secure_store, "OS API", "Credential storage & retrieval")

  UpdateLayoutConfig($c4ShapeInRow="3", $c4BoundaryInRow="2")
```

### External Systems Reference

| Actor | Technology | Protocol | Purpose |
|-------|-----------|----------|---------|
| AnkiConnect | Anki Add-on (HTTP) | REST over `http://127.0.0.1:8765` | Deck management, card creation |
| PaddleOCR API | Standalone / Docker | REST | OCR text extraction from images |
| LLM APIs | DeepSeek / OpenAI / Anthropic / etc | HTTP SSE streaming | Chat completions, embeddings |
| Web Search | Bing Search API / configurable | HTTP | Real-time search for RAG |
| Cloud Sync | WebDAV / S3 compatible | HTTP(s) | Cross-device backup sync |
| OS Secure Store | Keychain / libsecret / WinCred | Platform SDK | Encrypted credential storage |

---

## Container Diagram

The diagram below shows the major containers (runtime processes/datastores) within Deep Student.

```mermaid
C4Container
  title Container Diagram -- Deep Student Internal Architecture

  Person(user, "Student / User", "Desktop application user")

  System_Boundary(de, "Deep Student") {

    Container(webview, "Tauri WebView", "System WebView2/WKWebView", "Hosts the React SPA frontend")
    Container(backend, "Tauri Rust Backend", "Rust binary + Tauri v2", "All business logic, database access, external API orchestration")

    ContainerDb(main_db, "Main SQLite Database", "mistakes.db", "Chat messages, mistakes, settings, document tasks, anki_cards, templates, review data")
    ContainerDb(vfs_db, "VFS SQLite Database", "vfs.db", "Resources, notes, files, folders, exam_sheets, questions, translations, essays, mindmaps, index units, review plans, blobs")
    ContainerDb(chat_v2_db, "Chat V2 SQLite Database", "chat_v2.db", "Sessions, messages, blocks, attachments, session state, resources, workspace, agents, subagent tasks")
    ContainerDb(llm_usage_db, "LLM Usage SQLite Database", "llm_usage.db", "Token usage logs, cost tracking, per-caller and per-model statistics")
    ContainerDb(audit_db, "Audit SQLite Database", "audit.db", "Data governance audit logs, migration history")
    ContainerDb(question_db, "Question Bank SQLite Database", "question_bank.db", "Questions, answer submissions, learning history, heat map data")

    Container(file_store, "File System Storage", "App data directory", "PDF blobs, OCR images, attachment files, model downloads")

    Rel(user, webview, "GUI interactions", "Click, type, scroll")
    Rel(webview, backend, "Tauri IPC (invoke)", "Commands + JSON serialized results")
    Rel(backend, main_db, "rusqlite", "CRUD operations")
    Rel(backend, vfs_db, "rusqlite via r2d2 pool", "Unified resource storage CRUD")
    Rel(backend, chat_v2_db, "rusqlite via r2d2 pool", "Chat sessions, messages, blocks")
    Rel(backend, llm_usage_db, "rusqlite", "Usage statistics writes/queries")
    Rel(backend, audit_db, "rusqlite", "Audit log writes/queries")
    Rel(backend, question_db, "rusqlite", "Question bank CRUD")
    Rel(backend, file_store, "std::fs", "Read/write blobs, images, cached files")
  }

  System_Ext(anki_connect, "AnkiConnect", "Anki desktop add-on")
  System_Ext(paddle_ocr, "PaddleOCR API", "OCR service")
  System_Ext(llm_apis, "LLM API Providers", "DeepSeek, OpenAI, etc.")
  System_Ext(cloud_sync, "Cloud Sync", "WebDAV / S3")

  Rel(backend, anki_connect, "HTTP", "Card operations")
  Rel(backend, paddle_ocr, "HTTP", "OCR requests")
  Rel(backend, llm_apis, "HTTP/Stream", "LLM calls")
  Rel(backend, cloud_sync, "HTTP", "Backup sync")

  UpdateLayoutConfig($c4ShapeInRow="3", $c4BoundaryInRow="2")
```

### Container Summary

| Container | Technology | Persistence | Key Responsibility |
|-----------|-----------|-------------|-------------------|
| Tauri WebView | React 18 + TypeScript | None (ephemeral) | UI rendering, user interaction |
| Tauri Rust Backend | Rust 1.96 + Tauri 2 | See databases | All business logic, external API calls |
| Main SQLite DB (`mistakes.db`) | SQLite via rusqlite | File-based | Chat legacy, mistakes, settings, anki_cards, document_tasks |
| VFS SQLite DB (`vfs.db`) | SQLite via r2d2 pool | File-based | Unified resources, notes, files, folders, questions, review plans |
| Chat V2 SQLite DB (`chat_v2.db`) | SQLite via r2d2 pool | File-based | Chat sessions, messages, blocks, attachments, workspace agents |
| LLM Usage DB (`llm_usage.db`) | SQLite via rusqlite | File-based | Token/cost statistics |
| File System | Local OS filesystem | Directory on disk | PDF blobs, images, attachments, vector store (Lance) |

---

## Data Flow: Core Use Cases

### Chat with LLM
```
User Input --> WebView --> IPC invoke --> chat_v2::handlers
  --> LLMManager (calls external API) --> Streaming blocks written to DB
  --> Events emitted to WebView --> UI updated
```

### PDF OCR Processing
```
User uploads PDF --> VFS file_handlers --> PdfProcessingService
  --> PaddleOCR API (HTTP) --> OCR text stored in vfs.db
  --> Index units created --> Lance vector store indexed
```

### Question Bank Review (SM-2)
```
User answers question --> qbank_submit_answer
  --> review_plan_service (SM-2 algorithm) --> review_plans table updated
  --> review_history recorded --> Next review date computed
```

### Anki Card Generation
```
Document uploaded --> document_tasks created --> LLM generates cards
  --> anki_cards table populated --> AnkiConnect HTTP API (export)
```

---

## Legend

- **Person** (human actor): Green circle
- **System** (the software system): Blue rectangle
- **Container** (runtime process/datastore): Blue rectangle with database cylinder for DBs
- **System_Ext** (external system): Grey/yellow rectangle
- **Rel**: Relationship arrow with label
- **Rel_D**: Relationship to database

---

## Key Source References

| Item | File | Lines |
|------|------|-------|
| Module declarations | `src-tauri/src/lib.rs` | 6-92 |
| App initialization | `src-tauri/src/lib.rs` | 169-174, 268-842 |
| Main DB schema | `src-tauri/src/database/manager.rs` | 220-425 |
| VFS DB schema | `src-tauri/migrations/vfs/V20260130__init.sql` | 1-800+ |
| Chat V2 DB schema | `src-tauri/migrations/chat_v2/V20260130__init.sql` | 1-230+ |
| LLM Usage DB schema | `src-tauri/migrations/llm_usage/V20260130__init.sql` | 1-60+ |
| Tauri command registration | `src-tauri/src/lib.rs` | 847-1727 |
