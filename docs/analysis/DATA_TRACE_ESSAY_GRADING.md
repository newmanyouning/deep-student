# Data Trace: Essay Grading Pipeline

> Generated: 2026-06-01
> Scope: Full data flow from Tauri command invocation through LLM streaming to frontend display and persistence.

---

## 1. Entry Points (Tauri Commands)

**File:** `C:/deep-student/src-tauri/src/essay_grading/mod.rs`

| Command | Function | Purpose |
|---------|----------|---------|
| `essay_grading_stream` | `essay_grading_stream()` | Main stream grading entry point |
| `essay_grading_create_session` | `essay_grading_create_session()` | Create grading session |
| `essay_grading_get_session` | `essay_grading_get_session()` | Get session details |
| `essay_grading_update_session` | `essay_grading_update_session()` | Update session metadata |
| `essay_grading_delete_session` | `essay_grading_delete_session()` | Soft-delete session |
| `essay_grading_list_sessions` | `essay_grading_list_sessions()` | List sessions with pagination |
| `essay_grading_toggle_favorite` | `essay_grading_toggle_favorite()` | Toggle session favorite |
| `essay_grading_get_rounds` | `essay_grading_get_rounds()` | Get all rounds for a session |
| `essay_grading_get_round` | `essay_grading_get_round()` | Get specific round |
| `essay_grading_get_latest_round_number` | `essay_grading_get_latest_round_number()` | Get latest round number |
| `essay_grading_get_modes` | `essay_grading_get_modes()` | Get all grading modes |
| `essay_grading_get_mode` | `essay_grading_get_mode()` | Get specific mode |
| `essay_grading_get_models` | `essay_grading_get_models()` | Get available LLM models |
| `essay_grading_create_custom_mode` | `essay_grading_create_custom_mode()` | Create custom grading mode |
| `essay_grading_update_custom_mode` | `essay_grading_update_custom_mode()` | Update custom mode |
| `essay_grading_delete_custom_mode` | `essay_grading_delete_custom_mode()` | Delete custom mode |
| `essay_grading_list_custom_modes` | `essay_grading_list_custom_modes()` | List custom modes |
| `essay_grading_save_builtin_override` | `essay_grading_save_builtin_override()` | Override built-in mode |
| `essay_grading_reset_builtin_mode` | `essay_grading_reset_builtin_mode()` | Reset built-in mode to default |
| `essay_grading_has_builtin_override` | `essay_grading_has_builtin_override()` | Check if built-in has override |
| `cancel_stream` | `cancel_stream()` (in `commands.rs`) | Cancel any active stream (shared generic command) |

All commands return `EssayGradingResult<T>` (defined in `error.rs`).

---

## 2. Request Type (Rust -> Frontend)

### `GradingRequest` (Rust struct)

**File:** `C:/deep-student/src-tauri/src/essay_grading/types.rs`

```rust
struct GradingRequest {
    session_id: String,
    stream_session_id: String,
    round_number: i32,
    input_text: String,
    topic: Option<String>,
    mode_id: Option<String>,
    model_config_id: Option<String>,
    essay_type: String,
    grade_level: String,
    custom_prompt: Option<String>,
    previous_result: Option<String>,
    previous_input: Option<String>,
    image_base64_list: Option<Vec<String>>,
    topic_image_base64_list: Option<Vec<String>>,
}
```

The matching TypeScript type is in `C:/deep-student/src/essay-grading/useEssayGradingStream.ts`:

```typescript
interface GradingRequest { /* same fields, camelCase */ }
```

### `GradingResponse` (Rust struct)

**File:** `C:/deep-student/src-tauri/src/essay_grading/types.rs`

```rust
struct GradingResponse {
    round_id: String,
    session_id: String,
    round_number: i32,
    grading_result: String,         // full LLM output (markdown + XML markers)
    overall_score: Option<f32>,
    dimension_scores_json: Option<String>,  // serialized ParsedScore JSON
    created_at: String,             // RFC 3339
}
```

---

## 3. Pipeline Flow (End-to-End)

```
Frontend (invoke)          Backend                     LLM
    |                         |                          |
    |-- essay_grading_stream -|                          |
    |   (GradingRequest)      |                          |
    |                         |-- build prompts ---------|
    |                         |   (system + user prompt) |
    |                         |                          |
    |                         |-- stream_grade() ------->|
    |                         |   SSE/HTTP POST          |
    |                         |   (stream: true)         |
    |                         |                          |
    |   <-- SSE "data" ------ |   on_chunk callback      |
    |    {type:"data",        |                          |
    |     chunk, accumulated, |                          |
    |     char_count}         |                          |
    |                         |                          |
    |                         |   ... streaming ...      |
    |                         |                          |
    |   <-- SSE "complete" -- |   stream ends            |
    |    {type:"complete",    |                          |
    |     result, round_id,   |                          |
    |     score}              |                          |
    |                         |                          |
    |                         |-- parse_score_from_result|
    |                         |-- save to VFS            |
    |                         |-- return GradingResponse |
    |                         |                          |
    |   <-- invoke resolves   |                          |
```

### Step-by-step:

1. **Frontend** calls `invoke('essay_grading_stream', { request })` (via `startGrading` in the hook).
2. **`essay_grading_stream`** command in `mod.rs` constructs `GradingDeps` (LLMManager, VFS database, EventEmitter, custom modes) and calls `pipeline::run_grading()`.
3. **`run_grading()`** in `pipeline.rs`:
   - Resolves grading mode (built-in or custom).
   - Builds system prompt (mode prompt + marker instructions + section instructions + score format + dimensions + student Q&A + custom prompt).
   - Builds user prompt (topic + previous context + essay type/grade hints + text stats block + essay content).
   - Resolves LLM config (user-selected model or default Model2), decrypts API key.
   - Calls `stream_grade()` with callbacks.
4. **`stream_grade()`** constructs messages (text-only or multimodal with images) and delegates to `StreamingLLMPipeline::stream()`.
5. **`StreamingLLMPipeline::stream()`** in `C:/deep-student/src-tauri/src/utils/streaming_llm_pipeline.rs`:
   - Builds HTTP request via `ProviderAdapter` (vendor-specific request formatting).
   - Registers with cancel watch channel (`llm.subscribe_cancel_stream`).
   - Sends POST with `stream: true` flag.
   - Parses SSE chunks using `adapter.parse_stream()`.
   - For each `ContentChunk` event, calls `on_chunk(content)` which accumulates text and emits SSE to frontend.
   - On `[DONE]` or `Done` event, marks `stream_ended = true`.
   - Cancellation: checks `cancel_rx` watch channel via `tokio::select!` or polls `consume_pending_cancel`.
6. **Back in `run_grading()`**:
   - If `StreamStatus::Cancelled` -> emit `cancelled` event, return Ok(None).
   - If `StreamStatus::Incomplete` -> return error.
   - Double-check for pending cancel signals after stream completion (race window guard).
   - Parses score from accumulated text using regex `<score total="X" max="Y">...<dim .../>...</score>`.
   - Saves to VFS via `VfsEssayRepo::create_essay()`.
   - Emits `complete` event to frontend.
   - Returns `GradingResponse` as the command result.

---

## 4. SSE Event Types (Backend -> Frontend)

**File (Rust types):** `C:/deep-student/src-tauri/src/essay_grading/types.rs`
**File (TS listener):** `C:/deep-student/src-essay-grading/useEssayGradingStream.ts`

All events are emitted on channel `essay_grading_stream_{stream_session_id}`.

### 4a. `data` event (incremental)

```json
{
  "type": "data",
  "chunk": "最新增量文本",
  "accumulated": "累积的完整文本",
  "char_count": 1234
}
```
- Emitted: on each ContentChunk from LLM stream.
- Frontend: updates `gradingResult` state with `accumulated`.
- Timeout: 120s timer is reset on each event (if no data for 120s -> timeout error).

### 4b. `complete` event (final)

```json
{
  "type": "complete",
  "round_id": "uuid-string",
  "grading_result": "完整批改结果（含XML标记）",
  "overall_score": 85.0,
  "parsed_score": "{\"total\":85.0,\"max_total\":100.0,\"grade\":\"良好\",\"dimensions\":[...]}",
  "created_at": "2026-06-01T12:00:00Z"
}
```
- Emitted: after VFS save succeeds.
- Frontend: sets `isGrading = false`, stores `currentRoundId`, resolves promise with `'completed'`.

### 4c. `error` event

```json
{
  "type": "error",
  "message": "错误描述"
}
```
- Emitted: currently **not directly emitted** by the emitter; errors propagate via the async command's `Result`.
- The frontend catches `invoke()` rejection and `resetTimeout()` -> sets error state.

### 4d. `cancelled` event

```json
{
  "type": "cancelled"
}
```
- Emitted: when stream is cancelled (user cancellation or race window guard).
- Frontend: sets `isGrading = false`, resolves promise with `'cancelled'`.

---

## 5. Cancellation Flow

```
Frontend cancelGrading()      Backend
    |                             |
    |-- invoke('cancel_stream',{streamEventName:"essay_grading_stream_xxx"})
    |                             |-- llm_manager.request_cancel_stream(event_name)
    |                             |   (sets watch channel to true)
    |                             |
    |   (meanwhile in streaming pipeline)
    |                             |-- tokio::select! detects cancel_rx.changed()
    |                             |-- sets cancelled=true -> StreamStatus::Cancelled
    |                             |-- clear_cancel_stream() cleans up
    |                             |-- emit_cancelled() to frontend
```

**Mechanism:**
- `llm_manager.streaming` uses `tokio::sync::watch::channel` per stream_event.
- `subscribe_cancel_stream()` creates a receiver; `request_cancel_stream()` broadcasts `true`.
- The pipeline's `tokio::select!` races stream chunks against cancel notification.
- After stream ends, `clear_cancel_stream()` removes the channel.
- A secondary `cancel_registry` (HashMap) retains pending cancel signals for the race-window check in `run_grading()` at line 125.

---

## 6. Prompt Construction

**File:** `C:/deep-student/src-tauri/src/essay_grading/pipeline.rs`, function `build_grading_prompts()`

### System Prompt structure:
1. Grading mode's `system_prompt` (full exam rubric for the selected mode).
2. `MARKER_INSTRUCTIONS` (XML marker usage rules: `<del>`, `<ins>`, `<replace>`, `<note>`, `<good>`, `<err>`).
3. `SECTION_INSTRUCTIONS` (polish section with `<section-polish>`).
4. `MODEL_ESSAY_INSTRUCTIONS` (conditionally, if topic is provided).
5. `SCORE_FORMAT_INSTRUCTIONS` + dimension listing with max scores.
6. Student Q&A instruction block.
7. Custom user prompt (sanitized, max 2000 chars).

### User Prompt structure:
1. Topic/source material (sanitized, max 1000 chars).
2. Previous round context (if multi-round: previous input + previous result, max 8000 chars each).
3. Essay type hint + grade level hint.
4. Text statistics block (Chinese chars, English words, punctuation counts, etc.).
5. Essay content (not sanitized, as-is).

### Image handling (Multimodal):
- If `is_multimodal && has_images`: constructs content array with `image_url` parts (MIME guessed from base64 magic bytes).
- Text-only path: simple text content strings.

---

## 7. Score Parsing

**File:** `C:/deep-student/src-tauri/src/essay_grading/pipeline.rs`, function `parse_score_from_result()`

Regex-based extraction from LLM output:
- `<score total="X" max="Y"> ... </score>` (supports attribute order variation).
- `<dim name="..." score="..." max="...">comment</dim>` for dimension scores.
- Validates: finite values, max > 0, total clamped to [0, mode.max_total].
- Grade labels: >= 90% -> "优秀", >= 75% -> "良好", >= 60% -> "及格", else "不及格".

Output: `Option<ParsedScore>` containing:
```
ParsedScore { total: f32, max_total: f32, grade: String, dimensions: Vec<DimensionScore> }
```

---

## 8. Persistence (VFS)

**File:** `C:/deep-student/src-tauri/src/vfs/repos/essay_repo.rs`

- `VfsEssayRepo::create_essay()` stores:
  - `title`, `essay_type`, `content`, `session_id`, `round_number`, `grade_level`, `custom_prompt`
  - `grading_result_json`: `{"result": "<full LLM output>", "overall_score": ..., "dimension_scores": "..."}`
  - `score`, `dimension_scores_json`
  - `resource_id` (links to content storage in `resources` table)
- Sessions stored in `essay_sessions` table through `VfsEssayRepo::create_session()`.

---

## 9. Frontend State Model

**File:** `C:/deep-student/src/essay-grading/useEssayGradingStream.ts`

```typescript
interface GradingStreamState {
  isGrading: boolean;
  gradingResult: string;       // accumulated text from SSE data events
  error: string | null;
  streamSessionId: string | null;
  charCount: number;
  currentRoundId: string | null;
  canRetry: boolean;
  isPartialResult: boolean;    // true if error/cancel after partial data received
}
```

**Edge case protections:**
- Guard against duplicate starts (`isStartingRef`, `isActiveRef`).
- 120s timeout with reset on each SSE event.
- Component unmount: cancels backend stream via `cancel_stream`.
- Stale event guard: `isActiveRef.current || settledRef.current` check in data/complete handlers.
- Race-window cancellation: double-check after stream completion in `run_grading()`.

---

## 10. File Map

| Layer | File | Role |
|-------|------|------|
| Backend entry | `src-tauri/src/essay_grading/mod.rs` | Tauri command definitions (~20 commands) |
| Backend pipeline | `src-tauri/src/essay_grading/pipeline.rs` | Core business logic: prompt building, streaming, score parsing, VFS save |
| Backend types | `src-tauri/src/essay_grading/types.rs` | All data types: GradingRequest, GradingResponse, SSE payloads, GradingMode, ParsedScore |
| Backend events | `src-tauri/src/essay_grading/events.rs` | `GradingEventEmitter` - emits SSE via Tauri `window.emit()` |
| Backend error | `src-tauri/src/essay_grading/error.rs` | `EssayGradingError`, `EssayGradingResult<T>` |
| Backend text stats | `src-tauri/src/essay_grading/text_stats.rs` | `EssayTextStats`, `calculate_text_stats()`, `build_stats_prompt_block()` |
| Backend shared pipeline | `src-tauri/src/utils/streaming_llm_pipeline.rs` | `StreamingLLMPipeline::stream()` - HTTP request, SSE parsing, cancel watch |
| Backend cancel stream | `src-tauri/src/commands.rs:310` | Generic `cancel_stream` command |
| Backend cancel infra | `src-tauri/src/llm_manager/streaming.rs` | Watch channels + cancel registry |
| Backend VFS essay repo | `src-tauri/src/vfs/repos/essay_repo.rs` | CRUD for `essays` and `essay_sessions` tables |
| Frontend hook | `src/essay-grading/useEssayGradingStream.ts` | `useEssayGradingStream()` - state management + SSE listener |
| Frontend API | `src/essay-grading/essayGradingApi.ts` | Typed API wrappers (CRUD + grading modes + models) |
| Frontend component | `src/components/EssayGradingWorkbench.tsx` | Main workbench UI component |
| Frontend child components | `src/components/essay-grading/GradingMain.tsx` | Grading panel display |

---

## 11. Key Boundaries and Data Transforms

| Boundary | Data Transform |
|----------|---------------|
| Frontend `startGrading(request)` -> Backend `essay_grading_stream` | TS `GradingRequest` (camelCase) -> Rust `GradingRequest` (snake_case), serialized via serde |
| Backend `run_grading()` -> LLM API | Prompts + images -> JSON `messages` array via `ProviderAdapter.build_request()` |
| LLM API -> Backend `on_chunk` | SSE bytes -> `adapter.parse_stream()` -> `StreamEvent::ContentChunk(text)` |
| Backend pipeline -> Frontend SSE | Rust `window.emit()` -> Tauri event bus -> `listen()` in TS |
| LLM output -> ParsedScore | Regex extraction from `{result}` string: `<score>` and `<dim>` XML tags |
| ParsedScore -> VFS | `serde_json::to_string()` -> `VfsCreateEssayParams.grading_result_json` |
| VFS -> Frontend via `getRounds()` | `VfsEssay` rows -> `GradingRoundResponse` -> TS `GradingRound` |

---

## 12. Error Flow

```
Invoke fails (network)
  -> catch in useEssayGradingStream line 297
  -> setState error, canRetry=true, isPartialResult=(has data ? true : false)
  -> retryGrading() generates new stream_session_id and re-invokes

LLM returns error status
  -> StreamingLLMPipeline line 112-128 checks response.status()
  -> custom error handler (optional) or default format
  -> AppError::llm() propagated as command error

Stream ends without [DONE]
  -> StreamStatus::Incomplete
  -> run_grading() line 117 returns Err(AppError::llm)
  -> frontend catches as error

User cancels mid-stream
  -> cancel_stream command -> request_cancel_stream -> watch channel
  -> tokio::select! detects cancel -> StreamStatus::Cancelled
  -> emit_cancelled() -> frontend resolves with 'cancelled'
  -> returned Ok(None) from run_grading()

Race-window cancel (user cancels just as stream finishes)
  -> double-check at run_grading() line 125 via consume_pending_cancel()
  -> emit_cancelled(), return Ok(None)
```
