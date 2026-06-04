-- ============================================================================
-- V20260604: Add PDF scanned-detection fields
-- ============================================================================
--
-- Purpose: Track whether a PDF is scanned (image-based) or has extractable
-- text. Allows the frontend to show OCR-related UI cues.
-- ============================================================================

-- is_scanned: TRUE if extracted text per page < 50 chars (scanned/image PDF)
ALTER TABLE files ADD COLUMN is_scanned INTEGER;

-- needs_ocr: TRUE if the PDF has no extractable text and should be OCR'd
ALTER TABLE files ADD COLUMN needs_ocr INTEGER;

-- ============================================================================
-- Backfill for existing PDF files
-- ============================================================================
-- For existing PDFs with extracted_text shorter than 100 chars total (rough
-- heuristic: <50 chars per page is the rule, but we don't have per-page
-- lengths in the schema, so total <100 is a conservative approximation), mark
-- as scanned + needs_ocr. PDFs with no extracted_text at all are definitely
-- scanned.
UPDATE files
SET is_scanned = 1, needs_ocr = 1
WHERE mime_type = 'application/pdf'
  AND (extracted_text IS NULL OR LENGTH(extracted_text) < 100);

-- PDFs with plenty of text: not scanned, no OCR needed
UPDATE files
SET is_scanned = 0, needs_ocr = 0
WHERE mime_type = 'application/pdf'
  AND extracted_text IS NOT NULL
  AND LENGTH(extracted_text) >= 100;
