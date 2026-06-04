# Round 08: Chat V2 对话引擎 — 诊断报告

**日期**: 2026-05-29
**状态**: ✅ 完成

---

## 模块规模

```
src/features/chat/    474 源文件 (.ts/.tsx) — 全项目最大模块
├── core/             核心引擎 (Store + Session + Types)
│   ├── store/        18 文件, 7325 行 — ChatStore 工厂 + Actions
│   ├── session/      SessionManager (625 行) — LRU 会话缓存
│   ├── middleware/    autoSave, chunkBuffer, eventBridge
│   └── types/         共享/Block/Message/Store 类型定义
├── adapters/         后端通信层
│   └── TauriAdapter.ts  4104 行 — 最大单一文件
├── components/       UI 组件 (60+ 子目录)
│   ├── input-bar/    输入栏系统 (Composer, ModelPicker, Queue)
│   ├── message/      消息渲染 (Block, Citations, Tool)
│   ├── Variant/      多变体并排视图
│   ├── ActivityTimeline/  工具调用时间线
│   └── renderers/    代码/Markdown/Mermaid 渲染器
├── plugins/          插件化扩展点
│   ├── blocks/       块类型注册 (思考/工具/代码/图片...)
│   ├── chat/         聊天模式插件
│   ├── events/       事件处理插件
│   └── modes/        功能模式 (Anki, Research, Mindmap...)
├── skills/           技能系统
│   ├── builtin/      12 内置技能
│   └── builtin-tools/  内置工具定义
├── hooks/            Chat 专用 Hooks
├── pages/            页面级组件 (ChatV2Page, SessionSidebar)
├── queue/            消息队列
├── registry/         插件注册表 (modeRegistry, blockRegistry)
├── utils/            工具函数
└── workspace/        工作区与子代理
```

---

## 核心架构

### 设计原则 (来自 README.md)

1. **单一 SSOT Store** — 每个会话一个 ChatStore 实例
2. **插件化** — 新模式/块类型/事件只需加插件文件
3. **Callback 注入** — Store 不直接调用后端，通过 TauriAdapter 注入
4. **Map + 顺序数组** — O(1) 查找，细粒度更新
5. **操作守卫** — 状态机 + 操作前置检查

### Store 架构 (7325 行，18 个文件)

| 文件 | 行数 | 职责 |
|------|------|------|
| `messageActions.ts` | **1119** | 消息 CRUD (最复杂) |
| `variantStoreActions.ts` | **987** | 变体持久化操作 |
| `variantActions.ts` | **915** | 变体运行时管理 |
| `restoreActions.ts` | **837** | 会话恢复/水合 |
| `sessionActions.ts` | 519 | 会话级操作 |
| `skillActions.ts` | 379 | 技能执行 |
| `createChatStore.ts` | 388 | Store 工厂 + 状态定义 |
| `selectors.ts` | 271 | 派生状态查询 |
| `contextActions.ts` | 247 | 上下文引用管理 |
| `blockActions.ts` | 334 | 块级操作 |
| `queueActions.ts` | 246 | 消息队列 |
| `streamActions.ts` | 150 | 流式接收 |

**评价**: Store 被拆分为 18 个文件，职责清晰，但 messageActions (1119行) 和 variant 相关 (1902行) 仍然偏大。

### TauriAdapter (4104 行)

全项目最大的单一 TypeScript 文件！包含：
- `setup()` — 监听后端事件
- `sendMessage()` — 发送消息到后端 Pipeline
- `abortStream()` — 中断流式生成
- `loadSession()`, `createSession()` — 会话持久化
- Context 引用构建 (通过 contextHelper)
- 多模型并行 (variants)

**问题**: 4104 行违背了 Chat V2 自身的 "插件化" 设计原则。适配器应该是薄层。

### SessionManager (625 行)

- 单例模式，管理多个 ChatStore 实例
- **LRU 缓存** (maxSessions = 10)
- 会话元数据 (title, tags, lastAccess)
- 流式状态订阅
- 保存前驱逐 (save-before-eviction)

---

## 文档质量

Chat V2 有**项目中最好的文档**：

| 文档 | 内容 |
|------|------|
| README.md | 设计目标 + 插件化原则 |
| BLOCK_RENDERING_GUIDE.md | 18 章完整开发者参考 (Block类型/渲染链/工具/引用/变体...) |
| docs/01-可复用清单.md | 可复用组件/工具列表 |
| docs/02-完整架构.md | 插件化架构图 |
| docs/03-数据契约.md | 类型定义、块系统、插件接口 |
| docs/04-实现阶段.md | 实现优先级指南 |
| docs/05-多会话管理.md | SessionManager、LRU、并行流式 |
| docs/architecture/ | 4 个架构图文档 |

---

## 模块间依赖

```
ChatV2Page (UI Shell)
├── SessionManager → ChatStore (SSOT)
│   └── Middleware (autoSave, chunkBuffer, eventBridge)
├── TauriAdapter (Backend Bridge)
│   ├── contextHelper (上下文引用)
│   └── eventBridge (事件转发到 Store)
├── Plugin Registry (mode/block/event)
│   ├── Skills (builtin + custom)
│   └── Modes (Anki, Research, Mindmap, Default)
└── Components (InputBar, Message, Variant, Timeline)
```

---

## 发现的问题

- [ ] **P1** — `TauriAdapter.ts` **4104 行**，是项目最大单一文件。违背了 Chat V2 自身 "插件化" 的设计原则，应拆分为 send/load/abort/setup 等独立模块
- [ ] **P2** — Store 拆分为 18 个文件(7325行) 是好的实践，但 `messageActions.ts`(1119行) 和 `variantActions.ts`(915行) 仍然太大
- [ ] **P3** — Chat V2 有自己的 `skills/builtin/` 和 `hooks/` 目录，与全局 `src/skills/`、`src/hooks/` 形成嵌套层级。技能系统的全局/局部边界需要明确定义
- [ ] **P3** — `adapters/contextHelper.ts` 和 `contextHelperOptimized.ts` 并存，暗示有优化版本但名称暗示两者都在使用

### 正面发现

- **架构设计优秀**: 每个会话独立 Store、Callback 注入、操作守卫、插件注册表
- **文档最佳**: 5 篇核心文档 + 4 篇架构图 + 18 章开发者手册
- **Store 拆分到位**: 18 个文件按职责拆分是项目中最好的 Store 架构
- **LRU 驱逐**: SessionManager 正确实现了会话缓存上限和自动驱逐

---

## 建议优先处理

1. 拆分 `TauriAdapter.ts` (4104行) — 按功能域拆分为 send.ts / load.ts / abort.ts / setup.ts
2. 合并或决策 `contextHelper.ts` 和 `contextHelperOptimized.ts` 的去留
3. 明确 Chat V2 内部 skills/ 与全局 skills/ 的边界和引用规则
