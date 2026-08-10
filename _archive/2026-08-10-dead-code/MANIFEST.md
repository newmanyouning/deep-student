# MANIFEST — 2026-08-10 死代码清理归档

> 归档时间: 2026-08-10 20:30 CST | 分支: pr/3-pdf-reading
> 依据: `docs/dead-code-backend-dependency-audit-2026-08-10.md` (E 类孤儿成品审计)
> 规则: 本目录文件**不参与编译** (tsconfig include=["src"] 不覆盖根级 _archive/), 恢复时按原路径移回

## 归档清单

| # | 归档文件 | 原路径 | 审计结论 (完成度/必要性/冲突) | 恢复注意 |
|---|----------|--------|------------------------------|----------|
| 1 | `src/components/ReviewSession.tsx` | 同左 | SM-2 复习会话 (547行)。完成度: 高 (UI+store+后端 17 个 review_plan_* 命令全链路完整)。必要性: 间隔重复是学习软件核心功能。冲突: 与题库 session 制复习 (ExamContentView) + Anki 导出 (Anki 自身 SRS) **三套复习范式并存**, 产品定位未定 | ⚠️ 最高价值归档项。`src/stores/reviewPlanStore.ts` 保留在 src (mcp-debug/registerStores 引用), 恢复时只需移回组件 + 决定入口 (建议 learning-hub 复习标签)。**前提: 先回答与 Anki SRS 的产品关系** |
| 2 | `src/components/ReviewCalendarView.tsx` | 同左 | 复习日历热力图 (613行), 同上全链路完整, 依赖同一 reviewPlanStore | 同 #1, 建议与 ReviewSession 一起恢复 |
| 3 | `src/components/shared/MultimodalIndexButton.tsx` | 同左 | 资源级一键多模态索引按钮 (232行)。完成度: 高 (后端 vfs_multimodal_index_resource 在册)。必要性: 中低 — IndexStatusView 已提供索引管理。冲突: 与 IndexStatusView 功能重叠 | 恢复方案: 挂到 learning-hub FinderFileList 右键菜单 (注意该列表是虚拟化渲染, 2026-08 刚修复过滚动 bug, 改动需谨慎) |
| 4 | `src/features/mindmap/components/shared/EmojiPicker.tsx` | 同左 | mindmap 节点 emoji 选择器 (115行)。完成度: 高 (纯 UI)。必要性: 低 — 现役节点直接渲染 icon 字符串, 用户可手动输入 emoji。冲突: 无 | 恢复方案: 挂到 mindmap 节点编辑工具栏 |
| 5 | `src/features/settings/components/ChatMigrationSection.tsx` | 同左 | chat→chat_v2 手动迁移 UI (522行)。完成度: 高 (后端 chat_v2_migrate_legacy_chat 等命令在册)。必要性: 无 — 一次性迁移工具, 本分支迁移已完成。冲突: 无 | 恢复意义仅在于旧数据用户手动迁移; 后端命令仍在, 如需恢复移回 + 挂到 DataGovernanceDashboard |
| 6 | `src/components/TagTreeImportCheckModal.tsx` (+`.css`) | 同左 | 标签树导入前校验弹窗 (88行)。完成度: 高。必要性: 取决于现役导入流是否有校验 (未找到现役校验调用点)。冲突: 无 | 与 #7 成对恢复; 恢复前需确认题库/标签导入流是否需要校验步骤 |
| 7 | `src/utils/TagTreeValidator.ts` | 同左 | 标签树校验逻辑, #6 的依赖 | 与 #6 成对 |
| 8 | `src/promptkit/` (chat-container.tsx / prompt-input.tsx / ui/tooltip.tsx / lib/cn.ts) | 同左 | vendored promptkit 组件库残留 (4 文件), 零外部引用。决策点 3: 用户决定移入保留目录而非删除 (2026-08-10 先从批次 1e 的删除中经 git 历史恢复) | 整体恢复: `git mv _archive/2026-08-10-dead-code/src/promptkit src/`; 原契约测试 promptkitPromptInputActionTooltipContract.test.tsx 已删, 如需恢复见 commit c8c27243~1 |

## 未归档 (保留在 src/ 的孤儿成品)

| 文件 | 原因 |
|------|------|
| `src/components/ApiConfigRecovery.tsx` | 有良好接线方案: ApisTab 检测配置为空/损坏时显示恢复横幅 (后端 check_api_config_status/restore_default_api_configs 在册)。成本低, 待产品确认后接线 |
| `src/components/shared/AiContentLabel.tsx` | AI 生成内容合规标识 (《人工智能生成合成内容标识办法》), 合规必要性高; 接线需产品设计 (展示位置: AI 消息块/生成卡片), 待确认 |
| `src/debug-panel/plugins/PdfMultimodalDebugPlugin.tsx` | ✅ 已恢复注册 (DebugPanelHost.tsx) — MULTIMODAL_INDEX_ENABLED=true, 原"已禁用"注释过时 |

## 恢复操作示例

```bash
# 恢复复习计划 UI (store 仍在 src/, 无需动)
git mv _archive/2026-08-10-dead-code/src/components/ReviewSession.tsx src/components/ReviewSession.tsx
git mv _archive/2026-08-10-dead-code/src/components/ReviewCalendarView.tsx src/components/ReviewCalendarView.tsx
# 然后按上表"恢复注意"接线入口, npx tsc --noEmit 验证
```
