//! Handlers 共享辅助函数
//!
//! 提取跨 handler 模块使用的函数，避免兄弟模块间直接导入。

use crate::chat_v2::types::{MessageMeta, ReplayMode, SendOptions};

/// 应用原始 skill 快照覆盖
///
/// 在 replay_mode 为 Original 时，从消息快照中恢复 skill 状态。
/// 用于 send_message 和 variant_handlers 中。
pub(crate) fn apply_original_skill_snapshot_overrides(
    mut options: SendOptions,
    preferred_meta: Option<&MessageMeta>,
    fallback_meta: Option<&MessageMeta>,
) -> SendOptions {
    if options.replay_mode != Some(ReplayMode::Original) {
        return options;
    }

    let snapshot = preferred_meta
        .and_then(|meta| {
            meta.skill_snapshot_after
                .as_ref()
                .or(meta.skill_snapshot_before.as_ref())
        })
        .or_else(|| {
            fallback_meta.and_then(|meta| {
                meta.skill_snapshot_after
                    .as_ref()
                    .or(meta.skill_snapshot_before.as_ref())
            })
        });

    let runtime_snapshot = preferred_meta
        .and_then(|meta| {
            meta.skill_runtime_after
                .as_ref()
                .or(meta.skill_runtime_before.as_ref())
        })
        .or_else(|| {
            fallback_meta.and_then(|meta| {
                meta.skill_runtime_after
                    .as_ref()
                    .or(meta.skill_runtime_before.as_ref())
            })
        });

    if snapshot.is_none() && runtime_snapshot.is_none() {
        return options;
    }

    let mut replay_pinned_skill_ids = runtime_snapshot
        .map(|snapshot| snapshot.active_skill_ids.clone())
        .unwrap_or_default();
    if let Some(snapshot) = snapshot {
        replay_pinned_skill_ids = snapshot.manual_pinned_skill_ids.clone();
    }
    replay_pinned_skill_ids.sort();
    replay_pinned_skill_ids.dedup();

    if !replay_pinned_skill_ids.is_empty() {
        options.active_skill_ids = Some(replay_pinned_skill_ids);
    }

    if let Some(runtime_snapshot) = runtime_snapshot {
        if !runtime_snapshot.skill_contents.is_empty() {
            options.skill_contents = Some(runtime_snapshot.skill_contents.clone());
            options.replay_skill_contents = Some(runtime_snapshot.skill_contents.clone());
        }
        if !runtime_snapshot.skill_dependencies.is_empty() {
            options.skill_dependencies = Some(runtime_snapshot.skill_dependencies.clone());
        }
        if !runtime_snapshot.skill_embedded_tools.is_empty() {
            options.skill_embedded_tools = Some(runtime_snapshot.skill_embedded_tools.clone());
        }
        if !runtime_snapshot.mcp_tool_schemas.is_empty() {
            options.mcp_tool_schemas = Some(runtime_snapshot.mcp_tool_schemas.clone());
        }
        if !runtime_snapshot.selected_mcp_servers.is_empty() {
            options.mcp_tools = Some(runtime_snapshot.selected_mcp_servers.clone());
        }
    } else if let Some(snapshot) = snapshot {
        if !snapshot.effective_allowed_external_servers.is_empty() {
            options.mcp_tools = Some(snapshot.effective_allowed_external_servers.clone());
        }
    }

    options
}
