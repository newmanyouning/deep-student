use crate::models::ExamCardBBox;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::LazyLock;
use std::time::{Duration, Instant};
use uuid::Uuid;

// ==================== Registry types ====================

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct RegistryCapabilityFlags {
    pub(crate) vision: bool,
    pub(crate) function_calling: bool,
    pub(crate) reasoning: bool,
    #[serde(default)]
    pub(crate) max_context_tokens: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct RegistryModelRecord {
    pub(crate) model_id: String,
    #[serde(default)]
    pub(crate) provider_scope: Option<String>,
    #[serde(default)]
    pub(crate) provider_model_id: Option<String>,
    #[serde(default)]
    pub(crate) alias_of: Option<String>,
    pub(crate) capabilities: RegistryCapabilityFlags,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct RegistrySeriesRecord {
    models: Vec<RegistryModelRecord>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct RegistryDocument {
    records: Vec<RegistrySeriesRecord>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct ProviderProtocolRegistryRecord {
    pub(crate) provider_type: String,
    #[serde(default)]
    pub(crate) allowed_protocols: Vec<String>,
    pub(crate) default_protocol: String,
    #[serde(default)]
    pub(crate) official: bool,
    #[serde(default)]
    pub(crate) supports_openai_responses: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct ProviderProtocolRegistryDocument {
    pub(crate) providers: Vec<ProviderProtocolRegistryRecord>,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct CapabilityOverrides {
    pub(crate) is_multimodal: bool,
    pub(crate) supports_tools: bool,
    pub(crate) supports_reasoning: bool,
    pub(crate) context_window: Option<u32>,
}

// ==================== Static registries ====================

pub(crate) static MODEL_CAPABILITY_REGISTRY: LazyLock<Vec<RegistryModelRecord>> = LazyLock::new(|| {
    serde_json::from_str::<RegistryDocument>(include_str!(
        "../../../scripts/model-capability-registry.json"
    ))
    .map(|doc| {
        doc.records
            .into_iter()
            .flat_map(|record| record.models)
            .collect()
    })
    .unwrap_or_else(|err| {
        eprintln!(
            "[LLMManager] failed to parse model capability registry, falling back to empty: {}",
            err
        );
        Vec::new()
    })
});

pub(crate) static PROVIDER_PROTOCOL_REGISTRY: LazyLock<Vec<ProviderProtocolRegistryRecord>> =
    LazyLock::new(|| {
        serde_json::from_str::<ProviderProtocolRegistryDocument>(include_str!(
            "../../../scripts/provider-protocol-registry.json"
        ))
        .map(|doc| doc.providers)
        .unwrap_or_else(|err| {
            eprintln!(
                "[LLMManager] failed to parse provider protocol registry, falling back to empty: {}",
                err
            );
            Vec::new()
        })
    });

// ==================== Constants ====================

pub(crate) const EXAM_SEGMENT_MAX_IMAGE_BYTES: usize = 1_500_000;
pub(crate) const EXAM_SEGMENT_MAX_DIMENSION: u32 = 1_600;
pub(crate) const EXAM_SEGMENT_MAX_PAGES: usize = 36;
pub(crate) const STREAM_MAX_CTX_TOKENS: usize = 200_000;
pub(crate) const USER_PREFERENCES_SETTING_KEY: &str = "chat.user_preferences_profile";
pub(crate) const USER_PREFERENCE_FIELD_MAX_LEN: usize = 800;
pub(crate) const BUILTIN_MODEL_PROFILES_SNAPSHOT_KEY: &str = "builtin_model_profiles_snapshot";
pub(crate) const HIDDEN_BUILTIN_MODEL_PROFILES_KEY: &str = "hidden_builtin_model_profile_ids";

static CONTROL_CHARS_REGEX: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"[\u{0000}-\u{001F}\u{007F}]").unwrap());

// ==================== Helper: user preference storage ====================

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub(crate) struct StoredUserPreferenceProfile {
    pub(crate) enabled: bool,
    pub(crate) background: String,
    pub(crate) goals: String,
    pub(crate) communication: String,
    pub(crate) notes: String,
}

impl Default for StoredUserPreferenceProfile {
    fn default() -> Self {
        Self {
            enabled: false,
            background: String::new(),
            goals: String::new(),
            communication: String::new(),
            notes: String::new(),
        }
    }
}

pub(crate) fn sanitize_user_preference_field(value: &str) -> String {
    if value.is_empty() {
        return String::new();
    }
    let cleaned = CONTROL_CHARS_REGEX.replace_all(value, "");
    let trimmed = cleaned.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    let mut count = 0usize;
    let mut result = String::new();
    for ch in trimmed.chars() {
        if count >= USER_PREFERENCE_FIELD_MAX_LEN {
            break;
        }
        result.push(ch);
        count += 1;
    }
    result
}

pub(crate) fn build_user_preference_prompt_from_profile(
    profile: &StoredUserPreferenceProfile,
) -> Option<String> {
    if !profile.enabled {
        return None;
    }

    let background = sanitize_user_preference_field(&profile.background);
    let goals = sanitize_user_preference_field(&profile.goals);
    let communication = sanitize_user_preference_field(&profile.communication);
    let notes = sanitize_user_preference_field(&profile.notes);

    let mut lines: Vec<String> = Vec::new();
    if !background.is_empty() {
        lines.push(format!("- 学习背景 / Background: {}", background));
    }
    if !goals.is_empty() {
        lines.push(format!("- 学习目标 / Goals: {}", goals));
    }
    if !communication.is_empty() {
        lines.push(format!(
            "- 沟通偏好 / Communication Style: {}",
            communication
        ));
    }
    if !notes.is_empty() {
        lines.push(format!("- 补充说明 / Additional Notes: {}", notes));
    }

    if lines.is_empty() {
        return None;
    }

    Some(format!(
        "### 用户偏好（User Preferences）\n{}",
        lines.join("\n")
    ))
}

// ==================== MCP tool cache ====================

/// 前端 MCP 工具（通过桥接从前端SDK获取）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct FrontendMcpTool {
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) description: Option<String>,
    #[serde(default)]
    pub(crate) input_schema: Value,
}

/// MCP 工具缓存（前端来源）
#[derive(Debug, Clone)]
pub(crate) struct McpToolCache {
    pub(crate) tools: Vec<FrontendMcpTool>,
    cached_at: Instant,
    pub(crate) ttl: Duration,
}

impl McpToolCache {
    pub(crate) fn new(tools: Vec<FrontendMcpTool>, ttl: Duration) -> Self {
        Self {
            tools,
            cached_at: Instant::now(),
            ttl,
        }
    }
    pub(crate) fn is_expired(&self) -> bool {
        self.cached_at.elapsed() > self.ttl
    }
}

// ==================== OCR model config ====================

/// OCR 模型配置（用于多引擎支持）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OcrModelConfig {
    /// 模型配置 ID（对应 ApiConfig.id）
    pub config_id: String,
    /// 模型名称（如 deepseek-ai/DeepSeek-OCR）
    pub model: String,
    /// 引擎类型（deepseek_ocr, paddle_ocr_vl, generic_vlm）
    pub engine_type: String,
    /// 显示名称
    pub name: String,
    /// 是否免费
    #[serde(default)]
    pub is_free: bool,
    /// 是否启用（默认 true，向后兼容旧数据）
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// 优先级（数字越小越优先，默认 0）
    #[serde(default)]
    pub priority: u32,
}

fn default_true() -> bool {
    true
}

// ==================== ApiConfig ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiConfig {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub vendor_id: Option<String>,
    #[serde(default)]
    pub vendor_name: Option<String>,
    #[serde(default)]
    pub provider_type: Option<String>,
    #[serde(default)]
    pub provider_scope: Option<String>,
    #[serde(default)]
    pub api_protocol: Option<String>,
    #[serde(default)]
    pub supports_openai_responses: Option<bool>,
    pub api_key: String,
    pub base_url: String,
    pub model: String,
    pub is_multimodal: bool,
    pub is_reasoning: bool,
    pub is_embedding: bool,
    pub is_reranker: bool,
    #[serde(default)]
    pub is_image_generation: bool,
    pub enabled: bool,
    #[serde(default = "default_model_adapter")]
    pub model_adapter: String,
    #[serde(default = "default_max_output_tokens")]
    pub max_output_tokens: u32,
    #[serde(default = "default_temperature")]
    pub temperature: f32,
    #[serde(default, alias = "supports_tools")]
    pub supports_tools: bool,
    #[serde(default = "default_gemini_api_version")]
    pub gemini_api_version: String,
    #[serde(default)]
    pub is_builtin: bool,
    #[serde(default)]
    pub is_read_only: bool,
    #[serde(default)]
    pub reasoning_effort: Option<String>,
    #[serde(default)]
    pub thinking_enabled: bool,
    #[serde(default)]
    pub thinking_budget: Option<i32>,
    #[serde(default)]
    pub include_thoughts: bool,
    #[serde(default)]
    pub min_p: Option<f32>,
    #[serde(default)]
    pub top_k: Option<u32>,
    #[serde(default)]
    pub enable_thinking: Option<bool>,
    #[serde(default)]
    pub supports_reasoning: bool,
    #[serde(default)]
    pub headers: Option<HashMap<String, String>>,
    /// Top-P 核采样参数（运行时覆盖用）
    #[serde(default)]
    pub top_p_override: Option<f32>,
    /// 频率惩罚（运行时覆盖用）
    #[serde(default)]
    pub frequency_penalty_override: Option<f32>,
    /// 存在惩罚（运行时覆盖用）
    #[serde(default)]
    pub presence_penalty_override: Option<f32>,
    /// 重复惩罚（Qwen/豆包等模型使用）
    #[serde(default)]
    pub repetition_penalty: Option<f32>,
    /// MiniMax reasoning_split 参数
    #[serde(default)]
    pub reasoning_split: Option<bool>,
    /// Claude 4.5 Opus effort 参数 (high/medium/low)
    #[serde(default)]
    pub effort: Option<String>,
    /// OpenAI GPT-5.2 verbosity 参数 (low/medium/high)
    #[serde(default)]
    pub verbosity: Option<String>,
    /// 是否收藏
    #[serde(default)]
    pub is_favorite: bool,
    /// 供应商级别的 max_tokens 限制（API 最大允许值）
    #[serde(default)]
    pub max_tokens_limit: Option<u32>,
    /// 模型上下文窗口大小（tokens）
    #[serde(default, alias = "context_window")]
    pub context_window: Option<u32>,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            vendor_id: None,
            vendor_name: None,
            provider_type: None,
            provider_scope: None,
            api_protocol: None,
            supports_openai_responses: None,
            api_key: String::new(),
            base_url: String::new(),
            model: String::new(),
            is_multimodal: false,
            is_reasoning: false,
            is_embedding: false,
            is_reranker: false,
            is_image_generation: false,
            enabled: false,
            model_adapter: default_model_adapter(),
            max_output_tokens: default_max_output_tokens(),
            temperature: default_temperature(),
            supports_tools: false,
            gemini_api_version: default_gemini_api_version(),
            is_builtin: false,
            is_read_only: false,
            reasoning_effort: None,
            thinking_enabled: false,
            thinking_budget: None,
            include_thoughts: false,
            min_p: None,
            top_k: None,
            enable_thinking: None,
            supports_reasoning: false,
            headers: None,
            top_p_override: None,
            frequency_penalty_override: None,
            presence_penalty_override: None,
            repetition_penalty: None,
            reasoning_split: None,
            effort: None,
            verbosity: None,
            is_favorite: false,
            max_tokens_limit: None,
            context_window: None,
        }
    }
}

// ==================== VendorConfig ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VendorConfig {
    pub id: String,
    pub name: String,
    pub provider_type: String,
    #[serde(default)]
    pub api_protocol: Option<String>,
    #[serde(default)]
    pub supports_openai_responses: Option<bool>,
    pub base_url: String,
    pub api_key: String,
    #[serde(default)]
    pub headers: HashMap<String, String>,
    #[serde(default)]
    pub rate_limit_per_minute: Option<u32>,
    #[serde(default)]
    pub default_timeout_ms: Option<u64>,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default)]
    pub is_builtin: bool,
    #[serde(default)]
    pub is_read_only: bool,
    #[serde(default)]
    pub sort_order: Option<i32>,
    /// 供应商级别的 max_tokens 限制（API 最大允许值）
    #[serde(default)]
    pub max_tokens_limit: Option<u32>,
    /// 供应商官网链接
    #[serde(default)]
    pub website_url: Option<String>,
}

impl Default for VendorConfig {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: "New Vendor".to_string(),
            provider_type: "openai".to_string(),
            api_protocol: Some("openai_chat_completions".to_string()),
            supports_openai_responses: Some(false),
            base_url: String::new(),
            api_key: String::new(),
            headers: HashMap::new(),
            rate_limit_per_minute: None,
            default_timeout_ms: None,
            notes: None,
            is_builtin: false,
            is_read_only: false,
            sort_order: None,
            max_tokens_limit: None,
            website_url: None,
        }
    }
}

// ==================== ModelProfile ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelProfile {
    pub id: String,
    pub vendor_id: String,
    pub label: String,
    pub model: String,
    #[serde(default)]
    pub provider_scope: Option<String>,
    #[serde(default)]
    pub api_protocol: Option<String>,
    #[serde(default = "default_model_adapter")]
    pub model_adapter: String,
    #[serde(default)]
    pub is_multimodal: bool,
    #[serde(default)]
    pub is_reasoning: bool,
    #[serde(default)]
    pub is_embedding: bool,
    #[serde(default)]
    pub is_reranker: bool,
    #[serde(default)]
    pub is_image_generation: bool,
    #[serde(default)]
    pub supports_tools: bool,
    #[serde(default)]
    pub supports_reasoning: bool,
    #[serde(default = "default_profile_status")]
    pub status: String,
    #[serde(default = "default_profile_enabled")]
    pub enabled: bool,
    #[serde(default = "default_max_output_tokens")]
    pub max_output_tokens: u32,
    #[serde(default = "default_temperature")]
    pub temperature: f32,
    #[serde(default)]
    pub reasoning_effort: Option<String>,
    #[serde(default)]
    pub thinking_enabled: bool,
    #[serde(default)]
    pub thinking_budget: Option<i32>,
    #[serde(default)]
    pub include_thoughts: bool,
    #[serde(default)]
    pub enable_thinking: Option<bool>,
    #[serde(default)]
    pub min_p: Option<f32>,
    #[serde(default)]
    pub top_k: Option<u32>,
    #[serde(default)]
    pub gemini_api_version: Option<String>,
    #[serde(default)]
    pub is_builtin: bool,
    /// 重复惩罚
    #[serde(default)]
    pub repetition_penalty: Option<f32>,
    /// MiniMax reasoning_split 参数
    #[serde(default)]
    pub reasoning_split: Option<bool>,
    #[serde(default)]
    pub effort: Option<String>,
    #[serde(default)]
    pub verbosity: Option<String>,
    /// 是否收藏
    #[serde(default)]
    pub is_favorite: bool,
    /// 模型级别的 max_tokens 限制
    #[serde(default)]
    pub max_tokens_limit: Option<u32>,
    /// 模型上下文窗口大小
    #[serde(default, alias = "context_window")]
    pub context_window: Option<u32>,
}

impl Default for ModelProfile {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            vendor_id: String::new(),
            label: "New Model".to_string(),
            model: String::new(),
            provider_scope: None,
            api_protocol: None,
            model_adapter: default_model_adapter(),
            is_multimodal: false,
            is_reasoning: false,
            is_embedding: false,
            is_reranker: false,
            is_image_generation: false,
            supports_tools: false,
            supports_reasoning: false,
            status: default_profile_status(),
            enabled: default_profile_enabled(),
            max_output_tokens: default_max_output_tokens(),
            temperature: default_temperature(),
            reasoning_effort: None,
            thinking_enabled: false,
            thinking_budget: None,
            include_thoughts: false,
            enable_thinking: None,
            min_p: None,
            top_k: None,
            gemini_api_version: None,
            is_builtin: false,
            repetition_penalty: None,
            reasoning_split: None,
            effort: None,
            verbosity: None,
            is_favorite: false,
            max_tokens_limit: None,
            context_window: None,
        }
    }
}

// ==================== ResolvedModelConfig ====================

#[derive(Debug, Clone)]
pub struct ResolvedModelConfig {
    pub vendor: VendorConfig,
    pub profile: ModelProfile,
    pub runtime: ApiConfig,
}

// ==================== Default value functions ====================

pub(crate) fn default_model_adapter() -> String {
    "general".to_string()
}

pub(crate) fn default_max_output_tokens() -> u32 {
    8192
}

pub(crate) fn default_temperature() -> f32 {
    0.7
}

pub(crate) fn default_gemini_api_version() -> String {
    "v1".to_string()
}

pub(crate) fn default_profile_status() -> String {
    "enabled".to_string()
}

pub(crate) fn default_profile_enabled() -> bool {
    true
}

pub(crate) fn looks_like_image_generation_model_id(model: &str) -> bool {
    let model = model.to_lowercase();
    ["gpt-image", "dall-e", "imagen", "flux"]
        .iter()
        .any(|needle| model.contains(needle))
}

// ==================== Protocol helper functions ====================

pub(crate) fn normalize_provider_protocol_registry_value(value: Option<&str>) -> String {
    value.unwrap_or_default().trim().to_lowercase()
}

pub(crate) fn normalize_base_url_for_provider_protocol_registry(base_url: &str) -> String {
    base_url.trim().trim_end_matches('/').to_lowercase()
}

pub(crate) fn get_provider_protocol_record(
    provider_type: Option<&str>,
) -> Option<&'static ProviderProtocolRegistryRecord> {
    let normalized = normalize_provider_protocol_registry_value(provider_type);
    if normalized.is_empty() {
        return None;
    }
    PROVIDER_PROTOCOL_REGISTRY
        .iter()
        .find(|record| record.provider_type == normalized)
}

pub(crate) fn provider_allowed_protocols(provider_type: Option<&str>) -> Vec<String> {
    get_provider_protocol_record(provider_type)
        .map(|record| record.allowed_protocols.clone())
        .filter(|protocols| !protocols.is_empty())
        .unwrap_or_else(|| {
            vec![
                "openai_chat_completions".to_string(),
                "openai_responses".to_string(),
            ]
        })
}

fn resolves_to_official_openai(provider_type: Option<&str>, base_url: &str) -> bool {
    let normalized_provider = normalize_provider_protocol_registry_value(provider_type);
    let normalized_base_url = normalize_base_url_for_provider_protocol_registry(base_url);
    normalized_base_url.contains("api.openai.com")
        || (normalized_provider == "openai" && normalized_base_url.is_empty())
}

pub(crate) fn provider_supports_openai_responses(
    provider_type: Option<&str>,
    base_url: &str,
    supports_openai_responses: Option<bool>,
) -> bool {
    if supports_openai_responses == Some(true) {
        // ★ 第三方代理安全：对于有明确非 OpenAI 域名的 base_url
        // （如 ai98pro.xyz / api.qsl.fan），忽略自动检测的 Responses API 支持。
        // 这些代理的 Responses API 实现不完整，会导致上游 502 错误。
        // 空 base_url（未配置）或 api.openai.com 不受影响。
        let base_url_has_content = !base_url.is_empty();
        let is_third_party = base_url_has_content && !resolves_to_official_openai(provider_type, base_url);
        if !is_third_party {
            return true;
        }
        // 第三方端点：不回退到 auto-detection，直接返回 false（禁用 Responses API）
        return false;
    }
    if resolves_to_official_openai(provider_type, base_url) {
        return true;
    }
    get_provider_protocol_record(provider_type)
        .map(|record| record.supports_openai_responses)
        .unwrap_or(false)
}

pub(crate) fn should_honor_explicit_openai_responses_protocol(config: &ApiConfig) -> bool {
    config.model_adapter == "general"
        && provider_supports_openai_responses(
            config.provider_type.as_deref(),
            &config.base_url,
            config.supports_openai_responses,
        )
}

pub(crate) fn resolve_preferred_protocol_for_provider(
    provider_type: Option<&str>,
    adapter: Option<&str>,
    base_url: &str,
    supports_openai_responses: Option<bool>,
) -> String {
    match normalize_provider_protocol_registry_value(adapter).as_str() {
        "anthropic" => return "anthropic_messages".to_string(),
        "google" => return "google_generate_content".to_string(),
        _ => {}
    }

    let allowed = provider_allowed_protocols(provider_type);
    if provider_supports_openai_responses(provider_type, base_url, supports_openai_responses)
        && allowed
            .iter()
            .any(|protocol| protocol == "openai_responses")
    {
        return "openai_responses".to_string();
    }

    if let Some(record) = get_provider_protocol_record(provider_type) {
        if allowed
            .iter()
            .any(|protocol| protocol == &record.default_protocol)
        {
            return record.default_protocol.clone();
        }
    }

    allowed
        .into_iter()
        .next()
        .unwrap_or_else(|| "openai_chat_completions".to_string())
}

#[inline]
pub(crate) fn effective_max_tokens(max_output_tokens: u32, max_tokens_limit: Option<u32>) -> u32 {
    match max_tokens_limit {
        Some(limit) => max_output_tokens.min(limit),
        None => max_output_tokens,
    }
}

#[inline]
pub(crate) fn should_use_openai_responses_for_config(config: &ApiConfig) -> bool {
    if let Some(protocol) = config.api_protocol.as_deref() {
        let normalized = normalize_provider_protocol_registry_value(Some(protocol));
        return normalized == "openai_responses"
            && should_honor_explicit_openai_responses_protocol(config);
    }
    if config.model_adapter != "general" {
        return false;
    }

    resolve_preferred_protocol_for_provider(
        config.provider_type.as_deref(),
        Some(config.model_adapter.as_str()),
        &config.base_url,
        config.supports_openai_responses,
    ) == "openai_responses"
}

pub(crate) fn normalize_vendor_protocol_config(vendor: &mut VendorConfig) -> bool {
    let mut changed = false;

    if vendor.supports_openai_responses.is_none() {
        vendor.supports_openai_responses = Some(provider_supports_openai_responses(
            Some(vendor.provider_type.as_str()),
            &vendor.base_url,
            None,
        ));
        changed = true;
    }

    if let Some(protocol) = vendor.api_protocol.as_deref() {
        let normalized = normalize_provider_protocol_registry_value(Some(protocol));
        if normalized == "openai_responses"
            && !provider_supports_openai_responses(
                Some(vendor.provider_type.as_str()),
                &vendor.base_url,
                vendor.supports_openai_responses,
            )
        {
            vendor.api_protocol = Some(resolve_preferred_protocol_for_provider(
                Some(vendor.provider_type.as_str()),
                Some("general"),
                &vendor.base_url,
                vendor.supports_openai_responses,
            ));
            changed = true;
        }
    }

    changed
}

pub(crate) fn normalize_model_profile_protocol_config(
    profile: &mut ModelProfile,
    vendor: Option<&VendorConfig>,
) -> bool {
    let Some(protocol) = profile.api_protocol.as_deref() else {
        return false;
    };

    let normalized = normalize_provider_protocol_registry_value(Some(protocol));
    if normalized != "openai_responses" {
        return false;
    }

    let config = ApiConfig {
        provider_type: vendor.map(|item| item.provider_type.clone()),
        base_url: vendor.map(|item| item.base_url.clone()).unwrap_or_default(),
        model_adapter: profile.model_adapter.clone(),
        api_protocol: Some("openai_responses".to_string()),
        supports_openai_responses: vendor.and_then(|item| item.supports_openai_responses),
        ..Default::default()
    };

    if should_use_openai_responses_for_config(&config) {
        return false;
    }

    profile.api_protocol = Some(resolve_preferred_protocol_for_provider(
        vendor.map(|item| item.provider_type.as_str()),
        Some(profile.model_adapter.as_str()),
        vendor
            .map(|item| item.base_url.as_str())
            .unwrap_or_default(),
        vendor.and_then(|item| item.supports_openai_responses),
    ));
    true
}

pub(crate) fn build_provider_adapter(config: &ApiConfig) -> Box<dyn crate::providers::ProviderAdapter> {
    let use_responses = should_use_openai_responses_for_config(config);
    if matches!(
        config
            .api_protocol
            .as_deref()
            .map(|protocol| normalize_provider_protocol_registry_value(Some(protocol)))
            .as_deref(),
        Some("openai_responses")
    ) && !use_responses
    {
        log::warn!(
            "[LLM Manager] Downgrading unsupported openai_responses protocol to chat_completions: provider_type={:?}, base_url={}, model={}, adapter={}",
            config.provider_type,
            config.base_url,
            config.model,
            config.model_adapter
        );
    }
    if use_responses {
        Box::new(crate::providers::OpenAIResponsesAdapter)
    } else {
        match config.model_adapter.as_str() {
            "google" | "gemini" => Box::new(crate::providers::GeminiAdapter::new()),
            "anthropic" | "claude" => Box::new(crate::providers::AnthropicAdapter::new()),
            _ => Box::new(crate::providers::OpenAIAdapter),
        }
    }
}

pub(crate) fn normalize_nonstream_response_to_openai(
    config: &ApiConfig,
    response_json: &Value,
) -> Result<Value, crate::models::AppError> {
    if config.model_adapter == "google" {
        if let Some(safety_msg) = extract_gemini_safety_error(response_json) {
            return Err(crate::models::AppError::llm(safety_msg));
        }
        return crate::adapters::gemini_openai_converter::convert_gemini_nonstream_response_to_openai(
            response_json,
            &config.model,
        )
        .map_err(|e| crate::models::AppError::llm(format!("Gemini响应转换失败: {}", e)));
    }

    if matches!(config.model_adapter.as_str(), "anthropic" | "claude") {
        return crate::providers::convert_anthropic_response_to_openai(
            response_json,
            &config.model,
        )
        .ok_or_else(|| crate::models::AppError::llm("解析Anthropic响应失败".to_string()));
    }

    if should_use_openai_responses_for_config(config) {
        let mut text_segments: Vec<String> = Vec::new();
        if let Some(output) = response_json.get("output").and_then(|v| v.as_array()) {
            for item in output {
                if let Some(content_arr) = item.get("content").and_then(|v| v.as_array()) {
                    for entry in content_arr {
                        let entry_type = entry.get("type").and_then(|v| v.as_str()).unwrap_or("");
                        if matches!(entry_type, "output_text" | "text") {
                            if let Some(text) = entry.get("text").and_then(|v| v.as_str()) {
                                if !text.is_empty() {
                                    text_segments.push(text.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
        if text_segments.is_empty() {
            if let Some(output_text) = response_json.get("output_text").and_then(|v| v.as_str()) {
                if !output_text.is_empty() {
                    text_segments.push(output_text.to_string());
                }
            }
        }
        return Ok(serde_json::json!({
            "choices": [{
                "message": {
                    "content": text_segments.join("")
                }
            }],
            "usage": response_json.get("usage").cloned()
        }));
    }

    Ok(response_json.clone())
}

/// 检测 Gemini 非流式响应中的安全阻断，并返回结构化错误消息
pub(crate) fn extract_gemini_safety_error(resp: &serde_json::Value) -> Option<String> {
    gemini_safety_error_impl(resp)
}

fn gemini_safety_error_impl(resp: &serde_json::Value) -> Option<String> {
    if let Some(obj) = resp.as_object() {
        if let Some(prompt_feedback) = obj.get("promptFeedback") {
            if let Some(block_reason) =
                prompt_feedback.get("blockReason").and_then(|v| v.as_str())
            {
                let info = serde_json::json!({
                    "type": "safety_error",
                    "reason": block_reason,
                    "details": prompt_feedback
                });
                return Some(format!("Gemini安全阻断: {}", info.to_string()));
            }
        }
    }
    if let Some(cands) = resp.get("candidates").and_then(|v| v.as_array()) {
        for cand in cands {
            if let Some(fr) = cand.get("finishReason").and_then(|v| v.as_str()) {
                if fr == "SAFETY" {
                    let info = serde_json::json!({
                        "type": "safety_error",
                        "reason": fr,
                        "details": cand
                    });
                    return Some(format!("Gemini安全阻断: {}", info.to_string()));
                }
            }
        }
    }
    None
}

// ==================== Exam segmentation types ====================

#[derive(Debug, Clone)]
pub struct ExamSegmentationCard {
    pub question_label: String,
    pub bbox: ExamCardBBox,
    pub ocr_text: Option<String>,
    pub tags: Vec<String>,
    pub extra_metadata: Option<Value>,
    pub card_id: String,
}

#[derive(Debug, Clone)]
pub struct ExamSegmentationPage {
    pub page_index: usize,
    pub cards: Vec<ExamSegmentationCard>,
}

#[derive(Debug, Clone)]
pub struct ExamSegmentationOutput {
    pub pages: Vec<ExamSegmentationPage>,
    pub raw: Option<Value>,
}

// ==================== ImagePayload ====================

#[derive(Debug, Clone)]
pub(crate) struct ImagePayload {
    pub(crate) mime: String,
    pub(crate) base64: String,
}

// ==================== MergedChatMessage ====================

/// 🔧 P1修复：合并后的消息类型
/// 用于在消息序列化时合并连续的工具调用
#[derive(Debug, Clone)]
pub(crate) enum MergedChatMessage {
    /// 普通消息（直接传递）
    Regular(crate::models::ChatMessage),
    /// 合并的工具调用消息（多个 tool_calls）
    MergedToolCalls {
        tool_calls: Vec<crate::models::ToolCall>,
        content: String,
        thinking_content: Option<String>,
        thought_signature: Option<String>,
    },
}
