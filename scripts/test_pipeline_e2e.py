#!/usr/bin/env python3
"""
PaddleOCR Pipeline E2E Test

Tests the full PaddleOCR API pipeline matching the Rust paddleocr_api.rs implementation:
1. Auth header format comparison (Bearer vs bearer) on a public image URL
2. File upload mode with real scanned PDF (multipart form)
3. Job polling (5s interval, 60 attempts max)
4. JSONL download and parse
5. Results saved to markdown
6. Validation of output

Logging format matches Rust pipeline: [Pipeline::StepName] key=value
"""

import sys
import os
import json
import time
import requests
from pathlib import Path
from datetime import datetime, timezone

# ============================================================================
# Configuration
# ============================================================================

API_BASE = "https://paddleocr.aistudio-app.com/api/v2"
API_TOKEN = "950b164023143c0c95c1fe691c46e320758b00d0"
MODEL = "PaddleOCR-VL-1.6"

# Test image URL (public Wikimedia OCR test image)
TEST_IMAGE_URL = (
    "https://upload.wikimedia.org/wikipedia/commons/thumb/7/7e/"
    "Document_OCR_test.png/800px-Document_OCR_test.png"
)

# Real scanned PDF
PDF_PATH = Path(r"C:/deep-student/scripts/《实变函数解题指南》周民强.pdf")

# Output directory
OUTPUT_DIR = Path(r"C:/deep-student/scripts/ocr_output")
OUTPUT_MD = OUTPUT_DIR / "e2e_test_output.md"

POLL_INTERVAL = 5       # seconds
MAX_POLL_ATTEMPTS = 60  # 5 minutes max

# ============================================================================
# Logging helpers (Rust pipeline format)
# ============================================================================

def timestamp():
    return datetime.now(timezone.utc).strftime("%Y-%m-%d %H:%M:%S.%f")[:-3]

def log_pipeline(step, **kwargs):
    """Log in Rust pipeline format: [Pipeline::StepName] key=value ..."""
    parts = [f"{k}={v}" for k, v in kwargs.items()]
    print(f"[{timestamp()}] [Pipeline::{step}] {' '.join(parts)}", flush=True)

def log_info(msg):
    print(f"[{timestamp()}] [INFO] {msg}", flush=True)

def log_warn(msg):
    print(f"[{timestamp()}] [WARN] {msg}", flush=True)

def log_error(msg):
    print(f"[{timestamp()}] [ERROR] {msg}", flush=True)

# ============================================================================
# Test counters
# ============================================================================

PASS = 0
FAIL = 0
SKIP = 0

def result(status, test_name, detail=""):
    global PASS, FAIL, SKIP
    if status == "PASS":
        PASS += 1
        log_info(f"RESULT: {test_name} -> PASS")
    elif status == "FAIL":
        FAIL += 1
        log_info(f"RESULT: {test_name} -> FAIL -- {detail}")
    elif status == "SKIP":
        SKIP += 1
        log_info(f"RESULT: {test_name} -> SKIP -- {detail}")
    if detail and status != "FAIL":
        log_info(f"         {detail}")


# ============================================================================
# Step 1: Auth header comparison
# ============================================================================

def test_auth_header(variant: str) -> dict:
    """
    Test a specific Authorization header format.
    variant: "Bearer" (RFC 6750 standard) or "bearer" (lowercase, matches Rust code)
    """
    step = "AuthHeaderTest"
    log_pipeline(step, variant=variant, url=TEST_IMAGE_URL)

    headers = {
        "Authorization": f"{variant} {API_TOKEN}",
        "Content-Type": "application/json",
    }

    payload = {
        "model": MODEL,
        "fileUrl": TEST_IMAGE_URL,
        "optionalPayload": {
            "useDocOrientationClassify": False,
            "useDocUnwarping": False,
            "useChartRecognition": False,
            "useTextlineOrientation": False,
        },
    }

    start = time.time()
    try:
        resp = requests.post(
            f"{API_BASE}/ocr/jobs",
            headers=headers,
            json=payload,
            timeout=30,
        )
        elapsed_ms = int((time.time() - start) * 1000)
        status_code = resp.status_code
        body_preview = resp.text[:300]

        log_pipeline(
            f"{step}Response",
            variant=variant,
            status_code=status_code,
            elapsed_ms=elapsed_ms,
            body_preview=body_preview.replace("\n", "\\n"),
        )

        if status_code == 200:
            try:
                data = resp.json()
                job_id = data.get("data", {}).get("jobId", "unknown")
                log_pipeline(
                    "JobSubmitted",
                    job_id=job_id,
                    model=MODEL,
                    file_url=TEST_IMAGE_URL,
                    auth_variant=variant,
                )
                return {"status": "ok", "job_id": job_id, "auth": variant}
            except Exception as e:
                return {"status": "parse_error", "error": str(e), "auth": variant}
        elif status_code == 401:
            log_pipeline(f"{step}Unauthorized", variant=variant, status_code=status_code)
            return {"status": "unauthorized", "auth": variant}
        else:
            log_pipeline(f"{step}Unexpected", variant=variant, status_code=status_code)
            return {"status": f"http_{status_code}", "auth": variant}

    except requests.exceptions.Timeout:
        log_pipeline(f"{step}Timeout", variant=variant)
        return {"status": "timeout", "auth": variant}
    except requests.exceptions.ConnectionError as e:
        log_pipeline(f"{step}ConnectionError", variant=variant, error=str(e))
        return {"status": "connection_error", "auth": variant}
    except Exception as e:
        log_pipeline(f"{step}Exception", variant=variant, error=str(e))
        return {"status": "exception", "error": str(e), "auth": variant}


# ============================================================================
# Step 2: Submit file upload
# ============================================================================

def submit_file_job(file_path: Path, auth: str = "bearer") -> str:
    """Submit an OCR job via file upload (multipart form)."""
    file_size = file_path.stat().st_size
    filename = file_path.name
    log_pipeline(
        "FileUploadSubmit",
        file=filename,
        file_size_bytes=file_size,
        file_size_mb=f"{file_size / (1024 * 1024):.2f}",
        model=MODEL,
        auth=auth,
    )

    optional_payload = json.dumps({
        "useDocOrientationClassify": False,
        "useDocUnwarping": False,
        "useChartRecognition": False,
        "useTextlineOrientation": False,
    })

    with open(file_path, "rb") as f:
        files = {
            "model": (None, MODEL),
            "optionalPayload": (None, optional_payload),
            "file": (filename, f, "application/octet-stream"),
        }
        headers = {
            "Authorization": f"{auth} {API_TOKEN}",
        }
        resp = requests.post(
            f"{API_BASE}/ocr/jobs",
            headers=headers,
            files=files,
            timeout=120,
        )

    if not resp.ok:
        error_body = resp.text[:500]
        log_pipeline(
            "FileUploadFailed",
            status_code=resp.status_code,
            error=error_body.replace("\n", "\\n"),
        )
        raise RuntimeError(
            f"File job submission failed (HTTP {resp.status_code}): {error_body}"
        )

    data = resp.json()
    job_id = data["data"]["jobId"]

    log_pipeline(
        "FileUploadSuccess",
        job_id=job_id,
        file=filename,
        file_size_bytes=file_size,
    )
    return job_id


# ============================================================================
# Step 3: Poll job
# ============================================================================

def poll_job(job_id: str) -> tuple:
    """Poll job status until completion. Returns (jsonl_url, total_pages, extracted_pages)."""
    log_pipeline(
        "PollingStart",
        job_id=job_id,
        poll_interval_secs=POLL_INTERVAL,
        max_attempts=MAX_POLL_ATTEMPTS,
    )

    poll_timer_start = time.time()

    # Use same auth as our winning variant from Step 1
    headers = {"Authorization": f"bearer {API_TOKEN}"}

    for attempt in range(1, MAX_POLL_ATTEMPTS + 1):
        elapsed_ms = int((time.time() - poll_timer_start) * 1000)

        resp = requests.get(
            f"{API_BASE}/ocr/jobs/{job_id}",
            headers=headers,
            timeout=15,
        )

        if not resp.ok:
            raise RuntimeError(f"Poll failed (HTTP {resp.status_code}): {resp.text[:300]}")

        status_data = resp.json()["data"]
        state = status_data["state"]
        progress = status_data.get("extractProgress", {})
        extracted = progress.get("extractedPages", 0)
        total = progress.get("totalPages", 0)
        error_msg = status_data.get("errorMsg")

        if state == "done":
            jsonl_url = status_data["resultUrl"]["jsonUrl"]
            log_pipeline(
                "JobCompleted",
                job_id=job_id,
                pages=extracted,
                total_pages=total,
                elapsed_ms=elapsed_ms,
                attempt=attempt,
            )
            log_info(f"Job {job_id} completed: {extracted}/{total} pages in {elapsed_ms}ms")
            return jsonl_url, total, extracted

        elif state == "failed":
            log_pipeline(
                "JobFailed",
                job_id=job_id,
                error=error_msg or "Unknown error",
                elapsed_ms=elapsed_ms,
                attempt=attempt,
            )
            raise RuntimeError(f"Job failed: {error_msg or 'Unknown error'}")

        elif state == "running":
            log_pipeline(
                "JobPolling",
                job_id=job_id,
                state="running",
                pages_done=extracted,
                pages_total=total,
                attempt=attempt,
                elapsed_ms=elapsed_ms,
            )

        else:
            # pending / other states
            if attempt % 10 == 1 or attempt == MAX_POLL_ATTEMPTS:
                log_pipeline(
                    "JobPolling",
                    job_id=job_id,
                    state=state,
                    pages_done=extracted,
                    pages_total=total,
                    attempt=attempt,
                    elapsed_ms=elapsed_ms,
                )

        if attempt >= MAX_POLL_ATTEMPTS:
            log_pipeline(
                "JobTimeout",
                job_id=job_id,
                max_attempts=MAX_POLL_ATTEMPTS,
                elapsed_ms=elapsed_ms,
            )
            raise TimeoutError(f"Job {job_id} timed out after {MAX_POLL_ATTEMPTS} attempts")

        time.sleep(POLL_INTERVAL)

    raise TimeoutError(f"Job {job_id} timed out after {MAX_POLL_ATTEMPTS} attempts")


# ============================================================================
# Step 4: Download and parse JSONL
# ============================================================================

def download_and_parse(jsonl_url: str) -> list:
    """Download JSONL results and parse into pages."""
    log_pipeline("JsonlDownload", url=jsonl_url)

    resp = requests.get(jsonl_url, timeout=60)
    if not resp.ok:
        raise RuntimeError(f"Download failed (HTTP {resp.status_code})")

    body = resp.text
    lines = [l for l in body.strip().split("\n") if l.strip()]
    line_count = len(lines)
    size_bytes = len(body)

    log_pipeline(
        "JsonlDownloaded",
        url=jsonl_url,
        size_bytes=size_bytes,
        lines=line_count,
    )
    log_info(f"Downloaded {size_bytes} bytes, {line_count} JSONL lines")

    pages = []
    parse_errors = 0

    for line_num, line in enumerate(lines, 1):
        try:
            parsed = json.loads(line)
            result_block = parsed.get("result", {})

            # VL series: layoutParsingResults -> markdown.text + markdown.images
            layout_results = result_block.get("layoutParsingResults", [])
            for item in layout_results:
                md = item.get("markdown", {})
                text = md.get("text", "")
                images = md.get("images", {})

                pages.append({
                    "page_index": line_num - 1,
                    "markdown_text": text,
                    "images": list(images.keys()),
                    "image_urls": images,
                })

            # PP-OCRv5: ocrResults -> ocrImage
            ocr_results = result_block.get("ocrResults", [])
            for ocr_item in ocr_results:
                pages.append({
                    "page_index": line_num - 1,
                    "markdown_text": "",
                    "images": [],
                    "image_urls": {},
                    "ocr_image_url": ocr_item.get("ocrImage", ""),
                })

        except json.JSONDecodeError as e:
            parse_errors += 1
            log_warn(f"Parse error on line {line_num}: {e} -- line preview: {line[:100]}")

    total_text_chars = sum(len(p["markdown_text"]) for p in pages)
    total_images = sum(len(p["images"]) for p in pages)

    log_pipeline(
        "JsonlParsed",
        pages=len(pages),
        text_chars=total_text_chars,
        images=total_images,
        parse_errors=parse_errors,
        model=MODEL,
    )

    return pages


# ============================================================================
# Step 5: Save to markdown
# ============================================================================

def save_to_markdown(pages: list, output_path: Path):
    """Save extracted text and image references to a markdown file."""
    log_pipeline("MarkdownSave", output=str(output_path))

    OUTPUT_DIR.mkdir(parents=True, exist_ok=True)

    total_text_chars = sum(len(p["markdown_text"]) for p in pages)
    total_images = sum(len(p["images"]) for p in pages)

    with open(output_path, "w", encoding="utf-8") as f:
        f.write(f"# PaddleOCR E2E Test Output\n\n")
        f.write(f"- **Model**: {MODEL}\n")
        f.write(f"- **PDF**: {PDF_PATH.name}\n")
        f.write(f"- **Date**: {datetime.now(timezone.utc).strftime('%Y-%m-%d %H:%M UTC')}\n")
        f.write(f"- **Total Pages**: {len(pages)}\n")
        f.write(f"- **Total Text Length**: {total_text_chars} chars\n")
        f.write(f"- **Total Images Found**: {total_images}\n")
        f.write(f"\n---\n\n")

        for i, page in enumerate(pages):
            f.write(f"## Page {page['page_index'] + 1}\n\n")

            if page["markdown_text"]:
                f.write(page["markdown_text"])
                f.write("\n\n")
            else:
                f.write(f"*(No markdown text extracted for this page)*\n\n")

            if page["images"]:
                f.write("### Images on this page\n\n")
                for img_name in page["images"]:
                    img_url = page["image_urls"].get(img_name, "")
                    f.write(f"- **{img_name}**: `{img_url}`\n")
                f.write("\n")

            if page.get("ocr_image_url"):
                f.write(f"### OCR Image\n\n")
                f.write(f"- URL: `{page['ocr_image_url']}`\n\n")

            if i < len(pages) - 1:
                f.write("---\n\n")

    file_size = output_path.stat().st_size
    log_pipeline(
        "MarkdownSaved",
        output=str(output_path),
        file_size_bytes=file_size,
        pages=len(pages),
        text_chars=total_text_chars,
        images=total_images,
    )
    log_info(f"Markdown saved to {output_path} ({file_size} bytes)")


# ============================================================================
# Step 6: Validation
# ============================================================================

def validate_output(pages: list, expected_pages_min: int = 1) -> dict:
    """Validate that OCR results are meaningful."""
    log_pipeline("ValidationStart")

    checks = {
        "pages_extracted": False,
        "text_nonempty": False,
        "text_reasonable_length": False,
        "images_counted": False,
    }

    total_pages = len(pages)
    total_text_chars = sum(len(p["markdown_text"]) for p in pages)
    total_images = sum(len(p["images"]) for p in pages)

    # Check 1: Pages were extracted
    if total_pages >= expected_pages_min:
        checks["pages_extracted"] = True
        log_pipeline(
            "ValidationCheck",
            check="pages_extracted",
            result="PASS",
            pages=total_pages,
            expected_min=expected_pages_min,
        )
    else:
        log_pipeline(
            "ValidationCheck",
            check="pages_extracted",
            result="FAIL",
            pages=total_pages,
            expected_min=expected_pages_min,
        )

    # Check 2: Non-empty text exists
    if total_text_chars > 0:
        checks["text_nonempty"] = True
        log_pipeline(
            "ValidationCheck",
            check="text_nonempty",
            result="PASS",
            text_chars=total_text_chars,
        )
    else:
        log_pipeline(
            "ValidationCheck",
            check="text_nonempty",
            result="FAIL",
            text_chars=total_text_chars,
        )

    # Check 3: Reasonable text length (at least 100 chars for a real textbook PDF)
    min_text_length = 100
    if total_text_chars >= min_text_length:
        checks["text_reasonable_length"] = True
        log_pipeline(
            "ValidationCheck",
            check="text_reasonable_length",
            result="PASS",
            text_chars=total_text_chars,
            min_expected=min_text_length,
        )
    else:
        log_pipeline(
            "ValidationCheck",
            check="text_reasonable_length",
            result="FAIL",
            text_chars=total_text_chars,
            min_expected=min_text_length,
        )

    # Check 4: Images counted
    checks["images_counted"] = True
    log_pipeline(
        "ValidationCheck",
        check="images_counted",
        result="PASS" if total_images > 0 else "OK",
        images=total_images,
    )

    all_pass = all(checks.values())
    log_pipeline(
        "ValidationComplete",
        all_pass=str(all_pass),
        checks_passed=sum(1 for v in checks.values() if v),
        checks_total=len(checks),
    )

    return {
        "pass": all_pass,
        "checks": checks,
        "total_pages": total_pages,
        "total_text_chars": total_text_chars,
        "total_images": total_images,
    }


# ============================================================================
# Main
# ============================================================================

def main():
    global PASS, FAIL, SKIP

    start_time = time.time()

    print(f"\n{'='*70}", flush=True)
    print(f"  PaddleOCR Pipeline E2E Test", flush=True)
    print(f"  API: {API_BASE}", flush=True)
    print(f"  Model: {MODEL}", flush=True)
    print(f"  PDF: {PDF_PATH.name}", flush=True)
    print(f"  Timeout: {MAX_POLL_ATTEMPTS * POLL_INTERVAL}s ({MAX_POLL_ATTEMPTS} attempts x {POLL_INTERVAL}s)", flush=True)
    print(f"{'='*70}\n", flush=True)

    # ------------------------------------------------------------------
    # STEP 1: Auth header comparison (Bearer vs bearer)
    # ------------------------------------------------------------------
    log_info("=" * 50)
    log_info("STEP 1: Auth header comparison (Bearer vs bearer)")
    log_info("=" * 50)

    auth_results = {}
    for variant in ["Bearer", "bearer"]:
        log_info(f"Testing auth variant: '{variant}'")
        auth_results[variant] = test_auth_header(variant)

    # Determine which auth variant works
    working_auth = None
    for variant, res in auth_results.items():
        if res.get("status") == "ok":
            working_auth = variant
            result("PASS", f"Auth Header: {variant}", f"Successfully submitted job (job_id={res.get('job_id')})")
        elif res.get("status") == "unauthorized":
            result("FAIL", f"Auth Header: {variant}", "HTTP 401 Unauthorized")
        else:
            result("FAIL", f"Auth Header: {variant}", f"Status={res.get('status')}")

    if working_auth is None:
        log_error("Neither auth variant works! Proceeding with 'bearer' (as used in Rust code).")
        result("FAIL", "Auth Header: Both variants", "Neither Bearer nor bearer works")
        working_auth = "bearer"  # fallback to match Rust code
    else:
        log_pipeline("AuthDecision", working_variant=working_auth)
        log_info(f"Working auth variant: '{working_auth}'")

    # ------------------------------------------------------------------
    # STEP 2: File upload (real scanned PDF)
    # ------------------------------------------------------------------
    log_info("=" * 50)
    log_info("STEP 2: File upload with real scanned PDF")
    log_info("=" * 50)

    if not PDF_PATH.exists():
        result("SKIP", "File Upload", f"PDF not found: {PDF_PATH}")
        log_error(f"PDF not found at {PDF_PATH}. Aborting.")
        print(f"\n{'='*70}")
        print(f"  RESULTS: {PASS} passed, {FAIL} failed, {SKIP} skipped")
        print(f"  Overall: {'FAIL' if FAIL > 0 else 'PASS'}")
        print(f"{'='*70}")
        sys.exit(1 if FAIL > 0 else 0)

    file_size_mb = PDF_PATH.stat().st_size / (1024 * 1024)
    log_info(f"File: {PDF_PATH.name} ({file_size_mb:.2f} MB)")

    try:
        job_id = submit_file_job(PDF_PATH, auth=working_auth)
        result("PASS", "File Upload Submit", f"job_id={job_id}, file={PDF_PATH.name} ({file_size_mb:.2f}MB)")
    except Exception as e:
        result("FAIL", "File Upload Submit", str(e))
        log_error(f"File upload failed, cannot continue: {e}")
        print(f"\n{'='*70}")
        print(f"  RESULTS: {PASS} passed, {FAIL} failed, {SKIP} skipped")
        print(f"{'='*70}")
        sys.exit(1)

    # ------------------------------------------------------------------
    # STEP 3: Poll job
    # ------------------------------------------------------------------
    log_info("=" * 50)
    log_info("STEP 3: Poll job for completion")
    log_info("=" * 50)

    try:
        jsonl_url, total_pages, extracted_pages = poll_job(job_id)
        result("PASS", "Job Polling", f"job_id={job_id}, pages={extracted_pages}/{total_pages}")
    except Exception as e:
        result("FAIL", "Job Polling", str(e))
        log_error(f"Job polling failed: {e}")
        print(f"\n{'='*70}")
        print(f"  RESULTS: {PASS} passed, {FAIL} failed, {SKIP} skipped")
        print(f"{'='*70}")
        sys.exit(1)

    # ------------------------------------------------------------------
    # STEP 4: Download and parse JSONL
    # ------------------------------------------------------------------
    log_info("=" * 50)
    log_info("STEP 4: Download and parse JSONL results")
    log_info("=" * 50)

    try:
        pages = download_and_parse(jsonl_url)
        total_text_chars = sum(len(p["markdown_text"]) for p in pages)
        total_images = sum(len(p["images"]) for p in pages)
        result(
            "PASS", "JSONL Download & Parse",
            f"pages={len(pages)}, text_chars={total_text_chars}, images={total_images}",
        )
    except Exception as e:
        result("FAIL", "JSONL Download & Parse", str(e))
        log_error(f"JSONL download/parse failed: {e}")
        print(f"\n{'='*70}")
        print(f"  RESULTS: {PASS} passed, {FAIL} failed, {SKIP} skipped")
        print(f"{'='*70}")
        sys.exit(1)

    # ------------------------------------------------------------------
    # STEP 5: Save to markdown
    # ------------------------------------------------------------------
    log_info("=" * 50)
    log_info("STEP 5: Save extracted text to markdown")
    log_info("=" * 50)

    try:
        save_to_markdown(pages, OUTPUT_MD)
        result("PASS", "Markdown Save", f"output={OUTPUT_MD}")
    except Exception as e:
        result("FAIL", "Markdown Save", str(e))

    # ------------------------------------------------------------------
    # STEP 6: Validation
    # ------------------------------------------------------------------
    log_info("=" * 50)
    log_info("STEP 6: Validate output")
    log_info("=" * 50)

    # Expect at least 1 page for a real PDF (this PDF has 200+ pages normally)
    expected_pages = 1
    if total_pages:
        expected_pages = min(1, total_pages)

    try:
        validation = validate_output(pages, expected_pages_min=1)
        if validation["pass"]:
            result("PASS", "Validation", f"All {len(validation['checks'])} checks passed")
        else:
            failed_checks = [k for k, v in validation["checks"].items() if not v]
            result("FAIL", "Validation", f"Failed checks: {', '.join(failed_checks)}")
    except Exception as e:
        result("FAIL", "Validation", str(e))

    # ------------------------------------------------------------------
    # Summary
    # ------------------------------------------------------------------
    elapsed_total = time.time() - start_time
    print(f"\n{'='*70}")
    print(f"  PIPELINE E2E TEST SUMMARY")
    print(f"  Total time: {elapsed_total:.1f}s")
    print(f"  Auth variant used: {working_auth}")
    print(f"  PDF: {PDF_PATH.name} ({file_size_mb:.2f} MB)")
    print(f"  Pages extracted: {len(pages)}")
    print(f"  Total text chars: {total_text_chars}")
    print(f"  Total images found: {total_images}")
    print(f"  Output: {OUTPUT_MD}")
    print(f"")
    print(f"  RESULTS: {PASS} passed, {FAIL} failed, {SKIP} skipped")
    print(f"  Overall: {'PASS' if FAIL == 0 else 'FAIL'}")
    print(f"{'='*70}\n")

    log_pipeline(
        "E2ETestComplete",
        elapsed_seconds=f"{elapsed_total:.1f}",
        auth_variant=working_auth,
        pages=len(pages),
        text_chars=total_text_chars if 'total_text_chars' in dir() else 0,
        images=total_images if 'total_images' in dir() else 0,
        passed=PASS,
        failed=FAIL,
        skipped=SKIP,
    )

    if FAIL > 0:
        sys.exit(1)


if __name__ == "__main__":
    main()
