#!/usr/bin/env python3
"""
PaddleOCR Real PDF Test

Tests PaddleOCR API with:
1. URL mode: publicly accessible image URL to verify API works
2. File upload mode: actual PDF files from scripts/
3. File size limits: checks if large PDFs can be processed directly or need splitting

API: https://paddleocr.aistudio-app.com/api/v2
"""

import sys
import os
import json
import time
import requests
from pathlib import Path

# ============================================================================
# Configuration
# ============================================================================

API_BASE = "https://paddleocr.aistudio-app.com/api/v2"
API_TOKEN = "950b164023143c0c95c1fe691c46e320758b00d0"
MODEL = "PaddleOCR-VL-1.6"

HEADERS = {
    "Authorization": f"Bearer {API_TOKEN}",
}

# Publicly accessible test image URLs (small, well-known test images)
TEST_IMAGE_URLS = [
    "https://upload.wikimedia.org/wikipedia/commons/thumb/7/7e/Document_OCR_test.png/800px-Document_OCR_test.png",
    "https://www.africau.edu/images/default/sample.pdf",
]

PDF_FILES = [
    Path(r"C:/deep-student/scripts/13505804_数据结构C语言版.pdf"),  # 16MB
    Path(r"C:/deep-student/scripts/《实变函数解题指南》周民强.pdf"),  # 7.2MB
]

POLL_INTERVAL = 5  # seconds
MAX_POLL_ATTEMPTS = 60  # 5 minutes max

PASS = 0
FAIL = 0
SKIP = 0

def log_result(status, test_name, detail=""):
    global PASS, FAIL, SKIP
    if status == "PASS":
        PASS += 1
        print(f"  [PASS] {test_name}")
    elif status == "FAIL":
        FAIL += 1
        print(f"  [FAIL] {test_name} -- {detail}")
    elif status == "SKIP":
        SKIP += 1
        print(f"  [SKIP] {test_name} -- {detail}")
    if detail and status != "FAIL":
        print(f"         {detail}")

# ============================================================================
# Helper Functions
# ============================================================================

def submit_url_job(file_url):
    """Submit an OCR job via URL mode (JSON POST)."""
    payload = {
        "model": MODEL,
        "fileUrl": file_url,
        "optionalPayload": {
            "useDocOrientationClassify": False,
            "useDocUnwarping": False,
            "useChartRecognition": False,
            "useTextlineOrientation": False,
        }
    }
    resp = requests.post(
        f"{API_BASE}/ocr/jobs",
        headers={**HEADERS, "Content-Type": "application/json"},
        json=payload,
        timeout=30,
    )
    if not resp.ok:
        raise RuntimeError(f"URL job submission failed (HTTP {resp.status_code}): {resp.text[:500]}")
    data = resp.json()
    return data["data"]["jobId"]

def submit_file_job(file_path):
    """Submit an OCR job via file upload (multipart form)."""
    filename = os.path.basename(file_path)
    file_size = os.path.getsize(file_path)

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
        resp = requests.post(
            f"{API_BASE}/ocr/jobs",
            headers=HEADERS,
            files=files,
            timeout=60,
        )

    if not resp.ok:
        raise RuntimeError(f"File job submission failed (HTTP {resp.status_code}): {resp.text[:500]}")
    data = resp.json()
    return data["data"]["jobId"]

def poll_job(job_id, max_attempts=MAX_POLL_ATTEMPTS):
    """Poll job status until completion."""
    for attempt in range(1, max_attempts + 1):
        resp = requests.get(
            f"{API_BASE}/ocr/jobs/{job_id}",
            headers=HEADERS,
            timeout=15,
        )
        if not resp.ok:
            raise RuntimeError(f"Poll failed (HTTP {resp.status})")

        status = resp.json()
        state = status["data"]["state"]

        if state == "done":
            result_url = status["data"]["resultUrl"]["jsonUrl"]
            pages = status["data"]["extractProgress"]["extractedPages"]
            progress = status["data"]["extractProgress"]
            return result_url, pages, progress

        if state == "failed":
            error_msg = status["data"].get("errorMsg", "Unknown error")
            raise RuntimeError(f"Job failed: {error_msg}")

        print(f"    Poll {attempt}/{max_attempts}: state={state}", end="\r")
        time.sleep(POLL_INTERVAL)

    raise TimeoutError(f"Job {job_id} timed out after {max_attempts} attempts")

def download_result(result_url):
    """Download and parse JSONL result."""
    resp = requests.get(result_url, timeout=30)
    if not resp.ok:
        raise RuntimeError(f"Download failed (HTTP {resp.status})")

    text = resp.text
    lines = [l for l in text.strip().split("\n") if l.strip()]
    pages = []
    for line in lines:
        try:
            parsed = json.loads(line)
            result = parsed.get("result", {})
            # VL series: layoutParsingResults -> markdown
            layout_results = result.get("layoutParsingResults", [])
            for item in layout_results:
                md = item.get("markdown", {})
                pages.append({
                    "markdownText": md.get("text", ""),
                    "images": list(md.get("images", {}).keys()),
                })
        except json.JSONDecodeError:
            continue
    return pages

def run_test(name, fn):
    """Run a test function with timing."""
    print(f"\n{'='*60}")
    print(f"TEST: {name}")
    print(f"{'='*60}")
    try:
        start = time.time()
        result = fn()
        elapsed = time.time() - start
        print(f"  Duration: {elapsed:.1f}s")
        log_result("PASS", name, result.get("summary", ""))
        return result
    except Exception as e:
        elapsed = time.time() - start
        print(f"  Duration: {elapsed:.1f}s")
        log_result("FAIL", name, str(e))
        return None

# ============================================================================
# Tests
# ============================================================================

def test_url_mode_basic():
    """Test 1: Submit a small publicly accessible image URL."""
    url = TEST_IMAGE_URLS[0]
    print(f"  Submitting URL: {url}")
    job_id = submit_url_job(url)
    print(f"  Job ID: {job_id}")

    result_url, pages, progress = poll_job(job_id)
    print(f"  Result URL: {result_url}")
    print(f"  Pages extracted: {pages}")
    print(f"  Progress: {json.dumps(progress, indent=4)[:200]}")

    results = download_result(result_url)
    total_text = sum(len(p["markdownText"]) for p in results)
    return {
        "job_id": job_id,
        "pages": pages,
        "text_length": total_text,
        "summary": f"URL mode works. {pages} page(s), {total_text} chars extracted",
    }

def test_url_sample_pdf():
    """Test 2: Submit a sample PDF via URL (the small AfricaU.edu sample PDF)."""
    url = TEST_IMAGE_URLS[1]
    print(f"  Submitting URL: {url}")
    job_id = submit_url_job(url)
    print(f"  Job ID: {job_id}")

    result_url, pages, progress = poll_job(job_id)
    print(f"  Result URL: {result_url}")
    print(f"  Pages extracted: {pages}")
    print(f"  Progress: {json.dumps(progress, indent=4)[:200]}")

    results = download_result(result_url)
    total_text = sum(len(p["markdownText"]) for p in results)
    return {
        "job_id": job_id,
        "pages": pages,
        "text_length": total_text,
        "summary": f"URL PDF mode works. {pages} page(s), {total_text} chars extracted",
    }

def test_file_upload_pdf(file_path):
    """Test 3: Upload a real PDF file directly."""
    file_size_mb = os.path.getsize(file_path) / (1024 * 1024)
    filename = file_path.name
    print(f"  File: {filename} ({file_size_mb:.1f} MB)")
    print(f"  Submitting file upload...")
    job_id = submit_file_job(str(file_path))
    print(f"  Job ID: {job_id}")

    result_url, pages, progress = poll_job(job_id)
    print(f"  Result URL: {result_url}")
    print(f"  Pages extracted: {pages}")
    print(f"  Progress: {json.dumps(progress, indent=4)[:300]}")

    results = download_result(result_url)
    total_text = sum(len(p["markdownText"]) for p in results)

    return {
        "job_id": job_id,
        "file": filename,
        "file_size_mb": file_size_mb,
        "pages": pages,
        "text_length": total_text,
        "summary": f"File upload works! {pages} pages processed in {file_size_mb:.1f}MB PDF, {total_text} chars extracted",
    }

# ============================================================================
# Main
# ============================================================================

def main():
    global PASS, FAIL, SKIP
    print(f"{'='*60}")
    print(f"  PaddleOCR Real PDF Test")
    print(f"  API: {API_BASE}")
    print(f"  Model: {MODEL}")
    print(f"  Date: {time.strftime('%Y-%m-%d %H:%M:%S')}")
    print(f"{'='*60}")

    # --- Test 1: URL mode with image ---
    r1 = run_test("URL Mode with Public Image", test_url_mode_basic)

    # --- Test 2: URL mode with small PDF ---
    r2 = run_test("URL Mode with Sample PDF", test_url_sample_pdf)

    # --- Test 3: File upload with real PDFs ---
    for pdf_file in PDF_FILES:
        if not pdf_file.exists():
            log_result("SKIP", f"File Upload: {pdf_file.name}", "File not found")
            continue

        file_size_mb = os.path.getsize(pdf_file) / (1024 * 1024)
        test_name = f"File Upload: {pdf_file.name} ({file_size_mb:.1f}MB)"

        def make_test(p):
            return lambda: test_file_upload_pdf(p)

        run_test(test_name, make_test(pdf_file))

    # --- Summary ---
    print(f"\n{'='*60}")
    print(f"  RESULTS: {PASS} passed, {FAIL} failed, {SKIP} skipped")
    print(f"{'='*60}")

    if FAIL > 0:
        sys.exit(1)

if __name__ == "__main__":
    main()
