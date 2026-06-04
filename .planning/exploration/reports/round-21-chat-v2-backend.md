# Round 21: Chat V2 Pipeline 后端 — 诊断报告

**日期**: 2026-05-29
**状态**: ✅ 完成 (架构级扫描)

---

## 模块规模: 102 文件, 87,248 行 — 全项目最大后端模块

```
src-tauri/src/chat_v2/
├── 核心 (19 文件)
│   ├── types.rs              3670 行 🔴 类型定义
│   ├── repo.rs               4371 行 🔴 数据仓库层
│   ├── vfs_resolver.rs       2664 行 🟡 VFS 解析器
│   ├── pipeline_tests.rs     2094 行 🟡 测试
│   ├── event_bus.rs          — 事件总线
│   ├── state.rs              — 全局状态
│   └── ...
├── pipeline/ (13 文件) — 对话 Pipeline
│   ├── multi_variant.rs      3045 行 🔴 多变体引擎
│   ├── tool_loop.rs          2177 行 🟡 工具循环
│   └── ...
├── tools/ (35 文件) — 工具执行器
│   ├── chatanki_executor.rs  5758 行 🔴🔴 最大文件!
│   ├── builtin_resource_executor.rs 3625 行 🔴
│   ├── template_executor.rs  2369 行
│   ├── builtin_retrieval_executor.rs 2064 行
│   └── ...
├── handlers/ (15 文件) — 消息处理器
│   ├── send_message.rs       2064 行
│   └── ...
├── workspace/ (13 文件) — 工作区/子代理
├── adapters/ (4 文件) — LLM 适配器接口
└── migration/ (3 文件) — 数据迁移
```

---

## 关键发现

| 文件 | 行数 | 说明 |
|------|------|------|
| `chatanki_executor.rs` | **5758** | 🔴 全项目最大 Rust 文件 — Anki 卡片生成执行器 |
| `repo.rs` | 4371 | 🔴 数据仓库 — 所有 Chat 相关的数据库操作 |
| `types.rs` | 3670 | 🔴 类型定义 — God Type File |
| `builtin_resource_executor.rs` | 3625 | 🔴 内置资源执行器 |
| `multi_variant.rs` | 3045 | 🟡 多变体引擎 |

### 架构

Chat V2 后端采用 **Pipeline 架构**:
```
send_message → Pipeline (multi_variant) → ToolLoop → Handlers
                  ↓
              EventBus → Frontend (via Tauri events)
```

工具系统通过 `tools/` 目录的 35 个执行器文件实现，每个执行器负责一类工具调用。

---

## 发现的问题

- [ ] **P1** — `chatanki_executor.rs` **5758 行** — 可能是全项目最大的单一源代码文件。需要紧急拆分
- [ ] **P1** — `repo.rs` 4371 行 — 所有 Chat 相关的数据库操作集中在一个文件
- [ ] **P1** — `types.rs` 3670 行 — 与前端 types/index.ts 相同的问题在后端重演
- [ ] **P2** — `tools/` 35 个执行器文件 — 设计合理但 chatanki_executor 和其他文件大小差异巨大 (5758 vs 平均 ~500)
- [ ] **P3** — `pipeline_tests.rs` 2094 行 — 测试代码在生产目录中，应放在 `tests/`

---

## 建议优先处理

1. 拆分 `chatanki_executor.rs` (5758行) — 最高优先级
2. 拆分 `repo.rs` (4371行) — 按实体创建 repos/sessions.rs, repos/messages.rs...
3. 拆分 `types.rs` (3670行) — 按域创建 types/messages.rs, types/tools.rs...
4. 将 `pipeline_tests.rs` 移入 `tests/` 目录
