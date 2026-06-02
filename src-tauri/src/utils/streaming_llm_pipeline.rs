/// 共享流式 LLM 管线
///
/// 从 essay_grading, translation, qbank_grading 三个管线中提取的公共流式 LLM 调用骨架。
/// 处理：
/// - ProviderAdapter HTTP 请求构建
/// - SSE/stream 块解析
/// - 取消支持（LLMManager cancel registry）
/// - 进度事件回调
use futures_util::StreamExt;
use serde_json::json;
use std::sync::Arc;

use crate::llm_manager::{build_provider_adapter, ApiConfig, LLMManager};
use crate::models::AppError;

/// 流式 LLM 调用的结果状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamStatus {
    Completed,
    Cancelled,
    /// 流结束但未收到 DONE 标记（网络中断 / 服务端异常）。
    /// 累积的文本可能部分可用但不保证完整。
    Incomplete,
}

/// 共享流式 LLM 管线
///
/// 用法：调用方自行构造 messages 数组（支持纯文本或图文混合），
/// 传入此管线处理 HTTP 请求、SSE 解析和取消。
pub struct StreamingLLMPipeline;

impl StreamingLLMPipeline {
    /// 执行流式 LLM 调用
    ///
    /// # Parameters
    /// - `config`: API 供应商配置
    /// - `api_key`: 已解密的 API 密钥
    /// - `messages`: 已构造的 messages 数组（system + user content）
    /// - `stream_event`: 唯一事件名，用于取消协调
    /// - `llm`: LLMManager 引用（提供 HTTP 客户端和取消注册表）
    /// - `temperature`: 模型 temperature 参数
    /// - `max_tokens`: 最大输出 token 数（调用方通过 `effective_max_tokens` 计算）
    /// - `task_name`: 错误消息前缀（如 "批改", "翻译", "评判"）
    /// - `error_handler`: 可选的 HTTP 状态码 → 用户友好错误消息映射函数。
    ///   如果 handler 返回 `Some(msg)` 则使用该消息；返回 `None` 则使用默认格式。
    /// - `on_chunk`: 每个内容块到达时的回调
    pub async fn stream<F>(
        config: &ApiConfig,
        api_key: &str,
        messages: Vec<serde_json::Value>,
        stream_event: &str,
        llm: Arc<LLMManager>,
        temperature: f64,
        max_tokens: u32,
        task_name: &str,
        error_handler: Option<fn(u16) -> Option<&'static str>>,
        mut on_chunk: F,
    ) -> Result<StreamStatus, AppError>
    where
        F: FnMut(String),
    {
        let result = async {
            // --- 构造请求体 ---
            let mut request_body = json!({
                "model": config.model,
                "messages": messages,
                "temperature": temperature,
                "max_tokens": max_tokens,
                "stream": true,
            });

            crate::llm_manager::LLMManager::apply_reasoning_config(&mut request_body, config, None);

            // --- 选择适配器 ---
            let adapter: Box<dyn crate::providers::ProviderAdapter> =
                build_provider_adapter(config);

            // --- 构造 HTTP 请求 ---
            let preq = adapter
                .build_request(&config.base_url, api_key, &config.model, &request_body)
                .map_err(|e| {
                    AppError::llm(format!("{}请求构建失败: {}", task_name, e))
                })?;

            let mut header_map = reqwest::header::HeaderMap::new();
            for (k, v) in preq.headers.iter() {
                if let (Ok(name), Ok(val)) = (
                    reqwest::header::HeaderName::from_bytes(k.as_bytes()),
                    reqwest::header::HeaderValue::from_str(v),
                ) {
                    header_map.insert(name, val);
                }
            }

            // --- 复用 LLMManager 的 HTTP 客户端 ---
            let client = llm.get_http_client();

            // --- 注册取消监听 ---
            llm.consume_pending_cancel(stream_event).await;
            let mut cancel_rx = llm.subscribe_cancel_stream(stream_event).await;

            // --- 发送流式请求 ---
            let response = client
                .post(&preq.url)
                .headers(header_map)
                .json(&preq.body)
                .send()
                .await
                .map_err(|e| AppError::llm(format!("{}请求失败: {}", task_name, e)))?;

            // --- 检查响应状态 ---
            if !response.status().is_success() {
                let status = response.status();
                let error_text = response.text().await.unwrap_or_default();

                // 优先尝试自定义错误处理
                if let Some(handler) = error_handler {
                    if let Some(msg) = handler(status.as_u16()) {
                        return Err(AppError::llm(msg.to_string()));
                    }
                }

                // 默认错误格式
                return Err(AppError::llm(format!(
                    "{} API 返回错误 {}: {}",
                    task_name, status, error_text
                )));
            }

            // --- 解析 SSE 流 ---
            let mut stream = response.bytes_stream();
            let mut buffer = String::new();
            let mut stream_ended = false;
            let mut cancelled = false;

            while !stream_ended && !cancelled {
                if llm.consume_pending_cancel(stream_event).await {
                    cancelled = true;
                    break;
                }

                tokio::select! {
                    changed = cancel_rx.changed() => {
                        if changed.is_ok() && *cancel_rx.borrow() {
                            cancelled = true;
                        }
                    }
                    chunk_result = stream.next() => {
                        match chunk_result {
                            Some(chunk) => {
                                let bytes = chunk
                                    .map_err(|e| AppError::llm(format!("读取流失败: {}", e)))?;
                                buffer.push_str(&String::from_utf8_lossy(&bytes));

                                while let Some(pos) = buffer.find("\n\n") {
                                    let line = buffer[..pos].trim().to_string();
                                    buffer = buffer[pos + 2..].to_string();

                                    if line.is_empty() {
                                        continue;
                                    }

                                    if line == "data: [DONE]" {
                                        stream_ended = true;
                                        break;
                                    }

                                    let events = adapter.parse_stream(&line);
                                    for event in events {
                                        match event {
                                            crate::providers::StreamEvent::ContentChunk(content) => {
                                                on_chunk(content);
                                            }
                                            crate::providers::StreamEvent::Done => {
                                                stream_ended = true;
                                                break;
                                            }
                                            _ => {}
                                        }
                                    }

                                    if stream_ended {
                                        break;
                                    }
                                }
                            }
                            None => {
                                break;
                            }
                        }
                    }
                }
            }

            if cancelled {
                return Ok(StreamStatus::Cancelled);
            }

            if stream_ended {
                Ok(StreamStatus::Completed)
            } else {
                println!(
                    "⚠️ [{}] SSE 流未收到 DONE 标记就结束，结果可能不完整",
                    task_name
                );
                Ok(StreamStatus::Incomplete)
            }
        }
        .await;

        llm.clear_cancel_stream(stream_event).await;

        result
    }
}
