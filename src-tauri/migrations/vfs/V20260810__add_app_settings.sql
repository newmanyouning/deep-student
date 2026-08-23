-- ============================================================================
-- V20260810: 应用设置表（settings 单一存储迁移）
-- ============================================================================
--
-- 背景: docs/DATA_STORAGE_SINGLE_SOURCE_AUDIT.md (2026-08-05) 指出
-- mistakes.db.settings 承载 vendor_configs/model_profiles/API keys 等 LLM
-- 管线入口配置，是"唯一故障点"却不在目标存储清单内。本迁移在 vfs.db
-- 建立等价的 KV 表 app_settings，由 Database 的设置方法路由读写
-- (vfs 优先 + mistakes 回退 + 启动时一次性复制 + 双写过渡)。
--
-- 兼容: 老库 mistakes.db.settings 数据保留不动，读取回退保证零丢失;
-- 旧版本应用回滚使用时仍可读旧表（双写期数据两份都在）。
-- ============================================================================

CREATE TABLE IF NOT EXISTS app_settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_app_settings_updated_at ON app_settings(updated_at DESC);
