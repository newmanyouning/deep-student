# Data Trace: Translation Pipeline

> Generated: 2026-06-01
> Scope: Full trace from UI to LLM response and back

---

## Overview

There are **two independent translation flows** in the codebase, each with its own event namespace, persistence strategy, and frontend rendering approach. They share only the core `stream_translate()` function and `StreamingLLMPipeline::stream()` for the actual LLM call.

---

## Flow A: Standalone Translation Workbench (DSTU-persisted)

### Entry Points

| Layer | File | Symbol |
|-------|------|--------|
| Tauri command registration | `src-tauri/src/lib.rs:995` | `crate::translation::translate_text_stream` |
| Command impl | `src-tauri/src/translation/mod.rs:34` | `translate_text_stream()` |
| Pipeline | `src-tauri/src/translation/pipeline.rs:28` | `run_translation()` |
| Shared LLM streaming | `src-tauri/src/utils/streaming_llm_pipeline.rs:32` | `StreamingLLMPipeline::stream()` |
| Event emitter | `src-tauri/src/translation/events.rs:10` | `TranslationEventEmitter` |
| Frontend hook | `src/translation/useTranslationStream.ts:67` | `useTranslationStream()` |
| Workbench component | `src/components/TranslateWorkbench.tsx:65` | `TranslateWorkbench` |
| DSTU content view | `src/features/learning-hub/apps/views/TranslationContentView.tsx:32` | `TranslationContentView` |
| DSTU adapter | `src/dstu/adapters/translationDstuAdapter.ts:153` | `translationDstuAdapter` |

### Data Flow Sequence

```
 User types/imports text in SourcePanel
   |
   v
 TranslateWorkbench.handleTranslate()
   |
   v
 useTranslationStream.startTranslation(request)
   |  - generates sessionId = `translate_${Date.now()}`
   |  - subscribes to Tauri event `translation_stream_${sessionId}`
   |  - sets timeout (120s)
   v
 invoke('translate_text_stream', { request: TranslationRequest })
   |
   v
 translate_text_stream() in mod.rs
   |  - constructs TranslationDeps { llm, db, emitter, vfs_db }
   v
 pipeline::run_translation()
   |  1. Input validation (empty text, max 100K chars)
   |  2. build_translation_prompts(&request) -> (system_prompt, user_prompt)
   |     - domain_system_prompt(): 7 preset domains (academic/technical/literary/legal/medical/casual/general)
   |     - Optional: formality style injection, glossary term injection
   |     - lang_full_name(): maps codes like "zh-CN" -> "Simplified Chinese (简体中文)"
   |  3. llm.get_translation_model_config() -> ApiConfig
   |  4. llm.decrypt_api_key() -> plaintext key
   v
 stream_translate()
   |  - builds messages: [{"role":"system","content":...}, {"role":"user","content":...}]
   |  - temperature: 0.3
   |  - max_tokens from effective_max_tokens()
   |  - custom error_handler for HTTP status -> Chinese error messages
   v
 StreamingLLMPipeline::stream()
   |  - constructs ProviderAdapter (per API vendor)
   |  - sends HTTP POST with stream=true
   |  - parses SSE chunks in real time
   |  - checks cancellation via LLMManager cancel registry
   |  - calls on_chunk callback for each content delta
   |
   v  (per-chunk callback)
 pipeline.rs:66
   |  - accumulated.push_str(chunk)
   |  - emitter.emit_data(session_id, chunk, accumulated)
   v
 TranslationEventEmitter.emit_data()
   |  - event name: `translation_stream_{session_id}`
   |  - payload: TranslationStreamData { type:"data", chunk, accumulated, char_count, word_count }
   |
   v  (on frontend)
 useTranslationStream event listener
   |  - resets timeout on each chunk
   |  - updates state: translatedText, charCount, wordCount
   |  - React re-render flows to TargetPanel -> TranslationStreamRenderer
   |
   v  (when stream completes)
 pipeline.rs:86
   |  - emitter.emit_complete(session_id, session_id, accumulated, now)
   |  - payload: TranslationStreamComplete { type:"complete", id, translated_text, created_at }
   v
 useTranslationStream receives "complete"
   |  - sets isTranslating=false
   |  - resolve promise with 'completed'
   v
 TranslateWorkbench.handleTranslate() continues
   |  - reads latest translatedText via ref (avoids stale closure)
   |  - calls dstuMode.onSessionSave(session)
   v
 TranslationContentView.handleSessionSave()
   |  - translationDstuAdapter.updateTranslation(session)
   |  - writes metadata fields to VFS/DTSU node:
   |    { sourceText, translatedText, srcLang, tgtLang, formality, customPrompt, qualityRating, isFavorite }
```

### Key Files (Flow A)

| File | Role |
|------|------|
| `src-tauri/src/cmd/translation.rs` | `ocr_extract_text` command (image -> text for translation source) |
| `src-tauri/src/translation/mod.rs` | `translate_text_stream` Tauri command entry |
| `src-tauri/src/translation/pipeline.rs` | Core pipeline: prompt building, streaming orchestration |
| `src-tauri/src/translation/events.rs` | `TranslationEventEmitter` (data/complete/error/cancelled) |
| `src-tauri/src/translation/types.rs` | `TranslationRequest`, `TranslationResponse`, 4 SSE payload structs |
| `src-tauri/src/utils/streaming_llm_pipeline.rs` | Shared SSO/stream parsing, cancellation |
| `src/translation/useTranslationStream.ts` | React hook: event subscription, timeout, cancel |
| `src/translation/TranslationStreamRenderer.tsx` | Streaming text renderer with stats |
| `src/components/TranslateWorkbench.tsx` | Full workbench: orchestrates source/target/prompt panels |
| `src/components/translation/SourcePanel.tsx` | Source text editor (input/OCR/file drag-drop) |
| `src/components/translation/TargetPanel.tsx` | Target text display (with edit/speak/export/copy/rate) |
| `src/components/translation/ComparisonView.tsx` | Paragraph-level side-by-side comparison |
| `src/components/translation/TranslationMain.tsx` | Layout: horizontal/vertical resizable panels |
| `src/components/translation/PromptPanel.tsx` | Custom prompt editor, formality, domain, glossary |
| `src/components/translation/TranslationHeader.tsx` | Tab bar (translate/history) |
| `src/components/translation/TranslationHistory.tsx` | History list with search, favorites, delete |
| `src/features/learning-hub/apps/views/TranslationContentView.tsx` | DSTU-aware wrapper: loads/saves sessions |
| `src/dstu/adapters/translationDstuAdapter.ts` | CRUD operations on VFS/DTSU translation nodes |

---

## Flow B: Chat Popover Translation (no persistence)

### Entry Points

| Layer | File | Symbol |
|-------|------|--------|
| Tauri command (aligned) | `src-tauri/src/lib.rs:996` | `crate::translation::chat_popover::stream_chat_translation_aligned` |
| Tauri command (plain) | `src-tauri/src/lib.rs:997` | `crate::translation::chat_popover::stream_chat_translation_plain` |
| Implementation | `src-tauri/src/translation/chat_popover.rs:172` | `run_chat_translation()` |
| Frontend component | `src/features/chat/components/TranslationPopover.tsx:211` | `TranslationPopover` |
| NDJSON parser | `src/features/chat/components/translationNdjsonParser.ts:33` | `createNdjsonParser()` |
| LRU cache | `src/features/chat/components/translationCache.ts` | `readCache`/`writeCache` |
| Types | `src/features/chat/components/translationTypes.ts` | `AlignedSegment`, `TranslationDisplayMode` |

### Data Flow Sequence

```
 User selects text in chat -> clicks "Translate" in SelectionToolbar
   |
   v
 TranslationPopover mounts (isVisible=true)
   |  1. loadTranslationSettings() resolves model ID + display mode
   |     - reads model_assignments from Tauri (translation_model_config_id, fallback model2)
   |     - determines mode: 'aligned' (default) or 'streaming'
   |  2. detectSourceLang(sourceText) detects zh-CN/ja/ko via Unicode ranges
   |  3. getDefaultTargetLang() -> zh-CN if source is Chinese, else en
   v
 doTranslate()
   |  - increments reqIdRef (cancels any in-flight request with same pattern)
   |  - checks LRU cache via buildCacheKey() (SHA-256 of context+source)
   |  - if miss: generates requestId = nanoid()
   |  - subscribes to Tauri event `chat_translation_${requestId}`
   v
 invoke('stream_chat_translation_aligned' or '_plain', { request: ChatTranslationRequest })
   |
   v
 chat_popover.rs::run_chat_translation()
   |  1. validate_request() (max 8000 chars, 200 context chars)
   |  2. llm.get_translation_model_config() -> ApiConfig
   |  3. decrypt_api_key()
   |  4. Build prompts based on mode:
   |     aligned:  build_aligned_prompts() -> NDJSON line-per-segment format
   |     plain:    build_plain_prompts()    -> single plain text output
   |  5. stream_translate() (same function as Flow A)
   v
 (per-chunk callback)
   |  - accumulated.push_str(chunk)
   |  - emits ChatTranslationEvent::Chunk { delta, accumulated }
   |  - event: `chat_translation_${requestId}`
   v
 TranslationPopover handles chunk event
   |  mode==aligned:
   |    - createNdjsonParser().push(delta) parses line-by-line
   |    - each parsed {"src":"...","tgt":"..."} appended to segments[]
   |    - visual: dual-column with hover-linked highlighting
   |  mode==streaming:
   |    - accumulated text displayed directly in single column
   v
 On complete
   |  - parser.flush() handles trailing partial line
   |  - fallback: if aligned produced 0 segments, parseAlignedFallback() tries whole-buffer parsing
   |  - writes to LRU cache for future instant hits
   |  - cache cleared on `model_assignments_changed` window event
   v
 On cancel (close popover / switch language / unmount)
   |  - invoke('cancel_stream', { streamEventName })
   |  - unlisten removes Tauri event listener
```

### Chat Popover Specifics

**Display modes** controlled by `translation_display_mode` in model assignments:

- **`aligned`** (default): LLM outputs NDJSON, one JSON object per line. Frontend parses incrementally into `AlignedSegment[]`. Rendered as two columns with synchronized hover highlighting.
- **`streaming`**: LLM outputs plain text. Frontend renders token-by-token in single column with blinking cursor.

**Context injection**: `context_before` and `context_after` (max 200 chars each) are included in the prompt for disambiguation but explicitly marked "do NOT translate."

**Cancellation protocol**: Request-level gating via `reqIdRef` (atomic counter). Each new request increments the counter and cancels the previous stream. Stale event handlers self-ignore by comparing against `myId`.

### Key Files (Flow B)

| File | Role |
|------|------|
| `src-tauri/src/translation/chat_popover.rs` | Two Tauri commands, prompt building, event emission |
| `src/features/chat/components/TranslationPopover.tsx` | Full popover UI with aligned/streaming modes, cache, cancel |
| `src/features/chat/components/translationNdjsonParser.ts` | Incremental NDJSON line parser with error tolerance |
| `src/features/chat/components/translationCache.ts` | LRU cache (max 100), keyed by SHA-256(context+source) |
| `src/features/chat/components/translationTypes.ts` | `AlignedSegment`, `TranslationDisplayMode` types |

---

## Shared Infrastructure

### `stream_translate()` (pipeline.rs:233)

Signature:
```rust
pub(crate) async fn stream_translate<F>(
    config: &ApiConfig,
    api_key: &str,
    system_prompt: &str,
    user_prompt: &str,
    stream_event: &str,
    llm: Arc<LLMManager>,
    on_chunk: F,
) -> Result<StreamStatus, AppError>
where
    F: FnMut(String),
```

Creates a 2-message array `[system, user]`, calls `StreamingLLMPipeline::stream()` with temperature 0.3 and a custom HTTP error handler (maps 401/403/429/5xx to Chinese user-facing messages).

### `StreamingLLMPipeline::stream()` (streaming_llm_pipeline.rs:47)

The shared LLM pipeline that:
1. Resolves the API provider adapter from `config`
2. Constructs an HTTP streaming request
3. Parses SSE chunks, calling `on_chunk` for each content delta
4. Checks `LLMManager.cancel_registry` for cancellation
5. Returns `StreamStatus::Completed`, `Cancelled`, or `Incomplete`

Cancellation is invoked by the frontend via `invoke('cancel_stream', { streamEventName })`, which hits the `LLMManager`'s cancel registry.

### `lang_full_name()` (pipeline.rs:102)

Maps language codes (e.g., "zh-CN", "en", "ja") to full human-readable names in the prompt (e.g., "Simplified Chinese (简体中文)", "English") so the LLM precisely understands the target language.

---

## Event Namespace Isolation

| Flow | Event name pattern | Rust struct | TS type |
|------|-------------------|-------------|---------|
| Standalone | `translation_stream_{sessionId}` (generated as `translate_{timestamp}`) | `TranslationStreamData` / `TranslationStreamComplete` / etc. | `TranslationStreamEvent` (discriminated by `type` field) |
| Chat popover | `chat_translation_{requestId}` (generated as `nanoid()`) | `ChatTranslationEvent` (Rust enum, serde tag) | `ChatTranslationEvent` (TS discriminated union via `type`) |

---

## Persistence (DSTU/VFS)

Translation sessions are stored as VFS nodes with type `translation`. The `translationDstuAdapter` provides CRUD:

- **Create**: `dstu.create('/', { type: 'translation', name, metadata })`
- **Read**: `dstu.get('/{id}')` -> `dstuNodeToTranslationSession()`
- **Update**: `dstu.setMetadata('/{id}', metadata)`
- **Delete**: `dstu.delete('/{id}')`

Metadata fields: `sourceText`, `translatedText`, `srcLang`, `tgtLang`, `formality`, `customPrompt`, `domain`, `glossary`, `qualityRating`, `isFavorite`.

Note (from pipeline.rs:78-80): Backend no longer creates VFS records. The frontend creates an empty VFS node via Learning Hub, then updates it after translation. This prevents double-write orphan records.

---

## Input Validation Boundaries

| Parameter | Standalone (Flow A) | Chat popover (Flow B) |
|-----------|---------------------|-----------------------|
| Source text max | 100,000 chars | 8,000 chars |
| Context before/after | N/A (not supported) | 200 chars each |
| Empty text | `AppError::validation` | `AppError::validation` |
| Source: standalone | Backend validation only | Frontend + backend validation |

---

## Rust Types vs Frontend Types Verification

### TranslationRequest (Rust -> TS)

| Rust field | TS field | Match | Note |
|-----------|----------|-------|------|
| `text: String` | `text: string` | YES | Same name |
| `src_lang: String` | `src_lang: string` | YES | Same name (snake vs camel via serde) |
| `tgt_lang: String` | `tgt_lang: string` | YES | Same name |
| `prompt_override: Option<String>` | `prompt_override?: string` | YES | Same name |
| `session_id: String` | (generated internally) | YES | Frontend generates `translate_{timestamp}`, not exposed in TS interface |
| `formality: Option<String>` | `formality?: 'formal' \| 'casual' \| 'auto' \| null` | YES | String matching |
| `glossary: Option<Vec<(String,String)>>` | `glossary?: Array<[string, string]>` | YES | Same shape |
| `domain: Option<String>` | `domain?: string` | YES | Same name |

### SSE Events (Rust -> TS)

**TranslationStreamData** (standalone):
- `type: String` ("data") -> `type: 'data'`
- `chunk: String` -> `chunk?: string`
- `accumulated: String` -> `accumulated?: string`
- `char_count: usize` -> `char_count?: number`
- `word_count: usize` -> `word_count?: number`

**TranslationStreamComplete** (standalone):
- `type: String` ("complete") -> `type: 'complete'`
- `id: String` -> `id?: string`
- `translated_text: String` -> `translated_text?: string`
- `created_at: String` -> `created_at?: string`

**ChatTranslationEvent** (popover, Rust enum -> TS discriminated union):
- `Chunk { delta, accumulated }` -> `{ type: 'chunk'; delta: string; accumulated: string }`
- `Complete` -> `{ type: 'complete' }`
- `Error { message }` -> `{ type: 'error'; message: string }`
- `Cancelled` -> `{ type: 'cancelled' }`

All server-emitted fields match their frontend consumers.

---

## Appendix: File Index

```
src-tauri/src/translation/
  mod.rs                 # translate_text_stream command entry
  pipeline.rs            # Core: run_translation, stream_translate, build_translation_prompts, lang_full_name, domain_system_prompt
  events.rs              # TranslationEventEmitter
  types.rs               # TranslationRequest, TranslationResponse, 4 SSE event structs
  chat_popover.rs        # stream_chat_translation_aligned/plain commands, prompt building

src-tauri/src/utils/
  streaming_llm_pipeline.rs  # StreamingLLMPipeline::stream() - shared SSE HTTP client

src/
  translation/
    useTranslationStream.ts  # Standalone flow hook (event subscription, timeout, cancel)
    TranslationStreamRenderer.tsx  # Streaming text display with stats

  components/
    TranslateWorkbench.tsx   # Full workbench orchestration
    translation/
      TranslationMain.tsx    # Layout: resizable source/target panels
      SourcePanel.tsx        # Source text editor with OCR/drag-drop
      TargetPanel.tsx        # Target display with edit/speak/export/copy/rate
      ComparisonView.tsx     # Paragraph-level bilingual comparison
      PromptPanel.tsx        # Custom prompt, formality, domain, glossary
      TranslationHeader.tsx  # Tab bar
      TranslationHistory.tsx # History list

  features/
    learning-hub/apps/views/
      TranslationContentView.tsx  # DSTU-aware wrapper: load/save sessions
    chat/components/
      TranslationPopover.tsx      # Chat popover translation UI
      translationNdjsonParser.ts  # Incremental NDJSON parser
      translationCache.ts         # LRU cache (100 entries)
      translationTypes.ts         # AlignedSegment, TranslationDisplayMode

  dstu/adapters/
    translationDstuAdapter.ts     # VFS/DTSU CRUD for translation records
```
