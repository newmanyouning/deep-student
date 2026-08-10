# DeepStudent 记忆架构 vs 角色扮演机器人记忆架构 — 对比与优化报告

> 日期: 2026-07-26 | 对比基准: 席恩 (角色扮演机器人) 5 层记忆架构 (L0-L4)
> 分析范围: 完整记忆系统代码级审查 + 架构对比

---

## 执行摘要

DeepStudent 的记忆架构在 **语义检索、LLM 决策、分类摘要** 方面强于角色扮演机器人。但在 **会话工作记忆 (L1)、情感状态记忆 (L4)** 和 **事件日志压缩 (L3)** 三个关键层存在明显差距。

**核心建议**: 不必重写现有架构，只需增加 **L1 会话工作上下文** 和 **L3 事件日志** 两个新层，将现有的 MemoryService 作为 L2+L3 的事实层保留。

---

## 1. 现有架构 vs 席恩架构 — 完整对比

### 1.1 层映射

```
┌──────────────────────────────────────────────────────────────────┐
│  席恩 (角色扮演)                   DeepStudent (学习软件)           │
├──────────────────────────────────────────────────────────────────┤
│                                                                  │
│  L0 人格内核 (~2KB, 只读)         │  ❌ 无对应                     │
│    角色卡: 身份/性格/语言风格      │  最接近: 系统 prompt          │
│    永不自动更新                    │  但这不是持久化的              │
│                                  │  — 需要增加 L0               │
│                                  │                              │
│  L1 关系与约定 (~2KB, 极少更新)    │  ❌ 无对应                     │
│    称呼体系/关键承诺/暗号/纪念日   │  — 需要增加 L1               │
│    追加并标注日期                  │                              │
│                                  │                              │
│  L2 用户画像 (~4KB, 慢速更新)      │  ✅ MemoryService            │
│    身份/工作/经济/兴趣/习惯/雷区   │    fact/study/note 类型       │
│    过时标注而非删除                │    LLM 决策 ADD/UPDATE/APPEND │
│                                  │    自动提取 (auto_extractor)  │
│                                  │    用户画像摘要 (profile)      │
│                                  │  — 相当, 但缺情感/关系维度     │
│                                  │                              │
│  L3 事件日志 (滚动, 持续追加)       │  ⚠️ 部分对应                  │
│    重要事件/情绪高点/共同经历      │  MemoryEvolution             │
│    每次会话结束提炼 0-3 条         │    stale/hits 进化           │
│    每月压缩: 里程碑 + 时期摘要     │  CategoryManager              │
│    日常琐事合并为时期摘要          │    __cat_*__ 分类摘要          │
│                                  │  — 缺事件日志 + 压缩         │
│                                  │                              │
│  L4 工作上下文 (会话级, 易失)      │  ⚠️ 部分对应                  │
│    当前话题/未完成约定/今日情绪    │  Chat V2 Pipeline             │
│    每次会话重写, 旧内容进 L3       │    会话消息 (无独立 L1)      │
│                                  │  — 无独立工作上下文存储        │
│                                  │  — 消息历史被截断而非整理      │
└──────────────────────────────────────────────────────────────────┘
```

### 1.2 能力对比矩阵

| 能力 | 席恩 | DeepStudent | 差距 |
|------|------|-------------|------|
| **人格连续性** | ✅ L0 只读 | ❌ 无 | 🔴 高 |
| **关系/承诺追踪** | ✅ L1 日期标注 | ❌ 无 | 🔴 高 |
| **情感状态记忆** | ✅ L4 今日情绪 | ❌ 无 | 🔴 高 |
| **事件日志** | ✅ L3 按月压缩 | ⚠️ 有进化但无事件日志 | 🟡 中 |
| **用户画像** | ✅ L2 慢速更新 | ✅ profile summary | 🟢 低 |
| **语义检索** | ⚠️ 简单检索 | ✅ LanceDB 向量+FTS | 🟢 优 |
| **LLM 决策** | ⚠️ 无 | ✅ ADD/UPDATE/APPEND | 🟢 优 |
| **自动提取** | ⚠️ 无 | ✅ auto_extractor | 🟢 优 |
| **分类摘要** | ⚠️ 无 | ✅ __cat_*__ | 🟢 优 |
| **压缩/遗忘** | ✅ L3/L4 允许 | ⚠️ stale 标记但不删除 | 🟡 中 |
| **审计** | ⚠️ 无 | ✅ audit_log + idempotency | 🟢 优 |
| **隐私模式** | ⚠️ 无 | ✅ 禁用外部 API | 🟢 优 |

---

## 2. 具体差距分析

### 差距 1: L0 人格内核 — 无稳定身份

**席恩的做法**: 角色卡永不修改，L0 只读，确保"无论记忆怎么变，说话方式不变"。

**DeepStudent 的问题**: 系统 prompt 由 `build_system_prompt` 动态生成，包含用户画像但没有稳定的"系统身份"。没有 L0 意味着每次会话的"助手是谁"都可能漂移。

**优化方案**: 增加一个持久的 **L0 系统角色定义**，存储在 `memory_config` 或独立的系统笔记中:

```rust
// memory/config.rs 新增
pub struct SystemIdentityConfig {
    pub role_name: String,        // "学习助手"
    pub personality: String,      // "耐心、鼓励、简洁"
    pub teaching_style: String,   // "苏格拉底式提问"
    pub language_style: String,   // "中文为主，简洁明了"
    pub core_principles: Vec<String>, // ["永远鼓励用户", "不评判错误答案"]
}
```

注入到系统 prompt 作为固定前缀，**不参与记忆更新**。

### 差距 2: L1 关系与约定 — 无承诺追踪

**席恩的做法**: 关键承诺、称呼体系、纪念日、关系状态变化都记录在 L1，追加并标注日期，永不删除。

**DeepStudent 的问题**: 用户和 AI 之间的承诺（如"帮我记住下次要带错题本"、"每周五复习"）没有被持久化。`fact` 类型记忆可以存储这些，但没有"约定"的特殊处理。

**优化方案**: 增加 **Promise 类型** 记忆:

```rust
// memory/service.rs 新增
pub enum MemoryType {
    Fact,       // 原子事实
    Study,      // 学习记忆
    Note,       // 经验笔记
    Promise,    // 🆕 约定/承诺 — 永不删除，只能"完成"或"变更"
}
```

Promise 类型的记忆:
- 不进入时间衰减
- 不被 LLM 决策 DELETE
- 搜索时始终包含在 `user_profile` 注入中
- 支持状态: pending / completed / changed

### 差距 3: L3 事件日志 — 有进化但无日志

**席恩的做法**: 每次会话结束提炼 0-3 条事件，按月压缩成"里程碑 + 时期摘要"，日常琐事合并为摘要。

**DeepStudent 的问题**: `MemoryEvolution` 只处理 stale/hits 标记，不记录事件。CategoryManager 生成的是分类摘要，不是时间线事件。

**优化方案**: 增加 **EpisodicLog** 类型，独立于现有 MemoryType:

```rust
// 新文件: memory/episodic_log.rs
pub struct EpisodicLog {
    pub id: String,
    pub timestamp: String,
    pub event_type: EventType, // Milestone / EmotionalHigh / LearningProgress / Note
    pub summary: String,       // 一句话
    pub user_emotion: Option<String>, // 用户情绪
    pub ai_response: Option<String>,  // AI 应对方式
    pub session_id: String,
    pub archived: bool,
}
```

事件日志按月压缩:
- 保留 Milestone 类型全部
- EmotionalHigh 保留摘要
- LearningProgress 合并为"时期摘要"
- 原始条目归档到 `episodic_archive/YYYY-MM.md`

### 差距 4: L4 工作上下文 — 消息截断而非整理

**席恩的做法**: L4 会话开始时全文注入，会话结束时回写。当前话题、未完成约定、今日情绪都作为上下文。

**DeepStudent 的问题**: Chat V2 Pipeline 直接截断消息历史 (`truncateHistory`)，没有"工作上下文"概念。如果用户说"我们刚才在讨论第三章的内容，能继续吗？"，系统不知道"第三章"是什么，因为消息历史被截断了。

**优化方案**: 增加 **SessionContext** 概念:

```rust
// memory/session_context.rs
pub struct SessionContext {
    pub session_id: String,
    pub current_topic: Option<String>,     // 当前话题
    pub pending_commitments: Vec<String>,  // 未完成的约定
    pub user_emotion: Option<String>,      // 当前情绪
    pub last_topic_id: Option<String>,     // 上次讨论的资源/笔记 ID
    pub created_at: String,
    pub updated_at: String,
}
```

SessionContext 在每次会话开始时注入 system prompt，在会话结束时提炼到 L3 事件日志。

### 差距 5: L2 用户画像 — 缺情感/关系维度

**席恩的做法**: L2 记录身份、工作、经济、兴趣、习惯、情感模式、雷区。情感模式是"她喜欢被怎么称呼"、"她什么时候需要安慰"。

**DeepStudent 的问题**: `fact` 类型只记录事实，不记录"用户偏好"的元维度。例如"用户喜欢被鼓励"和"用户不喜欢被批评"是两种重要的元事实，但当前没有特殊处理。

**优化方案**: 增加 **Preference** 类型作为 `fact` 的子类型:

```rust
pub enum MemoryPurpose {
    Internalized,
    Memorized,
    Supplementary,
    Systemic,
    Preference, // 🆕 用户偏好/情感模式
}
```

Preference 类型:
- 搜索时权重更高 (1.6x)
- 注入 system prompt 时放在最前面
- LLM 决策时优先保留
- 例: "用户喜欢鼓励式反馈", "用户不喜欢被直接否定"

---

## 3. 优化路线图

### 阶段 1: 增加 L1 关系与约定 (低难度，高影响)

**改动范围**: `memory/service.rs` + `memory/handlers.rs` + `chat_v2/pipeline/prompt.rs`

1. 增加 `Promise` 类型到 `MemoryType`
2. 增加 `PromiseStatus` enum (pending/completed/changed)
3. 修改 `write_smart` 支持 Promise 类型
4. 修改 `load_user_profile` 将 pending Promise 注入 system prompt
5. 增加 `builtin-promise_*` 工具

**代码量**: ~200 行新增 + ~50 行修改

### 阶段 2: 增加 L3 事件日志 (中难度，高影响)

**改动范围**: 新文件 `memory/episodic_log.rs` + `memory/evolution.rs` + `chat_v2/pipeline/persistence.rs`

1. 新增 `EpisodicLog` 结构和 VFS 存储
2. 会话结束时自动提炼 0-3 条事件
3. 按月压缩事件日志
4. `load_user_profile` 加载近期事件日志摘要

**代码量**: ~300 行新增

### 阶段 3: 增加 L4 工作上下文 (高难度，最高影响)

**改动范围**: `memory/session_context.rs` + `chat_v2/pipeline.rs` + `chat_v2/handlers/send_message.rs`

1. 新增 `SessionContext` 结构
2. 会话开始时加载 L4 注入
3. 会话结束时提炼到 L3
4. 支持"继续上次讨论"的语义恢复

**代码量**: ~400 行新增 + ~100 行修改

### 阶段 4: L0 人格内核 (低难度，中影响)

**改动范围**: `memory/config.rs` + `chat_v2/pipeline/prompt.rs`

1. 新增 `SystemIdentityConfig` 配置
2. 注入到系统 prompt 固定前缀
3. 不参与记忆更新

**代码量**: ~50 行新增

### 阶段 5: L2 用户画像增强 (中难度，中影响)

**改动范围**: `memory/service.rs` + `memory/llm_decision.rs` + `memory/category_manager.rs`

1. 增加 `Preference` 类型到 `MemoryPurpose`
2. LLM 决策时优先考虑 Preference
3. CategoryManager 增加偏好维度

**代码量**: ~100 行新增

---

## 4. 席恩 vs DeepStudent 优化后架构

```
┌────────────────────────────────────────────────────────────────────┐
│                DeepStudent 优化后记忆架构 (L0-L4)                    │
├────────────────────────────────────────────────────────────────────┤
│                                                                  │
│  L0 系统角色 (只读, ~1KB)                                        │
│    身份: 学习助手 | 性格: 耐心鼓励 | 教学: 苏格拉底式 | 语言: 中文简洁 │
│    文件: memory_config.system_identity                           │
│    更新: 永不自动更新                                            │
│                                                                  │
│  L1 关系与约定 (~2KB, 极少更新)                                   │
│    约定: "每周五复习" | 承诺: "带错题本" | 纪念日: "高考倒计时"     │
│    类型: Promise (pending/completed/changed)                     │
│    更新: 新约定达成/旧约定变更时追加并标注日期                      │
│                                                                  │
│  L2 用户画像 (慢速更新, ~4KB)                                     │
│    事实: 身份/工作/兴趣/习惯 | 偏好: 喜欢鼓励/不喜欢批评             │
│    类型: Fact | Study | Note | Preference                        │
│    更新: 稳定事实修订, 过时标注而非删除                             │
│    自动提取: auto_extractor + LLM 决策                            │
│                                                                  │
│  L3 事件日志 (滚动, 按月压缩)                                     │
│    里程碑: 重要突破 | 情绪高点: 学习高峰 | 时期摘要: 日常琐事合并    │
│    类型: EpisodicLog (Milestone/EmotionalHigh/Progress/Note)     │
│    更新: 每次会话结束提炼 0-3 条, 按月压缩为里程碑+时期摘要          │
│    归档: episodic_archive/YYYY-MM.md                             │
│                                                                  │
│  L4 工作上下文 (会话级, 易失)                                     │
│    当前话题: "第三章微积分" | 未完成约定: "整理错题" | 今日情绪: 焦虑 │
│    类型: SessionContext                                          │
│    更新: 每次会话重写, 旧内容消化进 L3 后清空                       │
│                                                                  │
│  L5 分类摘要 (LLM 聚合, 按需召回)                                  │
│    __cat_偏好__ | __cat_知识__ | __cat_经验__ | __cat_情感__      │
│    类型: CategorySummary (__cat_*__)                            │
│    更新: LLM 聚合, 会话中按需召回                                 │
│                                                                  │
└────────────────────────────────────────────────────────────────────┘
```

---

## 5. 关键技术决策

### 5.1 为什么不用重写现有架构

DeepStudent 的现有架构在 **VFS 存储、向量检索、LLM 决策、审计** 方面是生产级的。重写会丢失这些优势。正确做法是:

- **保留**: MemoryService (L2 事实层), VFS 存储, LanceDB 检索
- **增加**: L1 (Promise), L3 (EpisodicLog), L4 (SessionContext), L0 (SystemIdentity)
- **增强**: Preference 类型, 情感维度

### 5.2 席恩 vs DeepStudent 的本质区别

| 维度 | 席恩 (角色扮演) | DeepStudent (学习) |
|------|----------------|-------------------|
| **核心需求** | 人格连续性 | 知识连续性 |
| **记忆主体** | 用户关系 | 用户知识 |
| **关键操作** | 情感记忆 | 学习进度 |
| **注入策略** | 全文注入 | 按需召回 |
| **遗忘策略** | 淡化琐事 | 保留知识 |
| **优化目标** | 人格稳定 | 学习效果 |

### 5.3 保留 DeepStudent 现有优势

1. **LanceDB 向量检索** — 席恩的简单关键词检索无法比拟
2. **LLM 决策** — ADD/UPDATE/APPEND 比席恩的规则判断更智能
3. **分类摘要** — __cat_*__ 是席恩没有的
4. **审计+幂等** — 生产级质量保证
5. **隐私模式** — 本地化降级

---

## 6. 优先级排序

| 优先级 | 阶段 | 难度 | 影响 | 工作量 |
|--------|------|------|------|--------|
| **P0** | 阶段 1: L1 关系与约定 | 低 | 高 | ~250 行 |
| **P1** | 阶段 3: L4 工作上下文 | 高 | 最高 | ~500 行 |
| **P2** | 阶段 2: L3 事件日志 | 中 | 高 | ~300 行 |
| **P3** | 阶段 5: L2 偏好增强 | 中 | 中 | ~100 行 |
| **P4** | 阶段 4: L0 人格内核 | 低 | 中 | ~50 行 |

**建议**: 从阶段 1 开始，因为它难度最低且能立即解决"用户和 AI 的承诺追踪"这个高频痛点。阶段 3 (L4) 影响最大但难度最高，可以后续实施。

---

> 🤖 报告由 Claude Code 通过两个并行深度分析代理 + 直接代码审查完成
> 对比基准: 席恩 (角色扮演机器人) 5 层记忆架构 (L0-L4)
> 分析代码量: ~20,000 行记忆系统代码 + ~5,000 行参考文档
