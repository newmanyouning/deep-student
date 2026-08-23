#!/usr/bin/env node
// Query the chat_v2.db to find PDF records and OCR status
const fs = require('fs');
const path = require('path');

const dbPath = 'C:/Users/1/AppData/Roaming/com.deepstudent.app/slots/slotB/chat_v2.db';

// Simple SQLite3 query using raw file parsing for key tables
// We'll look for the SQLite header and try to extract data

const buf = fs.readFileSync(dbPath);

// Check SQLite magic header
const magic = buf.toString('utf8', 0, 16);
console.log('SQLite header:', magic.substring(0, 15));

// Check database size
console.log('Database size:', (buf.length / 1024 / 1024).toFixed(1), 'MB');

// Look for PDF-related strings in the database
const text = buf.toString('utf8', 0, Math.min(buf.length, 50 * 1024 * 1024)); // read first 50MB as text

// Find all PDF file references by searching for '.pdf' in text
const pdfMatches = [];
let idx = 0;
while ((idx = text.indexOf('.pdf', idx)) !== -1) {
    // Get context around the match
    const start = Math.max(0, idx - 100);
    const end = Math.min(text.length, idx + 200);
    const context = text.substring(start, end).replace(/[\x00-\x1f\x7f-\x9f]/g, ' ');
    pdfMatches.push(context.trim());
    idx += 1;
    if (pdfMatches.length >= 15) break;
}

console.log('\n=== PDF references in database (up to 15) ===');
pdfMatches.forEach((m, i) => console.log(`[${i + 1}] ...${m}...`));

// Look for OCR-related records
const ocrCount = (text.match(/ocr_pages_json/gi) || []).length;
const ocrTextCount = (text.match(/resources\.ocr_text/gi) || []).length;
const noteCount = (text.match(/note_/gi) || []).length;

console.log('\n=== Database statistics ===');
console.log('ocr_pages_json references:', ocrCount);
console.log('resources.ocr_text references:', ocrTextCount);
console.log('note_ records:', noteCount);

// Look for file_ and att_ prefixed records
const fileCount = (text.match(/"file_[a-zA-Z0-9_-]{10,30}"/g) || []).length;
const attCount = (text.match(/"att_[a-zA-Z0-9_-]{10,30}"/g) || []).length;
const tbCount = (text.match(/"tb_[a-zA-Z0-9_-]{10,30}"/g) || []).length;
console.log('\nfile_ records:', fileCount);
console.log('att_ records:', attCount);
console.log('tb_ records:', tbCount);
