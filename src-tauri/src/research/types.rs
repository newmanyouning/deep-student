//! Research 领域类型定义（HPIAS 深度研究引擎）
//!
//! 本文件落地 `docs/plans/2026-08-06-type-unification-detailed-design.md` 第二部分
//! (§2.3) 的全部核心类型，与 vfs.db 迁移 `V20260807__add_research_tables.sql`
//! （§2.2 三张表）一一对应：
//!
//! | 表 | 类型 |
//! |----|------|
//! | `research_sessions` | [`ResearchSession`]（聚合根） |
//! | `research_rounds` | [`ResearchRound`]（轮次全部状态） |
//! | `research_artifacts` | [`ResearchArtifact`]（可扩展制品） |
//!
//! 事件契约与前端 `src/stores/researchStore.ts` 的 `HpiasEvent`（38 变体）对齐：
//! `ResearchEvent` 的 serde tag 即前端事件名（snake_case），见下方各变体注释。
//!
//! ## 序列化规则（仓库惯例）
//! - 结构体: `#[serde(rename_all = "camelCase")]`
//! - 枚举: `#[serde(rename_all = "snake_case")]`
//! - 事件: `#[serde(tag = "event", rename_all = "snake_case")]`（字段保持 snake_case，
//!   与前端 `HpiasEvent` 载荷键名一致）
//!
//! ## 扩展性
//! - 新阶段/新状态/新事件: 加枚举变体（`#[non_exhaustive]`），表结构不变
//! - 新制品类型: `artifact_type` 字符串 + `payload_json` 任意 JSON，类型系统不穷举
//! - 新检索源: `queries_json`/`retrieved_json` 是 JSON，`target` 字段自由

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

// ============================================================================
// 状态枚举
// ============================================================================

/// 研究阶段状态机 — 与前端 HpiasEvent 的 round_* 事件一一对应。
/// 非穷尽枚举: 新阶段（如 model_retrieval）只加变体，不改表结构。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ResearchStage {
    /// 计划已生成（plan_generated）
    PlanGenerated,
    /// 计划待审批（plan_pending_approval，监督模式）
    PlanPendingApproval,
    /// 检索查询已准备（queries_prepared）
    QueriesPrepared,
    /// 检索已完成（retrieval_completed）
    RetrievalCompleted,
    /// 去重已完成（dedupe_completed）
    DedupeCompleted,
    /// 每文档上限已应用（per_doc_cap_applied）
    PerDocCapApplied,
    /// 关键词过滤已应用（keyword_filter_applied）
    KeywordFilterApplied,
    /// 片段选择已完成（selection_completed）
    SelectionCompleted,
    /// 子代理运行中（subagent_started .. subagent_done）
    SubagentsRunning,
    /// 综合已更新（synthesis_updated）
    SynthesisUpdated,
    /// 批评者评审已更新（critic_updated）
    CriticUpdated,
    /// 宏洞察已生成（macro_insight_generated）
    MacroInsightGenerated,
}

/// 会话状态 — 顶层聚合根的生命周期。
/// 对应 `research_sessions.status` 列。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ResearchSessionStatus {
    /// 等待首轮计划生成（初始态）
    PendingPlan,
    /// 运行中（有轮次在执行）
    Running,
    /// 暂停（用户干预或等待审批超时）
    Paused,
    /// 完成（达到覆盖率目标或 run_to_full_coverage 结束）
    Done,
    /// 失败（不可恢复错误）
    Failed,
    /// 已取消（用户请求或超时取消）
    Cancelled,
}

/// 会话执行模式 — 对应 `research_sessions.mode` 列。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ResearchMode {
    /// 监督模式: 每轮计划需用户审批（plan_pending_approval）
    Supervised,
    /// 自主模式: silent_approval，计划自动通过
    Autonomous,
}

/// 轮次状态 — 对应 `research_rounds.status` 列。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum RoundStatus {
    /// 待开始（已建行未启动）
    Pending,
    /// 运行中
    Running,
    /// 等待审批（监督模式，plan_pending_approval）
    AwaitingApproval,
    /// 已完成
    Done,
    /// 失败
    Failed,
}

// ============================================================================
// 辅助值类型（对应 *_json 列的 JSON 结构）
// ============================================================================

/// 研究计划 — 对应 `research_rounds.plan_json`（{goals, sub_questions}）。
/// 前端 plan_pending_approval / plan_generated 事件的 plan 载荷。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResearchPlan {
    /// 本轮研究目标列表
    pub goals: Vec<String>,
    /// 拆解出的子问题列表（每个子问题交给一个子代理）
    pub sub_questions: Vec<String>,
}

/// 检索查询 — 对应 `research_rounds.queries_json`（[{query, target}]）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResearchQuery {
    /// 查询文本
    pub query: String,
    /// 检索目标/来源类型（自由字符串，如 web | corpus | model_retrieval，扩展位）
    pub target: Option<String>,
}

/// 检索到的文档 — 对应 `research_rounds.retrieved_json`（[{document_id, chunk_count}]）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RetrievedDoc {
    /// 资源文档 ID（vfs.resources.id，与研究产物同库便于引用）
    pub document_id: String,
    /// 命中分块数
    pub chunk_count: i32,
}

/// 去重统计 — 对应 `research_rounds.dedupe_json`（{before, after}）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DedupeStats {
    /// 去重前片段数
    pub before: i32,
    /// 去重后片段数
    pub after: i32,
}

/// 选中的片段 — 对应 `research_rounds.selection_json`（[{document_id, chunk_index, score, selected}]）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectedChunk {
    /// 资源文档 ID
    pub document_id: String,
    /// 分块下标（0-based）
    pub chunk_index: i32,
    /// 相关性评分
    pub score: f64,
    /// 是否最终选中（过滤后仍保留）
    pub selected: bool,
}

/// 引用条目 — 对应 `research_rounds.citations_json`（[{source_id, chunk_index, quote}]）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CitationRef {
    /// 来源资源 ID
    pub source_id: String,
    /// 来源分块下标
    pub chunk_index: i32,
    /// 引用原文（片段）
    pub quote: String,
}

/// 轮次指标 — 对应 `research_rounds.metrics_json`
/// （{tokens_in, tokens_out, duration_ms, subagent_count}）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoundMetrics {
    /// 输入 token 数
    pub tokens_in: i64,
    /// 输出 token 数
    pub tokens_out: i64,
    /// 轮次耗时（毫秒）
    pub duration_ms: i64,
    /// 本轮子代理数量
    pub subagent_count: i32,
}

/// 批评者评审 — 对应 `research_rounds.critic_json`（{gaps[], verdict}）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CriticReview {
    /// 发现的缺口列表
    pub gaps: Vec<String>,
    /// 评审结论
    pub verdict: String,
}

// ============================================================================
// 顶层聚合
// ============================================================================

/// 研究会话 — 顶层聚合根。
/// 对应 `research_sessions` 表，ID 格式 `rs_{nanoid}`。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResearchSession {
    /// 会话 ID（rs_{nanoid}）
    pub id: String,
    /// 会话标题（初始问题摘要）
    pub title: String,
    /// 会话状态
    pub status: ResearchSessionStatus,
    /// 执行模式（监督/自主）
    pub mode: ResearchMode,
    /// 最大轮次数（覆盖率循环上限）
    pub max_rounds: i32,
    /// 覆盖率目标: 最少选中片段数（run_to_full_coverage 判定）
    pub min_selected: i32,
    /// 扩展位: 模型选择、检索源配置等（对应 options_json 列）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options_json: Option<Value>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 最后更新时间
    pub updated_at: DateTime<Utc>,
}

/// 轮次 — 一轮研究的全部状态。JSON 字段与 researchStore.roundsView 逐字段对应。
/// 对应 `research_rounds` 表，ID 格式 `rr_{nanoid}`。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResearchRound {
    /// 轮次 ID（rr_{nanoid}）
    pub id: String,
    /// 所属会话 ID
    pub session_id: String,
    /// 轮次序号（1-based，表内 UNIQUE(session_id, round_no)）
    pub round_no: i32,
    /// 当前所处阶段（可恢复断点）
    pub stage: ResearchStage,
    /// 轮次状态
    pub status: RoundStatus,
    /// 本轮研究计划
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plan: Option<ResearchPlan>,
    /// 检索查询列表
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub queries: Vec<ResearchQuery>,
    /// 检索到的文档列表
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub retrieved: Vec<RetrievedDoc>,
    /// 去重统计
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dedupe: Option<DedupeStats>,
    /// 选中片段列表
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub selection: Vec<SelectedChunk>,
    /// 本轮综合（markdown）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub synthesis_md: Option<String>,
    /// 引用列表
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub citations: Vec<CitationRef>,
    /// 轮次指标
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metrics: Option<RoundMetrics>,
    /// 批评者评审
    #[serde(skip_serializing_if = "Option::is_none")]
    pub critic: Option<CriticReview>,
    /// 用户笔记（手写批注）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    /// 标签列表（对应 tags_json 列）
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
}

/// 制品 — 类型系统不穷举 payload，用 artifact_type + payload_json 扩展。
/// 对应 `research_artifacts` 表，ID 格式 `ra_{nanoid}`。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResearchArtifact {
    /// 制品 ID（ra_{nanoid}）
    pub id: String,
    /// 所属会话 ID
    pub session_id: String,
    /// 所属轮次（NULL = 会话级制品，如宏洞察）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub round_no: Option<i32>,
    /// 产出代理: planner | subagent:{name} | synthesizer | critic | macro
    pub agent: String,
    /// 制品类型（可扩展字符串: plan | report | macro_insight | chart | dataset | ...）
    pub artifact_type: String,
    /// 制品载荷（任意 JSON）
    pub payload_json: Value,
    /// 载荷大小（字节）
    pub size: i64,
    /// 创建时间
    pub created_at: DateTime<Utc>,
}

// ============================================================================
// 运行参数与命令辅助类型
// ============================================================================

/// 运行参数 — 对应 `research_run_until` 桩命令签名
/// （max_rounds / min_selected / silent_approval）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResearchRunOptions {
    /// 最大轮次数覆盖（None = 沿用会话默认）
    pub max_rounds: Option<i32>,
    /// 覆盖率目标: 最少选中片段数（None = 沿用会话默认）
    pub min_selected: Option<i32>,
    /// 自主审批（跳过 plan_pending_approval）
    pub silent_approval: Option<bool>,
}

/// 分块级上下文 — 对应 `research_get_chunk_context` 桩命令
/// （document_id / chunk_index / before / after）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResearchChunkRef {
    /// 资源文档 ID
    pub document_id: String,
    /// 分块下标（0-based）
    pub chunk_index: usize,
    /// 前文块数（None = 默认）
    pub before: Option<usize>,
    /// 后文块数（None = 默认）
    pub after: Option<usize>,
}

// ============================================================================
// 研究事件（后端 → 前端事件流）
// ============================================================================

/// 研究事件 — 后端向前端发射的事件，对齐 researchStore.HpiasEvent。
/// serde tag = 事件名（snake_case，即前端 `type` 字段），字段键保持 snake_case
/// 与前端载荷一致。为保留扩展性，新事件只加变体（`#[non_exhaustive]`），
/// 前端 switch 默认分支降级。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
#[non_exhaustive]
pub enum ResearchEvent {
    /// 会话已启动（前端: session_started）
    SessionStarted {
        /// 会话 ID
        session_id: String,
    },
    /// 轮次已启动（前端: round_started）
    RoundStarted {
        /// 会话 ID
        session_id: String,
        /// 轮次序号
        round_no: i32,
    },
    /// 阶段推进（对应前端 round_executing / plan_generated / retrieval_completed /
    /// dedupe_completed / per_doc_cap_applied / keyword_filter_applied /
    /// selection_completed / subagents_done 等阶段事件）
    StageAdvanced {
        /// 会话 ID
        session_id: String,
        /// 轮次序号
        round_no: i32,
        /// 新阶段
        stage: ResearchStage,
    },
    /// 计划待审批（前端: plan_pending_approval，监督模式）
    PlanPendingApproval {
        /// 会话 ID
        session_id: String,
        /// 轮次序号
        round_no: i32,
        /// 待审批的计划
        plan: ResearchPlan,
    },
    /// 子代理已启动（前端: subagent_started）
    SubagentStarted {
        /// 会话 ID
        session_id: String,
        /// 轮次序号
        round_no: i32,
        /// 子代理标识
        subagent: String,
        /// 分配的子问题
        task: String,
    },
    /// 子代理已完成（前端: subagent_completed）
    SubagentCompleted {
        /// 会话 ID
        session_id: String,
        /// 轮次序号
        round_no: i32,
        /// 子代理标识
        subagent: String,
        /// 产出摘要（markdown）
        summary: String,
        /// 置信度（0..1）
        confidence: f32,
    },
    /// 子代理工具调用（前端: subagent_tool_call）
    SubagentToolCall {
        /// 会话 ID
        session_id: String,
        /// 轮次序号
        round_no: i32,
        /// 子代理标识
        subagent: String,
        /// 工具名
        tool: String,
        /// 调用参数（任意 JSON）
        args: Value,
    },
    /// 子代理工具结果（前端: subagent_tool_result）
    SubagentToolResult {
        /// 会话 ID
        session_id: String,
        /// 轮次序号
        round_no: i32,
        /// 子代理标识
        subagent: String,
        /// 工具名
        tool: String,
        /// 工具返回（任意 JSON）
        result: Value,
    },
    /// 综合已更新（前端: synthesis_updated）
    SynthesisUpdated {
        /// 会话 ID
        session_id: String,
        /// 轮次序号
        round_no: i32,
        /// 本轮综合（markdown）
        markdown: String,
    },
    /// 批评者评审已更新（前端: critic_updated）
    CriticUpdated {
        /// 会话 ID
        session_id: String,
        /// 轮次序号
        round_no: i32,
        /// 发现的缺口
        gaps: Vec<String>,
        /// 评审结论
        verdict: String,
    },
    /// 宏洞察已生成（前端: macro_insight_generated，跨轮次，会话级）
    MacroInsightGenerated {
        /// 会话 ID
        session_id: String,
        /// 洞察载荷（任意 JSON）
        insight: Value,
    },
    /// 制品已创建（前端: artifact_created）
    ArtifactCreated {
        /// 会话 ID
        session_id: String,
        /// 制品（完整对象）
        artifact: ResearchArtifact,
    },
    /// 运行参数已变更（前端无同名事件，供 Cockpit 等订阅方使用）
    RunOptionsChanged {
        /// 会话 ID
        session_id: String,
        /// 新运行参数
        options: ResearchRunOptions,
    },
    /// 通用进度（前端: ingestion_progress / macro_insight_progress 可映射到此）
    Progress {
        /// 会话 ID
        session_id: String,
        /// 进度百分比（0..100）
        percent: f32,
        /// 进度消息
        message: String,
    },
    /// 失败（前端: session_failed / error）
    Failed {
        /// 会话 ID
        session_id: String,
        /// 错误消息
        error: String,
    },
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// 事件序列化 tag 断言（至少 3 个变体）: tag 值即前端 HpiasEvent 事件名
    #[test]
    fn test_research_event_tag_serialization() {
        // 1. SessionStarted → session_started
        let e = ResearchEvent::SessionStarted {
            session_id: "rs_abc".into(),
        };
        let v = serde_json::to_value(&e).unwrap();
        assert_eq!(v["event"], "session_started");
        assert_eq!(v["session_id"], "rs_abc");

        // 2. PlanPendingApproval → plan_pending_approval（载荷 camelCase）
        let e = ResearchEvent::PlanPendingApproval {
            session_id: "rs_abc".into(),
            round_no: 1,
            plan: ResearchPlan {
                goals: vec!["目标1".into()],
                sub_questions: vec!["问题1".into()],
            },
        };
        let v = serde_json::to_value(&e).unwrap();
        assert_eq!(v["event"], "plan_pending_approval");
        assert_eq!(v["round_no"], 1);
        assert_eq!(v["plan"]["subQuestions"][0], "问题1");

        // 3. SubagentToolCall → subagent_tool_call（任意 JSON 载荷直通）
        let e = ResearchEvent::SubagentToolCall {
            session_id: "rs_abc".into(),
            round_no: 1,
            subagent: "sub_1".into(),
            tool: "web_search".into(),
            args: json!({ "query": "rust" }),
        };
        let v = serde_json::to_value(&e).unwrap();
        assert_eq!(v["event"], "subagent_tool_call");
        assert_eq!(v["tool"], "web_search");
        assert_eq!(v["args"]["query"], "rust");
    }

    /// 事件带 tag 反序列化往返
    #[test]
    fn test_research_event_tag_roundtrip() {
        let e = ResearchEvent::CriticUpdated {
            session_id: "rs_abc".into(),
            round_no: 2,
            gaps: vec!["缺少对比实验".into()],
            verdict: "需补充".into(),
        };
        let json = serde_json::to_string(&e).unwrap();
        assert!(json.contains("\"event\":\"critic_updated\""));
        let back: ResearchEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(back, e);

        // 未知事件名反序列化失败（非穷尽枚举的读取侧约定: 引擎只发已知事件）
        assert!(serde_json::from_str::<ResearchEvent>(
            r#"{"event":"unknown_event","session_id":"rs_x"}"#
        )
        .is_err());
    }

    /// ResearchStage 全部 12 变体的 serde 值 + 往返
    #[test]
    fn test_research_stage_serde_values() {
        let cases = [
            (ResearchStage::PlanGenerated, "plan_generated"),
            (ResearchStage::PlanPendingApproval, "plan_pending_approval"),
            (ResearchStage::QueriesPrepared, "queries_prepared"),
            (ResearchStage::RetrievalCompleted, "retrieval_completed"),
            (ResearchStage::DedupeCompleted, "dedupe_completed"),
            (ResearchStage::PerDocCapApplied, "per_doc_cap_applied"),
            (ResearchStage::KeywordFilterApplied, "keyword_filter_applied"),
            (ResearchStage::SelectionCompleted, "selection_completed"),
            (ResearchStage::SubagentsRunning, "subagents_running"),
            (ResearchStage::SynthesisUpdated, "synthesis_updated"),
            (ResearchStage::CriticUpdated, "critic_updated"),
            (ResearchStage::MacroInsightGenerated, "macro_insight_generated"),
        ];
        for (stage, name) in cases {
            let s = serde_json::to_string(&stage).unwrap();
            assert_eq!(s, format!("\"{}\"", name), "stage {} 序列化值不符", name);
            let back: ResearchStage = serde_json::from_str(&s).unwrap();
            assert_eq!(back, stage);
        }
    }

    /// 状态枚举 serde 值 + 往返
    #[test]
    fn test_status_enums_serde() {
        assert_eq!(
            serde_json::to_string(&ResearchSessionStatus::PendingPlan).unwrap(),
            "\"pending_plan\""
        );
        assert_eq!(
            serde_json::to_string(&ResearchSessionStatus::Cancelled).unwrap(),
            "\"cancelled\""
        );
        assert_eq!(
            serde_json::to_string(&ResearchMode::Autonomous).unwrap(),
            "\"autonomous\""
        );
        assert_eq!(
            serde_json::to_string(&RoundStatus::AwaitingApproval).unwrap(),
            "\"awaiting_approval\""
        );
        let m: ResearchMode = serde_json::from_str("\"supervised\"").unwrap();
        assert_eq!(m, ResearchMode::Supervised);
        let s: RoundStatus = serde_json::from_str("\"done\"").unwrap();
        assert_eq!(s, RoundStatus::Done);
    }

    /// ResearchSession camelCase 键 + 往返
    #[test]
    fn test_research_session_camelcase_roundtrip() {
        let s = ResearchSession {
            id: "rs_abc".into(),
            title: "深度学习综述".into(),
            status: ResearchSessionStatus::Running,
            mode: ResearchMode::Supervised,
            max_rounds: 5,
            min_selected: 3,
            options_json: Some(json!({ "execution_mode": "supervised" })),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let json = serde_json::to_string(&s).unwrap();
        assert!(json.contains("\"maxRounds\":5"), "camelCase 键 maxRounds 缺失");
        assert!(json.contains("\"minSelected\":3"), "camelCase 键 minSelected 缺失");
        assert!(json.contains("\"optionsJson\""), "camelCase 键 optionsJson 缺失");
        assert!(json.contains("\"createdAt\""), "camelCase 键 createdAt 缺失");
        let back: ResearchSession = serde_json::from_str(&json).unwrap();
        assert_eq!(back, s);
    }

    /// ResearchRound 全字段 camelCase 往返（与表列和 roundsView 对齐）
    #[test]
    fn test_research_round_roundtrip() {
        let r = ResearchRound {
            id: "rr_abc".into(),
            session_id: "rs_abc".into(),
            round_no: 1,
            stage: ResearchStage::SelectionCompleted,
            status: RoundStatus::Running,
            plan: Some(ResearchPlan {
                goals: vec!["目标1".into()],
                sub_questions: vec!["问题1".into(), "问题2".into()],
            }),
            queries: vec![ResearchQuery {
                query: "rust async".into(),
                target: Some("web".into()),
            }],
            retrieved: vec![RetrievedDoc {
                document_id: "res_1".into(),
                chunk_count: 4,
            }],
            dedupe: Some(DedupeStats { before: 10, after: 7 }),
            selection: vec![SelectedChunk {
                document_id: "res_1".into(),
                chunk_index: 2,
                score: 0.87,
                selected: true,
            }],
            synthesis_md: Some("## 综合\n...".into()),
            citations: vec![CitationRef {
                source_id: "res_1".into(),
                chunk_index: 2,
                quote: "原文片段".into(),
            }],
            metrics: Some(RoundMetrics {
                tokens_in: 1000,
                tokens_out: 800,
                duration_ms: 12345,
                subagent_count: 3,
            }),
            critic: Some(CriticReview {
                gaps: vec!["缺少 X".into()],
                verdict: "基本合格".into(),
            }),
            note: Some("用户备注".into()),
            tags: vec!["深度".into()],
        };
        let json = serde_json::to_string(&r).unwrap();
        assert!(json.contains("\"roundNo\":1"), "camelCase 键 roundNo 缺失");
        assert!(json.contains("\"synthesisMd\""), "camelCase 键 synthesisMd 缺失");
        assert!(json.contains("\"subQuestions\""), "camelCase 键 subQuestions 缺失");
        assert!(json.contains("\"chunkCount\":4"), "camelCase 键 chunkCount 缺失");
        assert!(json.contains("\"stage\":\"selection_completed\""));
        let back: ResearchRound = serde_json::from_str(&json).unwrap();
        assert_eq!(back, r);
    }

    /// ResearchArtifact 往返（round_no = None 时字段省略）
    #[test]
    fn test_research_artifact_roundtrip() {
        let a = ResearchArtifact {
            id: "ra_abc".into(),
            session_id: "rs_abc".into(),
            round_no: None,
            agent: "macro".into(),
            artifact_type: "macro_insight".into(),
            payload_json: json!({ "insight": "跨轮次发现" }),
            size: 128,
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&a).unwrap();
        assert!(json.contains("\"artifactType\":\"macro_insight\""));
        assert!(!json.contains("roundNo"), "None 轮次应省略 roundNo 键");
        let back: ResearchArtifact = serde_json::from_str(&json).unwrap();
        assert_eq!(back, a);
    }

    /// ResearchRunOptions / ResearchChunkRef（对齐桩命令签名）camelCase 往返
    #[test]
    fn test_run_options_and_chunk_ref_roundtrip() {
        let o = ResearchRunOptions {
            max_rounds: Some(8),
            min_selected: None,
            silent_approval: Some(true),
        };
        let json = serde_json::to_string(&o).unwrap();
        assert!(json.contains("\"maxRounds\":8"));
        assert!(json.contains("\"silentApproval\":true"));
        let back: ResearchRunOptions = serde_json::from_str(&json).unwrap();
        assert_eq!(back, o);

        let c = ResearchChunkRef {
            document_id: "res_1".into(),
            chunk_index: 3,
            before: Some(1),
            after: Some(2),
        };
        let json = serde_json::to_string(&c).unwrap();
        assert!(json.contains("\"chunkIndex\":3"));
        let back: ResearchChunkRef = serde_json::from_str(&json).unwrap();
        assert_eq!(back, c);
    }
}
