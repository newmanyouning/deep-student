-- ============================================================================
-- OCR 结果存储模块 (VFS 扩展)
-- 创建: 2026-05-30
-- 用途: 独立存储所有 OCR 处理结果，支持来源追溯和导出
-- 兼容: 不影响现有 vfs 导入流程
-- ============================================================================

CREATE TABLE IF NOT EXISTS ocr_results (
    id TEXT PRIMARY KEY,                          -- ocr_{nanoid(12)}
    -- 来源标识
    source_type TEXT NOT NULL,                    -- 'file' | 'chat' | 'pdf_page' | 'clipboard' | 'manual'
    source_id TEXT,                               -- 来源资源ID (VFS resource_id 或 chat message_id)
    source_name TEXT,                             -- 原始文件名/会话标题
    source_page INTEGER,                          -- PDF页码 (0-based, NULL if not PDF)
    -- 内容
    content_hash TEXT NOT NULL,                   -- SHA-256 of raw input (用于去重)
    input_size_bytes INTEGER,                     -- 输入文件大小
    image_mime TEXT,                              -- 输入图片 MIME 类型
    image_width INTEGER,                          -- 图片宽度(px)
    image_height INTEGER,                         -- 图片高度(px)
    -- OCR 结果
    ocr_text TEXT NOT NULL,                       -- OCR 识别全文
    ocr_engine TEXT NOT NULL,                     -- 引擎: 'paddle' | 'deepseek-ocr' | 'hunyuan' | 'system-native' | 'vlm'
    ocr_confidence REAL,                          -- 置信度 0.0-1.0
    ocr_duration_ms INTEGER,                      -- 处理耗时(毫秒)
    ocr_lang TEXT,                                -- 识别语言: 'ch' | 'en' | 'auto'
    tags TEXT NOT NULL DEFAULT '[]',              -- JSON 数组: 识别出的标签
    mistake_type TEXT,                            -- 题型 (分析模式)
    -- 区块结果 (JSON)
    blocks_json TEXT,                             -- OCR 区块详情 (bbox + text + confidence)
    -- 元数据
    created_at TEXT NOT NULL,                     -- ISO 8601 创建时间
    updated_at TEXT NOT NULL,                     -- ISO 8601 更新时间
    deleted_at TEXT,                              -- 软删除时间戳
    -- 导出状态
    export_hash TEXT,                             -- 导出时的内容哈希 (用于增量导出)
    exported_at TEXT                              -- 最后导出时间
);

-- 索引
CREATE INDEX IF NOT EXISTS idx_ocr_source_type ON ocr_results(source_type);
CREATE INDEX IF NOT EXISTS idx_ocr_source_id ON ocr_results(source_id);
CREATE INDEX IF NOT EXISTS idx_ocr_content_hash ON ocr_results(content_hash);
CREATE INDEX IF NOT EXISTS idx_ocr_created ON ocr_results(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_ocr_engine ON ocr_results(ocr_engine);
CREATE INDEX IF NOT EXISTS idx_ocr_deleted ON ocr_results(deleted_at);
CREATE INDEX IF NOT EXISTS idx_ocr_export ON ocr_results(export_hash);
CREATE UNIQUE INDEX IF NOT EXISTS idx_ocr_dedup ON ocr_results(content_hash, source_id) WHERE deleted_at IS NULL;

-- 去重视图: 相同内容只保留最新
CREATE VIEW IF NOT EXISTS ocr_results_dedup AS
SELECT * FROM ocr_results
WHERE deleted_at IS NULL
  AND id IN (
    SELECT id FROM ocr_results r2
    WHERE r2.content_hash = ocr_results.content_hash
    ORDER BY r2.created_at DESC
    LIMIT 1
  );
