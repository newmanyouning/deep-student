/// 翻译模块 - 独立流式管线
///
/// 职责：
/// - 提供流式翻译命令
/// - 管理翻译会话状态
/// - 发送 SSE 事件到前端
///
/// 与 unified_chat 的关系：
/// - 完全独立的管线，不依赖 unified_chat 的类型或逻辑
/// - 仅复用 LLMManager 的底层能力
pub mod chat_popover;
pub mod events;
pub mod pipeline;
pub mod types;

use serde::Deserialize;
use tauri::{State, Window};

use crate::models::AppError;
use events::TranslationEventEmitter;
use types::{TranslationRequest, TranslationResponse};

/// 流式翻译命令
///
/// # 参数
/// - `request`: 翻译请求（包含源文本、语言对、提示词等）
/// - `window`: Tauri 窗口句柄（用于发送 SSE 事件）
/// - `state`: 应用状态（访问 LLMManager、Database 等）
///
/// # 事件流
/// 1. `translation_stream_data`: 增量译文片段
/// 2. `translation_stream_complete`: 翻译完成（含完整译文和 ID）
/// 3. `translation_stream_error`: 错误信息
#[tauri::command]
pub async fn translate_text_stream(
    request: TranslationRequest,
    window: Window,
    state: State<'_, crate::commands::AppState>,
) -> Result<Option<TranslationResponse>, AppError> {
    println!(
        "🌐 [Translation] 开始流式翻译：{} -> {}, 文本长度：{}",
        request.src_lang,
        request.tgt_lang,
        request.text.len()
    );

    // 获取 VFS 数据库（必需）
    let vfs_db = state
        .vfs_db
        .clone()
        .ok_or_else(|| AppError::database("VFS 数据库未初始化".to_string()))?;

    // 构造依赖
    let deps = pipeline::TranslationDeps {
        llm: state.llm_manager.clone(),
        db: state.database.clone(), // 仅用于迁移期读取旧数据
        emitter: TranslationEventEmitter::new(window.clone()),
        vfs_db, // ★ VFS 统一存储（必需）
    };

    // 运行翻译管线
    let result = pipeline::run_translation(request.clone(), deps).await?;

    if let Some(ref response) = result {
        println!(
            "✅ [Translation] 翻译完成：ID={}, 译文长度：{}",
            response.id,
            response.translated_text.len()
        );
    } else {
        println!(
            "🛑 [Translation] 用户取消翻译：session_id={}",
            request.session_id
        );
    }

    Ok(result)
}

/// 🔧 统一翻译入口 (合并工作台 + 弹窗)
///
/// 两个旧入口 (translate_text_stream / stream_chat_translation_*) 共用
/// `stream_translate` 核心，仅薄命令层不同。此命令提供统一模式分发：
///
/// | mode | 对应旧命令 | 持久化 | 事件 |
/// |------|-----------|--------|------|
/// | `workbench` | translate_text_stream | 前端 DSTU | translation_stream_{session_id} |
/// | `popover_aligned` | stream_chat_translation_aligned | 无 | chat_translation_{request_id} |
/// | `popover_plain` | stream_chat_translation_plain | 无 | chat_translation_{request_id} |
///
/// 后端统一入口；表现层 (工作台/弹窗) 由前端自由选择。
#[derive(Debug, Clone, Deserialize)]
pub struct UnifiedTranslationRequest {
    /// 模式: workbench | popover_aligned | popover_plain
    pub mode: String,
    /// 工作台模式: TranslationRequest 字段
    #[serde(flatten)]
    pub workbench: Option<types::TranslationRequest>,
    /// 弹窗模式: ChatTranslationRequest 字段
    #[serde(flatten)]
    pub popover: Option<chat_popover::ChatTranslationRequest>,
}

#[tauri::command]
pub async fn translate_unified(
    request: UnifiedTranslationRequest,
    window: Window,
    state: State<'_, crate::commands::AppState>,
) -> Result<Option<TranslationResponse>, AppError> {
    match request.mode.as_str() {
        "workbench" => {
            let req = request
                .workbench
                .ok_or_else(|| AppError::validation("workbench 模式缺少 TranslationRequest".to_string()))?;
            let vfs_db = state
                .vfs_db
                .clone()
                .ok_or_else(|| AppError::database("VFS 数据库未初始化".to_string()))?;
            let deps = pipeline::TranslationDeps {
                llm: state.llm_manager.clone(),
                db: state.database.clone(),
                emitter: TranslationEventEmitter::new(window.clone()),
                vfs_db,
            };
            pipeline::run_translation(req, deps).await
        }
        "popover_aligned" | "popover_plain" => {
            let req = request
                .popover
                .ok_or_else(|| AppError::validation("popover 模式缺少 ChatTranslationRequest".to_string()))?;
            let mode = if request.mode == "popover_aligned" {
                chat_popover::ChatTranslationMode::Aligned
            } else {
                chat_popover::ChatTranslationMode::Plain
            };
            chat_popover::run_chat_translation_public(req, mode, window, state).await?;
            Ok(None)
        }
        other => Err(AppError::validation(format!("未知翻译模式: {}", other))),
    }
}
