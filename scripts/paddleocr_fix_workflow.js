export const meta = {
  name: 'paddleocr-fix-and-verify',
  description: 'PaddleOCR API修复+MCP测试+全组件覆盖验证',
  phases: [
    { title: 'Phase 1: Audit', detail: '审计OCR代码 vs 官方API' },
    { title: 'Phase 2: Fix', detail: '修复API实现' },
    { title: 'Phase 3: MCP', detail: 'MCP配置' },
    { title: 'Phase 4: Call Sites', detail: '全OCR调用点审查' },
    { title: 'Phase 5: Test', detail: '连通性测试' }
  ]
};

phase('Phase 1: Audit');
log('=== Phase 1: Auditing PaddleOCR code ===');

var audit = await agent(
  '## Audit PaddleOCR Implementation',
  '',
  'Official API (from user):',
  '- POST https://paddleocr.aistudio-app.com/api/v2/ocr/jobs',
  '- Auth header: "bearer <token>" (LOWERCASE bearer)',
  '- Job-based: submit job, poll state, download JSONL',
  '- VL series: result.layoutParsingResults[].markdown.text + .markdown.images',
  '- PP-OCRv5: result.ocrResults[].ocrImage',
  '- PP-StructureV3: result.layoutParsingResults[].markdown.text + .markdown.images',
  '',
  'Read these files and report ALL discrepancies vs official API:',
  '1. C:/deep-student/src-tauri/src/paddleocr_api.rs',
  '2. C:/deep-student/src-tauri/src/cmd/ocr.rs',
  '3. C:/deep-student/src-tauri/src/ocr_adapters/factory.rs',
  '4. C:/deep-student/src-tauri/src/llm_manager/builtin_vendors.rs (lines 193-199, 811-855)',
  '',
  'Check: auth header case, base URL, endpoint path, file upload format, polling, response parsing.'
);

phase('Phase 2: Fix');
log('=== Phase 2: Fixing API ===');

var fix = await agent(
  '## Fix PaddleOCR API Implementation',
  '',
  'Based on Phase 1 audit, apply fixes:',
  '',
  '1. Auth header: MUST be "bearer " prefix (lowercase). Check paddleocr_api.rs line where Authorization header is set.',
  '2. Base URL: MUST be https://paddleocr.aistudio-app.com/api/v2',
  '3. Endpoint: MUST POST to /ocr/jobs (job-based API), NOT /chat/completions',
  '4. File upload: multipart with model + optionalPayload JSON string + file bytes',
  '5. URL mode: JSON body with fileUrl + model + optionalPayload',
  '6. Polling: check state=pending|running|done|failed every 3s, max 120 attempts',
  '7. Response: parse correctly for VL-1.6/VL-1.5/PP-StructureV3 (layoutParsingResults) vs PP-OCRv5 (ocrResults)',
  '',
  'Files: C:/deep-student/src-tauri/src/paddleocr_api.rs, cmd/ocr.rs, ocr_adapters/factory.rs',
  'Preserve ALL existing functionality. Only fix API format discrepancies.'
);

phase('Phase 3: MCP');
log('=== Phase 3: MCP configuration ===');

var mcp = await agent(
  '## Create PaddleOCR MCP Configuration',
  '',
  'Create C:/deep-student/.mcp/paddleocr.json with 3 server entries:',
  '',
  '1. PaddleOCR-VL: pipeline=PaddleOCR-VL, endpoint=kfh520eae7f4c3u0.aistudio-app.com',
  '2. PP-OCRv5: pipeline=OCR, endpoint=mbj4o1s04etb73ta.aistudio-app.com',
  '3. PP-StructureV3: pipeline=PP-StructureV3, endpoint=q2n4xe93b5yec393.aistudio-app.com',
  '',
  'Read API key from C:/deep-student/scripts/all_vendor_api_keys.json paddleocr section.',
  'Fill in access token. Output the file.'
);

phase('Phase 4: Call Sites');
log('=== Phase 4: Auditing all OCR call sites ===');

var callSites = await agent(
  '## Audit ALL OCR Call Sites',
  '',
  'Search ENTIRE codebase:',
  '1. Rust: grep for paddleocr, paddle_ocr, PaddleOCR, ocr_, OcrEngine, OcrResult',
  '2. Frontend: grep src/ for paddleocr, paddle_ocr, ocr, OCR, OcrConfig',
  '',
  'For EACH call site verify: uses correct API format, correct model name, correct response parsing.',
  'Write C:/deep-student/docs/analysis/OCR_CALL_SITE_AUDIT.md'
);

phase('Phase 5: Test');
log('=== Phase 5: Connectivity test ===');

var test = await agent(
  '## Test PaddleOCR Connectivity',
  '',
  'Read API key from scripts/all_vendor_api_keys.json paddleocr section.',
  '',
  'Run this Bash command to test job submission:',
  'python3 scripts/test_paddleocr_connectivity.py',
  '',
  'First create the test script at C:/deep-student/scripts/test_paddleocr_connectivity.py:',
  '- Read key from all_vendor_api_keys.json',
  '- POST to https://paddleocr.aistudio-app.com/api/v2/ocr/jobs with a test image URL',
  '- Use header Authorization: bearer <key>',
  '- Poll for result (max 60s)',
  '- Print job status and result summary',
  '',
  'Then execute it and report results.',
  '',
  'IMPORTANT: never log the API key.'
);

log('');
log('PADDLEOCR FIX COMPLETE');
return { audit, fix, mcp, callSites, test };