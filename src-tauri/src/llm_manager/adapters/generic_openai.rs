//! 通用 OpenAI 兼容适配器
//!
//! 处理标准 OpenAI API 兼容的请求参数。
//! 适用于 OpenAI 官方 API（GPT-5.2+）及兼容供应商。
//!
//! ## Chat Completions API 参数格式 (2026)
//! - `reasoning_effort`: none | minimal | low | medium | high | xhigh（**顶级参数**）
//! - `verbosity`: low | medium | high（**顶级参数**）
//! - temperature/top_p 仅在 `reasoning_effort="none"` 时支持，其他值会**报错**
//!
//! ## 注意：Chat Completions API vs Responses API
//! - Chat Completions API 使用顶级参数：`reasoning_effort`, `verbosity`
//! - Responses API 使用嵌套格式：`reasoning: { effort }`, `text: { verbosity }`
//! - 本适配器使用 Chat Completions API 格式
//!
//! 参考文档：https://platform.openai.com/docs/api-reference/chat

use super::{get_trimmed_effort, resolve_enable_thinking, PassbackPolicy, RequestAdapter};
use crate::llm_manager::ApiConfig;
use serde_json::{json, Map, Value};

// ============================================================
// ProviderOverrides — 为 GenericOpenAIAdapter 注入供应商特定行为
// ============================================================

/// 供应商特定覆盖配置，用于消除 5 个薄适配器 (Grok/Mimo/Ernie/Doubao/Mistral)
/// 对独立 RequestAdapter 实现的需求。
///
/// 所有字段都是可选的：`None` 表示使用 GenericOpenAIAdapter 的默认逻辑。
/// 因为是 INTERNAL 细节，不公开给模块外部。
#[derive(Clone)]
pub(crate) struct ProviderOverrides {
    /// 适配器标识符（如 "grok"）
    pub id: &'static str,
    /// 适配器显示名称
    pub label: &'static str,
    /// 适配器描述
    pub description: &'static str,
    /// 自定义推理配置函数。返回 `true` 表示提前返回（跳过 `apply_common_params`）。
    pub reasoning_config_fn: Option<fn(&mut Map<String, Value>, &ApiConfig, Option<bool>) -> bool>,
    /// 自定义采样参数移除逻辑
    pub sampling_removal_fn: Option<fn(&ApiConfig) -> bool>,
    /// 自定义思维链回传策略
    pub passback_policy_fn: Option<fn(&ApiConfig) -> PassbackPolicy>,
    /// 自定义通用参数处理
    pub common_params_fn: Option<fn(&mut Map<String, Value>, &ApiConfig)>,
}

// ============================================================
// Grok 覆盖函数
// ============================================================

fn is_grok3_mini(model: &str) -> bool {
    let model_lower = model.to_lowercase();
    model_lower.contains("grok-3-mini") || model_lower.contains("grok3-mini")
}

fn is_grok4(model: &str) -> bool {
    let model_lower = model.to_lowercase();
    model_lower.contains("grok-4") || model_lower.contains("grok4")
}

fn grok_reasoning_config(
    body: &mut Map<String, Value>,
    config: &ApiConfig,
    _enable_thinking: Option<bool>,
) -> bool {
    // reasoning_effort 仅 grok-3-mini 支持
    if is_grok3_mini(&config.model) {
        if let Some(effort) = get_trimmed_effort(config) {
            let normalized = match effort.to_lowercase().as_str() {
                "high" | "xhigh" | "medium" => "high",
                _ => "low",
            };
            body.insert("reasoning_effort".to_string(), json!(normalized));
        }
    }
    // Grok-4 不支持 presencePenalty, frequencyPenalty, stop
    if is_grok4(&config.model) {
        body.remove("presence_penalty");
        body.remove("frequency_penalty");
        body.remove("presencePenalty");
        body.remove("frequencyPenalty");
        body.remove("stop");
    }
    false // 继续处理通用参数
}

fn grok_should_remove_sampling(_config: &ApiConfig) -> bool {
    false // Grok 支持 temperature/top_p
}

fn grok_common_params(body: &mut Map<String, Value>, config: &ApiConfig) {
    // Grok-4 不支持 repetition_penalty（通过 frequency/presence_penalty 实现）
    if !is_grok4(&config.model) {
        if let Some(min_p) = config.min_p {
            body.insert("min_p".to_string(), json!(min_p));
        }
        if let Some(top_k) = config.top_k {
            body.insert("top_k".to_string(), json!(top_k));
        }
        if let Some(rep_penalty) = config.repetition_penalty {
            body.insert("repetition_penalty".to_string(), json!(rep_penalty));
        }
    }
    // Grok 不使用 reasoning_split, effort, verbosity
}

// ============================================================
// Mimo 覆盖函数
// ============================================================

fn mimo_supports_thinking(config: &ApiConfig) -> bool {
    let model = config.model.to_lowercase();
    !model.contains("tts") && (config.supports_reasoning || config.is_reasoning)
}

fn mimo_reasoning_config(
    body: &mut Map<String, Value>,
    config: &ApiConfig,
    enable_thinking: Option<bool>,
) -> bool {
    body.remove("enable_thinking");
    body.remove("thinking_budget");
    body.remove("include_thoughts");
    body.remove("reasoning_effort");

    if mimo_supports_thinking(config) {
        let thinking_enabled = resolve_enable_thinking(config, enable_thinking);
        let thinking_type = if thinking_enabled {
            "enabled"
        } else {
            "disabled"
        };
        body.insert("thinking".to_string(), json!({ "type": thinking_type }));
    } else {
        body.remove("thinking");
    }

    if let Some(choice) = body.get("tool_choice").and_then(|value| value.as_str()) {
        if choice != "auto" {
            body.insert("tool_choice".to_string(), json!("auto"));
        }
    }

    true // 提前返回，跳过通用参数
}

fn mimo_should_remove_sampling(_config: &ApiConfig) -> bool {
    false
}

fn mimo_passback_policy(config: &ApiConfig) -> PassbackPolicy {
    if mimo_supports_thinking(config) {
        PassbackPolicy::DeepSeekStyle
    } else {
        PassbackPolicy::NoPassback
    }
}

// ============================================================
// Ernie 覆盖函数
// ============================================================

fn is_ernie_thinking_model(model: &str) -> bool {
    let model_lower = model.to_lowercase();
    model_lower.contains("ernie-5.0-thinking")
        || model_lower.contains("ernie-5-thinking")
        || model_lower.contains("ernie5-thinking")
        || model_lower.contains("ernie-x1")
        || model_lower.contains("ernie_x1")
}

fn ernie_reasoning_config(
    body: &mut Map<String, Value>,
    config: &ApiConfig,
    _enable_thinking: Option<bool>,
) -> bool {
    // ERNIE 使用 max_output_tokens 而非 max_tokens
    if let Some(max_tokens) = body.remove("max_tokens") {
        body.insert("max_output_tokens".to_string(), max_tokens);
    }
    if let Some(max_completion) = body.remove("max_completion_tokens") {
        body.insert("max_output_tokens".to_string(), max_completion);
    }

    // reasoning_effort: low | medium | high
    if let Some(effort) = get_trimmed_effort(config) {
        let effort_lower = effort.to_lowercase();
        if matches!(effort_lower.as_str(), "low" | "medium" | "high") {
            body.insert("reasoning_effort".to_string(), json!(effort_lower));
        }
    }

    body.remove("enable_thinking");
    body.remove("thinking");
    body.remove("thinking_budget");

    false // 继续处理通用参数
}

fn ernie_should_remove_sampling(config: &ApiConfig) -> bool {
    is_ernie_thinking_model(&config.model) || config.is_reasoning || config.supports_reasoning
}

fn ernie_passback_policy(config: &ApiConfig) -> PassbackPolicy {
    if is_ernie_thinking_model(&config.model) || config.supports_reasoning || config.is_reasoning {
        PassbackPolicy::DeepSeekStyle
    } else {
        PassbackPolicy::NoPassback
    }
}

fn ernie_common_params(body: &mut Map<String, Value>, config: &ApiConfig) {
    if let Some(min_p) = config.min_p {
        body.insert("min_p".to_string(), json!(min_p));
    }
    if let Some(top_k) = config.top_k {
        body.insert("top_k".to_string(), json!(top_k));
    }
    // ERNIE 使用 penalty_score 而非 repetition_penalty
    if let Some(rep_penalty) = config.repetition_penalty {
        body.insert("penalty_score".to_string(), json!(rep_penalty));
    }
    // reasoning_effort 已在 apply_reasoning_config 中处理
}

// ============================================================
// Doubao 覆盖函数
// ============================================================

fn doubao_supports_auto_mode(model: &str) -> bool {
    let model_lower = model.to_lowercase();
    if model_lower.contains("seed") {
        return true;
    }
    if model_lower.contains("thinking-pro-m") || model_lower.contains("thinking-pro-m-") {
        return true;
    }
    false
}

fn doubao_is_thinking_model(model: &str) -> bool {
    let model_lower = model.to_lowercase();
    model_lower.contains("thinking") || model_lower.contains("seed")
}

fn doubao_reasoning_config(
    body: &mut Map<String, Value>,
    config: &ApiConfig,
    enable_thinking: Option<bool>,
) -> bool {
    if !config.supports_reasoning && !config.is_reasoning {
        return false;
    }

    let enable_thinking_value = resolve_enable_thinking(config, enable_thinking);

    let thinking_type = if enable_thinking_value {
        if doubao_supports_auto_mode(&config.model) {
            if let Some(ref effort) = config.reasoning_effort {
                let effort_lower = effort.to_lowercase();
                if effort_lower == "auto" || effort_lower == "medium" {
                    "auto"
                } else {
                    "enabled"
                }
            } else {
                "enabled"
            }
        } else {
            "enabled"
        }
    } else {
        "disabled"
    };

    body.insert("thinking".to_string(), json!({ "type": thinking_type }));

    false
}

fn doubao_should_remove_sampling(_config: &ApiConfig) -> bool {
    false // 豆包 thinking 模型保留采样参数
}

fn doubao_passback_policy(config: &ApiConfig) -> PassbackPolicy {
    if config.is_reasoning
        || config.supports_reasoning
        || doubao_is_thinking_model(&config.model)
    {
        PassbackPolicy::DeepSeekStyle
    } else {
        PassbackPolicy::NoPassback
    }
}

// ============================================================
// Mistral 覆盖函数
// ============================================================

fn is_magistral(model: &str) -> bool {
    let model_lower = model.to_lowercase();
    model_lower.contains("magistral")
}

fn mistral_reasoning_config(
    body: &mut Map<String, Value>,
    config: &ApiConfig,
    enable_thinking: Option<bool>,
) -> bool {
    if is_magistral(&config.model) {
        let should_enable = resolve_enable_thinking(config, enable_thinking);
        if should_enable {
            body.insert("prompt_mode".to_string(), json!("reasoning"));
        } else {
            body.insert("prompt_mode".to_string(), Value::Null);
        }
    }
    false
}

fn mistral_should_remove_sampling(_config: &ApiConfig) -> bool {
    false // Mistral 支持 temperature/top_p
}

fn mistral_common_params(body: &mut Map<String, Value>, config: &ApiConfig) {
    if let Some(min_p) = config.min_p {
        body.insert("min_p".to_string(), json!(min_p));
    }
    if let Some(top_k) = config.top_k {
        body.insert("top_k".to_string(), json!(top_k));
    }
    if let Some(rep_penalty) = config.repetition_penalty {
        body.insert("repetition_penalty".to_string(), json!(rep_penalty));
    }
    // Mistral 不使用 reasoning_split, effort, verbosity
}

// ============================================================
// Provider override 常量定义
// ============================================================

pub(crate) const GROK_OVERRIDES: ProviderOverrides = ProviderOverrides {
    id: "grok",
    label: "xAI Grok",
    description: "Grok 系列，grok-3-mini 支持 reasoning_effort",
    reasoning_config_fn: Some(grok_reasoning_config),
    sampling_removal_fn: Some(grok_should_remove_sampling),
    passback_policy_fn: None,
    common_params_fn: Some(grok_common_params),
};

pub(crate) const MIMO_OVERRIDES: ProviderOverrides = ProviderOverrides {
    id: "mimo",
    label: "Xiaomi MiMo",
    description: "MiMo 系列，支持 thinking.type 与 reasoning_content",
    reasoning_config_fn: Some(mimo_reasoning_config),
    sampling_removal_fn: Some(mimo_should_remove_sampling),
    passback_policy_fn: Some(mimo_passback_policy),
    common_params_fn: None,
};

pub(crate) const ERNIE_OVERRIDES: ProviderOverrides = ProviderOverrides {
    id: "ernie",
    label: "百度文心",
    description: "ERNIE 系列，支持 max_output_tokens/reasoning_effort/penalty_score 参数",
    reasoning_config_fn: Some(ernie_reasoning_config),
    sampling_removal_fn: Some(ernie_should_remove_sampling),
    passback_policy_fn: Some(ernie_passback_policy),
    common_params_fn: Some(ernie_common_params),
};

pub(crate) const DOUBAO_OVERRIDES: ProviderOverrides = ProviderOverrides {
    id: "doubao",
    label: "字节豆包",
    description: "豆包 Seed/Thinking 系列，支持 thinking.type (auto/enabled/disabled)",
    reasoning_config_fn: Some(doubao_reasoning_config),
    sampling_removal_fn: Some(doubao_should_remove_sampling),
    passback_policy_fn: Some(doubao_passback_policy),
    common_params_fn: None,
};

pub(crate) const MISTRAL_OVERRIDES: ProviderOverrides = ProviderOverrides {
    id: "mistral",
    label: "Mistral AI",
    description: "Mistral 系列，Magistral 支持 prompt_mode: reasoning",
    reasoning_config_fn: Some(mistral_reasoning_config),
    sampling_removal_fn: Some(mistral_should_remove_sampling),
    passback_policy_fn: None,
    common_params_fn: Some(mistral_common_params),
};

/// 通用 OpenAI 兼容适配器
///
/// 处理标准 OpenAI Chat Completions API 格式的推理参数：
/// - `reasoning_effort`: "none" | "minimal" | "low" | "medium" | "high" | "xhigh"（顶级参数）
/// - `verbosity`: "low" | "medium" | "high"（顶级参数）
/// - `enable_thinking`: 启用思维链（兼容其他 OpenAI 兼容供应商）
/// - `thinking_budget`: 思维 token 预算
pub struct GenericOpenAIAdapter {
    /// 可选的供应商特定覆盖配置
    pub overrides: Option<&'static ProviderOverrides>,
}

impl GenericOpenAIAdapter {
    /// 验证 reasoning_effort 值是否有效
    fn is_valid_effort(effort: &str) -> bool {
        matches!(
            effort.to_lowercase().as_str(),
            "none" | "minimal" | "low" | "medium" | "high" | "xhigh" | "unset"
        )
    }

    /// 检查是否需要移除采样参数
    ///
    /// GPT-5.2: temperature/top_p 仅在 reasoning_effort="none" 时支持
    ///
    /// ## 优先级规则
    /// 1. 如果显式设置了 `reasoning_effort`：
    ///    - `reasoning_effort="none"` → **保留**采样参数（用户明确想禁用推理）
    ///    - 其他值 → **移除**采样参数（推理模式不支持）
    /// 2. 如果没有设置 `reasoning_effort`：
    ///    - `is_reasoning || supports_reasoning` → **移除**采样参数
    fn should_remove_sampling_for_reasoning(config: &ApiConfig) -> bool {
        if let Some(ref effort) = config.reasoning_effort {
            let trimmed = effort.trim().to_lowercase();
            if !trimmed.is_empty() {
                // 用户显式设置了 reasoning_effort
                // "none" 或 "unset" 表示禁用推理，应保留采样参数
                if trimmed == "none" || trimmed == "unset" {
                    return false; // 保留采样参数
                }
                // 非法值不应触发移除采样参数
                if !Self::is_valid_effort(&trimmed) {
                    log::warn!(
                        "[GenericOpenAIAdapter] Invalid reasoning_effort: {}. Keeping sampling params.",
                        trimmed
                    );
                    return false;
                }
                // 其他有效值表示启用推理，应移除采样参数
                return true;
            }
        }
        // 没有显式设置 reasoning_effort 时，使用原有逻辑
        config.is_reasoning || config.supports_reasoning
    }
}

impl RequestAdapter for GenericOpenAIAdapter {
    fn id(&self) -> &'static str {
        self.overrides.map_or("general", |o| o.id)
    }

    fn label(&self) -> &'static str {
        self.overrides.map_or("OpenAI Compatible", |o| o.label)
    }

    fn description(&self) -> &'static str {
        self.overrides
            .map_or("适用于大多数 OpenAI 兼容模型参数格式；具体请求协议由 OpenAI 协议决定", |o| o.description)
    }

    fn apply_reasoning_config(
        &self,
        body: &mut Map<String, Value>,
        config: &ApiConfig,
        enable_thinking: Option<bool>,
    ) -> bool {
        // 如果有供应商特定的推理配置覆盖，使用它
        if let Some(config_fn) = self.overrides.and_then(|o| o.reasoning_config_fn) {
            return config_fn(body, config, enable_thinking);
        }

        let mut early_return = false;

        // Chat Completions API: temperature/top_p 仅在 reasoning_effort="none" 时支持
        // 其他 reasoning_effort 值会导致 API 报错（不是被忽略）
        if Self::should_remove_sampling_for_reasoning(config) {
            body.remove("temperature");
            body.remove("top_p");
            body.remove("logprobs");
        }

        // 处理 reasoning_effort（Chat Completions API 使用顶级参数）
        // 注意：reasoning_effort 和 enable_thinking 是互斥的
        // - OpenAI 官方 Chat Completions API 使用 reasoning_effort（顶级参数）
        // - 其他 OpenAI 兼容供应商使用 enable_thinking
        let has_reasoning_effort = get_trimmed_effort(config).is_some();

        if has_reasoning_effort {
            // OpenAI 官方 Chat Completions API 格式：使用顶级参数
            if let Some(effort) = get_trimmed_effort(config) {
                let normalized = effort.to_lowercase();
                if normalized == "none" || normalized == "unset" {
                    // "none" 或 "unset" 时不添加 reasoning_effort 参数
                    body.remove("reasoning_effort");
                    body.remove("reasoning"); // 清理可能存在的嵌套格式
                    early_return = true;
                } else if Self::is_valid_effort(effort) {
                    // Chat Completions API: 使用顶级 reasoning_effort 参数
                    body.insert("reasoning_effort".to_string(), json!(normalized));
                }
            }

            // Chat Completions API: verbosity 是顶级参数
            if let Some(ref verbosity) = config.verbosity {
                let v = verbosity.trim().to_lowercase();
                if !v.is_empty() && matches!(v.as_str(), "low" | "medium" | "high") {
                    body.insert("verbosity".to_string(), json!(v));
                }
            }
        } else if config.supports_reasoning {
            // OpenAI 兼容供应商格式（enable_thinking）
            let enable_thinking_value = resolve_enable_thinking(config, enable_thinking);
            body.insert("enable_thinking".to_string(), json!(enable_thinking_value));

            if let Some(budget) = config.thinking_budget {
                let sanitized = budget.max(0);
                body.insert("thinking_budget".to_string(), json!(sanitized));
            }

            if config.include_thoughts {
                body.insert("include_thoughts".to_string(), json!(true));
            }
        }

        early_return
    }

    fn should_remove_sampling_params(&self, config: &ApiConfig) -> bool {
        if let Some(fn_ptr) = self.overrides.and_then(|o| o.sampling_removal_fn) {
            return fn_ptr(config);
        }
        Self::should_remove_sampling_for_reasoning(config)
    }

    fn get_passback_policy(&self, config: &ApiConfig) -> PassbackPolicy {
        if let Some(fn_ptr) = self.overrides.and_then(|o| o.passback_policy_fn) {
            return fn_ptr(config);
        }
        if config.is_reasoning || config.supports_reasoning {
            PassbackPolicy::DeepSeekStyle
        } else {
            PassbackPolicy::NoPassback
        }
    }

    fn apply_common_params(&self, body: &mut Map<String, Value>, config: &ApiConfig) {
        if let Some(fn_ptr) = self.overrides.and_then(|o| o.common_params_fn) {
            return fn_ptr(body, config);
        }
        // 默认行为：使用 trait 默认实现
        if let Some(min_p) = config.min_p {
            body.insert("min_p".to_string(), json!(min_p));
        }
        if let Some(top_k) = config.top_k {
            body.insert("top_k".to_string(), json!(top_k));
        }
        if let Some(rep_penalty) = config.repetition_penalty {
            body.insert("repetition_penalty".to_string(), json!(rep_penalty));
        }
        if let Some(reasoning_split) = config.reasoning_split {
            body.insert("reasoning_split".to_string(), json!(reasoning_split));
        }
        if let Some(ref effort) = config.effort {
            body.insert("effort".to_string(), json!(effort));
        }
        if let Some(ref verbosity) = config.verbosity {
            body.insert("verbosity".to_string(), json!(verbosity));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========== 辅助函数 ==========

    fn adapter(overrides: Option<&'static ProviderOverrides>) -> GenericOpenAIAdapter {
        GenericOpenAIAdapter { overrides }
    }

    fn create_test_config(supports_reasoning: bool, is_reasoning: bool) -> ApiConfig {
        ApiConfig {
            supports_reasoning,
            is_reasoning,
            thinking_enabled: true,
            thinking_budget: Some(4096),
            include_thoughts: true,
            ..Default::default()
        }
    }

    // ============================================================
    // 通用 GenericOpenAI 测试（无覆盖）
    // ============================================================

    #[test]
    fn test_apply_reasoning_config_with_reasoning() {
        let adapter = adapter(None);
        let config = create_test_config(true, false);
        let mut body = Map::new();
        body.insert("temperature".to_string(), json!(0.7));

        adapter.apply_reasoning_config(&mut body, &config, None);

        assert!(body.contains_key("enable_thinking"));
        assert!(body.contains_key("thinking_budget"));
        assert!(body.contains_key("include_thoughts"));
        // supports_reasoning 时移除 temperature
        assert!(!body.contains_key("temperature"));
    }

    #[test]
    fn test_remove_sampling_params_for_reasoning_model() {
        let adapter = adapter(None);
        let config = create_test_config(false, true);
        let mut body = Map::new();
        body.insert("temperature".to_string(), json!(0.7));
        body.insert("top_p".to_string(), json!(0.9));

        adapter.apply_reasoning_config(&mut body, &config, None);

        assert!(!body.contains_key("temperature"));
        assert!(!body.contains_key("top_p"));
    }

    #[test]
    fn test_xhigh_reasoning_effort() {
        let adapter = adapter(None);
        let config = ApiConfig {
            reasoning_effort: Some("xhigh".to_string()),
            ..Default::default()
        };
        let mut body = Map::new();

        adapter.apply_reasoning_config(&mut body, &config, None);

        assert_eq!(body.get("reasoning_effort"), Some(&json!("xhigh")));
        assert!(!body.contains_key("reasoning"));
    }

    #[test]
    fn test_verbosity_parameter() {
        let adapter = adapter(None);
        let config = ApiConfig {
            reasoning_effort: Some("high".to_string()),
            verbosity: Some("high".to_string()),
            ..Default::default()
        };
        let mut body = Map::new();

        adapter.apply_reasoning_config(&mut body, &config, None);

        assert_eq!(body.get("verbosity"), Some(&json!("high")));
        assert!(!body.contains_key("text"));
    }

    #[test]
    fn test_temperature_removed_when_reasoning_medium() {
        let adapter = adapter(None);
        let config = ApiConfig {
            reasoning_effort: Some("medium".to_string()),
            is_reasoning: false,
            supports_reasoning: false,
            ..Default::default()
        };
        let mut body = Map::new();
        body.insert("temperature".to_string(), json!(0.7));
        body.insert("top_p".to_string(), json!(0.9));

        adapter.apply_reasoning_config(&mut body, &config, None);

        assert!(!body.contains_key("temperature"));
        assert!(!body.contains_key("top_p"));
    }

    #[test]
    fn test_temperature_kept_when_reasoning_none() {
        let adapter = adapter(None);
        let config = ApiConfig {
            reasoning_effort: Some("none".to_string()),
            is_reasoning: false,
            supports_reasoning: false,
            ..Default::default()
        };
        let mut body = Map::new();
        body.insert("temperature".to_string(), json!(0.7));

        adapter.apply_reasoning_config(&mut body, &config, None);

        assert!(body.contains_key("temperature"));
    }

    #[test]
    fn test_temperature_kept_when_reasoning_none_even_if_is_reasoning_true() {
        let adapter = adapter(None);
        let config = ApiConfig {
            reasoning_effort: Some("none".to_string()),
            is_reasoning: true,
            supports_reasoning: true,
            ..Default::default()
        };
        let mut body = Map::new();
        body.insert("temperature".to_string(), json!(0.7));
        body.insert("top_p".to_string(), json!(0.9));

        adapter.apply_reasoning_config(&mut body, &config, None);

        assert!(body.contains_key("temperature"));
        assert!(body.contains_key("top_p"));
    }

    #[test]
    fn test_temperature_removed_when_reasoning_high_and_is_reasoning_true() {
        let adapter = adapter(None);
        let config = ApiConfig {
            reasoning_effort: Some("high".to_string()),
            is_reasoning: true,
            supports_reasoning: true,
            ..Default::default()
        };
        let mut body = Map::new();
        body.insert("temperature".to_string(), json!(0.7));

        adapter.apply_reasoning_config(&mut body, &config, None);

        assert!(!body.contains_key("temperature"));
    }

    #[test]
    fn test_invalid_reasoning_effort_keeps_sampling_params() {
        let adapter = adapter(None);
        let config = ApiConfig {
            reasoning_effort: Some("foo".to_string()),
            is_reasoning: false,
            supports_reasoning: false,
            ..Default::default()
        };
        let mut body = Map::new();
        body.insert("temperature".to_string(), json!(0.7));
        body.insert("top_p".to_string(), json!(0.9));

        adapter.apply_reasoning_config(&mut body, &config, None);

        assert!(body.contains_key("temperature"));
        assert!(body.contains_key("top_p"));
        assert!(!body.contains_key("reasoning_effort"));
    }

    // ============================================================
    // Grok 覆盖测试
    // ============================================================

    #[test]
    fn test_grok3_mini_reasoning_effort() {
        let adapter = adapter(Some(&GROK_OVERRIDES));
        let config = ApiConfig {
            reasoning_effort: Some("high".to_string()),
            model: "grok-3-mini".to_string(),
            ..Default::default()
        };
        let mut body = Map::new();
        adapter.apply_reasoning_config(&mut body, &config, None);
        assert_eq!(body.get("reasoning_effort"), Some(&json!("high")));
    }

    #[test]
    fn test_grok3_no_reasoning_effort() {
        let adapter = adapter(Some(&GROK_OVERRIDES));
        let config = ApiConfig {
            reasoning_effort: Some("high".to_string()),
            model: "grok-3".to_string(),
            ..Default::default()
        };
        let mut body = Map::new();
        adapter.apply_reasoning_config(&mut body, &config, None);
        assert!(!body.contains_key("reasoning_effort"));
    }

    #[test]
    fn test_grok4_removes_unsupported_params() {
        let adapter = adapter(Some(&GROK_OVERRIDES));
        let config = ApiConfig {
            model: "grok-4-1-fast-reasoning".to_string(),
            ..Default::default()
        };
        let mut body = Map::new();
        body.insert("presence_penalty".to_string(), json!(0.5));
        body.insert("frequency_penalty".to_string(), json!(0.5));
        body.insert("stop".to_string(), json!(["END"]));

        adapter.apply_reasoning_config(&mut body, &config, None);

        assert!(!body.contains_key("presence_penalty"));
        assert!(!body.contains_key("frequency_penalty"));
        assert!(!body.contains_key("stop"));
    }

    #[test]
    fn test_grok_keeps_sampling_params() {
        let adapter = adapter(Some(&GROK_OVERRIDES));
        assert!(!adapter.should_remove_sampling_params(&ApiConfig::default()));
    }

    // ============================================================
    // Mimo 覆盖测试
    // ============================================================

    #[test]
    fn test_mimo_thinking_enabled() {
        let adapter = adapter(Some(&MIMO_OVERRIDES));
        let config = ApiConfig {
            supports_reasoning: true,
            thinking_enabled: true,
            model: "mimo-v2.5-pro".to_string(),
            enable_thinking: Some(true),
            ..Default::default()
        };
        let mut body = Map::new();

        adapter.apply_reasoning_config(&mut body, &config, None);

        let thinking = body.get("thinking").unwrap();
        assert_eq!(thinking.get("type"), Some(&json!("enabled")));
        assert!(!body.contains_key("enable_thinking"));
        assert!(!body.contains_key("thinking_budget"));
        assert!(!body.contains_key("include_thoughts"));
        assert!(!body.contains_key("reasoning_effort"));
    }

    #[test]
    fn test_mimo_thinking_disabled() {
        let adapter = adapter(Some(&MIMO_OVERRIDES));
        let config = ApiConfig {
            supports_reasoning: true,
            thinking_enabled: false,
            model: "mimo-v2.5-pro".to_string(),
            ..Default::default()
        };
        let mut body = Map::new();

        adapter.apply_reasoning_config(&mut body, &config, None);

        let thinking = body.get("thinking").unwrap();
        assert_eq!(thinking.get("type"), Some(&json!("disabled")));
    }

    #[test]
    fn test_mimo_early_return() {
        // Mimo 返回 true，意味着提前返回，跳过通用参数
        let adapter = adapter(Some(&MIMO_OVERRIDES));
        let config = ApiConfig {
            supports_reasoning: true,
            thinking_enabled: true,
            model: "mimo-v2.5-pro".to_string(),
            ..Default::default()
        };
        let mut body = Map::new();

        let early_return = adapter.apply_reasoning_config(&mut body, &config, None);
        assert!(early_return);
    }

    #[test]
    fn test_mimo_sampling_kept() {
        let adapter = adapter(Some(&MIMO_OVERRIDES));
        let config = ApiConfig {
            is_reasoning: true,
            ..Default::default()
        };
        assert!(!adapter.should_remove_sampling_params(&config));
    }

    // ============================================================
    // Ernie 覆盖测试
    // ============================================================

    #[test]
    fn test_ernie_max_tokens_conversion() {
        let adapter = adapter(Some(&ERNIE_OVERRIDES));
        let config = ApiConfig::default();
        let mut body = Map::new();
        body.insert("max_tokens".to_string(), json!(4096));

        adapter.apply_reasoning_config(&mut body, &config, None);

        assert!(!body.contains_key("max_tokens"));
        assert_eq!(body.get("max_output_tokens"), Some(&json!(4096)));
    }

    #[test]
    fn test_ernie_reasoning_effort() {
        let adapter = adapter(Some(&ERNIE_OVERRIDES));
        let config = ApiConfig {
            reasoning_effort: Some("high".to_string()),
            ..Default::default()
        };
        let mut body = Map::new();

        adapter.apply_reasoning_config(&mut body, &config, None);

        assert_eq!(body.get("reasoning_effort"), Some(&json!("high")));
    }

    #[test]
    fn test_ernie_invalid_reasoning_effort_ignored() {
        let adapter = adapter(Some(&ERNIE_OVERRIDES));
        let config = ApiConfig {
            reasoning_effort: Some("invalid".to_string()),
            ..Default::default()
        };
        let mut body = Map::new();

        adapter.apply_reasoning_config(&mut body, &config, None);

        assert!(!body.contains_key("reasoning_effort"));
    }

    #[test]
    fn test_ernie_removes_enable_thinking() {
        let adapter = adapter(Some(&ERNIE_OVERRIDES));
        let config = ApiConfig {
            supports_reasoning: true,
            ..Default::default()
        };
        let mut body = Map::new();
        body.insert("enable_thinking".to_string(), json!(true));
        body.insert("thinking".to_string(), json!({"type": "enabled"}));
        body.insert("thinking_budget".to_string(), json!(2048));

        adapter.apply_reasoning_config(&mut body, &config, Some(true));

        assert!(!body.contains_key("enable_thinking"));
        assert!(!body.contains_key("thinking"));
        assert!(!body.contains_key("thinking_budget"));
    }

    #[test]
    fn test_ernie_is_thinking_model() {
        assert!(is_ernie_thinking_model("ernie-5.0-thinking-latest"));
        assert!(is_ernie_thinking_model("ernie-5.0-thinking-preview"));
        assert!(is_ernie_thinking_model("ERNIE-5.0-THINKING-LATEST"));
        assert!(!is_ernie_thinking_model("ernie-5.0"));
    }

    #[test]
    fn test_ernie_should_remove_sampling_for_thinking_model() {
        let adapter = adapter(Some(&ERNIE_OVERRIDES));
        let config = ApiConfig {
            model: "ernie-5.0-thinking-latest".to_string(),
            ..Default::default()
        };
        assert!(adapter.should_remove_sampling_params(&config));
    }

    #[test]
    fn test_ernie_keep_sampling_for_non_thinking_model() {
        let adapter = adapter(Some(&ERNIE_OVERRIDES));
        let config = ApiConfig {
            model: "ernie-5.0".to_string(),
            is_reasoning: false,
            supports_reasoning: false,
            ..Default::default()
        };
        assert!(!adapter.should_remove_sampling_params(&config));
    }

    #[test]
    fn test_ernie_passback_policy_for_thinking_model() {
        let adapter = adapter(Some(&ERNIE_OVERRIDES));
        let config = ApiConfig {
            model: "ernie-5.0-thinking-latest".to_string(),
            ..Default::default()
        };
        assert_eq!(
            adapter.get_passback_policy(&config),
            PassbackPolicy::DeepSeekStyle
        );
    }

    #[test]
    fn test_ernie_penalty_score_conversion() {
        let adapter = adapter(Some(&ERNIE_OVERRIDES));
        let config = ApiConfig {
            repetition_penalty: Some(1.2),
            ..Default::default()
        };
        let mut body = Map::new();
        adapter.apply_common_params(&mut body, &config);
        assert_eq!(body.get("penalty_score"), Some(&json!(1.2)));
        assert!(!body.contains_key("repetition_penalty"));
    }

    // ============================================================
    // Doubao 覆盖测试
    // ============================================================

    #[test]
    fn test_doubao_thinking_type_enabled() {
        let adapter = adapter(Some(&DOUBAO_OVERRIDES));
        let config = ApiConfig {
            supports_reasoning: true,
            thinking_enabled: true,
            model: "doubao-1-5-thinking-pro-250415".to_string(),
            ..Default::default()
        };
        let mut body = Map::new();

        adapter.apply_reasoning_config(&mut body, &config, None);

        let thinking = body.get("thinking").unwrap();
        assert_eq!(thinking.get("type"), Some(&json!("enabled")));
    }

    #[test]
    fn test_doubao_thinking_type_disabled() {
        let adapter = adapter(Some(&DOUBAO_OVERRIDES));
        let config = ApiConfig {
            supports_reasoning: true,
            thinking_enabled: false,
            model: "doubao-1-5-thinking-pro-250415".to_string(),
            ..Default::default()
        };
        let mut body = Map::new();

        adapter.apply_reasoning_config(&mut body, &config, None);

        let thinking = body.get("thinking").unwrap();
        assert_eq!(thinking.get("type"), Some(&json!("disabled")));
    }

    #[test]
    fn test_doubao_seed_model_auto_mode() {
        let adapter = adapter(Some(&DOUBAO_OVERRIDES));
        let config = ApiConfig {
            supports_reasoning: true,
            thinking_enabled: true,
            reasoning_effort: Some("auto".to_string()),
            model: "doubao-seed-1-6-thinking-250715".to_string(),
            ..Default::default()
        };
        let mut body = Map::new();

        adapter.apply_reasoning_config(&mut body, &config, None);

        let thinking = body.get("thinking").unwrap();
        assert_eq!(thinking.get("type"), Some(&json!("auto")));
    }

    #[test]
    fn test_doubao_seed_model_medium_effort() {
        let adapter = adapter(Some(&DOUBAO_OVERRIDES));
        let config = ApiConfig {
            supports_reasoning: true,
            thinking_enabled: true,
            reasoning_effort: Some("medium".to_string()),
            model: "doubao-seed-1-6-vision-250715".to_string(),
            ..Default::default()
        };
        let mut body = Map::new();

        adapter.apply_reasoning_config(&mut body, &config, None);

        let thinking = body.get("thinking").unwrap();
        assert_eq!(thinking.get("type"), Some(&json!("auto")));
    }

    #[test]
    fn test_doubao_thinking_pro_m_supports_auto() {
        let adapter = adapter(Some(&DOUBAO_OVERRIDES));
        let config = ApiConfig {
            supports_reasoning: true,
            thinking_enabled: true,
            reasoning_effort: Some("auto".to_string()),
            model: "doubao-1-5-thinking-pro-m-250428".to_string(),
            ..Default::default()
        };
        let mut body = Map::new();

        adapter.apply_reasoning_config(&mut body, &config, None);

        let thinking = body.get("thinking").unwrap();
        assert_eq!(thinking.get("type"), Some(&json!("auto")));
    }

    #[test]
    fn test_doubao_non_m_no_auto() {
        let adapter = adapter(Some(&DOUBAO_OVERRIDES));
        let config = ApiConfig {
            supports_reasoning: true,
            thinking_enabled: true,
            reasoning_effort: Some("auto".to_string()),
            model: "doubao-1-5-thinking-pro-250415".to_string(),
            ..Default::default()
        };
        let mut body = Map::new();

        adapter.apply_reasoning_config(&mut body, &config, None);

        let thinking = body.get("thinking").unwrap();
        assert_eq!(thinking.get("type"), Some(&json!("enabled")));
    }

    #[test]
    fn test_doubao_keep_temperature() {
        let adapter = adapter(Some(&DOUBAO_OVERRIDES));
        let config = ApiConfig {
            is_reasoning: true,
            ..Default::default()
        };
        assert!(!adapter.should_remove_sampling_params(&config));
    }

    #[test]
    fn test_doubao_passback_policy_deepseek_style() {
        let adapter = adapter(Some(&DOUBAO_OVERRIDES));
        let config = ApiConfig {
            supports_reasoning: true,
            model: "doubao-seed-1-6-thinking-250715".to_string(),
            ..Default::default()
        };
        assert_eq!(
            adapter.get_passback_policy(&config),
            PassbackPolicy::DeepSeekStyle
        );
    }

    #[test]
    fn test_doubao_passback_policy_no_passback() {
        let adapter = adapter(Some(&DOUBAO_OVERRIDES));
        let config = ApiConfig {
            supports_reasoning: false,
            is_reasoning: false,
            model: "doubao-1.5-pro-32k".to_string(),
            ..Default::default()
        };
        assert_eq!(
            adapter.get_passback_policy(&config),
            PassbackPolicy::NoPassback
        );
    }

    #[test]
    fn test_doubao_non_reasoning_model_no_thinking() {
        let adapter = adapter(Some(&DOUBAO_OVERRIDES));
        let config = ApiConfig {
            supports_reasoning: false,
            is_reasoning: false,
            model: "doubao-1.5-pro-32k".to_string(),
            ..Default::default()
        };
        let mut body = Map::new();

        let result = adapter.apply_reasoning_config(&mut body, &config, None);

        assert!(!result);
        assert!(!body.contains_key("thinking"));
    }

    // ============================================================
    // Mistral 覆盖测试
    // ============================================================

    #[test]
    fn test_mistral_magistral_prompt_mode() {
        let adapter = adapter(Some(&MISTRAL_OVERRIDES));
        let config = ApiConfig {
            model: "magistral-medium-latest".to_string(),
            ..Default::default()
        };
        let mut body = Map::new();

        adapter.apply_reasoning_config(&mut body, &config, None);

        assert_eq!(body.get("prompt_mode"), Some(&json!("reasoning")));
    }

    #[test]
    fn test_mistral_magistral_with_prefix() {
        let adapter = adapter(Some(&MISTRAL_OVERRIDES));
        let config = ApiConfig {
            model: "mistral/magistral-medium-latest".to_string(),
            ..Default::default()
        };
        let mut body = Map::new();

        adapter.apply_reasoning_config(&mut body, &config, None);

        assert_eq!(body.get("prompt_mode"), Some(&json!("reasoning")));
    }

    #[test]
    fn test_mistral_large_no_prompt_mode() {
        let adapter = adapter(Some(&MISTRAL_OVERRIDES));
        let config = ApiConfig {
            model: "mistral-large-latest".to_string(),
            ..Default::default()
        };
        let mut body = Map::new();

        adapter.apply_reasoning_config(&mut body, &config, None);

        assert!(!body.contains_key("prompt_mode"));
    }

    #[test]
    fn test_mistral_keeps_sampling_params() {
        let adapter = adapter(Some(&MISTRAL_OVERRIDES));
        let config = ApiConfig {
            model: "mistral-large-latest".to_string(),
            is_reasoning: true,
            ..Default::default()
        };
        assert!(!adapter.should_remove_sampling_params(&config));
    }

    #[test]
    fn test_mistral_common_params() {
        let adapter = adapter(Some(&MISTRAL_OVERRIDES));
        let config = ApiConfig {
            model: "mistral-large-latest".to_string(),
            min_p: Some(0.1),
            top_k: Some(50),
            repetition_penalty: Some(1.1),
            ..Default::default()
        };
        let mut body = Map::new();

        adapter.apply_common_params(&mut body, &config);

        assert_eq!(body.get("min_p"), Some(&json!(0.1)));
        assert_eq!(body.get("top_k"), Some(&json!(50)));
        assert_eq!(body.get("repetition_penalty"), Some(&json!(1.1)));
    }

    // ============================================================
    // 元测试：override id/label/description 正确
    // ============================================================

    #[test]
    fn test_grok_override_identity() {
        let adapter = adapter(Some(&GROK_OVERRIDES));
        assert_eq!(adapter.id(), "grok");
        assert_eq!(adapter.label(), "xAI Grok");
    }

    #[test]
    fn test_mimo_override_identity() {
        let adapter = adapter(Some(&MIMO_OVERRIDES));
        assert_eq!(adapter.id(), "mimo");
        assert_eq!(adapter.label(), "Xiaomi MiMo");
    }

    #[test]
    fn test_ernie_override_identity() {
        let adapter = adapter(Some(&ERNIE_OVERRIDES));
        assert_eq!(adapter.id(), "ernie");
        assert_eq!(adapter.label(), "百度文心");
    }

    #[test]
    fn test_doubao_override_identity() {
        let adapter = adapter(Some(&DOUBAO_OVERRIDES));
        assert_eq!(adapter.id(), "doubao");
        assert_eq!(adapter.label(), "字节豆包");
    }

    #[test]
    fn test_mistral_override_identity() {
        let adapter = adapter(Some(&MISTRAL_OVERRIDES));
        assert_eq!(adapter.id(), "mistral");
        assert_eq!(adapter.label(), "Mistral AI");
    }
}
