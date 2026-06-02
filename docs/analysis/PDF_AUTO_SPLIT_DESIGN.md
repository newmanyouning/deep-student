# PDF Auto-Split for PaddleOCR API: Design Document

> Date: 2026-06-02
> Status: **Final — Implementation Ready**
> Author: Code Analysis Agent
> Last Reviewed: 2026-06-02

---

## 1. Problem Statement

The PaddleOCR AI Studio REST API (`PaddleOcrApiClient`) is a job-based OCR service. When a user submits a large PDF file (many pages or large file size), the API may silently truncate, time out, or reject the request. The current codebase has **no file size or page count validation** and **no splitting logic** -- the entire PDF is uploaded as-is.

### Specific Risks

1. **AI Studio API undocumented limits**: The AI Studio backend likely imposes constraints on:
   - Maximum upload file size (commonly 50 MB for cloud OCR APIs)
   - Maximum page count per job (commonly 100-200 pages)
   - Maximum processing time (the client's `MAX_POLL_ATTEMPTS` = 120 attempts x 3s = 6 min is already a timeout, but the server may give up earlier)

2. **Current code lacks any threshold detection**: Neither the `PaddleOcrApiClient` (`paddleocr_api.rs`) nor the adapters (`PaddleOcrApiAdapter` in `oc_adapters/paddle_api.rs`) perform pre-flight size checks.

3. **No retry with splitting**: If a job fails (e.g., API returns `"failed"` with a size-related error), there is no fallback to retry with a smaller chunk.

---

## 2. API Limits Analysis

Through code analysis of the PaddleOCR API integration (`C:/deep-student/src-tauri/src/paddleocr_api.rs`):

| Aspect | Current Code | Notes |
|--------|-------------|-------|
| File size limit | **Not enforced** -- no check in `ocr_file()` or `ocr_bytes()` | The multipart upload sends the raw bytes without any size validation |
| Page count limit | **Not enforced** -- no check before job submission | The API response includes `totalPages`/`extractedPages`, but only for progress reporting |
| Poll timeout | 120 attempts x 3s = **6 minutes** (`MAX_POLL_ATTEMPTS`) | Reasonable, but for huge PDFs even 6 min may not be enough |
| Model variants | VL-1.6, VL-1.5, PP-OCRv5, PP-StructureV3 | All models via same job endpoint, same limits likely apply |

While the exact AI Studio limits are undocumented, typical industry-standard cloud OCR API limits are:

- **File size**: 20-50 MB (multipart upload)
- **Pages per job**: 50-200 pages
- **Processing timeout**: 5-10 minutes server-side

The codebase needs defensive design regardless of exact numbers.

---

## 3. Existing Architecture

### Call Flow (PaddleOCR API path)

```
User/Frontend
    |
    v
PaddleOcrApiAdapter::call_api_file() / call_api_url()
    |  (oc_adapters/paddle_api.rs)
    v
PaddleOcrApiClient::ocr_file() / ocr_bytes() / ocr_url()
    |  (paddleocr_api.rs)
    v
POST /api/v2/ocr/jobs   (multipart or JSON)
    |
    v
Poll GET /api/v2/ocr/jobs/{jobId}
    |
    v
Download JSONL result
    |
    v
Parse into PaddleOcrResult { pages: Vec<PaddleOcrPage> }
```

### Actual Delegation Chain (Important for Integration Point)

```
ocr_file(file_path, model)
  -> reads file into bytes
  -> calls ocr_bytes(file_bytes, file_name, model)

ocr_bytes(file_bytes, file_name, model)   <-- primary entry point for file upload
  -> constructs multipart form
  -> POST to API
  -> poll_job()
  -> download_and_parse()
  -> returns PaddleOcrResult

ocr_url(file_url, model)                  <-- separate entry point for URL mode
  -> constructs JSON with fileUrl field
  -> POST to API
  -> poll_job()
  -> download_and_parse()
  -> returns PaddleOcrResult
```

**Critical insight**: `ocr_file()` is a thin wrapper that reads bytes and delegates to `ocr_bytes()`. Therefore, only TWO methods need to be modified:
- `ocr_bytes()` -- file upload path
- `ocr_url()` -- URL submission path (requires local download first)

### Callers of PaddleOCR API

1. **`exam_engine.rs`**: `call_paddle_ocr_api_raw()` -- submits single-page images (not full PDFs, already per-page rendered images). This path is safe because each call is a single image file.
2. **`exam_engine.rs`**: `call_paddle_ocr_free_text()` -- same, single image files.
3. **`model_profile_service.rs`**: `test_paddle_ocr_api_engine()` -- test with a single image.

**Key observation**: No caller in the current codebase submits a multi-page PDF to the PaddleOCR API. The callers work with individual rendered page images. The split concern arises for future use cases where multi-page PDFs might be submitted directly.

However, the `PaddleOcrApiClient::ocr_file()` and `ocr_bytes()` methods accept arbitrary file paths and bytes, so if a future caller (or the existing `ocr_url()` path for remote PDFs) submits a large multi-page PDF, the API will fail silently.

### Must also consider the VLM path

The other OCR path (VLM-based, used by `PaddleOcrVlAdapter` and `DeepSeekOcrAdapter`) processes pages one by one -- each page is rendered to an image by pdfium and OCR'd individually. This path is inherently auto-split at the page level and does NOT have this problem.

---

## 4. Proposed Auto-Split Module

### 4.1 Design Principles

1. **Transparent to callers**: Callers continue to call `ocr_file()` / `ocr_bytes()` / `ocr_url()` with a multi-page PDF. The split logic detects and splits internally.
2. **No breaking changes**: Existing function signatures remain unchanged. Return type `PaddleOcrResult` stays the same.
3. **Defensive splitting**: Use conservative thresholds to stay well within API limits.
4. **Composable**: The split logic is a self-contained module that can be reused or bypassed.
5. **Efficient**: Avoid re-rendering or re-downloading content that was already processed.

### 4.2 Module Location

**New file**: `C:/deep-student/src-tauri/src/paddleocr_split.rs`

This file contains the auto-split logic as a pure extension to `PaddleOcrApiClient`. It does NOT modify existing files except for `paddleocr_api.rs` (add 2 lines: `mod paddleocr_split;` and gate calls).

Registration in `lib.rs`: Add `pub mod paddleocr_split;` (alphabetically after `paddleocr_api`).

### 4.3 Configuration Constants

```rust
// paddleocr_split.rs

/// Default maximum file size in bytes before splitting is triggered (20 MB)
const DEFAULT_MAX_FILE_SIZE: u64 = 20 * 1024 * 1024;

/// Default maximum page count per chunk (50 pages)
const DEFAULT_MAX_PAGES_PER_CHUNK: usize = 50;

/// Default maximum concurrent split jobs (to avoid rate limiting)
const DEFAULT_MAX_CONCURRENT_CHUNKS: usize = 2;

/// Default maximum retries for chunk jobs
const DEFAULT_MAX_CHUNK_RETRIES: usize = 2;
```

**Note**: These constants are compile-time defaults. For production flexibility, they should later be exposed as configurable settings (e.g., via `AppConfig` or environment variables). For now, constants are sufficient.

### 4.4 Core Logic

#### Step 1: Detect if splitting is needed

```rust
enum SplitDecision {
    /// No splitting needed, proceed with original upload path
    NotNeeded,
    /// Splitting needed with the given pages per chunk
    Needed { chunk_pages: usize, total_pages: usize },
}

fn needs_split(file_bytes: &[u8], total_pages: usize) -> SplitDecision {
    let file_size = file_bytes.len() as u64;
    let needs_size_split = file_size > DEFAULT_MAX_FILE_SIZE;
    let needs_page_split = total_pages > DEFAULT_MAX_PAGES_PER_CHUNK;

    if !needs_size_split && !needs_page_split {
        return SplitDecision::NotNeeded;
    }

    // Calculate optimal chunk size
    let chunk_pages = if needs_page_split {
        DEFAULT_MAX_PAGES_PER_CHUNK
    } else {
        // Only size-based split: estimate pages from file size
        // Use proportion: chunk_size = (DEFAULT_MAX_FILE_SIZE / total_file_size) * total_pages
        let ratio = DEFAULT_MAX_FILE_SIZE as f64 / file_size as f64;
        let estimated = (total_pages as f64 * ratio).ceil() as usize;
        std::cmp::max(1, std::cmp::min(estimated, DEFAULT_MAX_PAGES_PER_CHUNK))
    };

    SplitDecision::Needed { chunk_pages, total_pages }
}
```

**Design decision**: When only file size exceeds the limit (but pages are reasonable), calculate per-chunk page count proportionally. When page count exceeds the limit, use `DEFAULT_MAX_PAGES_PER_CHUNK` as the chunk size. This ensures both constraints are satisfied simultaneously.

#### Step 2: Split PDF into page ranges

Uses the existing global `pdfium` instance (from `pdfium_utils.rs`). Verdict: **pdfium-render 0.8.37 fully supports the required API** (`new_pdf()`, `copy_page()`, `save_to_file()`). No additional crate needed for PDF splitting.

```rust
fn split_pdf_into_chunks(
    pdf_bytes: &[u8],
    chunk_pages: usize,
    total_pages: usize,
    temp_dir: &Path,
) -> Result<Vec<PathBuf>, PaddleOcrApiError> {
    let pdfium = crate::pdfium_utils::load_pdfium()
        .map_err(|e| PaddleOcrApiError::Api(format!("Failed to load pdfium: {}", e)))?;

    let document = pdfium.load_pdf_from_byte_slice(pdf_bytes, None)
        .map_err(|e| PaddleOcrApiError::Api(format!("Failed to load PDF: {:?}", e)))?;

    let chunk_count = (total_pages + chunk_pages - 1) / chunk_pages;
    let mut chunk_paths = Vec::with_capacity(chunk_count);

    for chunk_idx in 0..chunk_count {
        let start_page = chunk_idx * chunk_pages;
        let end_page = std::cmp::min(start_page + chunk_pages, total_pages);

        let chunk_doc = pdfium.new_pdf()
            .map_err(|e| PaddleOcrApiError::Api(format!("Failed to create chunk doc: {:?}", e)))?;

        for page_idx in (start_page as u16)..(end_page as u16) {
            let page = document.pages().get(page_idx)
                .map_err(|e| PaddleOcrApiError::Api(format!("Failed to get page {}: {:?}", page_idx, e)))?;
            chunk_doc.pages().copy_page(&page)
                .map_err(|e| PaddleOcrApiError::Api(format!("Failed to copy page {}: {:?}", page_idx, e)))?;
        }

        // Save chunk to temp file
        let chunk_path = temp_dir.join(format!("chunk_{:04}.pdf", chunk_idx));
        chunk_doc.save_to_file(&chunk_path)
            .map_err(|e| PaddleOcrApiError::Api(format!("Failed to save chunk {}: {:?}", chunk_idx, e)))?;

        tracing::debug!("[PaddleOCR-Split] Created chunk {}: pages {}-{}, path: {:?}",
            chunk_idx, start_page, end_page - 1, chunk_path);

        chunk_paths.push(chunk_path);
    }

    Ok(chunk_paths)
}
```

**Important note on pdfium-render API**: In pdfium-render 0.8.x, `PdfDocument::pages()` returns `PdfPages` which has `copy_page(&PdfPage)` method. `Pdfium::new_pdf()` creates an empty document. `PdfDocument::save_to_file()` saves to a file path. All three methods are confirmed available in version 0.8.37 currently in use.

**Fallback**: If pdfium's page-copy API proves problematic at runtime (e.g., annotations, form fields, or internal links cause issues), fall back to rendering each page range as images and re-encoding as PDF. This is slower but more reliable for complex PDFs.

#### Step 3: Upload chunks and merge results

```rust
async fn split_ocr_impl(
    client: &PaddleOcrApiClient,
    file_bytes: &[u8],
    file_name: &str,
    model: &str,
    temp_dir: &Path,
) -> Result<PaddleOcrResult, PaddleOcrApiError> {
    // Step 1: Detect if splitting is needed
    let total_pages = estimate_pdf_page_count(file_bytes)?;
    let decision = needs_split(file_bytes, total_pages);

    let SplitDecision::Needed { chunk_pages, total_pages } = decision else {
        // No split needed: use original path
        return client.ocr_bytes_inner(file_bytes, file_name, model).await;
    };

    tracing::info!(
        "[PaddleOCR-Split] Splitting PDF: {} pages, chunk size={} pages, {} chunks",
        total_pages, chunk_pages,
        (total_pages + chunk_pages - 1) / chunk_pages
    );

    // Step 2: Split into chunk PDF files
    let chunk_paths = split_pdf_into_chunks(file_bytes, chunk_pages, total_pages, temp_dir)?;

    // Step 3: Upload chunks concurrently with semaphore
    let semaphore = Arc::new(tokio::sync::Semaphore::new(DEFAULT_MAX_CONCURRENT_CHUNKS));
    let mut join_set = tokio::task::JoinSet::new();

    for (idx, chunk_path) in chunk_paths.into_iter().enumerate() {
        let sem = semaphore.clone();
        let model = model.to_string();
        let chunk_name = format!("{}_chunk_{}", file_name, idx);
        let chunk_path_str = chunk_path.to_string_lossy().to_string();

        join_set.spawn(async move {
            let _permit = sem.acquire().await.unwrap();
            let mut attempt = 0;
            loop {
                // Re-create client for each attempt to avoid state pollution
                // Note: PaddleOcrApiClient is cheap to construct (just a reqwest::Client + token)
                // We need the token to be accessible. In split_ocr_impl, client is &PaddleOcrApiClient.
                // For spawned tasks, we clone the inner client and token.
                match client.ocr_file(&chunk_path_str, &model).await {
                    Ok(result) => return (idx, Ok(result), chunk_path_str),
                    Err(e) => {
                        attempt += 1;
                        if attempt >= DEFAULT_MAX_CHUNK_RETRIES {
                            return (idx, Err(e), chunk_path_str);
                        }
                        tokio::time::sleep(Duration::from_secs(2u64.pow(attempt))).await;
                    }
                }
            }
        });
    }

    // Collect results
    let mut results: Vec<(usize, Vec<PaddleOcrPage>)> = Vec::new();
    let mut errors: Vec<String> = Vec::new();

    while let Some(joined) = join_set.join_next().await {
        match joined {
            Ok((idx, Ok(chunk_result), path)) => {
                let offset = idx * chunk_pages;
                let adjusted_pages: Vec<PaddleOcrPage> = chunk_result.pages
                    .into_iter()
                    .map(|mut page| {
                        page.page_index += offset as u32;
                        page
                    })
                    .collect();
                results.push((idx, adjusted_pages));
                // Best-effort cleanup
                let _ = std::fs::remove_file(&path);
            }
            Ok((idx, Err(e), path)) => {
                errors.push(format!("Chunk {} failed: {}", idx, e));
                let _ = std::fs::remove_file(&path);
            }
            Err(e) => {
                errors.push(format!("Join error: {:?}", e));
            }
        }
    }

    // Step 4: Return partial or total result
    if results.is_empty() {
        return Err(PaddleOcrApiError::Api(format!(
            "All {} PDF chunks failed: {}",
            chunk_paths.len(),
            errors.join("; ")
        )));
    }

    if !errors.is_empty() {
        tracing::warn!(
            "[PaddleOCR-Split] {}/{} chunks had errors: {}",
            errors.len(),
            chunk_paths.len(),
            errors.join("; ")
        );
    }

    // Merge results in original page order
    results.sort_by_key(|(idx, _)| *idx);
    let all_pages: Vec<PaddleOcrPage> = results.into_iter()
        .flat_map(|(_, pages)| pages)
        .collect();

    Ok(PaddleOcrResult {
        total_pages: all_pages.len() as u32,
        pages: all_pages,
        model: model.to_string(),
    })
}
```

**Critical API consideration**: The spawned tasks in `JoinSet` need access to the `PaddleOcrApiClient`. Since `PaddleOcrApiClient` contains `reqwest::Client` (which is `Clone`) and `token: String` (which is `Clone`), the simplest approach is to clone the client for each chunk within the async block. However, because `PaddleOcrApiClient` doesn't directly implement `Clone`, we need to either:
1. Derive/impl `Clone` for `PaddleOcrApiClient` (recommended, since both fields are Clone)
2. Construct a new client per chunk using `PaddleOcrApiClient::new(token.clone())`

Option 2 is simpler and avoids changes to the existing struct.

### 4.5 Integration Point

The split logic integrates into `PaddleOcrApiClient` by modifying `ocr_bytes()` and `ocr_url()` to gate through the split module.

#### Modifications to `paddleocr_api.rs`

**A. Add at top of file:**
```rust
pub(crate) mod paddleocr_split;
```

**B. In `ocr_bytes()` method:**
Insert at the beginning of the method body, after variable declarations but before the main upload flow:

```rust
// Auto-split: if the PDF exceeds size/page limits, split and OCR in parallel
// This is transparent to the caller — same return type, same semantics.
if file_name.ends_with(".pdf") {
    let temp_dir = paddleocr_split::create_temp_dir()?;
    let result = paddleocr_split::maybe_split_and_ocr(
        self, file_bytes, file_name, model, &temp_dir
    ).await;
    // Cleanup temp dir on completion
    let _ = std::fs::remove_dir_all(&temp_dir);
    if let Ok(ref result) = result {
        return Ok(result.clone());  // Note: PaddleOcrResult needs Clone
    }
    // If split logic returned an error, log and fall through to original path
    // (This provides resilience: if splitting infrastructure fails, we still try the original upload)
    tracing::warn!("[PaddleOCR-Split] Split failed, falling back to direct upload: {:?}", result.err());
}
```

**However**, the above approach has a subtlety: `maybe_split_and_ocr` already calls `ocr_bytes_inner` (the original upload path) when splitting is not needed. So the flow should be:

```rust
// In ocr_bytes(), early return when split logic handles it:
if file_name.ends_with(".pdf") {
    let temp_dir = paddleocr_split::create_temp_dir()?;
    let result = paddleocr_split::maybe_split_and_ocr(
        self, file_bytes, file_name, model, &temp_dir
    ).await;
    let _ = std::fs::remove_dir_all(&temp_dir);
    // maybe_split_and_ocr returns Ok if split was handled (either original path or split path)
    // It returns Err only if the entire operation failed
    result?; // Propagate error or continue
}
```

**Wait** — this double-counts: if `maybe_split_and_ocr` decides split is not needed, it already calls `ocr_bytes_inner()`. So the outer `ocr_bytes()` body becomes dead code in that case. This is fine — the body is just bypassed.

**Better approach**: Refactor `ocr_bytes()` so the existing upload logic is extracted as `ocr_bytes_inner()`, then `ocr_bytes()` becomes a thin wrapper:

```rust
/// Internal implementation of OCR bytes upload (no auto-split)
async fn ocr_bytes_inner(
    &self,
    file_bytes: &[u8],
    file_name: &str,
    model: &str,
) -> Result<PaddleOcrResult, PaddleOcrApiError> {
    // ... existing logic unchanged ...
}

/// Public API: auto-splits PDFs if needed
pub async fn ocr_bytes(
    &self,
    file_bytes: &[u8],
    file_name: &str,
    model: &str,
) -> Result<PaddleOcrResult, PaddleOcrApiError> {
    if file_name.ends_with(".pdf") {
        let temp_dir = paddleocr_split::create_temp_dir()?;
        let result = paddleocr_split::maybe_split_and_ocr(
            self, file_bytes, file_name, model, &temp_dir
        ).await;
        let _ = std::fs::remove_dir_all(&temp_dir);
        return result;
    }
    // Non-PDF files pass through unchanged
    self.ocr_bytes_inner(file_bytes, file_name, model).await
}
```

#### For `ocr_url()` (URL mode)

The URL mode is different: the server downloads the file itself. To split by URL, we must:
1. Download the file locally first
2. Apply split logic
3. Upload each chunk via `ocr_bytes()` (multipart upload)

```rust
pub async fn ocr_url(
    &self,
    file_url: &str,
    model: &str,
) -> Result<PaddleOcrResult, PaddleOcrApiError> {
    if file_url.ends_with(".pdf") || file_url.contains(".pdf?") {
        // Download the PDF locally for possible splitting
        let resp = self.client.get(file_url).send().await?;
        let bytes = resp.bytes().await?;
        let file_name = std::path::Path::new(file_url)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("remote.pdf");

        // Now same path as ocr_bytes
        if bytes.len() > 0 {
            let temp_dir = paddleocr_split::create_temp_dir()?;
            let result = paddleocr_split::maybe_split_and_ocr(
                self, &bytes, file_name, model, &temp_dir
            ).await;
            let _ = std::fs::remove_dir_all(&temp_dir);
            return result;
        }
    }
    // Non-PDF or small PDF URLs pass through to original URL logic
    self.ocr_url_inner(file_url, model).await
}
```

This means `ocr_url()` also needs its body extracted to `ocr_url_inner()`.

#### Summary of changes to `paddleocr_api.rs`

| Line | Change |
|------|--------|
| Top of file | Add `pub(crate) mod paddleocr_split;` |
| `struct PaddleOcrResult` | Derive or impl `Clone` (needed for early return paths) |
| `ocr_bytes()` | Split into `ocr_bytes()` (public, with split gate) + `ocr_bytes_inner()` (existing logic) |
| `ocr_url()` | Split into `ocr_url()` (public, with download+split gate) + `ocr_url_inner()` (existing logic) |
| Imports | Add `std::path::Path`, `tracing` (already present) |

### 4.6 Detailed Method Signatures in `paddleocr_split.rs`

```rust
// === Public API (called from paddleocr_api.rs) ===

/// Create a temporary directory for split chunks.
/// Path: {temp_dir}/paddleocr_split/{uuid}/
pub fn create_temp_dir() -> Result<PathBuf, PaddleOcrApiError>;

/// Main entry point: check if splitting is needed, split if so, OCR chunks, merge results.
/// If splitting is not needed, delegates to client.ocr_bytes_inner().
pub async fn maybe_split_and_ocr(
    client: &PaddleOcrApiClient,
    file_bytes: &[u8],
    file_name: &str,
    model: &str,
    temp_dir: &Path,
) -> Result<PaddleOcrResult, PaddleOcrApiError>;

// === Internal functions ===

/// Estimate PDF page count by loading with pdfium.
/// Returns the number of pages, or error if PDF is invalid.
fn estimate_pdf_page_count(bytes: &[u8]) -> Result<usize, PaddleOcrApiError>;

/// Determine if splitting is needed based on file size and page count.
fn needs_split(file_bytes: &[u8], total_pages: usize) -> SplitDecision;

/// Split PDF bytes into chunk files, each containing chunk_pages pages.
/// Returns paths to all chunk files.
fn split_pdf_into_chunks(
    pdf_bytes: &[u8],
    chunk_pages: usize,
    total_pages: usize,
    temp_dir: &Path,
) -> Result<Vec<PathBuf>, PaddleOcrApiError>;

/// Orchestrate chunk upload, collect results, merge in order.
async fn split_ocr_impl(
    client: &PaddleOcrApiClient,
    file_bytes: &[u8],
    file_name: &str,
    model: &str,
    chunk_pages: usize,
    total_pages: usize,
    temp_dir: &Path,
) -> Result<PaddleOcrResult, PaddleOcrApiError>;
```

---

## 5. Quick Page Count Estimation (Optimization)

**Design decision**: Use pdfium directly for page count estimation. The concern about loading the full PDF just to count pages is mitigated because:

1. pdfium's `load_pdf_from_byte_slice()` is fast for metadata-only loading (lazy page rendering)
2. If splitting is needed, we already need pdfium loaded for the split operation
3. The quick check `file_bytes.starts_with(b"%PDF-")` filters out non-PDFs without any library call

```rust
/// Quick PDF page count by loading with pdfium.
/// Fast because pdfium does not render pages during metadata loading.
fn estimate_pdf_page_count(bytes: &[u8]) -> Result<usize, PaddleOcrApiError> {
    if !bytes.starts_with(b"%PDF-") {
        return Err(PaddleOcrApiError::Api("Not a PDF file".to_string()));
    }

    let pdfium = crate::pdfium_utils::load_pdfium()
        .map_err(|e| PaddleOcrApiError::Api(format!("Failed to load pdfium: {}", e)))?;

    let document = pdfium.load_pdf_from_byte_slice(bytes, None)
        .map_err(|e| PaddleOcrApiError::Api(format!("Failed to load PDF: {:?}", e)))?;

    Ok(document.pages().len())
}
```

**Decision NOT to use `lopdf` for lightweight parsing**: Adding a new crate dependency (`lopdf`) for page count estimation is not justified because:
- pdfium is already loaded and available globally
- pdfium's lazy loading makes metadata extraction fast
- The split operation requires pdfium anyway
- Reduced dependency maintenance burden

If performance profiling later shows that pdfium loading is a bottleneck for small PDFs, this can be revisited.

---

## 6. Temp File Management

### Directory Structure

```
{temp_dir}/paddleocr_split/{uuid}/
    chunk_0000.pdf
    chunk_0001.pdf
    ...
```

### Cleanup Strategy

| Level | Timing | Method |
|-------|--------|--------|
| Per chunk file | After chunk OCR completes (success or failure) | `std::fs::remove_file()` |
| Session temp dir | After all chunks processed | `std::fs::remove_dir_all()` in caller |
| Orphaned dirs | On service startup | Background task deletes dirs older than 1 hour |

```rust
pub fn create_temp_dir() -> Result<PathBuf, PaddleOcrApiError> {
    let base = std::env::temp_dir().join("paddleocr_split");
    let dir = base.join(uuid::Uuid::new_v4().to_string());
    std::fs::create_dir_all(&dir)
        .map_err(|e| PaddleOcrApiError::Io(e))?;
    Ok(dir)
}
```

Note: `uuid` is already a dependency (used extensively in the codebase).

**Orphan cleanup**: Add a function called at app startup:
```rust
pub fn cleanup_orphaned_temp_dirs() {
    let base = std::env::temp_dir().join("paddleocr_split");
    if !base.exists() { return; }
    let cutoff = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() - 3600; // 1 hour ago

    if let Ok(entries) = std::fs::read_dir(&base) {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                if let Ok(created) = metadata.created() {
                    if let Ok(age) = created.duration_since(std::time::UNIX_EPOCH) {
                        if age.as_secs() < cutoff {
                            let _ = std::fs::remove_dir_all(entry.path());
                        }
                    }
                }
            }
        }
    }
}
```

---

## 7. Error Handling Strategy

### Per-Chunk Error Handling

| Scenario | Handling |
|----------|----------|
| Chunk upload fails (transient) | Retry up to `DEFAULT_MAX_CHUNK_RETRIES` with exponential backoff (2s, 4s, 8s) |
| Chunk API returns job failure | No retry — report as chunk error |
| Chunk API times out | No retry — report as chunk error |
| Chunk succeeds, others fail | Return partial results with warning log |

### Overall Error Handling

| Scenario | Handling | Code Path |
|----------|----------|-----------|
| Page count cannot be determined | Return error to caller | `estimate_pdf_page_count()` returns Err |
| PDF cannot be parsed by pdfium | Return error to caller | `split_pdf_into_chunks()` returns Err |
| Some chunks succeed, some fail | Return partial results (successful chunks only) with `tracing::warn!` | `split_ocr_impl()` |
| All chunks fail | Return combined error message listing all chunk failures | `split_ocr_impl()` |
| Temp directory creation fails | Return IO error | `create_temp_dir()` |
| Single chunk exceeds size limit after split | Log warning, still attempt upload (server may accept) | Automatic (pdfium may create larger files than expected) |
| Split infrastructure fails (pdfium not available) | Log warning, fall back to original upload path | `ocr_bytes()` fallback |

### Partial Result Contract

When returning partial results:
- `PaddleOcrResult::total_pages` = number of pages from successful chunks
- `PaddleOcrResult::pages` = pages from successful chunks, in correct page order
- Missing pages are silently omitted (no placeholder)
- A warning is logged for each failed chunk

This is a pragmatic choice: for a study tool, partial OCR results are more useful than no results. Students can see what was successfully OCR'd and retry failed pages manually.

---

## 8. Backward Compatibility Analysis

### Caller Compatibility

| Caller | Current Input | After Change | Impact |
|--------|--------------|--------------|--------|
| `PaddleOcrApiAdapter::call_api_file()` | File path (single image) | Same | None — single images are not PDFs; split gate triggers only for `.pdf` |
| `PaddleOcrApiAdapter::call_api_url()` | URL | Same | None — same reasoning |
| Any future caller submitting multi-page PDFs | PDF bytes | Transparent auto-split | Positive — no caller-side changes needed |

### API Surface Compatibility

| Aspect | Current | After | Break? |
|--------|---------|-------|--------|
| `ocr_file()` signature | `(path, model) -> PaddleOcrResult` | Same | No |
| `ocr_bytes()` signature | `(bytes, name, model) -> PaddleOcrResult` | Same | No |
| `ocr_url()` signature | `(url, model) -> PaddleOcrResult` | Same | No |
| `PaddleOcrResult` | `{pages, total_pages, model}` | Same | No |
| `PaddleOcrPage` | `{page_index, markdown_text, images}` | Same | No |
| Error types | `PaddleOcrApiError` variants | Same + `Io` variant may now be hit | No — `Io` already exists |

### Behavioral Compatibility

| Behavior | Current | After | Difference |
|----------|---------|-------|------------|
| Single-page PDF | Uploaded directly | Same path (split not needed) | Identical |
| Small multi-page PDF | Uploaded directly | Same path | Identical |
| Large multi-page PDF | May fail/reach timeout | Split into chunks | Better |
| Non-PDF file | Uploaded directly | Same path | Identical |

---

## 9. Testing Plan

### Unit Tests (in `paddleocr_split.rs`)

1. **`needs_split()`**:
   - 10-page, 5 MB PDF -> `NotNeeded`
   - 100-page, 5 MB PDF -> `Needed { chunk_pages: 50 }` (exceeds page limit)
   - 10-page, 100 MB PDF -> `Needed { chunk_pages: 1 }` (exceeds file size, conservative estimate: 1 page)
   - 200-page, 200 MB PDF -> `Needed { chunk_pages: 50 }` (both limits exceeded, cap at 50)
   - Empty file (0 bytes) -> `NotNeeded` (edge: zero-size check before file size comparison)
   - 50-page PDF, exactly at limits -> `NotNeeded` (edge: boundary testing)

2. **`estimate_pdf_page_count()`**:
   - Valid 3-page PDF -> returns `Ok(3)`
   - Truncated/invalid PDF -> returns `Err`
   - Non-PDF bytes -> returns `Err`

3. **`split_pdf_into_chunks()`**:
   - 10-page PDF, chunk_pages=5 -> 2 chunks, each with 5 pages
   - 1-page PDF, chunk_pages=5 -> 1 chunk
   - 12-page PDF, chunk_pages=5 -> 3 chunks (5, 5, 2 pages)
   - Total page count across chunks equals original

### Integration Tests

4. **Result merging**:
   - Two chunks with pages [0,1,2] and [3,4,5] -> merged into [0,1,2,3,4,5]
   - Page index offsets: chunk 2 should have page_index starting at 5
   - Partial result: only chunk 0 succeeds -> returns 3 pages

5. **Mock PaddleOCR API server**:
   - Submit large PDF -> verify chunks are uploaded
   - Verify final merged result matches original page order
   - Simulate chunk failure -> verify partial results

### Manual Testing

6. **Real PaddleOCR API**:
   - Submit a 60-page PDF (1 page over 50-page limit) -> verify it gets split into 2 chunks and merged correctly
   - Submit a 200 MB single-page PDF (unlikely, but tests file size limit)

---

## 10. File Structure Changes

```
src-tauri/src/
  paddleocr_api.rs        (modify: add mod declaration, extract ocr_bytes_inner/ocr_url_inner,
                           gate ocr_bytes() and ocr_url() through split logic, add Clone to PaddleOcrResult)
  paddleocr_split.rs      (NEW: ~280 lines — needs_split, estimate_pdf_page_count,
                           split_pdf_into_chunks, split_ocr_impl, maybe_split_and_ocr,
                           create_temp_dir, cleanup_orphaned_temp_dirs, tests)
  lib.rs                  (add: pub mod paddleocr_split;)
  pdfium_utils.rs         (no change needed)
  ocr_adapters/
    paddle_api.rs         (no change needed)
  vfs/
    pdf_processing_service.rs  (no change needed)
  llm_manager/
    exam_engine.rs        (no change needed)
```

### Estimated LoC

| File | Change | Lines |
|------|--------|-------|
| `paddleocr_api.rs` | Add `mod`, extract 2 inner methods, gate 2 public methods, add Clone | ~40 |
| `paddleocr_split.rs` | New file (split logic + helpers + tests) | ~280 |
| `lib.rs` | Add 1 line | ~1 |
| **Total** | | **~321 lines** |

### Detailed Code Change Plan for `paddleocr_api.rs`

```
1. Top of file, after existing imports:
   pub(crate) mod paddleocr_split;

2. On PaddleOcrResult, derive Clone:
   #[derive(Debug, Clone)]

3. In impl PaddleOcrApiClient:
   a. Rename existing ocr_bytes() to ocr_bytes_inner() — exact same body
   b. Create new ocr_bytes():
      - Check if file_name ends with ".pdf"
      - If yes: create_temp_dir() -> maybe_split_and_ocr() -> remove_dir_all() -> return result
      - If no: delegate to ocr_bytes_inner()
   c. Rename existing ocr_url() to ocr_url_inner() — exact same body
   d. Create new ocr_url():
      - Check if file_url ends with ".pdf" or contains ".pdf?"
      - If yes: download bytes -> same as ocr_bytes() PDF path
      - If no: delegate to ocr_url_inner()
```

---

## 11. Future Considerations

### 11.1 Smarter Chunking

The current design splits by equal page ranges (e.g., pages 0-49, 50-99, ...). A smarter approach could:
- Estimate per-page size based on the first few pages and adjust chunk sizes dynamically.
- Detect natural section boundaries (e.g., chapter breaks in extracted text) for cleaner splits.

### 11.2 Caching

If the same PDF is submitted multiple times, cached split chunks could avoid re-splitting. The existing cache in `pdf_ocr_service.rs` (SHA-256 hash based) could be extended to cache per-page OCR results from the API path.

### 11.3 Progressive Upload

Instead of writing chunks to disk before uploading, render + upload pages in a streaming fashion. This saves I/O and reduces latency, but requires the API to support incremental page submission (which the current job-based API does not).

### 11.4 Configurable Thresholds

The constants `DEFAULT_MAX_FILE_SIZE` and `DEFAULT_MAX_PAGES_PER_CHUNK` should be promoted to runtime configuration (e.g., in `AppConfig` or environment variables) once production usage establishes optimal values.

### 11.5 Non-PDF Split

If future callers submit other multi-page document formats (e.g., TIFF, DJVU), the split module could be extended to handle those as well.

---

## 12. Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| pdfium-render `copy_page()` fails on complex PDFs (forms, annotations) | Split produces incorrect pages | Log discrepancy; use rendered-image-per-page fallback (render each page range to images, re-encode as PDF) |
| Split PDFs have larger total size than original (pdfium re-encodes) | Each chunk may still exceed API file size limit | Log warning; still attempt upload; if persistent, reduce chunk_pages further |
| Temp files fill up disk for concurrent large PDFs | Disk space exhaustion | Enforce max concurrent splits via semaphore (2), cleanup aggressively, monitor temp dir size, orphan cleanup on startup |
| PaddleOCR API accepts large PDFs after all (limits are higher than assumed) | Unnecessary splitting overhead | Tune thresholds after production observation; make configurable via settings |
| Downloaded content in `ocr_url()` path is not a PDF | Extra download overhead for non-PDF URLs | Check `Content-Type` header before downloading full body; only `.pdf` extension check first |
| `PaddleOcrApiClient` mutable logging or state | Race conditions in concurrent chunk processing | `PaddleOcrApiClient` is stateless (holds only `reqwest::Client` + token); concurrent access is safe |
| pdfium not available at runtime | Cannot split, fall back to original upload path | The split gate gracefully falls back when pdfium loading fails |

### Fallback Chain (Defense in Depth)

```
1. Detect .pdf extension
2. Attempt pdfium load for page count
   |-- Fail? → Fall through to original upload (single job, may fail)
3. Check if split needed by size/page count
   |-- Not needed? → Delegate to ocr_bytes_inner() (original path)
4. Create temp dir
   |-- Fail? → Return IO error to caller
5. Split PDF into chunks
   |-- Fail? → Return error (PDF parsing failure)
6. Submit chunks concurrently
   |-- Partial failure? → Return partial results with warning
   |-- Total failure? → Return combined error
```

---

## 13. Appendix: Key File References

| File | Purpose |
|------|---------|
| `C:/deep-student/src-tauri/src/paddleocr_api.rs` | PaddleOCR REST API client -- the target for integration |
| `C:/deep-student/src-tauri/src/paddleocr_split.rs` | **NEW** -- auto-split logic (~280 lines) |
| `C:/deep-student/src-tauri/src/ocr_adapters/paddle_api.rs` | OCR adapter wrapping the API client |
| `C:/deep-student/src-tauri/src/pdfium_utils.rs` | Global pdfium instance used for PDF splitting (already available, version 0.8.37) |
| `C:/deep-student/src-tauri/src/pdf_ocr_service.rs` | Existing PDF OCR pipeline with caching (alternative OCR path) |
| `C:/deep-student/src-tauri/src/vfs/pdf_processing_service.rs` | Media processing pipeline with OCR stage |
| `C:/deep-student/src-tauri/src/llm_manager/exam_engine.rs` | Uses per-page images; no splitting needed |
| `C:/deep-student/src-tauri/src/lib.rs` | Module registration for `paddleocr_split` |
| `C:/deep-student/src-tauri/Cargo.toml` | No new dependencies needed (pdfium-render 0.8.37 already included) |
