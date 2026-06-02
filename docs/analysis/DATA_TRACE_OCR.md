# OCR Pipeline Data Trace

> Generated: 2026-06-01 (using `date +"%Y-%m-%d %H:%M CST"`)
>
> Trace covers: configuration, adapter dispatch, LLM invocation, response parsing, frontend display.

---

## 1. Architecture Overview

```
                    ┌────────────────────────────────────┐
                    │       Frontend (React/TSX)          │
                    │  OcrEngineCard → Settings page      │
                    │  OcrEngineTestPanel → Test panel    │
                    │  OcrResultHeader / OcrResultCardV2  │
                    │          ↑ ↓ invoke()               │
                    ├────────────────────────────────────┤
                    │       Backend (Tauri Commands)      │
                    │  cmd/ocr.rs — config & test         │
                    │  chat_v2/handlers/ocr.rs — analysis │
                    │  vfs/pdf_processing_service.rs      │
                    │  vfs/indexing/mod.rs                │
                    │  pdf_ocr_service.rs                 │
                    │  multimodal/embedding_service.rs    │
                    ├────────────────────────────────────┤
                    │       LLM Manager (Dispatch)        │
                    │  model2_pipeline::call_ocr_model    │
                    │  model_profile_service::test_ocr    │
                    ├────────────────────────────────────┤
                    │    OcrAdapter (Prompt + Parse)      │
                    │  DeepSeekOcrAdapter                 │
                    │  PaddleOcrVlAdapter                 │
                    │  Glm4vOcrAdapter                    │
                    │  GenericVlmAdapter                  │
                    │  SystemOcrAdapter                   │
                    ├────────────────────────────────────┤
                    │     ProviderAdapter (Transport)     │
                    │  OpenAIAdapter / GeminiAdapter      │
                    │  AnthropicAdapter                   │
                    └────────────────────────────────────┘
```

### Key Source Files

| File | Role |
|------|------|
| `src-tauri/src/ocr_adapters/types.rs` | `OcrEngineType`, `OcrMode`, `OcrRegion`, `OcrPageResult`, `OcrError` |
| `src-tauri/src/ocr_adapters/mod.rs` | `OcrAdapter` trait, `Glm4vOcrAdapter`, `GenericVlmAdapter` |
| `src-tauri/src/ocr_adapters/factory.rs` | `OcrAdapterFactory::create()`, engine info, model inference |
| `src-tauri/src/ocr_adapters/paddle.rs` | `PaddleOcrVlAdapter`, JSON/bbox parsing |
| `src-tauri/src/ocr_adapters/deepseek.rs` | `DeepSeekOcrAdapter`, grounding format parsing |
| `src-tauri/src/paddleocr_api.rs` | Direct PaddleOCR REST API client (async job mode) |
| `src-tauri/src/cmd/ocr.rs` | Tauri commands for engine config and testing |
| `src-tauri/src/chat_v2/handlers/ocr.rs` | `chat_v2_perform_ocr` command (analysis mode) |
| `src-tauri/src/llm_manager/model_profile_service.rs:1087-1350` | `get_ocr_adapter()`, `test_ocr_with_engine()`, `get_ocr_config_with_effective_engine()` |
| `src-tauri/src/llm_manager/model2_pipeline.rs:4863-5020+` | `call_ocr_model_raw_prompt()` — core LLM OCR invocation |
| `src/features/chat/plugins/modes/analysis.ts` | `performOcr()`, `retryOcr()` — frontend OCR trigger |
| `src/features/chat/components/panels/OcrResultCard.tsx` | OCR result display with LaTeX rendering |
| `src/features/chat/plugins/modes/components/OcrResultHeader.tsx` | Collapsible OCR result header |
| `src/features/settings/components/OcrEngineCard.tsx` | Engine priority/list management UI |
| `src/features/settings/components/OcrEngineTestPanel.tsx` | Multi-engine comparison testing UI |

---

## 2. Engine Selection Flow

### 2.1 Configuration Storage

All settings stored in app DB key-value table:

- `ocr.engine_type` — default engine type string, default `"paddle_ocr_vl"`
- `ocr.available_models` — JSON array of `OcrModelConfig` with priorities
- `ocr.enable_thinking` — `"true"` / `"false"` for VLM deep reasoning

### 2.2 OcrEngineType Enum (types.rs:8-26)

Seven variants, with `PaddleOcrVl` as default:

| Variant | Identifier | Grounding | Free | Dedicated | Backend |
|---------|-----------|-----------|------|-----------|---------|
| `DeepSeekOcr` | `deepseek_ocr` | Yes | No | Yes | OpenAI-compatible API |
| `PaddleOcrVl` | `paddle_ocr_vl` | Yes | Yes | Yes | OpenAI-compatible API |
| `PaddleOcrVlV1` | `paddle_ocr_vl_v1` | Yes | Yes | Yes | OpenAI-compatible API |
| `PaddleOcrApi` | `paddle_ocr_api` | No | Yes | Yes | Direct REST API (aistudio-app.com) |
| `Glm4vOcr` | `glm4v_ocr` | Yes | No | No | Zhipu API compatible |
| `GenericVlm` | `generic_vlm` | No | No | No | Any OpenAI/Gemini/Anthropic |
| `SystemOcr` | `system_ocr` | No | Yes | Yes | OS native (Win/Mac/iOS) |

### 2.3 Adapter Dispatch (factory.rs:23-37)

```rust
impl OcrAdapterFactory {
    pub fn create(engine_type: OcrEngineType) -> Arc<dyn OcrAdapter> {
        match engine_type {
            DeepSeekOcr    => Arc::new(DeepSeekOcrAdapter::new()),
            PaddleOcrVl    => Arc::new(PaddleOcrVlAdapter::new()),
            PaddleOcrVlV1  => Arc::new(PaddleOcrVlAdapter::with_engine(PaddleOcrVlV1)),
            PaddleOcrApi   => Arc::new(PaddleOcrVlAdapter::with_engine(PaddleOcrApi)),
            Glm4vOcr       => Arc::new(Glm4vOcrAdapter::new()),
            GenericVlm     => Arc::new(GenericVlmAdapter::new()),
            SystemOcr      => Arc::new(SystemOcrAdapter::new()),
        }
    }
}
```

**Key design**: `PaddleOcrApi` uses the same `PaddleOcrVlAdapter` for prompt/parse but the invocation bypasses the VLM path and uses the direct REST API client (`paddleocr_api.rs`).

---

## 3. Data Flow: Analysis Mode OCR (Primary Path)

### Step-by-step data transformation

#### Step 1: User uploads image (Frontend)

```
File (from drag-drop or file picker)
  → FileReader.readAsDataURL()
  → string: "data:image/png;base64,iVBORw0KGgo..."
```

**File**: `analysis.ts` → `performOcr(images: string[], onProgress?)`

#### Step 2: Tauri IPC invocation (Frontend → Backend)

```typescript
const response = await invoke<{
  ocr_text: string;
  tags: string[];
  mistake_type: string;
}>('chat_v2_perform_ocr', {
  request: {
    images: normalizedImages, // string[] of data URLs
  },
});
```

**Serialization**: `serde_json::to_string()` on Rust side, JSON parse on TS side.

#### Step 3: Backend entry point (chat_v2/handlers/ocr.rs:43-131)

```
OcrRequest { images: Vec<String> }
  → Parse base64 → Vec<u8> (raw image bytes)
  → Route based on engine_type.is_native_ocr()
```

**Two paths diverge here:**

**Path A — Native OCR** (SystemOcr):
```
Vec<u8> → system_ocr::perform_system_ocr() → String
```

**Path B — VLM OCR** (all others):
```
Vec<u8> → base64 re-encode → ImagePayload { mime, base64 }
  → adapter.build_prompt(OcrMode::FreeOcr) → String prompt
  → llm_manager.call_ocr_model_raw_prompt(prompt, Some(image_payloads))
  → StandardModel2Output { assistant_message: String }
```

#### Step 4: Core LLM invocation (model2_pipeline.rs:4863)

**4a. Resolve config + adapter:**
```
get_ocr_config_with_effective_engine()
  → read ocr.available_models from DB
  → match config_id to ApiConfig
  → infer OcrEngineType from model name (or declared type)
  → OcrAdapterFactory::create(effective_engine)
```

**4b. Build messages:**
```json
{
  "model": "PaddlePaddle/PaddleOCR-VL-1.5",
  "messages": [
    {
      "role": "user",
      "content": [
        {
          "type": "image_url",
          "image_url": {
            "url": "data:image/png;base64,...",
            "detail": "high"
          }
        },
        {
          "type": "text",
          "text": "OCR:"        // ← adapter.build_prompt()
        }
      ]
    }
  ],
  "temperature": 0.0,
  "max_tokens": 4096,
  "repetition_penalty": 1.1,    // PaddleOCR-VL specific
  "stream": false
}
```

**4c. Provider adapter selection:**
```
config.model_adapter:
  "google" | "gemini" → GeminiAdapter
  "anthropic" | "claude" → AnthropicAdapter
  _ → OpenAIAdapter (default, covers SiliconFlow / custom endpoints)
```

**4d. HTTP request:**
```
ProviderAdapter::build_request(base_url, api_key, model, request_body)
  → PrepRequest { url, headers, body }
  
reqwest::Client::post(preq.url)
  .headers(preq.headers)
  .json(&preq.body)
  .send()
  → Response JSON
```

**4e. Parse response:**
```
response.json() → serde_json::Value
  → response_json.pointer("/choices/0/message/content")
  → String (raw model output)
```

#### Step 5: Adapter response parsing

**PaddleOcrVlAdapter.parse_response()** (paddle.rs:95-184):

```
response_text (raw LLM output)
  → Try parse as JSON (extract_json_from_response)
    → Success: parse blocks array + markdown → OcrPageResult
    → Fail: check for DeepSeek-style grounding tags
      → Success: delegate to DeepSeekOcrAdapter.parse_response()
      → Fail: return as plain text region
```

**OcrPageResult** structure:
```rust
struct OcrPageResult {
    page_index: usize,
    image_path: String,
    image_width: u32,
    image_height: u32,
    regions: Vec<OcrRegion>,
    markdown_text: Option<String>,
    engine: OcrEngineType,
    mode: OcrMode,
    processing_time_ms: Option<u64>,
}

struct OcrRegion {
    label: String,           // "text", "formula", "table", "chart", "document"
    text: String,
    bbox_normalized: Option<Vec<f64>>,  // [x,y,w,h] in 0-1 range
    bbox_pixels: Option<Vec<f64>>,      // [x,y,w,h] in pixel coords
    confidence: Option<f64>,
    raw_output: Option<String>,
}
```

#### Step 6: Backend response assembly

```
regions → join text with "\n" → full_text (String)
OcrResponse {
    ocr_text: full_text,
    tags: Vec::new(),        // classification deprecated
    mistake_type: String::new(),
}
```

#### Step 7: Frontend display

```
OcrResponse.ocr_text  →  OcrMeta.question
                        → OcrMeta.rawText
                        → OcrResultCard.ocrText
                        → <LatexText content={ocrText} />
```

**Frontend components chain:**
1. `analysis.ts:performOcr()` → returns `OcrMeta`
2. Store `modeState.ocrMeta` updated
3. `OcrResultHeader` (collapsible status bar, with retry button)
4. `OcrResultCardV2` (subscribes to store)
5. `OcrResultCard` (pure display):
   - `LatexText` for OCR content (LaTeX math rendering)
   - Tags as `Badge` components
   - Optional images with click-to-preview
   - Editable learning notes section

---

## 4. Data Flow: OCR Engine Testing (Settings)

**Frontend**: `OcrEngineTestPanel.tsx`

```
Select image → base64 string
→ invoke('test_ocr_engine', {
    request: { imageBase64, engineType, configId }
  })
```

**Backend** (`cmd/ocr.rs:590-716`):

```
1. Save image bytes to temp file
2. File path → state.llm_manager.test_ocr_with_engine()
```

**test_ocr_with_engine** (model_profile_service.rs:1158-1350):

```
1. Create OcrAdapter via factory
2. Resolve ApiConfig (by configId or engine type)
3. adapter.build_prompt(OcrMode::Grounding)
4. Prepare image data URL with detail
5. Build request JSON with adapter params
6. Select ProviderAdapter → HTTP request
7. Parse response: choices[0].message.content
8. Get image dimensions (image::io::Reader)
9. adapter.parse_response(content, w, h, 0, path, Grounding)
10. Fallback: if parse fails, wrap raw content as single region
11. Return (full_text, regions)
```

---

## 5. Direct PaddleOCR REST API Path

**Separate path** — used when `engine_type == PaddleOcrApi`.

**Client**: `paddleocr_api.rs` − `PaddleOcrApiClient`

**Flow**:
```
Submit job → POST /api/v2/ocr/jobs (multipart or JSON with fileUrl)
  → Receive { data: { jobId } }
  → Poll GET /api/v2/ocr/jobs/{jobId} every 3s (max 120 attempts ~6 min)
    States: pending → running → done/failed
  → Download JSONL from resultUrl.json_url
```

**JSONL parsing**:
- VL models (PaddleOCR-VL-1.5/1.6, PP-StructureV3):
  ```
  JSONL line → layoutParsingResults[n].markdown.text + .images
  ```
- PP-OCRv5:
  ```
  JSONL line → ocrResults[n].ocrImage (base64 image, no text)
  ```

**Return**:
```rust
struct PaddleOcrResult {
    pages: Vec<PaddleOcrPage>,
    total_pages: u32,
    model: String,
}
```

**Note**: This path is NOT wired through the `OcrAdapter` trait for invocation — it's called directly when selected. The adapter's `parse_response` is not used here; instead the native JSONL parser produces results in a different `PaddleOcrResult` structure.

---

## 6. Data Types Compatibility Check (Per-Step)

| Step | Source Format | Target Format | Transformation | Compatibility |
|------|--------------|---------------|---------------|---------------|
| File input | `File` (binary) | `string` (data URL) | `FileReader.readAsDataURL()` | ✅ |
| IPC invoke | TS `object` | Rust `OcrRequest` | `serde_json` (camelCase) | ✅ — `#[serde(rename_all = "camelCase")]` on all request/response types |
| Base64 decode | `String` | `Vec<u8>` | `base64::engine::general_purpose::STANDARD.decode()` | ✅ |
| Base64 re-encode | `Vec<u8>` | `String` | `base64::engine::general_purpose::STANDARD.encode()` | ✅ — lossless roundtrip |
| Data URL | mime + base64 | `String` | `format!("data:{};base64,{}", mime, data)` | ✅ |
| Prompt | `OcrMode` | `String` | `adapter.build_prompt(mode)` | ✅ — deterministic, engine-specific |
| LLM request | Rust `Value` | HTTP JSON body | `ProviderAdapter::build_request()` | ✅ |
| LLM response | HTTP JSON body | `String` | `pointer("/choices/0/message/content")` | ✅ — assumes OpenAI-compatible format |
| Adapter parse | `String` | `OcrPageResult` | Adapter-specific parser | ✅ — always succeeds (fallback to plain text) |
| Backend return | `Vec<OcrRegion>` | `String` (`full_text`) | `regions.iter().map(|r| r.text).join("\n")` | ⚠️ — bbox data discarded at this layer |
| IPC response | Rust `OcrResponse` | TS `{ ocr_text, tags, mistake_type }` | `serde_json` (camelCase) | ✅ |
| Frontend display | `string` | React `LatexText` | `ocrText` prop | ✅ — LaTeX-safe |

### Potential Type Issues

1. **Image format inference** (paddle.rs:156-171): `infer_mime_from_data_url()` has limited detection (png/gif/webp/bmp/heic/heif — falls back to jpeg). If an image format not in this list is supplied, it gets the wrong MIME type, which could confuse some API endpoints.

2. **bbox data loss in chat_v2_perform_ocr**: The analysis mode OCR path only returns `ocr_text: String`, discarding all `OcrRegion` bounding box data. Grounding coordinates are only preserved in the test path (`test_ocr_engine`) and the internal `OcrPageResult` structure.

3. **Empty text guard** (`vfs/pdf_processing_service.rs:1991`): Empty OCR results are NOT written to DB, allowing retry. But if the model returns empty text repeatedly, the user hits a silent loop.

4. **PaddleOCR-VL repetition_penalty** (paddle.rs:91-93): Required for PaddleOCR-VL models. If omitted, these models may produce repeated text. The adapter returns `Some(1.1)`, and `call_ocr_model_raw_prompt` applies it (model2_pipeline.rs:4944-4948).

5. **Image dimensions**: `test_ocr_with_engine` reads actual image dimensions via `image::io::Reader` for bbox scaling. But `chat_v2_perform_ocr` never calls `parse_response` — it discards the adapter and gets raw text from `call_ocr_model_raw_prompt` which uses a *different* code path that parses the adapter's response differently.

---

## 7. Error Handling Chain

```
OcrError (ocr_adapters/types.rs:304-342)
  ├── Configuration(String)
  ├── Network(String)
  ├── Api { status, message }
  ├── Parse(String)
  ├── RateLimit { retry_after_ms }
  ├── ImageProcessing(String)
  └── Unsupported(String)

→ From<OcrError> for AppError (types.rs:345-370)
  → AppError::configuration / network / llm / file_system

AppError
  → chat_v2_perform_ocr returns Result<OcrResponse, String>
    (ChatV2Error::to_string() for all failures)

test_ocr_with_engine
  → Returns AppError, converted to OcrTestResponse { success: false, error: message }
  (never panics, always returns Ok with error field)
```

---

## 8. Key Files

- `C:/deep-student/src-tauri/src/ocr_adapters/types.rs` — All shared types (OcrEngineType, OcrMode, OcrRegion, OcrPageResult, OcrError)
- `C:/deep-student/src-tauri/src/ocr_adapters/mod.rs` — OcrAdapter trait + Glm4vOcrAdapter + GenericVlmAdapter
- `C:/deep-student/src-tauri/src/ocr_adapters/factory.rs` — Adapter creation + engine info + model validation
- `C:/deep-student/src-tauri/src/ocr_adapters/paddle.rs` — PaddleOcrVlAdapter (prompt + JSON bbox parsing)
- `C:/deep-student/src-tauri/src/ocr_adapters/deepseek.rs` — DeepSeekOcrAdapter (grounding tag parsing)
- `C:/deep-student/src-tauri/src/paddleocr_api.rs` — Direct PaddleOCR REST API client
- `C:/deep-student/src-tauri/src/cmd/ocr.rs` — Tauri commands for config/test (7 commands)
- `C:/deep-student/src-tauri/src/chat_v2/handlers/ocr.rs` — chat_v2_perform_ocr command
- `C:/deep-student/src-tauri/src/llm_manager/model_profile_service.rs` — get_ocr_adapter, test_ocr_with_engine, get_ocr_config_with_effective_engine
- `C:/deep-student/src-tauri/src/llm_manager/model2_pipeline.rs` — call_ocr_model_raw_prompt (core LLM invocation)
- `C:/deep-student/src/features/chat/plugins/modes/analysis.ts` — performOcr frontend trigger
- `C:/deep-student/src/features/chat/components/panels/OcrResultCard.tsx` — OCR result display
- `C:/deep-student/src/features/chat/plugins/modes/components/OcrResultHeader.tsx` — Collapsible OCR header
- `C:/deep-student/src/features/settings/components/OcrEngineCard.tsx` — Engine management UI
- `C:/deep-student/src/features/settings/components/OcrEngineTestPanel.tsx` — Multi-engine test UI
