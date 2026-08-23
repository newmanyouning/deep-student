//! # Research 模块 — HPIAS 深度研究引擎（类型层）
//!
//! 本模块落地 HPIAS (Hierarchical Progressive Information Acquisition System)
//! 分层渐进信息获取系统的**领域类型**与 vfs.db 表结构，不包含任何命令/引擎逻辑
//! （26 个 `research_*` 桩命令改造与引擎实现是后续任务）。
//!
//! ## HPIAS 设计意图（还原自 `docs/RESEARCH_MODULE_INTENT_ANALYSIS.md`）
//!
//! ```text
//! 研究会话 (session, 监督/自主双模式)
//!   └── 轮次循环 (round 1..N, 直到覆盖目标):
//!         plan → [approval] → queries → retrieval → dedupe → per-doc cap
//!         → keyword filter → selection → N × subagent(并行, 思考/工具调用)
//!         → synthesis → critic(找缺口) → macro insight(跨轮次)
//!   └── 完成 → session_report (可选落库为 DSTU Retrieval 节点)
//! ```
//!
//! ## 两条实现路径 (B/C) 兼容说明
//!
//! - **路径 B (推荐, 渐进)**: research-mode 技能 + chat_v2 `subagent_call` 工具，
//!   无独立编排器 — 本模块类型仅提供"存储骨架"（三张表 + 值类型）即可。
//! - **路径 C (完整 HPIAS)**: 独立编排器 + 事件流 — 本模块类型提供完整状态机
//!   (`ResearchStage` / `ResearchEvent`)，事件契约对齐前端
//!   `src/stores/researchStore.ts` 的 `HpiasEvent`（38 变体）。
//!
//! 类型层对两条路径同时兼容：表结构存"事实"，事件流存"过程"，
//! 引擎实现（B 或 C）按需取用，无需改动类型定义。
//!
//! ## 设计文档
//!
//! 详细设计见 `docs/plans/2026-08-06-type-unification-detailed-design.md` 第二部分：
//! - §2.2 领域表 SQL → 迁移 `V20260807__add_research_tables.sql`（vfs.db）
//! - §2.3 Rust 类型 → 本模块 `types` 子模块
//! - §2.4 前端契约对齐 → `ResearchEvent` 与 HpiasEvent 逐事件对齐
//! - §2.5 扩展性保障 → 非穷尽枚举 + 字符串扩展位

pub mod types;

pub use types::{
    CitationRef, CriticReview, DedupeStats, ResearchArtifact, ResearchChunkRef, ResearchEvent,
    ResearchMode, ResearchPlan, ResearchQuery, ResearchRound, ResearchRunOptions,
    ResearchSession, ResearchSessionStatus, ResearchStage, RetrievedDoc, RoundMetrics,
    RoundStatus, SelectedChunk,
};
