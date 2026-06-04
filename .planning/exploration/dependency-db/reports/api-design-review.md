# 后端 API 统一设计 — 模块分析与改进方案

**日期**: 2026-05-29
**数据来源**: api_functions 表 (697 个 Tauri 命令, 31 个模块)

---

## 全局统计

| 指标 | 数值 |
|------|------|
| Tauri 命令总数 | **697** |
| 涉及的 Rust 模块 | **31** |
| 使用 `State<'_, AppState>` | 268 (38%) |
| 使用 `String` 参数 | 510+ (73%) |
| 使用 `Result<T, AppError>` 返回 | ~550 (79%) |
| 使用 `serde_json::Value` 返回 | ~120 (17%) |
| 一致的模块前缀 (如 `chat_v2_`) | ~60% |
| 无前缀的孤立命令 | ~40% |

---

## 各模块 API 分析

### 1. Chat V2 — 78 命令

```
核心模式: State<'_, Arc<ChatV2Database>> + String + request struct
返回类型: Result<T, String>
```

**输入特征**:
| 参数 | 次数 | 类型 |
|------|------|------|
| `String` | 92 | session_id, tag, content... |
| `State<ChatV2Database>` | 51 | 数据库状态 |
| `State<WorkspaceCoordinator>` | 18 | 工作区 |
| `Option<String>` | 12 | 可选参数 |

**问题**:
- 返回类型用 `Result<T, String>` 而非 `Result<T, AppError>` — 与其他模块不一致
- `chat_v2_anki_cards_result` 和 `chat_v2_canvas_edit_result` 通过 AppHandle 发事件而非返回值
- 没有统一的请求体封装 — 部分用单独参数，部分用 request struct

**建议的统一字段**:
```rust
// 所有 Chat V2 命令应该:
// 1. 返回 Result<T, ChatV2Error> (不是 String)
// 2. 需要数据库的命令统一用 State<'_, Arc<ChatV2Database>>
// 3. 复杂参数封装为 request struct
pub struct ChatV2Api {
    // Session
    create_session(input: CreateSessionInput) -> Result<ChatSession, ChatV2Error>,
    delete_session(session_id: Uuid) -> Result<(), ChatV2Error>,
    load_session(session_id: Uuid) -> Result<ChatSession, ChatV2Error>,
    branch_session(session_id: Uuid) -> Result<ChatSession, ChatV2Error>,

    // Message
    send_message(input: SendMessageInput) -> Result<SendMessageOutput, ChatV2Error>,
    delete_message(session_id: Uuid, message_id: Uuid) -> Result<(), ChatV2Error>,

    // Group
    create_group(input: CreateGroupInput) -> Result<SessionGroup, ChatV2Error>,
    update_group(group_id: Uuid, input: UpdateGroupInput) -> Result<SessionGroup, ChatV2Error>,
    delete_group(group_id: Uuid) -> Result<(), ChatV2Error>,

    // Variant
    switch_variant(input: SwitchVariantInput) -> Result<(), ChatV2Error>,
    cancel_variant(session_id: Uuid, variant_id: Uuid) -> Result<(), ChatV2Error>,

    // Tool
    tool_approval_respond(input: ToolApprovalInput) -> Result<(), ChatV2Error>,
    ask_user_respond(input: AskUserInput) -> Result<(), ChatV2Error>,
}
```

### 2. VFS — 119 命令 (含 Pomodoro/Voice!)

```
问题: VFS 模块混合了文件操作 + 番茄钟 + 语音输入 + Todo
需要拆分
```

**实际职责分布**:
| 子域 | 命令数 | 应该属于 |
|------|--------|---------|
| 文件 CRUD | ~30 | VFS |
| 文件夹/路径 | ~20 | VFS |
| 向量索引 | ~15 | VFS |
| PDF 处理 | ~10 | VFS (或独立) |
| **番茄钟 (Pomodoro)** | ~8 | ❌ 应该独立模块 |
| **待办事项 (Todo)** | ~15 | ❌ 应该独立模块 |
| **语音输入** | ~5 | ❌ 应该独立模块 |
| 附件/Blob | ~10 | VFS |

**建议**: 将 Pomodoro/Todo/Voice 命令从 VFS 中拆出为独立模块。

**建议的统一字段**:
```rust
pub struct VfsApi {
    // File
    create_file(input: CreateFileInput) -> Result<VfsFile, VfsError>,
    get_file(file_id: Uuid) -> Result<VfsFile, VfsError>,
    update_file(file_id: Uuid, input: UpdateFileInput) -> Result<VfsFile, VfsError>,
    delete_file(file_id: Uuid) -> Result<(), VfsError>,
    list_files(folder_id: Option<Uuid>, filters: FileFilters) -> Result<Vec<VfsFile>, VfsError>,

    // Folder
    create_folder(input: CreateFolderInput) -> Result<VfsFolder, VfsError>,
    delete_folder(folder_id: Uuid) -> Result<(), VfsError>,
    move_resource(input: MoveResourceInput) -> Result<(), VfsError>,

    // Index
    get_index_status(resource_id: Uuid) -> Result<IndexStatus, VfsError>,
    reindex_resource(resource_id: Uuid, mode: IndexMode) -> Result<(), VfsError>,
    batch_index(mode: IndexMode, limit: Option<usize>) -> Result<BatchResult, VfsError>,

    // Blob
    upload_blob(input: UploadBlobInput) -> Result<BlobRef, VfsError>,
    get_blob(hash: String) -> Result<Vec<u8>, VfsError>,
}
```

### 3. DSTU — 54 命令

```
核心模式: State<'_, Arc<VfsDatabase>> + Window + String
返回类型: Result<T, String> — 同样用了 String 而非 AppError
```

**一致性最好**的模块 — 所有命令都有 `dstu_` 前缀。

**建议的统一字段**:
```rust
pub struct DstuApi {
    // Node CRUD
    create_node(input: CreateNodeInput) -> Result<DstuNode, DstuError>,
    get_node(node_id: Uuid) -> Result<DstuNode, DstuError>,
    update_node(node_id: Uuid, input: UpdateNodeInput) -> Result<DstuNode, DstuError>,
    delete_node(node_id: Uuid, soft: bool) -> Result<(), DstuError>,

    // Path
    build_path(folder_id: Option<Uuid>, resource_id: Uuid) -> Result<String, DstuError>,
    resolve_path(path: String) -> Result<DstuNode, DstuError>,

    // Batch
    batch_move(input: BatchMoveInput) -> Result<BatchResult, DstuError>,
    batch_copy(input: BatchCopyInput) -> Result<BatchResult, DstuError>,

    // Watch
    watch(node_id: Uuid) -> Result<(), DstuError>,
    unwatch(node_id: Uuid) -> Result<(), DstuError>,
}
```

### 4. Data Governance — 43 命令

```
核心模式: AppHandle + State<BackupJobManagerState> + String
返回类型: Result<T, String>
```

**建议的统一字段**:
```rust
pub struct DataGovernanceApi {
    // Backup
    start_backup(input: BackupInput) -> Result<BackupJobStartResponse, GovError>,
    cancel_backup(job_id: Uuid) -> Result<bool, GovError>,
    list_backups() -> Result<Vec<BackupInfo>, GovError>,
    verify_backup(backup_id: Uuid) -> Result<VerifyResponse, GovError>,
    delete_backup(backup_id: Uuid) -> Result<(), GovError>,

    // Restore
    restore_backup(backup_id: Uuid, input: RestoreInput) -> Result<RestoreResponse, GovError>,

    // Sync
    check_sync_status() -> Result<SyncStatus, GovError>,
    push_changes() -> Result<SyncResult, GovError>,
    pull_changes() -> Result<SyncResult, GovError>,

    // Audit
    query_audit_logs(filters: AuditFilters) -> Result<Vec<AuditEntry>, GovError>,
    cleanup_audit_logs(input: CleanupInput) -> Result<u64, GovError>,

    // Migration
    get_migration_status() -> Result<MigrationStatus, GovError>,
    run_migration(migration_id: String) -> Result<(), GovError>,
}
```

### 5. Memory — 27 命令

```
核心模式: State<'_, Arc<VfsDatabase>> + State<'_, Arc<VfsLanceStore>> + State<'_, Arc<LLMManager>>
返回类型: Result<T, String>
```

**问题**: 每个命令都需要 3 个 State 参数 — 应该封装为 MemoryContext。

**建议**:
```rust
// 封装重复的 State 参数
pub struct MemoryContext {
    vfs_db: Arc<VfsDatabase>,
    lance_store: Arc<VfsLanceStore>,
    llm_manager: Arc<LLMManager>,
}

pub struct MemoryApi {
    // Note CRUD
    create_note(input: CreateMemoryNoteInput, ctx: State<MemoryContext>) -> Result<String, MemoryError>,
    read_note(note_id: Uuid, ctx: State<MemoryContext>) -> Result<MemoryNote, MemoryError>,
    update_note(note_id: Uuid, input: UpdateMemoryNoteInput, ctx: State<MemoryContext>) -> Result<(), MemoryError>,
    delete_note(note_id: Uuid, ctx: State<MemoryContext>) -> Result<(), MemoryError>,

    // Batch
    batch_delete(ids: Vec<Uuid>, ctx: State<MemoryContext>) -> Result<BatchResult, MemoryError>,
    batch_move(ids: Vec<Uuid>, target: String, ctx: State<MemoryContext>) -> Result<BatchResult, MemoryError>,

    // Smart Write
    smart_write(input: SmartWriteInput, ctx: State<MemoryContext>) -> Result<SmartWriteOutput, MemoryError>,
}
```

### 6. Review Plan — 17 命令

```
核心模式: State<'_, Arc<VfsDatabase>> + String + Option<String>
返回类型: Result<T, String>
```

**建议**:
```rust
pub struct ReviewPlanApi {
    create_plan(input: CreateReviewPlanInput, db: State<VfsDb>) -> Result<ReviewPlan, ReviewError>,
    get_due_reviews(input: DueReviewsInput, db: State<VfsDb>) -> Result<DueReviewsResult, ReviewError>,
    submit_review(input: SubmitReviewInput, db: State<VfsDb>) -> Result<ReviewPlan, ReviewError>,
    delete_plan(plan_id: Uuid, db: State<VfsDb>) -> Result<(), ReviewError>,
    batch_create(question_ids: Vec<Uuid>, exam_id: Uuid, db: State<VfsDb>) -> Result<BatchResult, ReviewError>,
}
```

### 7. Essay Grading — 20 命令

```
核心模式: State<'_, AppState> + String + Option
返回类型: 直接返回类型 (非 Result 包裹)
```

**问题**: Essay 模块没有用 `Result<T, Error>` 包裹返回值。

**建议**:
```rust
pub struct EssayGradingApi {
    create_session(input: CreateEssaySessionInput) -> Result<EssaySession, EssayError>,
    grade_essay(input: GradeEssayInput) -> Result<GradingResult, EssayError>,
    get_session(session_id: Uuid) -> Result<EssaySession, EssayError>,
    delete_session(session_id: Uuid) -> Result<(), EssayError>,
    create_custom_mode(input: CreateModeInput) -> Result<GradingMode, EssayError>,
    list_custom_modes() -> Result<Vec<GradingMode>, EssayError>,
    delete_custom_mode(mode_id: Uuid) -> Result<(), EssayError>,
}
```

### 8. 小模块快速设计

**Anki Connect (13)**:
```rust
pub struct AnkiConnectApi {
    check_status() -> Result<bool, AnkiError>,
    list_decks() -> Result<Vec<String>, AnkiError>,
    list_models() -> Result<Vec<String>, AnkiError>,
    add_cards(input: AddCardsInput) -> Result<Vec<Option<u64>>, AnkiError>,
    export_cards(input: ExportCardsInput) -> Result<String, AnkiError>,
    import_package(path: String) -> Result<(), AnkiError>,
}
```

**OCR (14)**:
```rust
pub struct OcrApi {
    list_engines() -> Result<Vec<OcrEngineInfo>, OcrError>,
    get_active_engine() -> Result<String, OcrError>,
    set_active_engine(engine_type: String) -> Result<(), OcrError>,
    add_engine(input: AddOcrEngineInput) -> Result<bool, OcrError>,
    remove_engine(config_id: String) -> Result<bool, OcrError>,
    test_engine(input: TestOcrInput) -> Result<bool, OcrError>,
    validate_model(config_id: String) -> Result<bool, OcrError>,
}
```

**Cloud Storage (14)**:
```rust
pub struct CloudStorageApi {
    check_connection(config: CloudConfig) -> Result<bool, CloudError>,
    put(config: CloudConfig, key: String, data: Vec<u8>) -> Result<(), CloudError>,
    get(config: CloudConfig, key: String) -> Result<Option<Vec<u8>>, CloudError>,
    delete(config: CloudConfig, key: String) -> Result<(), CloudError>,
    exists(config: CloudConfig, key: String) -> Result<bool, CloudError>,
    list(config: CloudConfig, prefix: Option<String>) -> Result<Vec<String>, CloudError>,
    sync_upload(input: SyncUploadInput) -> Result<SyncResult, CloudError>,
    sync_download(input: SyncDownloadInput) -> Result<SyncResult, CloudError>,
}
```

---

## 全局改进方案

### 改进 1: 统一错误类型

```
现状:
  chat_v2      → Result<T, String>
  dstu         → Result<T, String>
  memory       → Result<T, String>
  vfs          → Result<T, String>
  commands     → Result<T, AppError>
  notes        → Result<T, AppError>

目标: 所有模块使用 Result<T, ModuleError>
  chat_v2      → Result<T, ChatV2Error>
  dstu         → Result<T, DstuError>
  memory       → Result<T, MemoryError>
  vfs          → Result<T, VfsError>  ← 已实现!
```

### 改进 2: 输入封装

```
现状: fn command(param1: String, param2: String, param3: Option<String>, ...)
目标: fn command(input: CommandInput, state: State<'_, Context>)
```

### 改进 3: 消除重复 State 参数

```
现状:
fn memory_add_relation(
    note_id_a: String,
    note_id_b: String,
    vfs_db: State<'_, Arc<VfsDatabase>>,
    lance_store: State<'_, Arc<VfsLanceStore>>,
    llm_manager: State<'_, Arc<LLMManager>>
) -> ...

目标:
fn memory_add_relation(
    input: AddRelationInput,
    ctx: State<'_, MemoryContext>  // 封装 3 个 State
) -> ...
```

### 改进 4: 拆分 VFS 模块

```
当前 VFS (119 命令) 应拆分为:
  VFS Core       (~45 命令) — 文件/文件夹/Blob/索引
  Pomodoro       (~8 命令)  — 独立模块
  Todo           (~15 命令) — 独立模块
  Voice Input    (~5 命令)  — 独立模块
  PDF Processing (~10 命令) — 可独立或保留在 VFS
```

### 改进 5: 统一命名规范

所有 Tauri 命令必须遵循 `模块_操作_目标` 格式:
```
✅ chat_v2_create_session
✅ vfs_upload_blob
✅ memory_batch_delete
✅ dstu_build_path

❌ cancel_stream           (哪个模块?)
❌ get_setting             (应该: settings_get)
❌ TauriAPI.readFileAsBytes (应该: vfs_read_file_bytes)
```

---

## 实施路线图

| 阶段 | 范围 | 工作量 |
|------|------|--------|
| 1 | 创建统一的错误类型 (每个模块一个 Error enum) | 31 个模块 |
| 2 | 为高频命令添加 Input struct | ~120 命令 |
| 3 | Memory/VFS 的 State 封装 | 2 个模块 |
| 4 | VFS 模块拆分 (Pomodoro/Todo/Voice 独立) | 1 个模块 → 4 个 |
| 5 | 命令命名规范化 (rename 保留旧名作为 deprecated) | ~40% 命令 |
| 6 | 前端 API 网关层同步更新 | 167 个文件 |
