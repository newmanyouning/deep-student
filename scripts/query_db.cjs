#!/usr/bin/env node
const fs = require('fs');

const dbPath = 'C:/Users/1/AppData/Roaming/com.deepstudent.app/slots/slotB/chat_v2.db';
const buf = fs.readFileSync(dbPath);
const text = buf.toString('utf8', 0, Math.min(buf.length, 50 * 1024 * 1024));

console.log('SQLite header:', text.substring(0, 15));
console.log('DB size:', (buf.length / 1024 / 1024).toFixed(1), 'MB');

// Count record types
const fileCount = (text.match(/"file_[a-zA-Z0-9_-]{10,30}"/g) || []).length;
const attCount = (text.match(/"att_[a-zA-Z0-9_-]{10,30}"/g) || []).length;
const tbCount = (text.match(/"tb_[a-zA-Z0-9_-]{10,30}"/g) || []).length;
const noteCount = (text.match(/"note_[a-zA-Z0-9_-]{10,30}"/g) || []).length;
const ocrRefs = (text.match(/ocr_pages_json/gi) || []).length;
const ocrTextRefs = (text.match(/ocr_text/gi) || []).length;

console.log('\n=== Resource Records ===');
console.log('file_ records:', fileCount);
console.log('att_ records:', attCount);
console.log('tb_ records:', tbCount);
console.log('note_ records:', noteCount);
console.log('ocr_pages_json refs:', ocrRefs);
console.log('ocr_text refs:', ocrTextRefs);

// Find PDF filenames in database
const pdfMatches = [];
let idx = 0;
while ((idx = text.indexOf('.pdf', idx)) !== -1) {
    const start = Math.max(0, idx - 80);
    const end = Math.min(text.length, idx + 30);
    const context = text.substring(start, end).replace(/[\x00-\x1f\x7f-\x9f]/g, ' ');
    pdfMatches.push(context.trim());
    idx += 1;
    if (pdfMatches.length >= 10) break;
}
console.log('\n=== PDF Filenames in DB ===');
pdfMatches.forEach((m, i) => console.log(`[${i+1}] ...${m}...`));
