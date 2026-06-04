# Round 22: Chat V2 Pipeline 后端诊断

**层级**: 4.3 — 后端模块（Rust）
**预计文件数**: 15-30
**状态**: ⏳ 待执行

## 目标

梳理 Chat V2 对话引擎的后端实现：Pipeline、工具执行器、workspace。

## 扫描文件清单

| # | 文件路径 | 关注点 |
|---|---------|--------|
| 1 | `src-tauri/src/chat_v2/` 全部 .rs 文件 | Chat V2 核心 |
| 2 | `src-tauri/src/chat_v2/pipeline/` 全部 .rs 文件 | 对话 Pipeline |
| 3 | `src-tauri/src/chat_v2/handlers/` 全部 .rs 文件 | 消息处理器 |
| 4 | `src-tauri/src/chat_v2/tools/` 全部 .rs 文件 | 工具定义与执行 |
| 5 | `src-tauri/src/chat_v2/workspace/` 全部 .rs 文件 | Workspace 管理 |
| 6 | `src-tauri/src/chat_v2/adapters/` 全部 .rs 文件 | 后端适配器 |
| 7 | `src-tauri/src/chat_v2/migration/` 全部 .rs 文件 | 数据迁移 |

## 诊断要点

1. **Pipeline 架构**: 消息流的处理阶段
2. **工具系统**: 工具定义格式、执行机制、结果回写
3. **会话管理**: 会话创建、存储、分支
4. **流式响应**: SSE/Stream 实现方式
5. **多模型**: 多模型对比的后端支持

## 输出格式

产出 `round-22-chat-v2-backend.md`
