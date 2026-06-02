# Chat V2 发送消息流水线数据追踪

> 生成日期: 2026-06-01
> 覆盖范围: 从后端 API 响应解析到前端 React 渲染的完整数据流

---

## 目录

1. [整体架构概览](#1-整体架构概览)
2. [Step 1: 前端触发 sendMessage (TauriAdapter.ts)](#2-step-1-前端触发-sendmessage)
3. [Step 2: 后端入口 send_message.rs](#3-step-2-后端入口-send_messagers)
4. [Step 3: Pipeline 编排引擎 pipeline.rs](#4-step-3-pipeline-编排引擎)
5. [Step 4: LLM 调用与 Provider 流式解析](#5-step-4-llm-调用与-provider-流式解析)
6. [Step 5: LLMStreamHooks 接收回调 (llm_adapter.rs)](#6-step-5-llmstreamhooks-接收回调)
7. [Step 6: 事件发射至前端 (events.rs)](#7-step-6-事件发射至前端)
8. [Step 7: 前端事件监听与桥接 (TauriAdapter.ts + eventBridge.ts)](#8-step-7-前端事件监听与桥接)
9. [Step 8: 事件注册表分发至处理器](#9-step-8-事件注册表分发至处理器)
10. [Step 9: Store 状态更新与 React 渲染](#10-step-9-store-状态更新与-react-渲染)
11. [关键数据结构对照表](#11-关键数据结构对照表)
12. [类型一致性验证](#12-类型一致性验证)
13. [错误处理覆盖分析](#13-错误处理覆盖分析)

---

## 1. 整体架构概览

```
┌─────────────────────────────────────────────────────────────────┐
│  Frontend (React/TS)                                            │
│  ┌────────────┐  ┌──────────────┐  ┌─────────────────────────┐ │
│  │ ChatStore  │  │ eventBridge  │  │ TauriAdapter            │ │
│  │ (Zustand)  │◄─│  (middleware) │◄─│  (事件监听+命令调用)      │ │
│  └──────┬─────┘  └──────────────┘  └──────────┬──────────────┘ │
│         │                                      │                │
│  ┌──────▼─────┐                  Tauri IPC     │                │
│  │  React     │              window.emit /      │                │
│  │  Components│              invoke             │                │
│  └────────────┘                                 │                │
└─────────────────────────────────────────────────┼────────────────┘
                                                   │
┌──────────────────────────────────────────────────┼────────────────┐
│  Backend (Rust/Tauri)                            │                │
│  ┌──────────────┐  ┌──────────┐  ┌────────────┐ │                │
│  │ events.rs    │◄─┤ pipeline │◄─┤ handlers/  │ │                │
│  │ (ChatV2Event │  │  .rs     │  │ send_msg.rs│ │                │
│  │  Emitter)    │  └────┬─────┘  └────────────┘ │                │
│  └──────────────┘       │                        │                │
│                   ┌─────▼─────────┐              │                │
│                   │ llm_adapter   │              │                │
│                   │ (LLMStreamHooks)             │                │
│                   └─────┬─────────┘              │                │
│                         │                        │                │
│              ┌──────────▼──────────┐             │                │
│              │ model2_pipeline.rs  │             │                │
│              │ (streaming loop)    │             │                │
│              └──────────┬──────────┘             │                │
│                         │                        │                │
│              ┌──────────▼──────────┐             │                │
│              │ providers/mod.rs    │             │                │
│              │ (StreamEvent enum)  │             │                │
│              │ parse_stream()      │             │                │
│              └──────────┬──────────┘             │                │
│                         │                        │                │
│                 LLM API (HTTP SSE)               │                │
└──────────────────────────────────────────────────┼────────────────┘
                                                    │
                                          LLM Provider API
```

---

## 2. Step 1: 前端触发 sendMessage

**文件**: `C:/deep-student/src/features/chat/adapters/TauriAdapter.ts`

### 流程

1. 用户在聊天界面输入消息并发送。
2. React 组件调用 `ChatStore.sendMessage()`。
3. Store 调用通过 `setSendCallback` 注入的 `this.executeSendMessage()`。
4. `executeSendMessage` 构造 `SendMessageRequest` 并调用 `invoke('chat_v2_send_message', request)`。
5. 同时，本地状态中的 `currentStreamingMessageId` 被设置为后端将返回的 `assistant_message_id`。
6. 在调用 `invoke` 前，通过 `resetBridgeState(sessionId)` 清理前一次流式的事件桥状态。

### 输入

```typescript
// SendMessageRequest (Rust 结构体映射)
interface SendMessageRequest {
  sessionId: string;
  userMessage: string;
  attachments?: AttachmentMeta[];
  options?: SendOptions;
  contextRefs?: SendContextRef[];
  // ...
}
```

### 输出（后端 invoke 返回）

```typescript
// Rust 返回 ChatV2Result<String>，即 assistant_message_id
string; // 如 "msg_a1b2c3d4-..."
```

### 类型一致性

- `SendMessageRequest` Rust 结构体定义在 `C:/deep-student/src-tauri/src/chat_v2/types.rs`。
- 前端同名接口定义在 `C:/deep-student/src/features/chat/adapters/types.ts`。
- 字段通过 serde `camelCase` 序列化，前端 TypeScript 使用 `camelCase`。

---

## 3. Step 2: 后端入口 send_message.rs

**文件**: `C:/deep-student/src-tauri/src/chat_v2/handlers/send_message.rs`

### 关键函数

```rust
#[tauri::command]
pub async fn chat_v2_send_message(
    window: Window,
    state: State<'_, ChatV2State>,
    request: SendMessageRequest,
) -> ChatV2Result<String>
```

### 流程

1. 接收 `SendMessageRequest`。
2. 检查是否有并行模型 ID (multi-variant) 或单模型。
3. 创建 `CancellationToken` 并通过 `ChatV2State` 注册以支持取消。
4. 调用 `ChatV2Pipeline::execute(window, request, cancel_token, chat_v2_state).await`。
5. 返回 `assistant_message_id` (String)。

### 输入/输出对照

| 输入 | 类型 | 来源 |
|------|------|------|
| `SendMessageRequest` | JSON deserialize | 前端 invoke |
| `CancellationToken` | tokio_util::sync | `ChatV2State` 注册 |
| `ChatV2State` | tauri::State | Tauri 状态管理 |

| 输出 | 类型 | 去向 |
|------|------|------|
| `ChatV2Result<String>` | `Ok(String)` / `Err(ChatV2Error)` | 前端 invoke 返回 |

---

## 4. Step 3: Pipeline 编排引擎

**文件**: `C:/deep-student/src-tauri/src/chat_v2/pipeline.rs`

### `ChatV2Pipeline::execute()` (line 298)

完整流水线阶段:

```
execute()
├── 创建 PipelineContext
├── 创建 ChatV2EventEmitter
├── 解析 model_name
├── emit_stream_start (会话级事件)
├── 立即保存用户消息 (P0防闪退)
└── execute_internal()
    ├── Stage 0: 初始化上下文快照
    ├── Stage 1: 检查取消
    ├── Stage 2: load_chat_history
    ├── Stage 3: execute_retrievals (并行 RAG/图谱/记忆/搜索)
    │   └── 每个检索工具 emit start → end 事件
    ├── Stage 3.5: 创建检索资源并添加到快照
    ├── Stage 4: build_system_prompt
    ├── Stage 5: execute_with_tools (LLM调用+工具递归)
    │   └── 核心: self.call_llm_with_adapter() → tool_loop
    ├── Stage 5.5: 工作区空闲期检测
    ├── Stage 6: save_results
    └── Stage 7: run_compaction (P1)
```

### 结果处理 (line 506-663)

| 结果 | 动作 |
|------|------|
| `Ok(())` | `emit_stream_complete_with_usage` + 自动生成 metadata |
| `Err(ChatV2Error::Cancelled)` | 尝试保存已累积内容 + `emit_stream_cancelled` |
| `Err(e)` | 尝试保存已累积内容 + `emit_stream_error` |

### 核心: LLM 调用路径

`execute_with_tools()` 位于 `C:/deep-student/src-tauri/src/chat_v2/pipeline/tool_loop.rs`:

1. 创建 `ChatV2LLMAdapter` 实例（实现 `LLMStreamHooks` trait）。
2. 注册 hook 到 LLMManager: `llm_manager.register_stream_hooks(stream_event, adapter).await`。
3. 调用 `llm_manager.call_unified_model_2_stream()` —— 传入统一请求体 `body`。
4. 构建请求体时包含 `messages`、`tools`、`stream: true`、`model` 等。

---

## 5. Step 4: LLM 调用与 Provider 流式解析

### 5a. Provider 适配器解析

**文件**: `C:/deep-student/src-tauri/src/providers/mod.rs`

`ProviderAdapter` trait 定义:

```rust
pub trait ProviderAdapter: Send + Sync {
    fn build_request(...) -> Result<ProviderRequest, ProviderError>;
    fn parse_stream(&self, line: &str) -> Vec<StreamEvent>;
}
```

`StreamEvent` 枚举 (line 33):

```rust
pub enum StreamEvent {
    ContentChunk(String),       // 正常内容片段
    ReasoningChunk(String),     // 推理内容片段 (DeepSeek-R1/Claude thinking)
    ThoughtSignature(String),   // Gemini 3 思维签名
    ToolCall(Value),            // 工具调用
    Usage(Value),               // Token 使用统计
    SafetyBlocked(Value),       // 安全阻断
    Done,                       // 流式结束
}
```

### 5b. OpenAIAdapter.parse_stream() (line 86)

解析 SSE line `data: {...}`:

```
data: {"choices":[{"delta":{"content":"Hello"}}]}
→ StreamEvent::ContentChunk("Hello")

data: {"choices":[{"delta":{"reasoning_content":"..."}}]}
→ StreamEvent::ReasoningChunk("...")

data: {"choices":[{"delta":{"tool_calls":[{"index":0,"function":{"arguments":"{"}}]}}]}
→ StreamEvent::ToolCall(...)

data: {"usage":{"prompt_tokens":100,"completion_tokens":50}}
→ StreamEvent::Usage(...)

data: [DONE]
→ StreamEvent::Done
```

### 5c. AnthropicAdapter.parse_stream() (line 1099)

解析 Anthropic SSE 事件类型:
- `content_block_delta` → ContentChunk / ReasoningChunk
- `content_block_start` → 开始工具调用累积
- `content_block_stop` → ToolCall (完整)
- `message_delta` → Usage / SafetyBlocked
- `message_stop` → Done

### 5d. 流式循环 (model2_pipeline.rs)

**文件**: `C:/deep-student/src-tauri/src/llm_manager/model2_pipeline.rs`, line ~1880

关键循环:

```rust
// 读取 SSE 行
while let Some(line_result) = rx.next().await {
    let events = adapter.parse_stream(&line);
    for event in events {
        match event {
            StreamEvent::ContentChunk(content) => {
                if let Some(h) = self.get_hook(stream_event).await {
                    h.on_content_chunk(&content);  // → hook 回调
                }
            }
            StreamEvent::ReasoningChunk(reasoning) => {
                if let Some(h) = self.get_hook(stream_event).await {
                    h.on_reasoning_chunk(&reasoning);
                }
            }
            StreamEvent::ToolCall(tc) => {
                // 聚合分块的工具调用
                // 累积到 pending_tool_calls map
                if let Some(h) = self.get_hook(stream_event).await {
                    h.on_tool_call_start(id, name);    // 首次
                    h.on_tool_call_args_delta(id, args_fragment); // 后续片段
                }
            }
            StreamEvent::Usage(usage) => {
                captured_usage = Some(usage);
                if let Some(h) = self.get_hook(stream_event).await {
                    h.on_usage(&usage);
                }
            }
            StreamEvent::Done => {
                stream_ended = true;
                // 完成未聚合的工具调用，通过 on_tool_call 回调
                // 通过 h.on_tool_call(&msg) 传递完整的 ChatMessage
                // 通过 h.on_complete(final_text, reasoning) 通知结束
            }
        }
    }
}
```

---

## 6. Step 5: LLMStreamHooks 接收回调

**文件**: `C:/deep-student/src-tauri/src/chat_v2/pipeline/llm_adapter.rs`

`ChatV2LLMAdapter` 实现 `LLMStreamHooks` trait，将 Provider 层的流式事件转换为 Chat V2 块级 BackendEvent。

### 6a. 事件转换映射

| LLMStreamHooks 回调 | 产生的 BackendEvent |
|---------------------|-------------------|
| `on_content_chunk(text)` | content → chunk |
| `on_reasoning_chunk(text)` | thinking → chunk |
| `on_tool_call_start(id, name)` | tool_call_preparing → start |
| `on_tool_call_args_delta(id, delta)` | tool_call_preparing → chunk (节流) |
| `on_tool_call(msg)` | 收集到 `collected_tool_calls`，由 execute_single_tool 发射 `tool_call → start/end` |
| `on_usage(usage)` | 存储到 `api_usage`（随 stream_complete 传递） |
| `on_complete(text, reasoning)` | finalize所有活跃块（thinking→end, content→end） |

### 6b. `<think>` 标签解析 (line 548-735)

部分中转站不支持 Claude Extended Thinking API，将思维链嵌入为 `<think>...</think>` 标签。适配器实时解析:
1. 缓冲每个 chunk 到 `think_tag_buffer`。
2. `process_think_tag_buffer()` 查找 `<think>` / `<thinking>` 开始标签。
3. 标签前内容 → `content` 块，标签内内容 → `thinking` 块。
4. 支持跨 chunk 边界（不完整的标签前缀会暂留缓冲区待下一 chunk）。

### 6c. 块生命周期管理

```
ensure_thinking_started() → emit THINKING→start
ensure_content_started() → emit CONTENT→start (先finalize thinking)
finalize_all() → emit THINKING→end + CONTENT→end
```

### 6d. 工具调用参数节流

`on_tool_call_args_delta` 累积参数到 `args_delta_buffer`，每满 500 字符发射一次 `TOOL_CALL_PREPARING→chunk`，避免事件风暴。

---

## 7. Step 6: 事件发射至前端

**文件**: `C:/deep-student/src-tauri/src/chat_v2/events.rs`

### 7a. 事件通道

```
块级事件:   chat_v2_event_{session_id}    → BackendEvent (JSON)
会话级事件: chat_v2_session_{session_id}  → SessionEvent (JSON)
```

### 7b. BackendEvent 结构 (line 136)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackendEvent {
    pub sequence_id: u64,         // 递增序列号（会话级）
    pub session_id: Option<String>,
    pub r#type: String,           // 'content', 'thinking', 'tool_call', 'rag', ...
    pub phase: String,            // 'start', 'chunk', 'end', 'error'
    pub message_id: Option<String>,
    pub block_id: Option<String>,
    pub block_type: Option<String>,
    pub chunk: Option<String>,    // 流式内容片段
    pub result: Option<Value>,    // 最终结果
    pub error: Option<String>,
    pub payload: Option<Value>,   // 附加数据（toolName, toolInput等）
    pub skill_state_version: Option<u64>,
    pub round_id: Option<String>,
    pub variant_id: Option<String>,
    pub model_id: Option<String>,
    pub status: Option<String>,   // variant_end 时: success/error/cancelled
    pub usage: Option<TokenUsage>,
}
```

### 7c. SessionEvent 结构 (line 430)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionEvent {
    pub session_id: String,
    pub event_type: String,       // 'stream_start', 'stream_complete', 'stream_error', ...
    pub message_id: Option<String>,
    pub model_id: Option<String>,
    pub error: Option<String>,
    pub duration_ms: Option<u64>,
    pub timestamp: i64,
    pub usage: Option<TokenUsage>,
    // ...
}
```

### 7d. 发射链路

```rust
// events.rs line 829
fn emit(&self, mut event: BackendEvent) {
    let event_name = self.block_event_channel();  // "chat_v2_event_{session_id}"
    self.window.emit(&event_name, &event);
}

// events.rs line 853
fn emit_session(&self, event: SessionEvent) {
    let event_name = self.session_event_channel();  // "chat_v2_session_{session_id}"
    self.window.emit(&event_name, &event);
}
```

### 7e. 序列号生成

`SESSION_SEQUENCE_COUNTERS` (DashMap) 按会话 ID 维护 `AtomicU64`，每个事件发射时递增。前端用此检测乱序和丢失。

### 7f. 事件流动示例 (content block)

```
时刻 0: emit_start('content', msg_id, block_id, None, None)
         → window.emit('chat_v2_event_{sid}', {
             sequenceId: 0, type: 'content', phase: 'start',
             messageId: 'msg_xxx', blockId: 'blk_yyy' })

时刻 1: emit_chunk('content', block_id, "Hello ", None)
         → window.emit('chat_v2_event_{sid}', {
             sequenceId: 1, type: 'content', phase: 'chunk',
             blockId: 'blk_yyy', chunk: 'Hello ' })

时刻 2: emit_chunk('content', block_id, "World", None)
         → { sequenceId: 2, type: 'content', phase: 'chunk',
             blockId: 'blk_yyy', chunk: 'World' }

时刻 3: emit_end('content', block_id, None, None)
         → { sequenceId: 3, type: 'content', phase: 'end',
             blockId: 'blk_yyy' }
```

---

## 8. Step 7: 前端事件监听与桥接

**文件**: `C:/deep-student/src/features/chat/adapters/TauriAdapter.ts`
**文件**: `C:/deep-student/src/features/chat/core/middleware/eventBridge.ts`

### 8a. 监听器注册 (TauriAdapter.ts line 500-514)

```typescript
const blockEventChannel = `chat_v2_event_${this.sessionId}`;
const sessionEventChannel = `chat_v2_session_${this.sessionId}`;

const listenPromise = Promise.all([
  listen<BackendEvent>(blockEventChannel, (event) => {
    this.handleBlockEvent(event.payload);
  }),
  listen<SessionEventPayload>(sessionEventChannel, (event) => {
    this.handleSessionEvent(event.payload);
  }),
]);
```

### 8b. handleBlockEvent → eventBridge (line 1248)

```typescript
private handleBlockEvent(event: BackendEvent): void {
  // 1. ChatAnki 工具调用拦截（调试面板）
  // 2. 多变体事件诊断日志
  // 3. 核心分发:
  handleBackendEventWithSequence(this.getCurrentState(), event);
}
```

### 8c. 序列号检测与乱序缓冲 (eventBridge.ts line 329)

`handleBackendEventWithSequence()`:

1. **去重**: 检查 `processedEventIds` Set 是否已处理过该 `sequenceId`。
2. **过期检查**: `sequenceId <= lastSequenceId` → 忽略。
3. **期望检查**: `sequenceId === expectedSeqId` → 直接处理。
4. **乱序缓冲**: 未来事件存入 `pendingEvents` Map，启动 gap 超时定时器。
5. **Gap 恢复**: 超时后跳过丢失的 sequenceId，按序消费缓冲区。
6. **首包保护**: 如果第一个非 start 事件先到，缓冲等待 start。

### 8d. 事件分发 (eventBridge.ts line 567)

`processEventInternal()`:

```
processEventInternal(store, event)
├── 如果 type === 'variant_start' → handleVariantStart()
├── 如果 type === 'variant_end'   → handleVariantEnd()
├── 如果 variantId 存在 → handleBlockEventWithVariant()
└── 否则 → handleBlockEventWithoutVariant() → handleBackendEvent()
```

### 8e. 核心分发: handleBackendEvent() (line 950)

对 `event.type` 从 `eventRegistry` 获取 `EventHandler`，根据 `event.phase` 调用对应方法:

| phase | 调用 |
|-------|------|
| `start` | `handler.onStart(store, messageId, payload, backendBlockId?)` |
| `chunk` | `handler.onChunk(store, blockId, chunk)` |
| `end` | `handler.onEnd(store, blockId, result)` |
| `error` | `handler.onError(store, blockId, error)` |

### 8f. 会话级事件处理 (TauriAdapter.ts line 1418)

| SessionEvent.eventType | 动作 |
|------------------------|------|
| `stream_start` | 设置 `currentStreamingMessageId`，创建占位消息（子代理场景），更新 meta |
| `stream_reconnect` | 通知重连 UI |
| `stream_complete` | `completeStream('success')` + `handleStreamComplete()` (更新 usage) |
| `stream_error` | `completeStream('error')` + `handleStreamAbort()` (保存) |
| `stream_cancelled` | `completeStream('cancelled')` + `handleStreamAbort()` (保存) |
| `summary_updated` | 更新会话标题和简介 |
| `variant_deleted` | 同步变体删除 |

---

## 9. Step 8: 事件注册表分发至处理器

**文件**: `C:/deep-student/src/features/chat/registry/eventRegistry.ts`
**注册的 Handler**:

| 事件类型 | 处理器文件 | Block 类型 |
|----------|-----------|-----------|
| `content` | `plugins/events/content.ts` | `content` |
| `thinking` | `plugins/events/thinking.ts` | `thinking` |
| `tool_call` | `plugins/events/toolCall.ts` | `mcp_tool` |
| `image_gen` | `plugins/events/toolCall.ts` | `image_gen` |
| `tool_call_preparing` | `plugins/events/toolCall.ts` | 工具参数准备中 |
| `rag` | `plugins/events/retrieval.ts` | `rag` |
| `memory` | `plugins/events/retrieval.ts` | `memory` |
| `web_search` | `plugins/events/retrieval.ts` | `web_search` |
| `multimodal_rag` | `plugins/events/retrieval.ts` | `multimodal_rag` |
| `anki_cards` | `plugins/events/ankiCards.ts` | `anki_cards` |
| `tool_approval_request` | `plugins/events/approval.ts` | — |
| `tool_limit` | `plugins/events/toolLimit.ts` | — |

### content EventHandler (content.ts line 33)

```typescript
onStart:  → store.createBlock(messageId, 'content') 或 createBlockWithId
onChunk:  → store.updateBlockContent(blockId, chunk)  // 追加内容
onEnd:    → store.updateBlockStatus(blockId, 'success')
onError:  → store.setBlockError(blockId, error)
```

### thinking EventHandler (thinking.ts line 34)

```typescript
onStart:  → store.createBlock(messageId, 'thinking')  // 置顶逻辑
onChunk:  → store.updateBlockContent(blockId, chunk)
onEnd:    → store.updateBlockStatus(blockId, 'success')
onError:  → store.setBlockError(blockId, error)
```

### Retrieval EventHandler (retrieval.ts line 54)

```typescript
onStart:  → store.createBlock(messageId, type)
onChunk:  → 忽略（检索通常不流式）
onEnd:    → store.updateBlock(blockId, { toolOutput: result }) + updateBlockStatus(blockId, 'success')
onError:  → store.setBlockError(blockId, error)
```

---

## 10. Step 9: Store 状态更新与 React 渲染

### 10a. chunk 缓冲优化

**文件**: `C:/deep-student/src/features/chat/core/middleware/chunkBuffer.ts`

`content` 和 `thinking` 的 `chunk` 事件不直接更新 Store，而是通过 `ChunkBufferImpl`：

```typescript
// eventBridge.ts line 1031-1048
if ((type === 'content' || type === 'thinking') && chunk) {
  chunkBuffer.setStore(store);
  chunkBuffer.push(effectiveBlockId, chunk, store.sessionId);
  // streamingBlockSaver 定期保存到后端
}
```

ChunkBuffer 按会话分组，收集 ~16ms 窗口内的 chunk 合并后一次更新 Store：
- `push(blockId, chunk, sessionId)` → 追加到缓冲区
- 定时器触发 `flush()` → 合并缓冲区内容 → `store.updateBlockContent(blockId, mergedChunk)`
- `flushSession(sessionId)` → 强制刷新

### 10b. Store 更新方法

`ChatStore` (Zustand) 提供以下 actions（由 event handlers 调用）:

| Store Action | 效果 |
|-------------|------|
| `createBlock(messageId, blockType)` | 创建新块，添加到 `message.blockIds` |
| `createBlockWithId(messageId, blockType, blockId)` | 使用预定义 ID 创建块 |
| `updateBlockContent(blockId, chunk)` | 追加内容，状态 → `running` |
| `updateBlockStatus(blockId, status)` | 设置状态 (`success`/`error`)，从 `activeBlockIds` 移除 |
| `setBlockError(blockId, error)` | 设置错误状态 |
| `updateBlock(blockId, partial)` | 直接更新块的部分字段 |
| `completeStream(status)` | 重置 `sessionStatus` → `idle` |
| `updateMessageMeta(messageId, meta)` | 更新消息元数据（usage, modelId, etc.） |

### 10c. React 渲染

Store 变更驱动 React 组件渲染 `MessageList` → `MessageItem` → `Block` 组件链。

块类型到渲染组件的映射在 `BLOCK_RENDERING_GUIDE.md` 中定义:
- `content` → 文本渲染器
- `thinking` → 可折叠思维链渲染器
- `rag`/`memory`/`web_search` → 来源引用卡
- `mcp_tool` → 工具调用结果渲染器
- `anki_cards` → Anki 制卡渲染器

---

## 11. 关键数据结构对照表

| Rust Struct | TypeScript Interface | 序列化格式 | 一致性 |
|-------------|---------------------|-----------|--------|
| `BackendEvent` | `BackendEvent` | camelCase | ✅ |
| `SessionEvent` | `SessionEventPayload` | camelCase | ✅ |
| `TokenUsage` | `TokenUsage` | camelCase | ✅ |
| `StreamEvent` | (无直接映射, 内部 Rust) | — | ✅ (不跨 FFI) |
| `SendMessageRequest` | `SendMessageRequest` | camelCase | ✅ |
| `ContentBlock` | (无直接映射, 中间 Rust 内部) | — | ✅ |
| `ToolCall` (Rust) | `ToolCall` (TS) | camelCase | ✅ |
| `ChatMessage` (Rust) | `ChatMessage` (TS stores) | camelCase | ✅ |

---

## 12. 类型一致性验证

### 12a. BackendEvent Rust → TypeScript

Rust (events.rs line 136):
```rust
#[serde(rename_all = "camelCase")]
pub struct BackendEvent {
    pub sequence_id: u64,          // → sequenceId?: number
    pub session_id: Option<String>,  // → sessionId?: string
    pub r#type: String,             // → type: string
    pub phase: String,              // → phase: EventPhase ('start'|'chunk'|'end'|'error')
    pub message_id: Option<String>,  // → messageId?: string
    pub block_id: Option<String>,    // → blockId?: string
    pub chunk: Option<String>,       // → chunk?: string
    pub result: Option<Value>,       // → result?: unknown
    pub error: Option<String>,       // → error?: string
    pub variant_id: Option<String>,  // → variantId?: string
    pub model_id: Option<String>,    // → modelId?: string
    pub usage: Option<TokenUsage>,   // → usage?: TokenUsage
}
```

TypeScript (eventBridge.ts line 33):
```typescript
export interface BackendEvent {
  sequenceId?: number;
  sessionId?: string;
  type: string;
  phase: EventPhase;
  messageId?: string;
  blockId?: string;
  chunk?: string;
  result?: unknown;
  error?: string;
  variantId?: string;
  modelId?: string;
  usage?: TokenUsage;
}
```

**差异**: Rust 中 `sequence_id` 是 `u64` 非 Optional，前端为 `number` 可选。其他字段一致。

### 12b. SessionEvent Rust → TypeScript

Rust (events.rs line 430):
```rust
#[serde(rename_all = "camelCase")]
pub struct SessionEvent {
    pub session_id: String,         // → sessionId: string
    pub event_type: String,         // → eventType: string
    pub message_id: Option<String>,  // → messageId?: string
    pub model_id: Option<String>,    // → modelId?: string
    pub error: Option<String>,       // → error?: string
    pub duration_ms: Option<u64>,    // → durationMs?: number
    pub timestamp: i64,             // → timestamp: number
    pub usage: Option<TokenUsage>,   // → usage?: TokenUsage
}
```

TypeScript `SessionEventPayload` 在 `types.ts` 中定义:
```typescript
interface SessionEventPayload {
  sessionId: string;
  eventType: string;
  messageId?: string;
  modelId?: string;
  error?: string;
  durationMs?: number;
  timestamp: number;
  usage?: TokenUsage;
}
```

**一致性**: 所有字段名和类型匹配 ✅

---

## 13. 错误处理覆盖分析

### 13a. 后端错误路径

| 错误场景 | 处理方式 | 最终效果 |
|---------|---------|---------|
| LLM API 网络错误 | `pipeline.execute()` 捕获 `ChatV2Error` | `emit_stream_error` + save partial + 前端显示错误 |
| 用户取消 | `CancellationToken` → `ChatV2Error::Cancelled` | `emit_stream_cancelled` + save partial |
| 工具调用失败 | `execute_single_tool` → emit `tool_call→error` | 前端更新块状态为 `error` |
| Provider 适配器解析失败 | `parse_stream()` 返回空 Vec | 静默忽略，继续下一行 |
| JSON 解析失败 | `serde_json::from_str` Err | 跳过该行 |
| SSE 流中断 | 循环退出 → stream_ended = true | 最终 message_stop 或超时处理 |

### 13b. 前端错误路径

| 错误场景 | 处理方式 |
|---------|---------|
| 序列号不连续 | Gap 超时后跳过丢失序号，按序处理缓冲事件 |
| 重复事件 | `processedEventIds` 去重 |
| 首包非 start | 缓冲等待 start |
| 无 handler 的事件类型 | 打印 warning，忽略 |
| Stale stream (过期消息) | `isStaleByExpectationTimestamp` / `isTargetingCurrentStreamMessage` 过滤 |
| Skill 版本不匹配 | `shouldDropEventBySkillVersion` 丢弃过期事件 |
| 监听器注册失败 | 重试机制 (`retrySetupListeners`), 用户通知 |
| 块启动失败 (无 messageId) | 打印 error，跳过 |
| Chunk 无 blockId | 从 `context.blockIdMap` 查找，找不到则 warning 跳过 |
| 流式错误 | `stream_error` 事件 → 显示全局通知 |
| 会话加载失败 | 降级为新会话，标记 `isDataLoaded=true` |

### 13c. 防数据丢失机制

1. **P0 防闪退**: 用户消息在 Pipeline 开始时立即保存 (`save_user_message_immediately`)。
2. **Partial save on cancel**: 取消时从 adapter 获取已累积内容并保存。
3. **Partial save on error**: 错误时同样保存已累积内容。
4. **streamingBlockSaver**: 前端定期保存流式块内容到后端，防止浏览器崩溃丢失。
5. **autoSave**: 事件处理后调度自动保存，有节流和并发控制。
6. **Cleanup on cleanup()**: 组件卸载时防御性清理所有中间件资源。

---

## 附录: 涉及文件清单

| 文件 | 角色 |
|------|------|
| `src-tauri/src/chat_v2/handlers/send_message.rs` | 消息发送命令入口 |
| `src-tauri/src/chat_v2/pipeline.rs` | 流水线编排引擎 |
| `src-tauri/src/chat_v2/pipeline/tool_loop.rs` | LLM 调用 + 工具递归循环 |
| `src-tauri/src/chat_v2/pipeline/llm_adapter.rs` | LLMStreamHooks 实现，事件转换 |
| `src-tauri/src/chat_v2/events.rs` | BackendEvent/SessionEvent 定义与发射 |
| `src-tauri/src/providers/mod.rs` | StreamEvent 枚举 + 各 Provider parse_stream |
| `src-tauri/src/llm_manager/model2_pipeline.rs` | SSE 流式循环，StreamEvent→hook 调用 |
| `src-tauri/src/llm_manager/streaming.rs` | LLMStreamHooks trait 定义 |
| `src/features/chat/adapters/TauriAdapter.ts` | 前端事件监听 + Tauri 命令调用 |
| `src/features/chat/core/middleware/eventBridge.ts` | 事件桥接、序列号检测、分发 |
| `src/features/chat/core/middleware/chunkBuffer.ts` | 流式 chunk 缓冲优化 |
| `src/features/chat/registry/eventRegistry.ts` | 事件处理器注册表 |
| `src/features/chat/plugins/events/content.ts` | content 事件处理器 |
| `src/features/chat/plugins/events/thinking.ts` | thinking 事件处理器 |
| `src/features/chat/plugins/events/retrieval.ts` | 检索事件处理器 (rag/memory/web_search) |
| `src/features/chat/plugins/events/toolCall.ts` | 工具调用事件处理器 |

---

## 附录: 数据流速查 (最小完整路径)

```
用户输入 → invoke('chat_v2_send_message', msg)
  → handlers/send_message.rs::chat_v2_send_message()
    → ChatV2Pipeline::execute()
      → execute_internal()
        → execute_with_tools()
          → call_llm_with_adapter()
            → llm_manager.call_unified_model_2_stream(body)
              → [HTTP SSE → adapter.parse_stream(line)]
                → [for each StreamEvent → LLMStreamHooks callback]
                  → ChatV2LLMAdapter::on_content_chunk(text)
                    → emit_chunk('content', block_id, text)
                      → window.emit('chat_v2_event_{sid}', BackendEvent)
                        → [Tauri IPC → frontend]
                          → listen<BackendEvent>(channel)
                            → handleBlockEvent()
                              → handleBackendEventWithSequence()
                                → processEventInternal()
                                  → eventRegistry.get('content')
                                    → contentEventHandler.onChunk()
                                      → chunkBuffer.push()
                                        → store.updateBlockContent()
                                          → React re-render
```
