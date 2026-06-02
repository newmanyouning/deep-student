# Functional Group Analysis: `dstu`

**Analysis date**: 2026-06-01  
**Group**: DSTU (Deep Study -- Unified Finder Protocol)  
**Module path**: `src-tauri/src/dstu/` (Rust backend)  
**Frontend**: `src/dstu/` (TypeScript)  
**Total Rust files**: 27 files  
**Total Rust LOC**: 16,024

---

## 1. Purpose

DSTU ("DS-Tauri-Unified Finder Protocol") is described as a protocol layer analogous to an OS file manager. It provides a unified, filesystem-semantics API (paths, directories, move/copy/list) over all resource types stored in VFS (notes, textbooks, exams, translations, essays, images, files, mind maps, folders). The frontend interacts with DSTU commands rather than accessing VFS repos directly.

---

## 2. File Inventory with LOC

| File | LOC | Role |
|------|-----|------|
| `handlers.rs` | 6,382 | God file: ALL 28+ resource Tauri commands |
| `folder_handlers.rs` | 1,097 | 16 folder-management Tauri commands |
| `trash_handlers.rs` | 798 | Recycle bin: soft-delete, restore, list, purge, empty |
| `handler_utils/search_helpers.rs` | 853 | Cross-type search functions |
| `handler_utils/content_helpers.rs` | 804 | Content read/write by type |
| `handler_utils/node_converters.rs` | 716 | VFS-type to DstuNode conversion |
| `path_parser.rs` | 741 | Path normalization, parse/build, validation |
| `path_types.rs` | 412 | ParsedPath struct, resource ID prefix mapping |
| `handler_utils/list_helpers.rs` | 388 | Smart-folder listing by type |
| `export/mod.rs` | 382 | Export system + ExportRegistry |
| `exam_formatter.rs` | 350 | Exam preview to ContentBlock formatting |
| `handler_utils/delete_helpers.rs` | 333 | Soft-delete/restore/purge by type |
| `handler_utils/crud.rs` | 269 | get_resource_by_type_and_id, fetch, fallback |
| `types.rs` | 1,007 | DstuNode, DstuNodeType, DstuListOptions, DstuCreateOptions, DstuWatchEvent, BatchMoveRequest, PathCacheEntry, ResourceLocation, SubjectMigrationStatus |
| `error.rs` | 174 | DstuError enum + DstuResult + From impls |
| `handler_utils/path_utils.rs` | 154 | extract_resource_info, infer_resource_type_from_id, is_uuid_format |
| `export/essay_adapter.rs` | 180 | Essay export adapter |
| `export/translation_adapter.rs` | 137 | Translation export adapter |
| `export/exam_adapter.rs` | 136 | Exam export adapter |
| `export/mindmap_adapter.rs` | 150 | MindMap export adapter |
| `export/file_adapter.rs` | 93 | File export adapter |
| `export/image_adapter.rs` | 90 | Image export adapter |
| `export/note_adapter.rs` | 90 | Note export adapter |
| `export/textbook_adapter.rs` | 84 | Textbook export adapter |
| `mod.rs` | 143 | Module root, re-exports |
| `handler_utils/mod.rs` | 61 | Sub-module declarations, re-exports |
| **Total** | **16,024** | |

---

## 3. External Crate Dependencies

### Directly used by DSTU source files

| Crate | Classification | Usage | Notes |
|-------|---------------|-------|-------|
| `serde` / `serde_json` | **ESSENTIAL** | Serialize structs, deserialize Tauri args, parse JSON metadata | Cannot remove |
| `thiserror` | **ESSENTIAL** | `#[derive(Error)]` for DstuError | Cannot remove |
| `tauri` | **ESSENTIAL** | `#[tauri::command]`, `State<>`, `Window`, event emission | This IS a Tauri command module |
| `rusqlite` | **ESSENTIAL** | SQLite queries in breadcrumbs, search, content helpers | Cannot remove |
| `chrono` | **ESSENTIAL** | Timestamp parsing (RFC3339, SQLite format), Utc::now() | Cannot remove |
| `base64` | **ESSENTIAL** | Binary file upload/download encoding | Cannot remove |
| `tokio` | **ESSENTIAL** | `spawn_blocking` for DB operations, async commands | Cannot remove |
| `tracing` / `log` | **ESSENTIAL** | Structured logging, instrumentation | Cannot remove |
| `uuid` | **ESSENTIAL** | UUID generation for temp exam IDs | Cannot remove |
| `nanoid` | **REPLACEABLE** | Resource ID generation | Could be replaced by uuid::Uuid::new_v4().to_string() with prefix stripping |
| `tempfile` | **ESSENTIAL** | Export temp file path | Used in export adapters |

### Indirect dependencies (brought in via VFS)

The massive coupling to `crate::vfs` pulls in all of VFS's transitive dependencies as indirect dependencies. This includes the full LanceDB stack, OCR pipeline, PDF renderer, document parsers (docx-rs, calamine, pptx-to-md, csv, quick-xml, rtf-parser), and more. These are not directly imported by DSTU but are compile-time dependencies.

---

## 4. Internal Coupling Analysis

### Sub-module call graph

```
src/dstu/mod.rs
├── types.rs          (standalone, only serde)
├── error.rs          (standalone, only serde/thiserror + ctx From<..>)
├── path_types.rs     (standalone, only serde)
├── path_parser.rs    → path_types, error
├── handlers.rs       → error, types, path_parser, handler_utils/*, trash_handlers, VFS (17+ types)
├── folder_handlers.rs → error, VFS (7 types)
├── trash_handlers.rs → error, handler_utils/node_converters, types, VFS (all repos)
├── exam_formatter.rs → VFS, chat_v2, models
├── export/mod.rs     → error, handler_utils/path_utils, types, VFS
│   ├── note_adapter.rs    → VFS
│   ├── textbook_adapter.rs → VFS
│   ├── exam_adapter.rs    → VFS
│   ├── translation_adapter.rs → VFS
│   ├── essay_adapter.rs   → VFS
│   ├── image_adapter.rs   → VFS
│   ├── file_adapter.rs    → VFS
│   └── mindmap_adapter.rs → VFS
└── handler_utils/mod.rs (re-exports)
    ├── node_converters.rs   → path_parser, types, VFS (8 types), unified_file_manager
    ├── path_utils.rs        → error, path_parser
    ├── crud.rs              → types, VFS (all repos)
    ├── list_helpers.rs      → types, VFS (all repos)
    ├── delete_helpers.rs    → error, VFS (all repos)
    ├── content_helpers.rs   → error, VFS (6 types)
    └── search_helpers.rs    → types, VFS (all repos + VfsResourceRepo)
```

### Cross-module import count (outside DSTU)

| External module | Imported by (DSTU files) | Count |
|----------------|-------------------------|-------|
| `crate::vfs::*` | ALL 27 files (directly or via handler_utils) | 27+ |
| `crate::chat_v2::resource_types` | exam_formatter.rs | 1 |
| `crate::models::*` | exam_formatter.rs | 1 |
| `crate::background_tasks` | handlers.rs, trash_handlers.rs | 2 |
| `crate::unified_file_manager` | node_converters.rs | 1 |

**Conclusion**: DSTU has a **monodirectional but extreme** coupling to the VFS module. Virtually every operation is a thin delegation to a VFS repository. There is also a small but architecturally significant coupling to `chat_v2` and `models` in `exam_formatter.rs`.

---

## 5. Code Duplication and Overlapping Functionality

### Critical: 5+ resource-type inference functions

| Function | Location | Returns | Prefixes covered |
|----------|----------|---------|-----------------|
| `DstuNodeType::from_str()` | `types.rs:67` | `Option<DstuNodeType>` | note, textbook, exam, translation, essay, image, file, retrieval, mindmap, + Chinese aliases |
| `item_type_to_dstu_node_type()` | `node_converters.rs:146` | `Option<DstuNodeType>` | note, textbook, exam, translation, essay, image, file, folder, mindmap |
| `infer_resource_type_from_id()` | `path_utils.rs:90` | `&'static str` (plural) | note_, file_, tb_, att_, img_, tr_, exam_, essay_, fld_, mm_, res_ + UUID |
| `get_resource_type_from_id()` | `path_types.rs:235` | `Option<String>` (singular) | note_, tb_, exam_, tr_, essay_, fld_, att_, img_, file_, mm_ |
| `DstuParsedPath::infer_resource_type()` | `types.rs:709` | `Option<String>` (singular) | note_, file_, tb_, att_, exam_, tr_, essay_, fld_, mm_ |

All five functions map resource ID prefixes to types, but with different return types (enum vs string) and different prefix sets (e.g., `res_` is only in `infer_resource_type_from_id`; `Retrieval` is only in `DstuNodeType`). Any change to prefix mappings requires updating all five.

### Critical: Resource-type dispatch switch statements

The pattern:
```rust
match resource_type {
    "notes" => VfsNoteRepo::...,
    "textbooks" => VfsTextbookRepo::...,
    "exams" => VfsExamRepo::...,
    ...
}
```

This identical switch appears in approximately 40-50+ locations:
- `handlers.rs`: dstu_get, dstu_create, dstu_update, dstu_delete, dstu_move, dstu_rename, dstu_copy, is_subfolder_of, dstu_batch_move, dstu_set_favorite, dstu_search (11+ times)
- `folder_handlers.rs`: validate_string_input, dstu_folder_add_item (2 times)
- `trash_handlers.rs`: is_resource_in_trash, lookup_resource_id, dstu_soft_delete, dstu_trash_restore, dstu_list_trash, dstu_empty_trash, dstu_permanently_delete (7 times)
- `crud.rs`: get_resource_by_type_and_id, fetch_resource_as_dstu_node, fallback_lookup_uuid_resource (3 times)
- `delete_helpers.rs`: delete_resource_by_type, delete_resource_by_type_with_conn, purge_resource_by_type, restore_resource_by_type, restore_resource_by_type_with_conn (5 times)
- `content_helpers.rs`: get_content_by_type, get_content_by_type_paged, update_content_by_type (3 times)
- `search_helpers.rs`: search_all (1 time -- but searches 9 types)
- `list_helpers.rs`: list_resources_by_type_with_folder_path (1 time -- 9 match arms)
- `path_utils.rs`: extract_resource_info (1 time -- delegates to infer_resource_type_from_id)

Adding a new resource type (e.g., "flashcard") requires editing all 40+ locations.

### Moderate: Two overlapping `ParsedPath` structs

- `path_types::ParsedPath` (exported as `RealParsedPath`, also as `NewParsedPath`) -- 15 fields including `virtual_type`
- `types::DstuParsedPath` -- 7 fields, no `virtual_type`

Both represent the same concept (a parsed DSTU path) but with different field sets. `DstuParsedPath` appears to be the older version; `ParsedPath` is the "C1 contract" version.

---

## 6. Error Propagation Pattern

```
External errors → VfsError, rusqlite::Error, serde_json::Error, io::Error
       ↓
  DstuError (via From impls)
       ↓
  DstuResult<T> = Result<T, DstuError>
       ↓
  String (via From<DstuError>) ← Tauri command return
```

The pattern is well-structured with proper `From` conversions for all expected error types (`VfsError`, `rusqlite::Error`, `serde_json::Error`, `std::io::Error`, `String`). However, many locations bypass this with `DstuError::from(e.to_string())` instead of relying on `From` impls, losing type information.

Several helper modules (`crud.rs`, `content_helpers.rs`, `search_helpers.rs`, `list_helpers.rs`, `delete_helpers.rs`) return `Result<_, String>` rather than `DstuResult<_>`, which is inconsistent.

---

## 7. SOLID Assessment

| Principle | Assessment | Details |
|-----------|-----------|---------|
| **S**ingle Responsibility | **FAIL** | `handlers.rs` (6382 lines) handles routing, validation, event emission, vector cleanup, error conversion -- far too many concerns |
| **O**pen/Closed | **FAIL** | Every new resource type requires edits to 40+ switch statements across the entire group |
| **L**iskov Substitution | **OK** | Standard Rust type patterns, no inheritance abuse |
| **I**nterface Segregation | **N/A** | No trait implementations beyond `ResourceExportAdapter` |
| **D**ependency Inversion | **PARTIAL** | `ResourceExportAdapter` trait in export/ is good; but DSTU as a whole depends directly on concrete VFS repos rather than abstractions |

---

## 8. DSTU's Relationship to VFS

### Can DSTU exist independently of VFS? **No.**

DSTU is fundamentally a thin protocol/adapter layer over VFS. It:
- Delegates every database operation to VFS repos
- Imports VFS types directly in every file
- Uses `VfsDatabase` as its primary state dependency
- Uses `From<VfsError>` for all error conversion
- Has no independent storage layer

If VFS were removed, DSTU would need its own database access layer -- essentially reimplementing VFS repositories.

### What would independence require?

1. Abstract the VFS dependencies behind traits (e.g., `ResourceRepository` trait)
2. Define DSTU's own storage types instead of reusing VFS types
3. Move the resource-type dispatch into a strategy pattern

---

## 9. Frontend TypeScript Counterpart

The `src/dstu/` TypeScript directory contains 41 files (~5000+ LOC) covering:
- `types.ts` + `types/` -- Frontend DSTU types matching Rust structs
- `api.ts` + `api/` -- Tauri command invocations
- `adapters/` -- Resource-type adapters (notes, exams, textbooks, etc.)
- `editors/` -- Editor wrappers (NoteEditor, ExamEditor, etc.)
- `hooks/` -- React hooks for DSTU state management
- `factory.ts`, `naming.ts`, `encoding.ts`, `logger.ts`, `openResource.ts`

The frontend mirror likely has similar resource-type switch duplication.

---

## 10. Health Score: **5 / 10**

### Breakdown

| Metric | Score | Rationale |
|--------|-------|-----------|
| Architectural vision | 8/10 | Well-motivated facade/protocol layer concept |
| Error handling pattern | 7/10 | Good DstuError design, but inconsistently applied |
| Test coverage | 7/10 | Strong unit tests in types.rs, path_parser.rs, path_types.rs, node_converters.rs |
| Code duplication | 2/10 | Extreme -- 40+ switch statements, 5+ type inference functions |
| File size distribution | 2/10 | `handlers.rs` at 6382 lines is a critical maintainability problem |
| Modularity / Cohesion | 4/10 | Sub-modules are logically organized but tightly coupled internally |
| Dependency management | 3/10 | Monolithic VFS coupling, no abstraction layer |
| Open/Closed compliance | 2/10 | Adding a new resource type is a cross-cutting change touching 40+ locations |

### Recommendation: Mitigation priorities

1. **Refactor `handlers.rs`** -- Split into per-resource-type handler files (e.g., `note_handlers.rs`, `exam_handlers.rs`), reducing the god file
2. **Introduce a `ResourceDispatcher` trait** -- Centralize the resource-type switch into a single trait with per-type implementations
3. **Consolidate type-inference functions** -- Merge the 5+ functions into one canonical source
4. **Remove duplicate `ParsedPath`** -- Keep only `path_types::ParsedPath` and remove `types::DstuParsedPath`
5. **Normalize error returns** -- Convert all `Result<_, String>` helpers to use `DstuResult<T>`

---

## 11. Key File Paths

- **Module root**: `C:\deep-student\src-tauri\src\dstu\mod.rs`
- **God handler**: `C:\deep-student\src-tauri\src\dstu\handlers.rs`
- **Folder handlers**: `C:\deep-student\src-tauri\src\dstu\folder_handlers.rs`
- **Trash handlers**: `C:\deep-student\src-tauri\src\dstu\trash_handlers.rs`
- **Error types**: `C:\deep-student\src-tauri\src\dstu\error.rs`
- **Core types**: `C:\deep-student\src-tauri\src\dstu\types.rs`
- **Path types**: `C:\deep-student\src-tauri\src\dstu\path_types.rs`
- **Path parser**: `C:\deep-student\src-tauri\src\dstu\path_parser.rs`
- **Export system**: `C:\deep-student\src-tauri\src\dstu\export\mod.rs`
- **Node converters**: `C:\deep-student\src-tauri\src\dstu\handler_utils\node_converters.rs`
- **CRUD helpers**: `C:\deep-student\src-tauri\src\dstu\handler_utils\crud.rs`
- **Path utils**: `C:\deep-student\src-tauri\src\dstu\handler_utils\path_utils.rs`
- **Frontend root**: `C:\deep-student\src\dstu\index.ts`
- **Frontend types**: `C:\deep-student\src\dstu\types.ts`
