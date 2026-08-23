/**
 * Research 领域共享类型（HPIAS 深度研究引擎）
 *
 * 本文件是后端 `src-tauri/src/research/types.rs` 的 TypeScript 镜像，
 * 与 vfs.db 三张表（research_sessions / research_rounds / research_artifacts）一一对应。
 * 供 `src/stores/researchStore.ts`、`src/utils/settingsApi.ts` 等前端消费方共享，
 * 消除各处的重复/内联形状（如旧版 ResearchArtifact id 为 number，与后端 ra_ 字符串不符）。
 *
 * ## 序列化规则（与后端一致）
 * - 结构体: camelCase 字段（serde rename_all = "camelCase"）
 * - 枚举: snake_case 值（serde rename_all = "snake_case"）
 * - 事件: `type` 字段为判别 tag（serde tag = "event"），值 snake_case，
 *   载荷键名 snake_case；嵌套结构体（如 plan）仍为 camelCase。
 * - 后端 `Option<T>` 字段 → `?: T | null`；`Vec<T>` 字段 → 必填数组。
 * - 后端 `#[non_exhaustive]` 枚举 → TS 无法表达，注释标注扩展位，
 *   消费方 switch 需保留 default 分支降级。
 * - `serde_json::Value` → `JsonValue`（任意 JSON）。
 * - `DateTime<Utc>` → ISO 8601 字符串；i64 / usize / f64 → number。
 */

// ============================================================================
// 通用 JSON 值（对应 serde_json::Value）
// ============================================================================

/** 任意 JSON 值（后端 serde_json::Value 的 TS 镜像） */
export type JsonValue = string | number | boolean | null | JsonValue[] | { [key: string]: JsonValue };

// ============================================================================
// 状态枚举（snake_case 字符串字面量联合）
// ============================================================================

/** 研究阶段状态机 — 与 HpiasEvent 的 round_* 事件一一对应。非穷尽枚举，新阶段只加字面量。 */
export type ResearchStage =
  /** 计划已生成 */
  | 'plan_generated'
  /** 计划待审批（监督模式） */
  | 'plan_pending_approval'
  /** 检索查询已准备 */
  | 'queries_prepared'
  /** 检索已完成 */
  | 'retrieval_completed'
  /** 去重已完成 */
  | 'dedupe_completed'
  /** 每文档上限已应用 */
  | 'per_doc_cap_applied'
  /** 关键词过滤已应用 */
  | 'keyword_filter_applied'
  /** 片段选择已完成 */
  | 'selection_completed'
  /** 子代理运行中（subagent_started .. subagent_done） */
  | 'subagents_running'
  /** 综合已更新 */
  | 'synthesis_updated'
  /** 批评者评审已更新 */
  | 'critic_updated'
  /** 宏洞察已生成 */
  | 'macro_insight_generated';

/** 会话状态 — 顶层聚合根的生命周期（对应 research_sessions.status）。非穷尽枚举。 */
export type ResearchSessionStatus =
  /** 等待首轮计划生成（初始态） */
  | 'pending_plan'
  /** 运行中（有轮次在执行） */
  | 'running'
  /** 暂停（用户干预或等待审批超时） */
  | 'paused'
  /** 完成（达到覆盖率目标或 run_to_full_coverage 结束） */
  | 'done'
  /** 失败（不可恢复错误） */
  | 'failed'
  /** 已取消（用户请求或超时取消） */
  | 'cancelled';

/** 会话执行模式（对应 research_sessions.mode）。非穷尽枚举。 */
export type ResearchMode =
  /** 监督模式: 每轮计划需用户审批（plan_pending_approval） */
  | 'supervised'
  /** 自主模式: silent_approval，计划自动通过 */
  | 'autonomous';

/** 轮次状态（对应 research_rounds.status）。非穷尽枚举。 */
export type RoundStatus =
  /** 待开始（已建行未启动） */
  | 'pending'
  /** 运行中 */
  | 'running'
  /** 等待审批（监督模式，plan_pending_approval） */
  | 'awaiting_approval'
  /** 已完成 */
  | 'done'
  /** 失败 */
  | 'failed';

// ============================================================================
// 辅助值类型（对应 *_json 列的 JSON 结构，camelCase）
// ============================================================================

/** 研究计划 — 对应 research_rounds.plan_json（{goals, sub_questions}）。 */
export interface ResearchPlan {
  /** 本轮研究目标列表 */
  goals: string[];
  /** 拆解出的子问题列表（每个子问题交给一个子代理） */
  sub_questions: string[];
}

/** 检索查询 — 对应 research_rounds.queries_json（[{query, target}]）。 */
export interface ResearchQuery {
  /** 查询文本 */
  query: string;
  /** 检索目标/来源类型（自由字符串，如 web | corpus | model_retrieval，扩展位） */
  target?: string | null;
}

/** 检索到的文档 — 对应 research_rounds.retrieved_json（[{document_id, chunk_count}]）。 */
export interface RetrievedDoc {
  /** 资源文档 ID（vfs.resources.id，与研究产物同库便于引用） */
  document_id: string;
  /** 命中分块数 */
  chunk_count: number;
}

/** 去重统计 — 对应 research_rounds.dedupe_json（{before, after}）。 */
export interface DedupeStats {
  /** 去重前片段数 */
  before: number;
  /** 去重后片段数 */
  after: number;
}

/** 选中的片段 — 对应 research_rounds.selection_json（[{document_id, chunk_index, score, selected}]）。 */
export interface SelectedChunk {
  /** 资源文档 ID */
  document_id: string;
  /** 分块下标（0-based） */
  chunk_index: number;
  /** 相关性评分 */
  score: number;
  /** 是否最终选中（过滤后仍保留） */
  selected: boolean;
}

/** 引用条目 — 对应 research_rounds.citations_json（[{source_id, chunk_index, quote}]）。 */
export interface CitationRef {
  /** 来源资源 ID */
  source_id: string;
  /** 来源分块下标 */
  chunk_index: number;
  /** 引用原文（片段） */
  quote: string;
}

/** 轮次指标 — 对应 research_rounds.metrics_json（{tokens_in, tokens_out, duration_ms, subagent_count}）。 */
export interface RoundMetrics {
  /** 输入 token 数 */
  tokens_in: number;
  /** 输出 token 数 */
  tokens_out: number;
  /** 轮次耗时（毫秒） */
  duration_ms: number;
  /** 本轮子代理数量 */
  subagent_count: number;
}

/** 批评者评审 — 对应 research_rounds.critic_json（{gaps[], verdict}）。 */
export interface CriticReview {
  /** 发现的缺口列表 */
  gaps: string[];
  /** 评审结论 */
  verdict: string;
}

// ============================================================================
// 顶层聚合（camelCase）
// ============================================================================

/** 研究会话 — 顶层聚合根。对应 research_sessions 表，ID 格式 rs_{nanoid}。 */
export interface ResearchSession {
  /** 会话 ID（rs_{nanoid}） */
  id: string;
  /** 会话标题（初始问题摘要） */
  title: string;
  /** 会话状态 */
  status: ResearchSessionStatus;
  /** 执行模式（监督/自主） */
  mode: ResearchMode;
  /** 最大轮次数（覆盖率循环上限） */
  max_rounds: number;
  /** 覆盖率目标: 最少选中片段数（run_to_full_coverage 判定） */
  min_selected: number;
  /** 扩展位: 模型选择、检索源配置等（对应 options_json 列） */
  options_json?: JsonValue | null;
  /** 创建时间 */
  created_at: string;
  /** 最后更新时间 */
  updated_at: string;
}

/** 轮次 — 一轮研究的全部状态。对应 research_rounds 表，ID 格式 rr_{nanoid}。 */
export interface ResearchRound {
  /** 轮次 ID（rr_{nanoid}） */
  id: string;
  /** 所属会话 ID */
  session_id: string;
  /** 轮次序号（1-based，表内 UNIQUE(session_id, round_no)） */
  round_no: number;
  /** 当前所处阶段（可恢复断点） */
  stage: ResearchStage;
  /** 轮次状态 */
  status: RoundStatus;
  /** 本轮研究计划 */
  plan?: ResearchPlan | null;
  /** 检索查询列表 */
  queries: ResearchQuery[];
  /** 检索到的文档列表 */
  retrieved: RetrievedDoc[];
  /** 去重统计 */
  dedupe?: DedupeStats | null;
  /** 选中片段列表 */
  selection: SelectedChunk[];
  /** 本轮综合（markdown） */
  synthesis_md?: string | null;
  /** 引用列表 */
  citations: CitationRef[];
  /** 轮次指标 */
  metrics?: RoundMetrics | null;
  /** 批评者评审 */
  critic?: CriticReview | null;
  /** 用户笔记（手写批注） */
  note?: string | null;
  /** 标签列表（对应 tags_json 列） */
  tags: string[];
}

/**
 * 制品 — 类型系统不穷举 payload，用 artifact_type + payload_json 扩展。
 * 对应 research_artifacts 表，ID 格式 ra_{nanoid}。
 * ⚠️ id 为 string（与后端一致）；旧前端本地定义曾为 number，消费处已做兼容。
 */
export interface ResearchArtifact {
  /** 制品 ID（ra_{nanoid}） */
  id: string;
  /** 所属会话 ID */
  session_id: string;
  /** 所属轮次（null = 会话级制品，如宏洞察） */
  round_no?: number | null;
  /** 产出代理: planner | subagent:{name} | synthesizer | critic | macro */
  agent: string;
  /** 制品类型（可扩展字符串: plan | report | macro_insight | chart | dataset | ...） */
  artifact_type: string;
  /** 制品载荷（任意 JSON） */
  payload_json: JsonValue;
  /** 载荷大小（字节） */
  size: number;
  /** 创建时间 */
  created_at: string;
}

// ============================================================================
// 运行参数与命令辅助类型（camelCase）
// ============================================================================

/** 运行参数 — 对应 research_run_until 命令（max_rounds / min_selected / silent_approval）。 */
export interface ResearchRunOptions {
  /** 最大轮次数覆盖（None = 沿用会话默认） */
  max_rounds?: number | null;
  /** 覆盖率目标: 最少选中片段数（None = 沿用会话默认） */
  min_selected?: number | null;
  /** 自主审批（跳过 plan_pending_approval） */
  silent_approval?: boolean | null;
}

/** 分块级上下文 — 对应 research_get_chunk_context 命令（document_id / chunk_index / before / after）。 */
export interface ResearchChunkRef {
  /** 资源文档 ID */
  document_id: string;
  /** 分块下标（0-based） */
  chunk_index: number;
  /** 前文块数（None = 默认） */
  before?: number | null;
  /** 后文块数（None = 默认） */
  after?: number | null;
}

// ============================================================================
// 研究事件（后端 → 前端事件流）
// ============================================================================

/**
 * 研究事件 — 后端 `ResearchEvent`（serde tag = "event"，snake_case）的 TS 镜像。
 * `type` 字段值与后端 serde tag 完全一致；载荷键名 snake_case，
 * 嵌套结构体（plan / artifact / options 等）为 camelCase。
 * 非穷尽联合（后端 #[non_exhaustive]）: 新事件只加成员，消费方 switch 保留 default 降级。
 */
export type ResearchEvent =
  /** 会话已启动 */
  | { type: 'session_started'; session_id: string }
  /** 轮次已启动 */
  | { type: 'round_started'; session_id: string; round_no: number }
  /**
   * 阶段推进（对应前端 round_executing / plan_generated / retrieval_completed /
   * dedupe_completed / per_doc_cap_applied / keyword_filter_applied /
   * selection_completed / subagents_done 等阶段事件）
   */
  | { type: 'stage_advanced'; session_id: string; round_no: number; stage: ResearchStage }
  /** 计划待审批（监督模式） */
  | { type: 'plan_pending_approval'; session_id: string; round_no: number; plan: ResearchPlan }
  /** 子代理已启动 */
  | { type: 'subagent_started'; session_id: string; round_no: number; subagent: string; task: string }
  /** 子代理已完成 */
  | { type: 'subagent_completed'; session_id: string; round_no: number; subagent: string; summary: string; confidence: number }
  /** 子代理工具调用 */
  | { type: 'subagent_tool_call'; session_id: string; round_no: number; subagent: string; tool: string; args: JsonValue }
  /** 子代理工具结果 */
  | { type: 'subagent_tool_result'; session_id: string; round_no: number; subagent: string; tool: string; result: JsonValue }
  /** 综合已更新 */
  | { type: 'synthesis_updated'; session_id: string; round_no: number; markdown: string }
  /** 批评者评审已更新 */
  | { type: 'critic_updated'; session_id: string; round_no: number; gaps: string[]; verdict: string }
  /** 宏洞察已生成（跨轮次，会话级） */
  | { type: 'macro_insight_generated'; session_id: string; insight: JsonValue }
  /** 制品已创建 */
  | { type: 'artifact_created'; session_id: string; artifact: ResearchArtifact }
  /** 运行参数已变更（供 Cockpit 等订阅方使用） */
  | { type: 'run_options_changed'; session_id: string; options: ResearchRunOptions }
  /** 通用进度（前端 ingestion_progress / macro_insight_progress 可映射到此） */
  | { type: 'progress'; session_id: string; percent: number; message: string }
  /** 失败（前端 session_failed / error 可映射到此） */
  | { type: 'failed'; session_id: string; error: string };
