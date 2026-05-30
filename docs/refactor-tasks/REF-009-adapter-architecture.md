# REF-009: 统一适配器架构设计

> 基于全部调研: 10 LLM + 9 OCR + 6 TTS/ASR + 11 Image 模型
> 覆盖: 官方平台 vs SiliconFlow 聚合平台差异

## 架构概览

```
Application (DeepStudent)
  |
Unified Adapter Layer
  |-- ChatAdapter    (LLM 对话)
  |-- OcrAdapter     (文档识别)
  |-- TtsAdapter     (文本转语音)
  |-- ImageAdapter   (图像生成)
  |
Platform Router (Official | Aggregator | Local)
  |-- OpenAI Adapter, DeepSeek Adapter, PaddleOCR Adapter, ...
```

## E. 语音/TTS/ASR 官方 vs 聚合差异

| 维度 | OpenAI 官方 | SiliconFlow |
|------|------------|-------------|
| TTS 端点 | `POST /v1/audio/speech` | `POST /v1/audio/speech` |
| TTS 模型 | tts-1/tts-1-hd/gpt-4o-mini-tts | Fish-Speech/CosyVoice/SenseVoice |
| voice | 13 预设声音名 | 参考音频 URL/ID |
| speed | 0.25-4.0 | 不支持 |
| instructions | gpt-4o-mini-tts 支持情感/语调 | 不支持 |
| output_format | mp3/opus/aac/flac/wav/pcm (6种) | mp3/wav (2种) |
| ASR 端点 | `POST /v1/audio/transcriptions` | Chat Completions / 专用端点 |
| stream | 不支持 | 支持流式 |
| sample_rate | 固定 24kHz | 可配置 |

## F. 图像生成 官方 vs 聚合差异

| 参数 | OpenAI DALL-E 3 | FLUX.2-pro | FLUX.1-schnell | Qwen-Image | Kolors |
|------|----------------|------------|----------------|------------|--------|
| prompt | 4000 字符 | 1024 | 1024 | 1024 | 1024 |
| count | 1 | 1 | 1-4 | 1 | 1-4 |
| steps | 固定 | 6-50 | 1-4 | 默认50 | 1-100 |
| CFG | 不支持 | 3.0-20.0 | 3.0-20.0 | 0.1-20 | 0-20 |
| seed | 不支持 | 支持 | 支持 | 支持 | 支持 |
| negative_prompt | 不支持 | 支持 | 支持 | 支持 | 支持 |
| 图像编辑 | 不支持 | 不支持 | 不支持 | 支持(Edit版) | 不支持 |
| 尺寸数 | 3 | 5 | 3 | 7 | 5 |

## G. Chat Adapter (LLM)

```rust
struct UnifiedChatRequest {
    model: String,
    messages: Vec<Message>,
    max_tokens: Option<u32>,
    temperature: Option<f32>,
    top_p: Option<f32>,
    stream: Option<bool>,
    stop: Option<Vec<String>>,
    tools: Option<Vec<Tool>>,
    tool_choice: Option<ToolChoice>,
    thinking: Option<ThinkingConfig>,     // DeepSeek V4 / OpenAI Responses
    extra: Option<HashMap<String, Value>>,// 平台特有参数透传
}

trait ChatAdapter: Send + Sync {
    fn provider_type(&self) -> ProviderType;
    fn build_request(&self, req: &UnifiedChatRequest) -> Result<HttpRequest>;
    fn parse_response(&self, resp: HttpResponse) -> Result<ChatResponse>;
    fn supports_feature(&self, feature: ChatFeature) -> bool;
    fn list_models(&self) -> Vec<ModelInfo>;
}

enum ChatFeature { Thinking, Vision, ToolCalling, Streaming, JsonMode, FrequencyPenalty }
```

## H. OCR Adapter

```rust
struct UnifiedOcrRequest {
    images: Vec<ImageInput>,          // URL | Base64 | FilePath
    prompt: Option<String>,           // 任务提示: "OCR"/"Convert to markdown"
    output_format: OcrOutputFormat,   // TextOnly | WithBoxes | Markdown | Structured | FullArchive
    lang: Option<String>,             // PaddleOCR/MinerU
    use_angle_cls: Option<bool>,      // PaddleOCR
    use_table_rec: Option<bool>,      // MinerU/PP-Structure
    use_formula_rec: Option<bool>,    // MinerU/PP-Structure
    device: Option<Device>,           // CPU/GPU (官方)
    precision: Option<Precision>,     // FP32/FP16 (官方)
    max_tokens: Option<u32>,          // 聚合平台
    temperature: Option<f32>,         // 聚合平台
    stream: Option<bool>,
}

enum OcrOutputFormat { TextOnly, WithBoxes, Markdown, Structured, FullArchive }

struct UnifiedOcrResult {
    text: String,
    confidence: Option<f32>,
    blocks: Vec<OcrBlock>,
    markdown: Option<String>,
    metadata: OcrMetadata,
}

struct OcrBlock {
    block_type: BlockType,
    text: String,
    bbox: Option<Rect>,
    polygon: Option<Vec<Point>>,
    confidence: Option<f32>,
    children: Vec<OcrBlock>,
}

enum BlockType { Text, Table, Formula, Figure, Header, Footer, List, Title, Handwriting }

struct OcrMetadata { engine: String, page_count: u32, total_time_ms: u64, platform: Platform }

struct OcrCapabilities {
    text_detection: bool, text_recognition: bool, layout_analysis: bool,
    table_recognition: bool, formula_recognition: bool, chart_recognition: bool,
    multi_language: bool, confidence_scores: bool,
    pdf_input: bool, office_input: bool, streaming: bool,
}

trait OcrAdapter {
    fn engine_name(&self) -> &str;
    fn platform(&self) -> Platform;
    fn recognize(&self, req: &UnifiedOcrRequest) -> Result<UnifiedOcrResult>;
    fn capabilities(&self) -> OcrCapabilities;
}

enum Platform { Official, Aggregator, Local }

enum PlatformOcrRequest {
    ChatCompletions { model: String, messages: Vec<Message>, max_tokens: Option<u32>, temperature: Option<f32>, stream: Option<bool> },
    PaddleOcr { images: Vec<String>, lang: String, use_cls: bool, use_det: bool, use_rec: bool },
    MinerU { files: Vec<FilePath>, backend: String, lang_list: Vec<String>, formula_enable: bool, table_enable: bool, return_md: bool, return_json: bool },
}
```

## I. TTS Adapter

```rust
struct UnifiedTtsRequest {
    model: String,
    input: String,
    voice: VoiceSelection,
    speed: Option<f32>,                 // 0.25-4.0 (OpenAI 独有)
    instructions: Option<String>,       // 情感控制 (gpt-4o-mini-tts 独有)
    response_format: Option<AudioFormat>,
    sample_rate: Option<u32>,           // SiliconFlow 独有
    stream: Option<bool>,               // SiliconFlow 独有
}

enum VoiceSelection { Preset(String), ReferenceAudio(AudioRef) }
struct AudioRef { audio_url: String, audio_text: Option<String> }
enum AudioFormat { Mp3, Opus, Aac, Flac, Wav, Pcm }

trait TtsAdapter {
    fn synthesize(&self, req: &UnifiedTtsRequest) -> Result<Vec<u8>>;
    fn capabilities(&self) -> TtsCapabilities;
}

struct TtsCapabilities {
    max_input_length: u32, speed_control: bool, voice_instructions: bool,
    reference_audio: bool, streaming: bool, output_formats: Vec<AudioFormat>, platform: Platform,
}
```

## J. Image Generation Adapter

```rust
struct UnifiedImageRequest {
    model: String,
    prompt: String,
    negative_prompt: Option<String>,    // SF 独有
    size: ImageSize,
    count: Option<u8>,                  // Kolors/FLUX.schnell
    quality: Option<ImageQuality>,      // OpenAI 独有
    style: Option<ImageStyle>,          // OpenAI 独有
    steps: Option<u32>,                 // SF 独有
    guidance: Option<f32>,              // SF 独有
    seed: Option<u64>,                  // SF 独有
    reference_image: Option<ImageInput>,// Qwen-Image-Edit/FLUX-Kontext
    output_format: Option<ImageOutputFormat>,
}

struct ImageSize { width: u32, height: u32 }
enum ImageQuality { Standard, Hd }
enum ImageStyle { Vivid, Natural }
enum ImageOutputFormat { Png, Jpeg }

trait ImageAdapter {
    fn generate(&self, req: &UnifiedImageRequest) -> Result<ImageResult>;
    fn capabilities(&self) -> ImageCapabilities;
}

struct ImageCapabilities {
    max_prompt_length: u32, max_images_per_request: u8,
    negative_prompt: bool, quality_control: bool, style_control: bool,
    seed_control: bool, image_editing: bool, platform: Platform,
}

struct ImageResult { images: Vec<ImageOutput>, seed: Option<u64>, timings: Option<ImageTimings> }
struct ImageOutput { data: Vec<u8>, format: ImageOutputFormat, width: u32, height: u32 }
```

## K. 平台选型矩阵

| 场景 | Chat | OCR | TTS | Image |
|------|------|-----|-----|-------|
| **最高精度** | OpenAI GPT-5.5 | HunyuanOCR 1B | OpenAI gpt-4o-mini-tts | FLUX.2-pro |
| **最低成本** | DeepSeek V4-Flash | PaddleOCR v5 (CPU) | Fish-Speech (免费) | FLUX.1-schnell |
| **最快速度** | DeepSeek V4-Flash | DeepSeek-OCR 2 | OpenAI tts-1 | FLUX.1-schnell |
| **统一API** | SiliconFlow | SiliconFlow | SiliconFlow | SiliconFlow |
| **完整能力** | 官方 OpenAI/DeepSeek | PaddleOCR 官方 | OpenAI 官方 | FLUX 官方 |
| **本地部署** | vLLM Ollama | PaddleOCR MinerU | Fish-Speech CosyVoice | FLUX-dev |

## L. 迁移路径

```
阶段1: 统一接口定义 ✅  调研 + 架构 + 自动探测
阶段2: ProviderDiscovery 实现 → 对接 LLMManager
阶段3: Chat Adapter 统一 → 包装 RequestAdapter + ProviderAdapter
阶段4: OCR Adapter 扩展 → 对接 ocr_adapters/ 模块
阶段5: TTS + Image Adapter 新建
阶段6: 整合测试

---

## N. 现有代码集成映射

诊断发现项目已有完整的 LLM 适配器系统，新架构需对接而非替代。

### 现有架构组件

| 组件 | 路径 | 职责 |
|------|------|------|
| `RequestAdapter` trait | `llm_manager/adapters/mod.rs` | LLM 请求体参数适配 (14个实现) |
| `AdapterRegistry` | 同上 | 静态 HashMap, provider_type -> adapter |
| `get_adapter()` | 同上 | 3-tier 查找: scope > type > model_adapter |
| `ProviderAdapter` trait | `providers/mod.rs` | HTTP 请求构建 + 流解析 |
| `OcrAdapterFactory` | `ocr_adapters/mod.rs` | OCR 引擎工厂 (DeepSeek/Paddle) |
| `ApiConfig` struct | `llm_manager/mod.rs` | 模型配置 (含 enable_thinking, is_multimodal 等) |
| `VendorConfig` struct | 同上 | 供应商配置 (含 provider_type, base_url) |
| `BuiltinVendor` | `llm_manager/builtin_vendors.rs` | 10 个内置供应商定义 |
| `SiliconFlow vendor` | `vendors/siliconflow.rs` | SF 平台 3 个免费模型配置 |

### 新架构 → 现有代码 映射

```
ProviderDiscovery (NEW)
  ├── detect_api_type()    → 输出 ApiConfig.provider_type
  ├── fetch_models()       → 输出 ApiConfig.model + ModelProfile
  ├── probe_capabilities() → 输出 ApiConfig.is_multimodal/is_reasoning/is_embedding
  ├── probe_parameters()   → 输出 RequestAdapter 选择依据
  └── ProviderProfile      → 序列化为 VendorConfig + Vec<ApiConfig>

ChatAdapter (NEW, 包装层)
  ├── 复用 AdapterRegistry::get_adapter() → RequestAdapter
  ├── 复用 OpenAIAdapter::build_request() → ProviderRequest
  └── 新增 thinking/vision/streaming 能力查询接口

OcrAdapter (NEW, 扩展层)
  ├── 对接 ocr_adapters::OcrAdapterFactory
  ├── 新增 SiliconFlow OCR 路径 (ChatCompletions 模式)
  └── 新增 MinerU/HunyuanOCR/Surya 本地路径

TtsAdapter (NEW, 新建)
  ├── 对接 providers::ProviderAdapter HTTP 层
  └── 新增 AudioGeneration 请求构建

ImageAdapter (NEW, 新建)
  ├── 对接 providers::ProviderAdapter HTTP 层
  └── 新增 ImageGeneration 请求构建
```

### 现有 Adapter 已支持的供应商

```rust
// llm_manager/adapters/ 已实现 (14 个)
"openai"     → GenericOpenAIAdapter    // 默认
"general"    → GenericOpenAIAdapter    // 通用
"siliconflow"→ GenericOpenAIAdapter    // 聚合平台
"nvidia"     → GenericOpenAIAdapter    // NVIDIA NIM
"deepseek"   → DeepSeekAdapter        // 深度求索
"qwen"       → QwenAdapter            // 通义千问
"zhipu"      → ZhipuAdapter           // 智谱
"doubao"     → DoubaoAdapter          // 豆包
"moonshot"   → MoonshotAdapter        // 月之暗面
"kimi"       → MoonshotAdapter        // Kimi 别名
"ernie"      → ErnieAdapter           // 百度文心
"baidu"      → ErnieAdapter           // 百度别名
"anthropic"  → AnthropicAdapter       // Claude
"claude"     → AnthropicAdapter       // Claude 别名
"google"     → GeminiAdapter          // Google Gemini
"gemini"     → GeminiAdapter          // 别名
"xai"        → GrokAdapter            // xAI Grok
"grok"       → GrokAdapter            // 别名
"mistral"    → MistralAdapter         // Mistral
"minimax"    → MiniMaxAdapter         // MiniMax
"mimo"       → MimoAdapter            // Xiaomi MiMo
```

### 调用方接入点

| 调用方 | 路径 | 使用的适配器 |
|--------|------|------------|
| LLMManager::select_model() | `llm_manager/mod.rs` | ProviderAdapter + RequestAdapter |
| ChatV2 Pipeline | `chat_v2/pipeline/llm_adapter.rs` | RequestAdapter |
| OCR 服务 | `ocr_adapters/factory.rs` | OcrAdapterFactory |
| PDF OCR | `pdf_ocr_service.rs` | OcrAdapterFactory |
| 流式 Anki | `streaming_anki_service.rs` | ProviderAdapter |
| DSTU 导出 | `dstu/export/mod.rs` | ProviderAdapter |
| 作文批改 | `essay_grading/pipeline.rs` | ProviderAdapter |
| 翻译流水线 | `translation/pipeline.rs` | ProviderAdapter |
| 题库打分 | `qbank_grading/pipeline.rs` | ProviderAdapter |

### ProviderDiscovery 集成点

```
用户添加新供应商
  |
  v
ProviderDiscovery::discover(base_url, api_key)
  |
  v
ProviderProfile { api_type, models[], capabilities, parameters }
  |
  ├──→ VendorConfig { id, name, provider_type, base_url }
  ├──→ Vec<ApiConfig> { model, is_multimodal, is_reasoning, ... }
  └──→ AdapterRegistry 已包含该 provider_type 的适配器
```

**关键**: 新供应商探测结果自动生成 `VendorConfig` + `ApiConfig`，与现有 `LLMManager::save_vendor_configs()` 兼容。`AdapterRegistry` 已覆盖所有 14 个主要供应商类型，新发现的供应商如果使用 OpenAI 兼容 API (如 SiliconFlow 上的任何模型)，自动落入 `GenericOpenAIAdapter`。

---

## O. 现有适配器参数处理分析

> 基于 4 个核心适配器源码的深入研究

### ApiConfig 结构完整性

`ApiConfig` (llm_manager/mod.rs:1291) 已包含新架构所需的全部字段:

```rust
// 供应商识别
provider_type: Option<String>,    // "deepseek" | "qwen" | "openai" | ...
provider_scope: Option<String>,   // 实际托管方 (SiliconFlow 上的 DeepSeek → "deepseek")
model_adapter: String,            // 适配器选择键

// 能力标记
is_multimodal: bool,              // 视觉能力
is_reasoning: bool,               // 推理模型
is_embedding: bool,               // 嵌入模型
is_reranker: bool,                // 重排序
is_image_generation: bool,        // 图像生成
supports_tools: bool,             // 工具调用

// 推理配置 (4 种模式)
reasoning_effort: Option<String>, // OpenAI: none/minimal/low/medium/high/xhigh
thinking_enabled: bool,           // 通用: 是否启用思考
thinking_budget: Option<i32>,     // Qwen/SF: token 预算
enable_thinking: Option<bool>,    // 外部覆盖标志

// 采样参数
temperature: f32, top_p_override: Option<f32>,
frequency_penalty_override: Option<f32>, presence_penalty_override: Option<f32>,

// 供应商特有
repetition_penalty: Option<f32>,  // Qwen/Doubao 重复惩罚
reasoning_split: Option<bool>,    // MiniMax 思维分离
effort: Option<String>,           // Anthropic effort
verbosity: Option<String>,        // OpenAI verbosity
min_p: Option<f32>, top_k: Option<u32>,
headers: Option<HashMap<String,String>>, // 自定义请求头
```

### 四大适配器参数处理策略

#### 1. GenericOpenAIAdapter ("openai" / "general")
**策略**: 探测 + 移除不兼容参数
```
temperature/top_p + reasoning_effort="medium" → ❌ API报错
  → 适配器自动移除 temperature/top_p/logprobs
temperature/top_p + reasoning_effort="none" → ✅ 保留
非法 reasoning_effort (如 "foo") → ⚠️ 记录警告, 保留采样参数
```
**数据流**: `ApiConfig` → `apply_reasoning_config(&mut body)` → body 被就地修改

#### 2. DeepSeekAdapter ("deepseek")
**策略**: 版本感知 + 平台检测
```
模型版本: V3.1 / V3.2 / V4 / LegacyAlias
托管平台: Official (api.deepseek.com) | SiliconFlow | OtherHosted

V4 Official + thinking=true:
  → body.insert("thinking", { "type": "enabled" })
  → body.insert("reasoning_effort", "high"/"max")  (normalized)
  
V3.2 SiliconFlow + thinking=true:
  → body.insert("enable_thinking", true)
  → reasoning_effort "low"/"medium" → thinking_budget 2048/8192

V3.1 + tools present:
  → body.remove("thinking")  // 完全禁用
```
**关键属性**: 版本号从 model 字段解析 (`deepseek-ai/deepseek-v4-pro` → V4)

#### 3. MiniMaxAdapter ("minimax")
**策略**: 拒绝不支持的参数
```
enable_thinking=true → ❌ MiniMax API不支持 → body.remove("enable_thinking")
thinking_budget=4096 → ❌ 不支持 → body.remove("thinking_budget")
temperature=0.7     → ✅ 支持 → 保留
reasoning_split      → ✅ 独有 → body.insert("reasoning_split", true)
```
**关键属性**: MiniMax 不支持 enable_thinking，必须移除（否则 API 报错）

#### 4. QwenAdapter ("qwen")
**策略**: 平台双轨 + SiliconFlow 特殊处理
```
DashScope 官方: enable_thinking + thinking_budget
SiliconFlow 托管: 相同格式, 但 thinking_budget clamp(128, 32768)
```
**关键属性**: 既支持 thinking.budget 又支持 reasoning_effort

### 参数处理规则总结

| 规则 | 适配器 | 触发条件 | 动作 |
|------|--------|----------|------|
| 采样参数移除 | OpenAI/DeepSeek | reasoning_effort ≠ "none" | remove temperature/top_p/logprobs |
| 不兼容参数清除 | MiniMax | API 不支持 | remove enable_thinking/thinking_budget |
| 非法值防御 | GenericOpenAI | 未知 reasoning_effort | 记录警告, 保留参数 |
| 值范围限制 | DeepSeek/SF | budget < 128 | clamp(128, 32768) |
| 版本感知 | DeepSeek | model 包含 v4/v3.1 | 选择不同参数格式 |
| 平台检测 | DeepSeek/Qwen | base_url/provider_type | Official vs SiliconFlow 格式 |

### 新架构数据流兼容性

```
ProviderDiscovery::discover()
  → ProviderProfile { api_type, models[], capabilities, parameters }
  → 映射到 ApiConfig {
      provider_type: 从 api_type 推断,
      model: models[0].id,
      is_multimodal: capabilities.can_vision,
      is_reasoning: capabilities 中有 thinking 支持,
      is_embedding: capabilities 中有 embedding,
      thinking_enabled: parameters 中有 thinking=true,
    }
  → AdapterRegistry::get_adapter(provider_type)
  → GenericOpenAIAdapter (新供应商默认)
  → body 构建时自动应用参数处理规则
```

### 模型更新时的参数保护机制

```
1. 探测阶段 (ProviderDiscovery)
   /models API 返回最新模型列表 → 自动更新模型目录
   
2. 参数验证 (Adapter)
   未知参数 → GenericOpenAIAdapter 记录警告保留
   不支持参数 → 适配器主动移除 (如 MiniMax remove enable_thinking)
   非法值 → clamp/正常化 (如 SF budget 128-32768)
   
3. 降级路径 (AdapterRegistry)
   新模型 unknown → 尝试 GenericOpenAIAdapter
   完全不兼容 → 返回清晰错误信息
   
4. 用户侧 (UI)
   探测报告展示 supported/unsupported 参数
   用户可在保存前调整参数值
```

### 保留 vs 舍弃规则

| 场景 | 保留 | 舍弃 | 原因 |
|------|------|------|------|
| 新模型添加了 `reasoning_effort` | ✅ 保留 | — | OpenAI 兼容, 自动支持 |
| 新模型移除了 `frequency_penalty` | — | ✅ 移除 | apply_common_params 跳过 None |
| 未知参数名 | ✅ 保留 | — | 透传给 API (可能有用) |
| 已知但平台不支持 | — | ✅ 移除 | 防止 API 400 错误 |
| 边界值超出范围 | 转换为最近合法值 | — | clamp 到安全区间 |
| API 版本升级 | 探测后更新 ApiConfig | — | 定期重新探测 |

---

## M. 供应商自动探测系统

> 用户添加新供应商+API Key 时，自动探测 API 类型、模型模态和能力

### 探测流程（4 阶段）

```
用户输入: { base_url: "https://api.example.com/v1", api_key: "sk-xxx" }

Phase 1: 端点探测 — 顺序探测定性 API 类型
  /v1/chat/completions     → 200/400 → Chat API
  /v1/audio/speech         → 200/400 → TTS API
  /v1/images/generations   → 200/400 → Image Gen API
  /predict/ocr_system      → 200/400 → PaddleOCR API
  /file_parse              → 200/400 → MinerU API
  (任一命中即停止, 401=Auth错误记录)

Phase 2: 模型清单 — GET /models 解析 JSON, 提取模型ID列表

Phase 3: 能力探测 — 对前N个模型发送测试请求
  Text:   POST /chat/completions + text content
  Vision: POST /chat/completions + image_url content
  Audio:  POST /chat/completions + audio 检测
  Image:  POST /images/generations 探测

Phase 4: 参数发现 — 发送含边界值参数的请求
  测试 temperature, top_p, stream, tools, thinking, response_format
  200=supported, 400+param_name_in_error=unsupported

输出: ProviderProfile { api_type, models[], capabilities, supported_params[] }
```

### 核心数据结构

```rust
struct ProviderProfile {
    endpoint: String,
    api_type: ApiType,              // OpenAiChat | OpenAiTts | PaddleOcr | ...
    models: Vec<ModelEntry>,
    capabilities: ProviderCapabilities,
    parameters: Vec<ParameterSupport>,
    errors: Vec<String>,
}

enum ApiType { OpenAiChat, OpenAiTts, OpenAiImage, PaddleOcr, MinerU, AnthropicMessages }

struct ModelEntry {
    id: String, owned_by: String,
    modality: ModelModality,        // Text | Vision | Audio | Omni
}

struct ProviderCapabilities {
    can_chat: bool, can_vision: bool, can_audio: bool,
    can_image_gen: bool, can_embed: bool, can_rerank: bool,
}

struct ParameterSupport { name: String, supported: bool, note: Option<String> }
```

### 端点探测实现

```rust
async fn detect_api_type(&self, profile: &mut ProviderProfile, key: &str) {
    let probes = [
        ("/v1/chat/completions",  ApiType::OpenAiChat,  r#"{"model":"_p_","messages":[{"role":"user","content":"hi"}],"max_tokens":1}"#),
        ("/v1/audio/speech",      ApiType::OpenAiTts,   r#"{"model":"_p_","input":"hi","voice":"alloy"}"#),
        ("/v1/images/generations",ApiType::OpenAiImage, r#"{"model":"_p_","prompt":"test","size":"1024x1024"}"#),
        ("/predict/ocr_system",   ApiType::PaddleOcr,   r#"{"images":[""]}"#),
    ];
    for (path, tp, body) in &probes {
        let resp = self.http_client.post(format!("{}{}", profile.endpoint, path))
            .header("Authorization", format!("Bearer {}", key))
            .body(body).timeout(10s).send().await;
        match resp {
            Ok(r) if r.status() == 200 || r.status() == 400 => { profile.api_type = tp; return; }
            Ok(r) if r.status() == 401 => { profile.errors.push("Auth failed".into()); profile.api_type = tp; return; }
            _ => continue,
        }
    }
}
```

### 模型列表获取

```rust
async fn fetch_models(&self, profile: &mut ProviderProfile, key: &str) {
    let url = format!("{}/models", profile.endpoint);
    let resp = self.http_client.get(&url).bearer_auth(key).send().await?;
    let json: Value = resp.json().await?;
    for item in json["data"].as_array().unwrap_or(&vec![]).iter().take(N) {
        profile.models.push(ModelEntry {
            id: item["id"].as_str()?.into(),
            owned_by: item["owned_by"].as_str().unwrap_or("?").into(),
            modality: Unknown,
        });
    }
}
```

### 能力探测

```rust
async fn probe_chat(&self, profile: &Profile, key: &str, model: &str) -> bool {
    let body = json!({"model":model,"messages":[{"role":"user","content":"hi"}],"max_tokens":1});
    self.post(profile, key, "/chat/completions", &body).await.is_ok()
}

async fn probe_vision(&self, profile: &Profile, key: &str, model: &str) -> bool {
    let body = json!({"model":model,"messages":[{"role":"user","content":[
        {"type":"image_url","image_url":{"url":"data:image/png;base64,iVBORw0KGgo="}},
        {"type":"text","text":"describe"}
    ]}],"max_tokens":1});
    self.post(profile, key, "/chat/completions", &body).await.is_ok()
}
```

### 完整工作流

```
用户输入 base_url + api_key
  → Phase 1: 确定 API 类型
  → Phase 2: 获取模型列表
  → Phase 3: 探测每个模型模态
  → Phase 4: 探测参数支持
  → ProviderProfile 存储到配置
  → 自动选择对应 Adapter 实现
  → UI 根据 capabilities 启用/禁用功能选项
```
```
