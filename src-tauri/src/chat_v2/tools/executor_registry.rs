//! 工具执行器注册表
//!
//! 管理所有已注册的工具执行器，提供统一的执行入口。
//!
//! ## 设计文档
//! 参考：`src/chat-v2/docs/29-ChatV2-Agent能力增强改造方案.md` 第 2.3.3 节

use std::sync::Arc;
use tokio::time::{timeout, Duration};

use super::executor::{ExecutionContext, ToolError, ToolExecutor, ToolResult, ToolSensitivity};
use crate::chat_v2::types::{ToolCall, ToolResultInfo};
use crate::chat_v2::workspace::WorkspaceCoordinator;

// 工具执行器类型 — 仅在工厂方法中引入，避免 Pipeline 依赖具体执行器
use super::{
    AcademicSearchExecutor, AskUserExecutor, AttemptCompletionExecutor,
    AttachmentToolExecutor, BuiltinResourceExecutor, BuiltinRetrievalExecutor,
    CanvasToolExecutor, ChatAnkiToolExecutor, CoordinatorSleepExecutor,
    DocxToolExecutor, FetchExecutor, GeneralToolExecutor, ImageGenerationExecutor,
    KnowledgeExecutor, MemoryToolExecutor, PaperSaveExecutor, PptxToolExecutor,
    QBankExecutor, SessionToolExecutor, SkillsExecutor, SubagentExecutor,
    TemplateDesignerExecutor, TodoListExecutor, UserTodoExecutor,
    WorkspaceToolExecutor, XlsxToolExecutor,
};

// ============================================================================
// 已弃用工具注册表
// ============================================================================

/// 已弃用工具名到其替代工具的映射
///
/// 当软件更新后旧工具被重命名或移除，此映射用于：
/// 1. 在 get_executor 中给出友好的错误提示
/// 2. 在 execute 中提供替代方案建议
/// 3. 日志中记录清晰警告
const DEPRECATED_TOOL_MAP: &[(&str, &str)] = &[
    // CardForge 2.0 => ChatAnki 3.0
    ("builtin-anki_generate_cards", "builtin-chatanki_run"),
    ("builtin-anki_control_task", "builtin-chatanki_control"),
    ("builtin-anki_export_cards", "builtin-chatanki_export"),
    ("builtin-anki_list_templates", "builtin-chatanki_list_templates"),
    ("builtin-anki_analyze_content", "builtin-chatanki_analyze"),
    ("builtin-anki_query_progress", "builtin-chatanki_status"),
    // 无前缀的兼容格式
    ("anki_generate_cards", "builtin-chatanki_run"),
    ("anki_control_task", "builtin-chatanki_control"),
    ("anki_export_cards", "builtin-chatanki_export"),
    ("anki_list_templates", "builtin-chatanki_list_templates"),
    ("anki_analyze_content", "builtin-chatanki_analyze"),
    ("anki_query_progress", "builtin-chatanki_status"),
];

/// 检查是否为已弃用的工具
pub fn is_deprecated_tool(tool_name: &str) -> bool {
    DEPRECATED_TOOL_MAP.iter().any(|(old, _)| *old == tool_name)
        || tool_name.strip_prefix("builtin-").unwrap_or(tool_name).starts_with("anki_")
}

/// 获取已弃用工具的替代工具名
pub fn get_deprecated_tool_replacement(tool_name: &str) -> Option<&'static str> {
    DEPRECATED_TOOL_MAP
        .iter()
        .find(|(old, _)| *old == tool_name)
        .map(|(_, replacement)| *replacement)
}

// ============================================================================
// 全局超时配置
// ============================================================================

/// 默认工具执行超时时间（秒）
const DEFAULT_TOOL_TIMEOUT_SECS: u64 = 120;
const NO_TOOL_TIMEOUT_SECS: u64 = 0;

/// 获取工具特定的超时时间（秒）
///
/// 某些工具可能需要更长的执行时间，在此处配置特例。
///
/// ## 工具命名规范
/// - 内置工具使用 `builtin-` 前缀，如 `builtin-rag_search`、`builtin-web_search`
/// - MCP 工具使用 `mcp_` 前缀，如 `mcp_brave_search`
fn get_tool_timeout_secs(tool_name: &str) -> u64 {
    // 去掉 builtin- 前缀用于统一匹配
    let stripped = tool_name.strip_prefix("builtin-").unwrap_or(tool_name);

    if stripped == "ask_user" {
        return NO_TOOL_TIMEOUT_SECS;
    }

    // 精确匹配：内置检索和搜索工具
    match tool_name {
        // 网络搜索工具（需要较长时间）
        "builtin-web_search" => 180, // 3 分钟
        // 学术论文搜索工具（arXiv / OpenAlex API）
        "builtin-arxiv_search" | "builtin-scholar_search" => 180, // 3 分钟
        // 论文保存工具（下载 PDF + VFS 存储，批量最多 5 篇）
        "builtin-paper_save" => 600, // 10 分钟（批量下载+处理）
        // 引用格式化工具（纯计算，无网络）
        "builtin-cite_format" => 30, // 30 秒
        // 网络请求和 HTML 解析工具（涉及网络请求和 HTML 解析）
        "builtin-web_fetch" => 180, // 3 分钟
        // RAG 检索工具（可能涉及大量数据）
        "builtin-rag_search" | "builtin-multimodal_search" | "builtin-unified_search" => 180, // 3 分钟
        // 文档写入/转换工具（大文件处理可能耗时较长）
        "builtin-docx_create"
        | "builtin-pptx_create"
        | "builtin-xlsx_create"
        | "builtin-docx_to_spec"
        | "builtin-pptx_to_spec"
        | "builtin-xlsx_to_spec"
        | "builtin-docx_replace_text"
        | "builtin-pptx_replace_text"
        | "builtin-xlsx_replace_text" => 300, // 5 分钟
        // 子代理调用工具（可能执行复杂任务）
        "subagent_call" => 300, // 5 分钟
        _ => {
            // ChatAnki 工具：chatanki_wait 内部有 30 分钟超时，外层需匹配
            if stripped == "chatanki_wait" {
                35 * 60 // 35 分钟（比内部 30 分钟稍长，避免竞态）
            } else if stripped.starts_with("chatanki_") {
                600 // 10 分钟（chatanki_run/start/export/sync 可能涉及大量 IO）
            } else if stripped == "image_generate" {
                300 // 5 分钟（第三方生图 API 可能排队）
            } else if tool_name.starts_with("mcp_") {
                // 前缀匹配：MCP 工具通常需要网络请求
                180 // 3 分钟
            } else {
                DEFAULT_TOOL_TIMEOUT_SECS
            }
        }
    }
}

// ============================================================================
// 执行器注册表
// ============================================================================

/// 工具执行器注册表
///
/// 管理多个工具执行器，按注册顺序遍历查找能处理指定工具的执行器。
pub struct ToolExecutorRegistry {
    /// 已注册的执行器列表（按注册顺序）
    executors: Vec<Arc<dyn ToolExecutor>>,
}

impl ToolExecutorRegistry {
    /// 创建空的注册表
    pub fn new() -> Self {
        Self {
            executors: Vec::new(),
        }
    }

    /// 注册执行器
    ///
    /// ## 参数
    /// - `executor`: 要注册的执行器
    ///
    /// ## 注意
    /// 执行器的注册顺序决定了查找顺序，先注册的优先匹配。
    pub fn register(&mut self, executor: Arc<dyn ToolExecutor>) {
        log::debug!(
            "[ToolExecutorRegistry] Registering executor: {}",
            executor.name()
        );
        self.executors.push(executor);
    }

    /// 获取能处理指定工具的执行器
    ///
    /// ## 参数
    /// - `tool_name`: 工具名称
    ///
    /// ## 返回
    /// - `Some(executor)`: 找到的执行器
    /// - `None`: 没有执行器能处理此工具
    ///
    /// ## 已弃用工具检测
    /// 如果工具名已知已弃用，会记录警告日志并提示替代工具。
    pub fn get_executor(&self, tool_name: &str) -> Option<Arc<dyn ToolExecutor>> {
        for executor in &self.executors {
            if executor.can_handle(tool_name) {
                return Some(executor.clone());
            }
        }

        // 未找到执行器，检查是否为已弃用工具
        if is_deprecated_tool(tool_name) {
            if let Some(replacement) = get_deprecated_tool_replacement(tool_name) {
                log::warn!(
                    "[ToolExecutorRegistry] Deprecated tool '{}' is no longer registered. \
                     Its replacement is '{}'. Old sessions referencing this tool will still \
                     display historical data via the 'deprecated_tool' block type.",
                    tool_name,
                    replacement,
                );
            } else {
                log::warn!(
                    "[ToolExecutorRegistry] Deprecated tool '{}' is no longer registered. \
                     No direct replacement found. Old sessions referencing this tool will still \
                     display historical data via the 'deprecated_tool' block type.",
                    tool_name,
                );
            }
        }

        None
    }

    /// 执行工具调用
    ///
    /// 遍历所有执行器，找到能处理的执行器并执行。
    ///
    /// ## 参数
    /// - `call`: 工具调用信息
    /// - `ctx`: 执行上下文（包含可选的取消令牌）
    ///
    /// ## 返回
    /// - `Ok(ToolResultInfo)`: 执行结果
    /// - `Err`: 没有执行器能处理、执行异常、超时或取消
    ///
    /// ## 超时保护
    /// 每个工具调用都有全局超时保护，防止 Pipeline 因单个工具执行卡死。
    /// 默认超时为 120 秒，某些特殊工具（如网络请求、代码执行）有更长的超时时间。
    /// `ask_user` 例外：它表示显式等待用户交互，不应被通用工具 watchdog 截断。
    ///
    /// ## 🆕 取消支持（2026-02）
    /// 如果 `ctx.cancellation_token` 存在，执行会在取消时提前终止。
    /// 取消优先级高于超时，可以立即响应用户取消请求。
    pub async fn execute(
        &self,
        call: &ToolCall,
        ctx: &ExecutionContext,
    ) -> ToolResult<ToolResultInfo> {
        // 🆕 取消检查：在执行前检查是否已取消
        if ctx.is_cancelled() {
            log::info!(
                "[ToolExecutorRegistry] Tool execution cancelled before start: {} (id={})",
                call.name,
                call.id
            );
            return Err(ToolError::Cancelled);
        }

        // 查找能处理的执行器
        let executor = self.get_executor(&call.name).ok_or_else(|| {
            // 增强已弃用工具的报错信息，提供替代工具建议
            if let Some(replacement) = get_deprecated_tool_replacement(&call.name) {
                ToolError::NotFound(format!(
                    "工具 '{name}' 已不再可用（已弃用），请使用替代工具 '{replacement}'。历史数据已保留。",
                    name = call.name,
                    replacement = replacement,
                ))
            } else if is_deprecated_tool(&call.name) {
                ToolError::NotFound(format!(
                    "工具 '{name}' 已不再可用（已弃用），且无直接替代工具。历史数据已保留。",
                    name = call.name,
                ))
            } else {
                ToolError::NotFound(format!("未找到可处理工具 '{name}' 的执行器", name = call.name))
            }
        })?;

        log::debug!(
            "[ToolExecutorRegistry] Executing tool '{}' with executor '{}'",
            call.name,
            executor.name()
        );

        // 🆕 P1 修复：获取工具特定的超时时间并添加超时保护
        let timeout_secs = get_tool_timeout_secs(&call.name);
        // 执行工具（带超时和取消保护）
        // 🆕 取消支持：使用 tokio::select! 同时监听取消信号
        let execute_future = executor.execute(call, ctx);

        if timeout_secs == NO_TOOL_TIMEOUT_SECS {
            log::debug!(
                "[ToolExecutorRegistry] Tool '{}' timeout disabled",
                call.name,
            );

            if let Some(cancel_token) = ctx.cancellation_token() {
                tokio::select! {
                    result = execute_future => result,
                    _ = cancel_token.cancelled() => {
                        log::info!(
                            "[ToolExecutorRegistry] Tool execution cancelled: {} (id={})",
                            call.name,
                            call.id
                        );
                        Err(ToolError::Cancelled)
                    }
                }
            } else {
                execute_future.await
            }
        } else {
            let timeout_duration = Duration::from_secs(timeout_secs);

            log::debug!(
                "[ToolExecutorRegistry] Tool '{}' timeout set to {}s",
                call.name,
                timeout_secs
            );

            let timeout_future = timeout(timeout_duration, execute_future);

            if let Some(cancel_token) = ctx.cancellation_token() {
                tokio::select! {
                    result = timeout_future => {
                        match result {
                            Ok(inner_result) => inner_result,
                            Err(_elapsed) => {
                                log::error!(
                                    "[ToolExecutorRegistry] Tool execution timeout after {}s: {} (id={})",
                                    timeout_secs,
                                    call.name,
                                    call.id
                                );
                                Err(ToolError::Timeout(format!(
                                    "Tool '{}' execution timed out after {}s",
                                    call.name, timeout_secs
                                )))
                            }
                        }
                    }
                    _ = cancel_token.cancelled() => {
                        log::info!(
                            "[ToolExecutorRegistry] Tool execution cancelled: {} (id={})",
                            call.name,
                            call.id
                        );
                        Err(ToolError::Cancelled)
                    }
                }
            } else {
                match timeout_future.await {
                    Ok(result) => result,
                    Err(_elapsed) => {
                        log::error!(
                            "[ToolExecutorRegistry] Tool execution timeout after {}s: {} (id={})",
                            timeout_secs,
                            call.name,
                            call.id
                        );
                        Err(ToolError::Timeout(format!(
                            "Tool '{}' execution timed out after {}s",
                            call.name, timeout_secs
                        )))
                    }
                }
            }
        }
    }

    /// 获取工具敏感等级
    ///
    /// ## 参数
    /// - `tool_name`: 工具名称
    ///
    /// ## 返回
    /// - `Some(sensitivity)`: 工具敏感等级
    /// - `None`: 没有执行器能处理此工具
    pub fn get_sensitivity(&self, tool_name: &str) -> Option<ToolSensitivity> {
        self.get_executor(tool_name)
            .map(|e| e.sensitivity_level(tool_name))
    }

    /// 检查是否有执行器能处理指定工具
    pub fn can_handle(&self, tool_name: &str) -> bool {
        self.get_executor(tool_name).is_some()
    }

    /// 获取已注册的执行器数量
    pub fn len(&self) -> usize {
        self.executors.len()
    }

    /// 检查注册表是否为空
    pub fn is_empty(&self) -> bool {
        self.executors.is_empty()
    }

    /// 获取所有执行器名称（用于调试）
    pub fn executor_names(&self) -> Vec<&'static str> {
        self.executors.iter().map(|e| e.name()).collect()
    }

    // ====================================================================
    // 工厂方法 — 创建预配置的注册表（替代 Pipeline 中的直接实例化）
    // ====================================================================

    /// 创建默认注册表（不含工作区执行器）
    ///
    /// 包含所有非工作区内置工具执行器，按注册顺序排列。
    /// `GeneralToolExecutor` 始终最后注册，充当兜底 fallback。
    pub fn create_default() -> Self {
        Self::create_inner(None)
    }

    /// 创建含工作区执行器的注册表
    ///
    /// 在默认注册表基础上，额外注册工作区相关执行器：
    /// - `WorkspaceToolExecutor`
    /// - `SubagentExecutor`
    /// - `CoordinatorSleepExecutor`
    pub fn create_with_workspace(coordinator: Arc<WorkspaceCoordinator>) -> Self {
        Self::create_inner(Some(coordinator))
    }

    /// 内部工厂方法 — 保持与原 `create_executor_registry_with_workspace` 完全一致
    fn create_inner(coordinator: Option<Arc<WorkspaceCoordinator>>) -> Self {
        let mut registry = Self::new();

        registry.register(Arc::new(AttemptCompletionExecutor::new()));
        registry.register(Arc::new(CanvasToolExecutor::new()));
        // AnkiToolExecutor 已移除 — 旧 CardForge 2.0 管线由 ChatAnki 完全接管
        registry.register(Arc::new(ChatAnkiToolExecutor::new()));
        registry.register(Arc::new(BuiltinRetrievalExecutor::new()));
        registry.register(Arc::new(BuiltinResourceExecutor::new()));
        registry.register(Arc::new(AttachmentToolExecutor::new())); // 🆕 附件工具执行器（解决 P0 断裂点）
        registry.register(Arc::new(FetchExecutor::new())); // 🆕 内置 Web Fetch 工具
        registry.register(Arc::new(AcademicSearchExecutor::new())); // 🆕 学术论文搜索工具（arXiv + OpenAlex）
        registry.register(Arc::new(PaperSaveExecutor::new())); // 🆕 论文保存+引用格式化工具
        registry.register(Arc::new(KnowledgeExecutor::new()));
        registry.register(Arc::new(TodoListExecutor::new()));
        registry.register(Arc::new(QBankExecutor::new()));
        registry.register(Arc::new(MemoryToolExecutor::new()));
        registry.register(Arc::new(UserTodoExecutor::new()));
        registry.register(Arc::new(SkillsExecutor::new())); // 🆕 Skills 工具执行器（渐进披露架构）
        registry.register(Arc::new(TemplateDesignerExecutor::new())); // 🆕 模板设计师工具执行器
        registry.register(Arc::new(AskUserExecutor::new())); // 🆕 用户提问工具执行器
        registry.register(Arc::new(SessionToolExecutor::new())); // 🆕 会话管理工具执行器
        registry.register(Arc::new(DocxToolExecutor::new())); // 🆕 DOCX 文档读写工具执行器
        registry.register(Arc::new(PptxToolExecutor::new())); // 🆕 PPTX 演示文稿读写工具执行器
        registry.register(Arc::new(XlsxToolExecutor::new())); // 🆕 XLSX 电子表格读写工具执行器
        registry.register(Arc::new(ImageGenerationExecutor::new())); // 🆕 内置图片生成工具执行器

        if let Some(coordinator) = coordinator {
            registry.register(Arc::new(WorkspaceToolExecutor::new(coordinator.clone())));
            // 注册 SubagentExecutor（subagent_call 语法糖）
            registry.register(Arc::new(SubagentExecutor::new(coordinator.clone())));
            // 🆕 注册 CoordinatorSleepExecutor（主代理睡眠/唤醒机制）
            registry.register(Arc::new(CoordinatorSleepExecutor::new(coordinator)));
        }

        registry.register(Arc::new(GeneralToolExecutor::new()));

        log::info!(
            "[ChatV2::pipeline] ToolExecutorRegistry initialized with {} executors: {:?}",
            registry.len(),
            registry.executor_names()
        );

        registry
    }
}

impl Default for ToolExecutorRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// 单元测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;

    /// 测试用执行器
    struct TestExecutor {
        name: &'static str,
        handles: Vec<String>,
    }

    #[async_trait]
    impl ToolExecutor for TestExecutor {
        fn can_handle(&self, tool_name: &str) -> bool {
            self.handles.contains(&tool_name.to_string())
        }

        async fn execute(
            &self,
            call: &ToolCall,
            _ctx: &ExecutionContext,
        ) -> ToolResult<ToolResultInfo> {
            Ok(ToolResultInfo::success(
                Some(call.id.clone()),
                Some("test_block".to_string()),
                call.name.clone(),
                call.arguments.clone(),
                serde_json::json!({"executed_by": self.name}),
                10,
            ))
        }

        fn name(&self) -> &'static str {
            self.name
        }
    }

    #[test]
    fn test_registry_creation() {
        let registry = ToolExecutorRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn test_register_executor() {
        let mut registry = ToolExecutorRegistry::new();
        let executor = Arc::new(TestExecutor {
            name: "test",
            handles: vec!["tool_a".to_string()],
        });
        registry.register(executor);
        assert_eq!(registry.len(), 1);
        assert!(registry.can_handle("tool_a"));
        assert!(!registry.can_handle("tool_b"));
    }

    #[test]
    fn test_executor_priority() {
        let mut registry = ToolExecutorRegistry::new();

        // 第一个执行器处理 tool_a
        let executor1 = Arc::new(TestExecutor {
            name: "executor1",
            handles: vec!["tool_a".to_string()],
        });
        registry.register(executor1);

        // 第二个执行器也处理 tool_a
        let executor2 = Arc::new(TestExecutor {
            name: "executor2",
            handles: vec!["tool_a".to_string()],
        });
        registry.register(executor2);

        // 应该返回第一个注册的执行器
        let found = registry.get_executor("tool_a").unwrap();
        assert_eq!(found.name(), "executor1");
    }

    #[test]
    fn test_get_sensitivity() {
        let mut registry = ToolExecutorRegistry::new();
        let executor = Arc::new(TestExecutor {
            name: "test",
            handles: vec!["tool_a".to_string()],
        });
        registry.register(executor);

        // 默认敏感等级是 Low
        assert_eq!(
            registry.get_sensitivity("tool_a"),
            Some(ToolSensitivity::Low)
        );
        assert_eq!(registry.get_sensitivity("unknown_tool"), None);
    }

    #[test]
    fn image_generation_tool_uses_five_minute_timeout() {
        assert_eq!(get_tool_timeout_secs("builtin-image_generate"), 300);
        assert_eq!(get_tool_timeout_secs("image_generate"), 300);
    }

    #[test]
    fn ask_user_tool_is_not_subject_to_global_timeout() {
        assert_eq!(get_tool_timeout_secs("builtin-ask_user"), 0);
        assert_eq!(get_tool_timeout_secs("ask_user"), 0);
    }
}
