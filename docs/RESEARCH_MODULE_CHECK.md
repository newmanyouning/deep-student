# 研究模块检查报告

> 日期: 2026-08-06 | 检查范围: 研究模块全部调用与功能实现

---

## 一、研究模块组成

| 部分 | 位置 | 状态 |
|------|------|------|
| **Stub 命令 (26个)** | `src-tauri/src/cmd/research_stubs.rs` | ❌ 全部返回 not_implemented |
| **报告 CRUD (4个)** | `src-tauri/src/commands.rs:4111-4175` | ⚠️ list/get/delete 可用，insert 无调用者 |
| **数据库表** | `research_reports` (manager.rs:729) | ⚠️ MIGRATION_DEBT，无 Refinery 迁移 |
| **前端 API wrapper (26个)** | `src/utils/settingsApi.ts:85-188` | ❌ 注释明确"尚未实现" |
| **前端 Store** | `src/stores/researchStore.ts` (506行) | ⚠️ 无 UI 组件调用 |
| **真实深度研究** | `research-mode` 技能 (chat_v2) | ✅ 唯一活跃实现 |

## 二、26 个 Stub 命令功能清单

**会话/轮次管理 (8)**:
- research_get_round / get_round_visual_summary / delete_round
- research_generate_round_report / set_round_note / get_round_note / get_round_notes
- research_generate_session_report

**执行控制 (5)**:
- research_run_until / run_macro / run_to_full_coverage
- research_update_session_options / delete_session

**内容检索 (6)**:
- research_get_chunk_text / get_chunk_context / get_full_chat_history
- research_deep_read_by_docs / deep_read_by_tag / get_full_content

**分析辅助 (3)**:
- research_audit_user_questions / find_similar_questions / count_tokens

**设置/产物 (4)**:
- research_get_setting / set_setting / delete_setting / list_artifacts

## 三、调用关系结论

```
前端 settingsApi.ts (26 wrapper) ──→ 26 个 stub 命令 (全部报错)
                                        ↓
后端 database (research_reports 表) ←── 4 个 CRUD 命令 (insert 无调用者)
                                        ↓
researchStore.ts (506行) ──→ 无 UI 组件消费
                                        ↓
真实功能 = chat_v2 research-mode 技能 (web_search + todo + canvas + ask-user)
```

**关键发现**:
1. 26 个 stub 命令**从未工作过**，前端 wrapper 有注释承认"尚未实现"
2. `insert_research_report` 数据库写入无任何调用者 → 表永远为空
3. researchStore.ts 无 UI 组件消费 → 纯死代码
4. 深度研究已由 chat_v2 `research-mode` 技能实现 (唯一活跃入口)

## 四、建议

| 行动 | 内容 | 风险 |
|------|------|------|
| **删除** | research_stubs.rs (26 stub) + lib.rs 30 处注册 | 低 (零调用) |
| **删除** | research_reports 表 + 4 个 CRUD 命令 + database 方法 | 低 (无写入) |
| **删除** | settingsApi.ts 26 个 research wrapper + researchStore.ts | 低 (无 UI) |
| **保留** | research-mode 技能 (chat_v2 唯一入口) | — |
