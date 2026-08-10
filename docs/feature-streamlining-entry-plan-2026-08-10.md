# 功能精简与前端入口调整计划

> 日期: 2026-08-10 19:13 CST | 分支: pr/3-pdf-reading (独立版本, 不合并 main)
> 方法: import 语句级可达性分析 (1,470 个 ts/tsx 文件全图) + 5 套入口系统全量盘点
> 前版关系: 取代 `type-unification-dedup-frontend-partition-audit-2026-08-10.md` 第三部分 (其思路错误已在该文档附录二勘误)
> 状态: **阶段 1 已完成 (2026-08-10 20:40)**: 死代码 145 文件 ~31,470 行已删 (4 批 commit), E 类孤儿 8 文件归档 `_archive/` — 详见 `dead-code-backend-dependency-audit-2026-08-10.md` 附录执行记录。阶段 2 (入口合并) 起待开工

---

## 0. 本次调研的方法纪律

前版审计的三类教训, 本次逐条对应:

| 前版错误 | 本次对策 |
|----------|----------|
| 死代码用词匹配, 被同名 API 对象 `templateManager` 骗过 | 只认 import/export/动态 import() 语句, 构建全量引用图从入口 BFS |
| 把组合关系当功能重复 (DataImportExport⊂Settings) | 每条"重复"结论先验证引用方向 |
| 跨域搬进入口 (skills→settings 的 MCP 标签页) | 入口收敛只做**同域合并**, 跨域归属一律不动 |
| 只分析 ModernSidebar 一处 | 覆盖全部 5 套入口系统 + 视图路由 + 重定向 |

---

## 1. 现状基线 (全部经 import 级验证)

### 1.1 入口系统全景 — 5 套

| # | 系统 | 条目 | 来源 |
|---|------|------|------|
| 1 | 桌面 ModernSidebar | **7+1**: 新会话 / 学习资源 / 待办 / 技能管理 / 制卡任务 / 模板管理 / 设置 (+隐藏 ui-lab) | `config/navigation.ts:47` → `ModernSidebar.tsx:423-430` |
| 2 | 移动端 (MobileSidebarNavigation + BottomTabBar) | **5**: 新会话 / 技能管理 / 学习资源 / 制卡任务 / 设置 — **本无待办和模板管理** | `MobileSidebarNavigation.tsx:13-19` 白名单过滤; `BottomTabBar.tsx:14-59` |
| 3 | Topbar | **不渲染导航项** (仅虚拟标题栏: 标题/命令面板/窗口控制) | `Topbar.tsx:131` 调 createNavItems 但未用于渲染 |
| 4 | 命令面板 | **31 个 ready** + ~48 个 hidden (hidden 已从面板过滤, 用户不可见) | `capabilityRegistry.ts` |
| 5 | 视图路由 | `currentView` 状态机 + 17 条废弃视图重定向 + 兜底回退 chat-v2 | `app/navigation/canonicalView.ts:7-53` |

设置页内部 11 个标签: apis / models / general / appearance / **mcp** / search / statistics / **data-governance** / params / **shortcuts** / about (`useSettingsNavigation.tsx:33-55`)。

### 1.2 一级入口的真实归属 (前版误判的修正)

| 入口 | 实际内容 | 关键事实 |
|------|----------|----------|
| 模板管理 | **Anki 卡片模板** CRUD (`CustomAnkiTemplate` + `data/ankiTemplates`) | 与制卡任务同域且已耦合: task-dashboard 通过 `handleTemplateSelectionRequest` 把它当模板选择器用, 选完返回上一视图 (`App.tsx:1669-1690`) |
| 技能管理 | **chat-v2 技能 IDE** (skill markdown 创作, `@/features/chat/skills`) | 聊天输入栏已有技能选择面板 (`InputBarUI.tsx:2421` `btn-toggle-skill` + `Ctrl/Cmd+Shift+S`); 命令面板有 `nav.goto.skills-management` |
| 制卡任务 | Anki 制卡任务队列/监控 (`components/anki/TaskDashboardPage`) | 与 chat 内 Anki 面板是**有意双 surface**, 不动 |
| 待办 | 待办 + 番茄钟深度耦合 (`TodoMainPanel` 内嵌 `PomodoroPanel`) | 移动端本就没有此入口 |

### 1.3 死代码清单 (import 级确认, 共 138 个组件级文件 ≈ 28,919 行)

按目录分组 (完整 138 行清单在执行时以引用图输出为准逐批核对):

| 组 | 代表文件 | 规模 |
|----|----------|------|
| 模板死代码 | `TemplateManager.tsx` (1407行) + `RealTimeTemplateEditor/` (706行) | ~2,100 行 |
| 开发测试残留 | `dev/ChatSaveTestPanel.tsx` (1482行) + scenarios | ~1,600 行 |
| 题库旧组件 | `BatchOperationToolbar/` (1300行), `VirtualQuestionList`, `ExamSheetMobileLayout`, `CroppedExamCardImage`, `ExamCardImage`, `ExamPageImage`, `ReviewSession` (547), `ReviewCalendarView` (613) 等 | ~5,000 行 |
| anki 旧模块 | `components/anki/panels/*` (DocumentUpload/Export/MaterialQueue), 各 index barrel | ~1,300 行 |
| features/notes 死子模块 | `PreviewPanel` (581), `components/NoteEditorView`, `reference-selector/`, `NoteTagsEditor`*, `NotesSidebarSearch`* 等 | ~2,500 行 |
| features/chat 死子模块 | `workspace/` 整个目录 (AgentCard/WorkspacePanel/SubagentContainer 等, ~1665行), `plugins/chat/MultiSelectModelPanel` (580), `folder/FolderSelector` (443), 各 barrel | ~4,000 行 |
| settings 废弃 section | `AnkiConnectSettingsSection`, `ChatMigrationSection` (522), `McpToolsManager`, `McpToolEditorModal`, `ModelAssignmentPresets`, `SystemSettingsSection`, `data-governance/MigrationTab` | ~2,400 行 |
| learning-hub 死代码 | `LearningHubSidebarV2.tsx`, `FolderSelectorDialog`, `FinderSearchBar`, `IndexDiagnosticPanel` (538), apps barrels | ~1,100 行 |
| debug-panel 未注册插件 | 8 个插件 (ChatAnkiParse/PdfMultimodal/Streaming 等调试器) | ~4,150 行 |
| skills 死代码 | `skills-management/SkillsSidebar.tsx` (382) | ~400 行 |
| 杂项 | `DocumentViewer`, `LoadingScreen`, `ModernSelect`, `McpStatusIndicator`, `ui/shad/Command.tsx`, `ui/shad/Combobox.tsx`, `ui/unified-sidebar/*`, `promptkit/*` 等 | ~2,500 行 |

(* = 仅被测试引用, 见决策点 2)

### 1.4 可达但废弃/桩

| 项 | 现状 |
|----|------|
| `NoteEditorPortal.tsx` | App.tsx 引用但**固定返回 null** ("白板功能已移除") — 占位组件 |
| dstu 编辑器 create 模式 | NoteEditorWrapper 等 5 个 wrapper 的 create 分支渲染 "coming soon"/报错 |
| `ImportConversationDialog` | 完整 UI 但后端永远返回"导入功能尚未实现" |
| 开发视图 (tree-test/crepe-demo/chat-v2-test/llm-playground) | 仅 DEV 可达, 生产构建被 DevNull 替换 — 保留 |

---

## 2. 精简原则

1. **只删不可达代码** — 功能零损失; 每个删除批次以引用图 + tsc + build 三重验证
2. **双 surface 是有意架构, 不动** — chat 面板与独立页并存 (Anki/技能/资源) 是设计模式, 不是重复
3. **入口收敛只做同域合并** — 模板管理 (Anki 域) 并入制卡任务; 不做跨域搬运 (skills/todo 归属不动)
4. **删入口 ≠ 删页面** — 视图路由保留 + canonicalizeView 加重定向, 旧入口名可逆
5. **每阶段独立验收、独立提交、可独立回滚**

---

## 3. 目标入口结构

### 桌面 ModernSidebar: 7 → 6

```
新会话 (chat-v2)        不变
学习资源 (learning-hub)  不变
待办 (todo)             不变 (番茄钟耦合, 且移动端本就无此入口)
技能管理 (skills-management) 不变 (chat 技能 IDE, 聊天栏/命令面板均可达)
制卡 (task-dashboard)    ★ 吸收模板管理: 页内加"模板"标签
设置 (settings)          不变
```

- **模板管理视图保留**: 作为 task-dashboard 的内部标签 + 选择器模式 (picker 流程不变)
- `canonicalizeView` 新增: `template-management → task-dashboard` (旧入口名/命令面板/事件全部平滑落地)
- 移动端: 已是 5 入口且无模板管理, **无需改动** (天然一致)
- 命令面板: `nav.goto.template-management` 重定向到制卡; 可新增 `nav.goto` 直达模板标签

### 不做的事 (前版建议的否决项)

- ❌ skills-management → settings (张冠李戴: 它是 chat 技能 IDE, 且 MCP 配置早已在 settings)
- ❌ todo → learning-hub (撕裂番茄钟耦合, 混淆资源/任务隐喻)
- ❌ template-management → settings (它是 Anki 卡片模板, 与设置无关)
- ❌ feature 目录归位大搬运 (零用户价值, churn 大)

---

## 4. 分阶段计划

### 阶段 0: 基线保护 (0.5 天)

| 步骤 | 内容 |
|------|------|
| 0.1 | 打 git 检查点 (tag 或备份分支), 记录当前 commit |
| 0.2 | 基线验证: `npx tsc --noEmit` + `cargo check --tests` 0 错误存档 |
| 0.3 | 列出引用死代码源码的契约测试清单 (`secondarySurfaceShellContract.test.ts` 等 readFileSync 类测试), 标注需同步更新处 |

### 阶段 1: 死代码清除 (1-2 天, 零风险, 5 批次)

每批次流程: 删文件 → `tsc --noEmit` → `vite build` → 单批 commit (可整批 revert)。批次划分按耦合内聚, 避免半删状态:

| 批次 | 内容 | 验证重点 |
|------|------|----------|
| 1a | 模板死代码: TemplateManager + RealTimeTemplateEditor (+template.worker) | 确认 `data/ankiTemplates.ts` (API 对象) 不删 |
| 1b | features/chat 死子模块: workspace/ 整目录, plugins/chat 死面板, folder/, 各 barrel | InputBar 相关逐个复核 (聊天页是高敏感区) |
| 1c | 题库/复习旧组件: BatchOperationToolbar, ReviewSession, ReviewCalendarView, Exam*Image 等 | ExamContentView 现役链路不受影响 (抽查 grep) |
| 1d | anki 旧 panels + notes 死子模块 + learning-hub 死代码 + skills/SkillsSidebar | components/anki/TaskDashboardPage.tsx 现役, 只删 panels/barrels |
| 1e | settings 废弃 sections + debug-panel 未注册插件 + 杂项 (DocumentViewer/LoadingScreen/ui 组件/promptkit*) | Settings.tsx 现役 11 标签不受影响 |

预期产出: 删 ~138 文件 ~29,000 行, 构建产物体积小幅下降, 无行为变化。

### 阶段 2: Anki 域入口合并 (1 天)

| 步骤 | 内容 |
|------|------|
| 2.1 | `TaskDashboardPage` 加"模板"标签, 内嵌 `TemplateManagementPage` 浏览模式 (组件已支持 isSelectingMode 双态, 复用不重写) |
| 2.2 | `navigation.ts` 移除 template-management 项; `NavViewType` 类型清理 |
| 2.3 | `canonicalView.ts` 加 `template-management → task-dashboard` 重定向 (保留视图路由: 选择器模式仍渲染模板页) |
| 2.4 | `App.tsx` picker 流改道: `handleTemplateSelectionRequest` 目标从独立视图改为 task-dashboard 内标签, 选择回流畅通 |
| 2.5 | 命令面板 `nav.goto.template-management` 更新; i18n key (zh/en) 调整 |
| 2.6 | 契约测试更新: `secondarySurfaceShellContract.test.ts` / `segmentedControlMigrationContract.test.ts` (readFileSync 源码断言) |
| 2.7 | 走查: 桌面 6 入口 / 移动 5 入口 / picker 回流 / 命令面板跳转 / 旧事件 `template-management` 重定向生效 |

### 阶段 3: 命令面板死注册清理 (0.5 天)

- 48 个 hidden 命令: 删除注册代码 (保留 `capabilityRegistry` 机制与 'hidden' 状态类型), 或按决策点 5 处理
- `learning.commands.ts` 等模块的 TODO 桩函数一并清理

### 阶段 4: 杂项收尾 (0.5 天)

- `NoteEditorPortal` null 组件: 删组件 + 清 App.tsx 引用 (决策点 7)
- dstu create 模式 "coming soon": 按决策点 6 处理 (禁用新建入口 or 保留)
- `ImportConversationDialog` 永远失败: 按决策点 4 处理

### 阶段 5: 可选深化 (需逐项用户确认, 不在本计划默认范围)

- 技能管理入口进一步收敛 (并入聊天技能面板的"管理"按钮)
- 移动端补待办入口 (是增强不是精简, 单独评估)
- dev 视图 (ui-lab/tree-test 等) 从 CurrentView 类型中剥离为 dev-only 类型

---

## 5. 决策点 (开工前需用户拍板)

| # | 问题 | 选项 | 建议 |
|---|------|------|------|
| 1 | 桌面目标入口数 | A. 6 个 (模板并入制卡) / B. 5 个 (再把技能并入聊天面板) | **A** (技能 IDE 是创作工具, 保留入口; B 可做阶段 5 选项) |
| 2 | 仅被测试引用的组件 (NoteTagsEditor/NotesSidebarSearch/UnifiedSidebarSection/prompt-input) | A. 组件+测试成对删 / B. 保留 | **A** (测试死代码无意义; NoteContentView 的标签编辑走的是自己的 handleTagsChange, 不依赖这些) |
| 3 | `promptkit/` 目录 ( vendored 组件库, 3 文件) | A. 删 / B. 留作备用组件库 | **A** (需要时随时可从 git 历史恢复) |
| 4 | ImportConversationDialog (永远失败的导入对话框) | A. 隐藏入口 / B. 保留待实现 | **A** (隐藏按钮, 代码保留, 后端实现后恢复) |
| 5 | 48 个 hidden 命令 | A. 删注册代码 / B. 保留 | **A** (registry 机制保留, 未来命令按 ready 注册) |
| 6 | dstu create 模式 "coming soon" | A. 新建按钮暂时禁用 / B. 保留现状 | **B** (learning-hub 的新建走的是自己的创建流程, 不经过这些 wrapper; 不动) |
| 7 | NoteEditorPortal (返回 null 的占位组件) | A. 删除并清理引用 / B. 保留 | **A** |

## 6. 验收标准

- **每批次**: `tsc --noEmit` 0 错误; `vite build` 通过; 单批 git commit
- **阶段 2 专项**: 桌面 6 入口渲染正确; 移动端 5 入口不变; 制卡→模板→选择回流无死路; `template-management` 旧视图名重定向生效; 命令面板可跳制卡
- **全程**: 无功能丢失声明成立 — 所有被删文件在删除前经引用图复核; 视图路由保留 + 重定向保证旧入口名不死链
- **最终**: cargo check --tests + tsc + 生产构建三包 (exe/NSIS/MSI) 通过

## 7. 风险与回滚

| 风险 | 缓解 |
|------|------|
| 引用图漏判 (字符串事件/动态跳转) | 事件只触发已可达组件的行为, 不引入文件依赖 (已全库排除动态 import 模板字符串); 每批 build 验证兜底; 单批 commit 可 revert |
| 契约测试读源码断言误炸 | 阶段 0 已列清单, 阶段 1/2 同步更新 |
| 入口合并破坏 picker 回流 | 阶段 2.4/2.7 专项走查; 视图路由保留使回退 = 改回 navigation.ts 一行 |
| 删 NoteTagsEditor 影响笔记标签编辑 | 现役链路是 NoteContentView 自带 handleTagsChange, 删除前 grep 复核确认 |

---

> 调研方法: 2 个并行代理 (import 图可达性分析 + 入口系统盘点) + 主会话人工复核关键结论
> 关键复核: TemplateManager/RealTimeTemplateEditor 死代码确认 (前版漏删); template-management=Anki 卡片模板且与 task-dashboard 双态耦合; skills-management=chat 技能 IDE 且聊天栏可达; 移动端本无 todo/template 入口
