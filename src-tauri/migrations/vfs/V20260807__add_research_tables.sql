-- ============================================================================
-- Research 领域表 (V20260807__add_research_tables.sql)
-- ============================================================================
--
-- HPIAS (Hierarchical Progressive Information Acquisition System) 深度研究引擎
-- 的三张核心表（设计文档 §2.2）：
-- - research_sessions:   研究会话聚合根（监督/自主双模式）
-- - research_rounds:     轮次 — 一轮研究的全部状态（plan → ... → critic → macro）
-- - research_artifacts:  制品 — plan/report/macro_insight/chart/dataset/... 可扩展
--
-- 归属决策: 三张表放入 vfs.db（与 resources 资源表同库），便于研究轮次/制品
-- 直接引用资源 ID（document_id/source_id 均指向 vfs.resources.id）；
-- 而 research_reports 报告表保留在主库（database/manager.rs 运行时建表）。
--
-- 生成时间: 2026-08-07 23:37 CST
-- ============================================================================

-- ============================================================================
-- 1. 研究会话表（聚合根）
-- ============================================================================
CREATE TABLE IF NOT EXISTS research_sessions (
    id TEXT PRIMARY KEY,                        -- 格式: rs_{nanoid}
    title TEXT NOT NULL,                        -- 会话标题（初始问题摘要）
    status TEXT NOT NULL,                       -- pending_plan | running | paused | done | failed | cancelled
    mode TEXT NOT NULL DEFAULT 'supervised',    -- supervised | autonomous (silent_approval)
    max_rounds INTEGER NOT NULL DEFAULT 5,      -- 轮次循环上限
    min_selected INTEGER NOT NULL DEFAULT 3,    -- 覆盖率目标: 最少选中片段数
    options_json TEXT,                          -- 未来选项 (模型选择/检索源配置, 预留)
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_research_sessions_updated ON research_sessions(updated_at DESC);

-- ============================================================================
-- 2. 研究轮次表（一轮研究的全部状态）
-- ============================================================================
CREATE TABLE IF NOT EXISTS research_rounds (
    id TEXT PRIMARY KEY,                        -- 格式: rr_{nanoid}
    session_id TEXT NOT NULL REFERENCES research_sessions(id),
    round_no INTEGER NOT NULL,                  -- 轮次序号 (1-based)
    stage TEXT NOT NULL DEFAULT 'plan_generated', -- 当前所处阶段 (可恢复断点)
    status TEXT NOT NULL,                       -- pending | running | awaiting_approval | done | failed
    plan_json TEXT,                             -- {goals, sub_questions[]}
    queries_json TEXT,                          -- [{query, target}]
    retrieved_json TEXT,                        -- [{document_id, chunk_count}]
    dedupe_json TEXT,                           -- {before, after} (去重统计)
    selection_json TEXT,                        -- [{document_id, chunk_index, score, selected}]
    synthesis_md TEXT,                          -- 本轮综合 (markdown)
    citations_json TEXT,                        -- [{source_id, chunk_index, quote}]
    metrics_json TEXT,                          -- {tokens_in, tokens_out, duration_ms, subagent_count}
    critic_json TEXT,                           -- {gaps[], verdict}
    note TEXT,                                  -- 用户手写批注
    tags_json TEXT,                             -- 标签 JSON 数组
    UNIQUE(session_id, round_no)
);

CREATE INDEX IF NOT EXISTS idx_research_rounds_status ON research_rounds(status);

-- ============================================================================
-- 3. 研究制品表（类型系统不穷举 payload）
-- ============================================================================
CREATE TABLE IF NOT EXISTS research_artifacts (
    id TEXT PRIMARY KEY,                        -- 格式: ra_{nanoid}
    session_id TEXT NOT NULL REFERENCES research_sessions(id),
    round_no INTEGER,                           -- NULL = 会话级制品 (宏洞察等)
    agent TEXT NOT NULL,                        -- planner | subagent:{name} | synthesizer | critic | macro
    artifact_type TEXT NOT NULL,                -- 可扩展字符串: plan | report | macro_insight | chart | dataset | ...
    payload_json TEXT NOT NULL,
    size INTEGER NOT NULL DEFAULT 0,            -- 载荷大小 (字节)
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_research_artifacts_session_round ON research_artifacts(session_id, round_no);
