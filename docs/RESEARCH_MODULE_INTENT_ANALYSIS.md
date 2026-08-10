# 研究模块设计意图分析

> 日期: 2026-08-06 | 方法: 26 个 stub 签名 + researchStore 事件协议 + skill 实现三重还原

---

## 一、核心意图: HPIAS 多子代理深度研究系统

研究模块试图实现一个 **HPIAS (Hierarchical Progressive Information Acquisition System) 分层渐进信息获取系统** — 一种多子代理、多轮次、覆盖驱动的深度研究引擎。

从 `researchStore.ts` (506行) 的完整事件协议和 26 个 stub 签名还原的设计蓝图:

```
研究会话 (session)
  └── 轮次 (round) 循环执行直到覆盖目标:
        1. plan_generated           → LLM 生成本轮研究计划
        2. plan_pending_approval    → 监督模式下等待用户审批
        3. queries_prepared         → 生成检索查询
        4. retrieval_completed      → 检索文档 (fetched=N)
        5. dedupe_completed         → 去重 (before→after)
        6. per_doc_cap_applied      → 每文档上限
        7. keyword_filter_applied   → 关键词过滤
        8. selection_completed      → 选择高价值片段
        9. subagent_started × N     → 启动多个子代理，各自研究一个子问题
        10. subagent_thought        → 子代理思考 (LLM JSON)
        11. subagent_tool_call      → 子代理调用工具
        12. subagent_tool_result    → 工具结果
        13. subagent_completed      → 子代理产出摘要+引用+关键发现+置信度
        14. synthesis_updated       → 综合所有子代理发现
        15. critic_updated          → 批评者评审，找出缺口
        16. macro_insight_generated → 跨轮次宏洞察
  └── 会话完成 → 生成报告 (session_report)
```

## 二、设计特点

| 特点 | 证据 |
|------|------|
| **多子代理** | subagent_* 事件 (started/thought/tool_call/tool_result/completed/failed/done) |
| **覆盖率驱动** | run_to_full_coverage / run_until(max_rounds, min_selected) |
| **监督/自主双模式** | plan_pending_approval / silent_approval 参数 |
| **文档分块级检索** | get_chunk_text / get_chunk_context(before/after) |
| **宏洞察** | macro_insight_generated / macro_insight_progress |
| **问答审计** | audit_user_questions(date_range, keywords, group_by) |
| **相似问题** | find_similar_questions(question_text, top_k) |
| **制品管理** | artifact_created / list_artifacts / mergeArtifacts |
| **令牌预算** | count_tokens(document_ids, precise) |

## 三、当前实现状态

| 部分 | 状态 | 说明 |
|------|------|------|
| 前端事件协议 | ✅ 完整 | researchStore.ts 定义全部 HpiasEvent (506行) |
| 前端 API wrapper | ✅ 完整 | settingsApi.ts 26 个 research_* wrapper |
| 后端命令 | ❌ 26 个 stub | 全部返回 not_implemented |
| research_reports 表 | ⚠️ 部分 | list/get/delete 实现，insert 无调用者 |
| **chat_v2 subagent 工具** | ✅ 已有 | `subagent_call` (subagent_executor.rs，含深度限制) |
| **research-mode 技能** | ✅ 简化实现 | 单代理版: web_search + todo + canvas-note + ask-user |
| 前端 HPIAS UI | ⚠️ 无独立页面 | 仅 debug 插件 (MultiAgent/SubagentTest) |

## 四、意图 vs 现实: 差距分析

### 已实现的 (通过 research-mode 技能)
- 调研流程 (搜索 → 整理 → 报告)
- 任务进度管理 (todo-tools)
- 报告撰写 (canvas-note)
- 用户偏好确认 (ask-user)

### 未实现的 (HPIAS 特有)
| 能力 | 现状 | 已有替代 |
|------|------|----------|
| 多子代理并行研究 | ❌ 后端无 | chat_v2 `subagent_call` 工具可用 |
| 分层规划 (plan→approval) | ❌ | research-mode 直接执行，无计划审批 |
| 覆盖率驱动迭代 | ❌ | 无 |
| 批评者评审 (critic) | ❌ | 无 |
| 宏洞察跨轮次 | ❌ | 无 |
| 文档分块级上下文 | ❌ | unified_search 可近似 |
| 结构化事件流 | ❌ 后端不发射 | store 契约已定义 |
| 问答审计/相似问题 | ❌ | memory search 可近似 |

## 五、结论与建议

**意图**: 一个比当前 research-mode 技能更宏大的多子代理研究引擎，具备分层规划、覆盖率驱动、批评者评审、宏洞察等能力。前端协议已完整设计，但后端从未实现。

**关键发现**: 
1. 底层子代理能力 (`subagent_call`) 已在 chat_v2 存在 — HPIAS 需要的只是"编排层"
2. research-mode 技能是 HPIAS 的**单代理简化版**，已满足基本调研需求
3. 完整 HPIAS 是一个大型功能，非短期可实现

**建议**:
| 选项 | 说明 |
|------|------|
| A. 保留 research-mode 技能作为唯一入口 | 删除 26 个 stub + research_reports 表，接受简化 |
| B. 渐进升级技能 | 在 research-mode 中逐步加入子代理调用 (subagent_call)，无需后端 stub |
| C. 实现 HPIAS 后端 | 大工程，需规划 session/round 表 + 事件流 + 编排器 |

**推荐**: 选项 B — 利用已有的 `subagent_call` 工具增强 research-mode 技能，实现"多子代理"效果，同时删除 26 个未实现的 stub 死代码。这样既保留了 HPIAS 的核心意图（多子代理研究），又不需要从零实现整个后端。
