#!/usr/bin/env node

/**
 * Live test of the full PaddleOCR API flow:
 * 1. Submit file upload
 * 2. Poll for completion
 * 3. Download JSONL results
 * 4. Save to ocr_output/
 */
import { readFileSync, writeFileSync, existsSync } from 'fs';

const API_BASE = 'https://paddleocr.aistudio-app.com/api/v2';
const TOKEN = '950b164023143c0c95c1fe691c46e320758b00d0';
const PDF_PATH = 'C:/deep-student/scripts/《实变函数解题指南》周民强.pdf';
const MODEL = 'PaddleOCR-VL-1.6';

function sleep(ms) { return new Promise(r => setTimeout(r, ms)); }

// Step 1: Submit
console.log('=== STEP 1: Submit job ===');
const fileBuf = readFileSync(PDF_PATH);
const fileName = PDF_PATH.split(/[\/]/).pop();
const boundary = `----FormBoundary${Date.now()}`;

function encodeField(name, value, boundary) {
  return `\r\n--${boundary}\r\nContent-Disposition: form-data; name="${name}"\r\n\r\n${value}`;
}

function encodeFile(fieldName, fileName, buffer, boundary) {
  const header = `\r\n--${boundary}\r\nContent-Disposition: form-data; name="${fieldName}"; filename="${fileName}"\r\nContent-Type: application/octet-stream\r\n\r\n`;
  return Buffer.concat([Buffer.from(header), buffer]);
}

const optionalPayload = JSON.stringify({
  useDocOrientationClassify: false,
  useDocUnwarping: false,
  useChartRecognition: false,
  useTextlineOrientation: false,
});

const parts = [
  Buffer.from(encodeField('model', MODEL, boundary)),
  Buffer.from(encodeField('optionalPayload', optionalPayload, boundary)),
  encodeFile('file', fileName, fileBuf, boundary),
  Buffer.from(`\r\n--${boundary}--\r\n`),
];
const body = Buffer.concat(parts);

let resp = await fetch(`${API_BASE}/ocr/jobs`, {
  method: 'POST',
  headers: {
    'Authorization': `Bearer ${TOKEN}`,
    'Content-Type': `multipart/form-data; boundary=${boundary}`,
  },
  body,
});

if (!resp.ok) {
  const text = await resp.text();
  console.log(`FAIL: Job submission failed (${resp.status}): ${text}`);
  process.exit(1);
}

const submitData = await resp.json();
const jobId = submitData.data.jobId;
console.log(`PASS: Job submitted, jobId=${jobId}`);

// Step 2: Poll
console.log('=== STEP 2: Poll for completion ===');
let attempts = 0;
let resultUrl = null;
while (attempts < 120) {
  resp = await fetch(`${API_BASE}/ocr/jobs/${jobId}`, {
    headers: { 'Authorization': `Bearer ${TOKEN}` },
  });
  if (!resp.ok) {
    console.log(`FAIL: Poll failed (${resp.status})`);
    process.exit(1);
  }
  const status = await resp.json();
  const state = status.data.state;
  const progress = status.data.extractProgress || {};
  if (attempts % 10 === 0) {
    console.log(`  State=${state}, pages=${progress.extractedPages ?? '?'}/${progress.totalPages ?? '?'}`);
  }
  if (state === 'done') {
    resultUrl = status.data.resultUrl?.jsonUrl;
    console.log(`PASS: Job done, resultUrl=${resultUrl}`);
    break;
  }
  if (state === 'failed') {
    console.log(`FAIL: Job failed: ${status.data.errorMsg || 'Unknown error'}`);
    process.exit(1);
  }
  await sleep(3000);
  attempts++;
}
if (!resultUrl) {
  console.log('FAIL: Job timed out');
  process.exit(1);
}

// Step 3: Download JSONL
console.log('=== STEP 3: Download results ===');
resp = await fetch(resultUrl);
if (!resp.ok) {
  console.log(`FAIL: Download failed (${resp.status})`);
  process.exit(1);
}
const jsonlText = await resp.text();
const lines = jsonlText.trim().split('\n').filter(Boolean);
console.log(`PASS: Downloaded ${lines.length} JSONL lines`);

// Parse and display sample
const parsed = JSON.parse(lines[0]);
const hasLayoutResults = parsed.result?.layoutParsingResults !== undefined;
const hasOcrResults = parsed.result?.ocrResults !== undefined;
const blockCount = parsed.result?.layoutParsingResults?.[0]?.prunedResult?.parsing_res_list?.length || 0;
console.log(`  Layout blocks on page 1: ${blockCount}`);
console.log(`  Layout results found: ${hasLayoutResults}, OCR results found: ${hasOcrResults}`);

// Extract markdown text from all pages
const mdTexts = [];
for (const line of lines) {
  try {
    const p = JSON.parse(line);
    const results = p.result?.layoutParsingResults || [];
    for (const item of results) {
      const md = item.markdown || {};
      if (md.text) mdTexts.push(md.text);
    }
  } catch(e) {}
}
console.log(`PASS: Extracted ${mdTexts.length} markdown text blocks`);
console.log(`  First 200 chars: ${mdTexts[0]?.substring(0, 200) || '(empty)'}`);

// Save results
const outputDir = 'C:/deep-student/scripts/ocr_output';
const baseName = fileName.replace(/[《》]/g, '').replace(/\.pdf$/i, '');
writeFileSync(`${outputDir}/${baseName}_live_test.jsonl`, jsonlText);
writeFileSync(`${outputDir}/${baseName}_live_test.txt`, mdTexts.join('\n\n--- PAGE BREAK ---\n\n'));
console.log(`PASS: Results saved to ${outputDir}/${baseName}_live_test.jsonl and .txt`);

console.log('\n=== FULL FLOW: PASS ===');
