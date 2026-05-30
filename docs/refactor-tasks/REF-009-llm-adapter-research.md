# REF-009: LLM 适配器接口调研数据库

> 完成: 2026-05-30 11:00 CST | 来源: 官方文档 + API 参考 (web_search)
> 更新: 新增多模态视觉/视频/音频参数调研，硅基流动完整模型清单

## 供应商 API 参数矩阵

### 通用 Chat Completions 参数 (10个供应商交集)

| 参数 | OpenAI | DeepSeek | Qwen | GLM | Doubao | MiniMax | Kimi | SiliconFlow | NVIDIA | MiMo |
|------|--------|----------|------|-----|--------|---------|------|-------------|--------|------|
| `model` | ✅ R | ✅ R | ✅ R | ✅ R | ✅ R | ✅ R | ✅ R | ✅ R | ✅ R | ✅ R |
| `messages` | ✅ R | ✅ R | ✅ R | ✅ R | ✅ R | ✅ R | ✅ R | ✅ R | ✅ R | ✅ R |
| `max_tokens` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| `temperature` | ✅ | ✅(ignored in think) | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| `top_p` | ✅ | ✅(ignored in think) | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| `stream` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| `stop` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| `tools` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| `tool_choice` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| `frequency_penalty` | ✅ | ❌(V4 removed) | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| `presence_penalty` | ✅ | ❌(V4 removed) | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| `seed` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| `logprobs` | ✅ | ✅ | ✅ | ❌ | ❌ | ✅ | ✅ | ✅ | ❌ | ✅ |
| `top_logprobs` | ✅ | ✅ | ✅ | ❌ | ❌ | ✅ | ✅ | ✅ | ❌ | ✅ |
| `thinking` | ✅Responses | ✅V4 | ✅extra | ✅extra | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ |
| `reasoning_effort` | ✅Responses | ✅high/max | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ✅low/med/high |
| `response_format` | ✅ | ✅(json) | ✅ | ✅ | ❌ | ✅(json) | ✅ | ✅ | ❌ | ✅ |
| `parallel_tool_calls` | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ | ✅ | ✅ | ❌ | ✅ |
| `top_k` | ❌ | ❌ | ✅ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ✅ |
| `repetition_penalty` | ❌ | ❌ | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ |
| `stream_options` | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ | ✅ | ✅ | ❌ | ❌ |
| `n` | ✅ | ❌ | ✅ | ✅ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ |

图例: ✅=支持, ❌=不支持, R=必填, "ignored"=思考模式下被忽略, "extra"=需通过extra_body传递

### 供应商特有参数

| 供应商 | 特有参数 | 说明 |
|--------|----------|------|
| OpenAI | `instructions`, `reasoning`, `store`, `metadata` | Responses API 专有 |
| DeepSeek | `thinking.type`, `reasoning_effort` | 思考模式控制 |
| Qwen/阿里 | `enable_thinking`, `thinking_budget`, `enable_search`, `vl_high_resolution_images` | extra_body 传递 |
| GLM/智谱 | `thinking.type`, `do_sample`, `chat_template_kwargs.enable_thinking` | 多平台差异 |
| Doubao/火山 | `thinking` (多档深度) | Seed 2.0 Mini 支持4档 |
| MiMo | `min_p`, `search_mode`, `search_domain_filter`, `search_recency_filter` | 丰富搜索控制 |
| Kimi | `web_search_options`, `metadata`(16 pairs), `store` | 蒸馏/评估支持 |

## 模型清单 (按供应商)

### 1. OpenAI (openai)
| 模型ID | 类型 | 上下文 | 输入$/1M | 输出$/1M |
|--------|------|--------|----------|----------|
| gpt-5.5 | chat | 1,050K | $5.00 | $30.00 |
| gpt-5.5-pro | chat(pro) | 1,050K | $30.00 | $180.00 |
| gpt-5.4 | chat | 1,050K | $2.50 | $15.00 |
| gpt-5.4-mini | chat | 400K | $0.75 | $4.50 |
| gpt-5.4-nano | chat | 400K | $0.20 | $1.25 |
| gpt-4.1 | chat | 1,048K | $2.00 | $8.00 |
| gpt-4.1-mini | chat | 1,048K | $0.40 | $1.60 |
| gpt-4.1-nano | chat | 1M | $0.10 | $0.40 |
| o3 | reasoning | 200K | $2.00 | $8.00 |
| o4-mini | reasoning | 200K | $1.10 | $4.40 |
| gpt-5.3-codex | code | - | $1.75 | $14.00 |
| **协议**: Chat Completions + Responses API (新) |

### 2. DeepSeek (deepseek)
| 模型ID | 类型 | 上下文 | 输入$/1M | 输出$/1M | 备注 |
|--------|------|--------|----------|----------|------|
| deepseek-v4-pro | chat | 1M | $0.435 | $0.87 | 1.6T参数, 49B激活 |
| deepseek-v4-flash | chat | 1M | $0.14 | $0.28 | 284B, 13B激活 |
| **协议**: OpenAI Chat Completions + Anthropic Messages |
| **即将下线**: deepseek-chat, deepseek-reasoner (2026-07-24) |

### 3. 通义千问/阿里百炼 (qwen)
| 模型ID | 类型 | 上下文 | 备注 |
|--------|------|--------|------|
| qwen3.7-max | chat | 1M+ | 最新旗舰, Agent专精 |
| qwen3.6-max-preview | chat | - | GatedDeltaNet架构 |
| qwen3.5-plus | chat | - | 397B MoE, 思考默认开 |
| qwen3.5-flash | chat | - | 性价比 |
| qwen3-coder-plus | code | - | Coding Agent |
| qwq-plus | reasoning | - | 纯思考模式 |
| qwen-vl-max | vision | - | 多模态 |
| qwen3-omni-flash | omni | - | 全模态(语音+视觉) |
| **协议**: OpenAI Chat Completions + DashScope 原生 |

### 4. 智谱AI (zhipu)
| 模型ID | 类型 | 上下文 | 备注 |
|--------|------|--------|------|
| glm-5 | chat | 200K | 744B MoE, 旗舰Agent |
| glm-5-turbo | chat | 200K | OpenClaw增强 |
| glm-5.1 | chat | 200K | 全自治Agent |
| glm-4.7 | chat | 200K | Agentic Coding |
| glm-4.7-flash | chat | 200K | 免费开源 |
| glm-4.6 | chat | 200K | 均衡型 |
| **协议**: OpenAI Chat Completions |

### 5. 豆包/火山方舟 (doubao)
| 模型ID | 类型 | 上下文 | 输入¥/1M | 输出¥/1M |
|--------|------|--------|-----------|-----------|
| doubao-seed-2-0-pro | chat | 256K | ¥3.20 | ¥16.00 |
| doubao-seed-2-0-lite | chat | 256K | ¥0.60 | ¥3.60 |
| doubao-seed-2-0-mini | chat | 256K | ¥0.20 | ¥2.00 |
| doubao-seed-2-0-code | code | 256K | ¥3.20 | ¥16.00 |
| **协议**: OpenAI Chat Completions v3 |

### 6. MiniMax (minimax)
| 模型ID | 类型 | 上下文 | 输入$/1M | 输出$/1M |
|--------|------|--------|----------|----------|
| MiniMax-M2.7 | chat | - | - | - | 自进化LLM |
| MiniMax-M2.5 | chat | 196K | $0.15 | $1.15 |
| MiniMax-M2.1 | chat | 128K | - | - |
| MiniMax-M2 | chat | 128K | - | - |
| **协议**: OpenAI Chat Completions |

### 7. 月之暗面/Kimi (moonshot)
| 模型ID | 类型 | 上下文 | 输入$/1M | 输出$/1M |
|--------|------|--------|----------|----------|
| kimi-k2.5 | vision+reasoning | 256K | $0.60 | $3.00 |
| kimi-k2-thinking | reasoning | 256K | - | - |
| kimi-k2-thinking-turbo | reasoning | 256K | - | - |
| **协议**: OpenAI Chat Completions |
| **⚠️**: K2.5 计划 2026-05-30 停用 (部分平台) |

### 8. SiliconFlow/硅基流动 (siliconflow)
**聚合平台**: 托管 100+ 第三方模型，按类别:
- **Chat/Text**: DeepSeek系列, Qwen系列, GLM系列, MiniMax, Kimi, Llama, Mistral等
- **Vision**: Qwen-VL, InternVL, DeepSeek-VL等
- **Embedding**: BAAI/bge-m3, Qwen3-Embedding, BCE等
- **Reranker**: BAAI/bge-reranker-v2-m3等
- **Image Gen**: FLUX, Stable Diffusion等

| 类别 | 代表模型 | 输入$/1M | 输出$/1M |
|------|----------|----------|----------|
| Chat 免费 | Qwen2.5-7B, DeepSeek-R1-Distill等 | ¥0 | ¥0 |
| Chat 付费 | DeepSeek-V3, Qwen3-235B等 | ¥1-4 | ¥1-16 |
| Embedding | BAAI/bge-m3 | ¥0.7 | - |
| Rerank | BAAI/bge-reranker-v2-m3 | ¥1.0 | - |

### 9. NVIDIA NIM (nvidia)
| 模型ID | 类型 |
|--------|------|
| 通过 /models 端点动态拉取 | chat/embedding |
| **协议**: OpenAI Chat Completions |
| **特点**: 模型列表动态变化, 无 thinking 专用参数 |

### 10. Xiaomi MiMo (mimo)
| 模型ID | 类型 | 上下文 | 输入$/1M | 输出$/1M |
|--------|------|--------|----------|----------|
| xiaomi/mimo-v2.5-pro | chat | 1M | $1.00 | $3.00 |
| xiaomi/mimo-v2.5 | chat | 1.1M | $0.40 | $2.00 |
| **协议**: OpenAI Chat Completions |
| **特点**: MoE架构, 混合注意力, MIT开源 |

## 适配器接口设计建议

### 统一请求参数 (ChatCompletionsRequest)
```rust
struct ChatCompletionsRequest {
    // 必填
    model: String,
    messages: Vec<Message>,        // {role, content}
    
    // 通用可选
    max_tokens: Option<u32>,
    temperature: Option<f32>,
    top_p: Option<f32>,
    stream: Option<bool>,
    stop: Option<Vec<String>>,
    
    // 工具 (所有供应商均支持)
    tools: Option<Vec<Tool>>,
    tool_choice: Option<ToolChoice>,
    
    // 思考模式 (需适配器转换)
    thinking_config: Option<ThinkingConfig>,
    
    // 供应商特有 (通过 extra 透传)
    extra: Option<HashMap<String, Value>>,
}
```

### 适配器接口
```rust
trait LlmAdapter: Send + Sync {
    fn provider_type(&self) -> &str;
    fn base_url(&self, config: &ApiConfig) -> String;
    fn build_request(&self, req: ChatCompletionsRequest) -> Result<Request>;
    fn parse_response(&self, resp: Response) -> Result<ChatCompletionResponse>;
    fn supports_thinking(&self) -> bool;
    fn list_models(&self) -> Vec<ModelInfo>;
}
```

## 附录 A: 硅基流动完整模型定价清单

### Chat/Text 模型 (按价格排序)

| 模型ID | 类型 | 上下文 | 输入$/1M | 输出$/1M | 备注 |
|--------|------|--------|----------|----------|------|
| Qwen2.5-7B-Instruct | chat | 131K | 免费 | 免费 | 基础对话 |
| DeepSeek-R1-Distill-Qwen-7B | reasoning | 131K | 免费 | 免费 | 推理蒸馏 |
| DeepSeek-R1-Distill-Qwen-32B | reasoning | 131K | 免费 | 免费 | 推理蒸馏 |
| Qwen2.5-Coder-7B-Instruct | code | 131K | 免费 | 免费 | 代码生成 |
| BGE-M3 | chat | 8K | 免费 | 免费 | 通用 |
| Ling-mini-2.0 | chat | 131K | $0.07 | $0.28 | 16B MoE |
| Ling-flash-2.0 | chat | 131K | $0.14 | $0.57 | 100B MoE |
| GLM-4.5-Air | chat | 131K | $0.14 | $0.86 | 106B/12B act |
| Qwen3.5-9B | chat | - | $0.10 | $0.15 | 紧凑型 |
| GPT-OSS-20B | chat | 131K | $0.04 | $0.18 | OpenAI 开源 |
| GPT-OSS-120B | chat | 131K | $0.05 | $0.45 | OpenAI 开源 |
| Qwen3.5-35B-A3B | chat | - | $0.24 | $1.80 | 性价比 |
| Qwen3.5-27B | chat | - | $0.25 | $2.00 | Dense |
| Qwen3.5-122B-A10B | chat | - | $0.26 | $2.08 | 精英版 |
| DeepSeek-V3 | chat | 164K | $0.25 | $1.00 | 推理+工具 |
| DeepSeek-V3.1 | chat | 164K | $0.27 | $1.00 | 混合思考 |
| DeepSeek-V3.2-Exp | chat | 164K | $0.27 | $0.41 | 实验版 |
| DeepSeek-R1 | reasoning | 164K | $0.50 | $2.18 | 高级推理 |
| Qwen3.5-397B-A17B | chat | - | $0.39 | $2.34 | 旗舰 |
| GLM-4.5 | chat | 131K | $0.40 | $2.00 | 355B MoE |
| GLM-4.6 | chat | 205K | $0.50 | $1.90 | 200K ctx |
| Kimi-K2-Instruct-0905 | chat | 262K | $0.40 | $2.00 | 256K ctx |
| MiniMax-M2 | chat | 197K | $0.30 | $1.20 | 230B MoE |
| Ring-1T | chat | 131K | $0.57 | $2.28 | 1T params |
| Ling-1T | chat | 131K | $0.57 | $2.28 | 1T/50B act |

### Vision/Multimodal 模型

| 模型ID | 上下文 | 输入$/1M | 输出$/1M | 能力 |
|--------|--------|----------|----------|------|
| Qwen3-VL-8B-Instruct | 262K | $0.18 | $0.68 | 紧凑VL, OCR |
| Qwen3-VL-8B-Thinking | 262K | $0.18 | $2.00 | VL+思考 |
| Qwen3-VL-32B-Instruct | 262K | $0.20 | $0.60 | 百万像素图 |
| Qwen3-VL-32B-Thinking | 262K | $0.20 | $1.50 | VL+深度推理 |
| Qwen3-VL-30B-A3B-Instruct | 262K | $0.29 | $1.00 | MoE VL |
| Qwen3-VL-235B-A22B-Instruct | 262K | $0.30 | $1.50 | 旗舰MoE VL |
| Qwen3-VL-235B-A22B-Thinking | 262K | $0.45 | $3.50 | 旗舰VL+推理 |
| GLM-4.6V | 131K | $0.30 | $0.90 | 原生工具调用 |
| GLM-4.6V-Flash | 131K | 免费 | 免费 | **免费视觉** |
| GLM-4.5V | 66K | $0.14 | $0.86 | 基础视觉 |
| GLM-5V-Turbo | 205K | $1.20 | $4.00 | 视觉编码 |
| Gemma-4-31B | 262K | $0.13 | $0.40 | 文本+视觉+音频 |
| Gemma-4-26B-A4B | 262K | $0.12 | $0.40 | MoE 4B act |

### Embedding 模型

| 模型ID | 价格$/1M | 维度 |
|--------|----------|------|
| BAAI/bge-large-zh-v1.5 | ¥0.7 | 1024 |
| BAAI/bge-m3 | ¥0.7 | 1024 |
| Qwen3-Embedding-8B | ¥1.0 | 4096 |
| BCE-Embedding-base_v1 | ¥0.7 | 768 |
| BGE-Reranker-v2-m3 | ¥1.0 | rerank |

### Image Generation 模型

| 模型 | 单价 |
|------|------|
| Tongyi-MAI | $0.005/img |
| Stable Diffusion 3 | ¥0.035/img |
| Qwen-Image | $0.02/img |
| FLUX.1-dev | $0.03-0.06/img |

### Video Generation 模型

| 模型 | 单价 |
|------|------|
| Wan2.1/Wan2.2 | $0.21/video起 |

### Audio/Speech 模型

| 模型 | 类型 |
|------|------|
| Fish-Speech | TTS |
| CosyVoice | TTS/clone |
| SenseVoice-Small | ASR |

## 附录 B: 全平台多模态参数矩阵

### 视觉/图像输入参数

| 参数 | OpenAI | Qwen/阿里 | Kimi | GLM | Doubao | SiliconFlow |
|------|--------|-----------|------|-----|--------|-------------|
| `image_url` | ✅ base64/URL | ✅ base64/URL/file | ✅ base64/URL | ✅ | ✅ base64/URL/binary | ✅ |
| `detail`(分辨率) | ✅ auto/original | ❌(自动) | ❌ | ❌ | ✅ high/low/auto | ❌ |
| `max_images` | - | 256 | - | - | - | - |
| `max_videos` | - | 64 | - | - | - | - |
| `max_video_duration` | 3min(GPT-5) | 2hr | - | - | - | - |
| `max_video_size` | - | 2GB | - | - | - | - |
| `video_fps` | ❌ | ✅(可设) | ❌ | ❌ | ✅ 0.2-5 | ❌ |
| `image_format` | ❌ | ❌ | PNG/JPEG/WebP/GIF | - | ✅ JPEG/PNG/WEBP/GIF | ❌ |
| `text_verbosity` | ✅ GPT-5.4+ | ❌ | ❌ | ❌ | ❌ | ❌ |

### 视觉特有参数

| 供应商 | 特有参数 | 说明 |
|--------|----------|------|
| OpenAI | `detail: auto/original`, `text.verbosity: high` | GPT-5.4+ patches算法 |
| Qwen/阿里 | `vl_high_resolution_images`, `vl_enable_image_hw_output`, `ocr_options` | 1600万像素/图 |
| Kimi | `thinking.type: enabled/disabled` | 思考模式控制视觉推理深度 |
| GLM | 原生 Function Call (GLM-4.6V) | 视觉工具调用 |
| Doubao | `image_format`, `video_fps`, `source_type: binary/base64/url` | 火山视觉API |
| SiliconFlow | 托管 GLM-4.6V-Flash (**免费**), Qwen3-VL, Gemma-4 | 聚合平台 |

### 音频/语音参数

| 供应商 | 模型 | 能力 |
|--------|------|------|
| OpenAI | gpt-4o-audio, gpt-4o-transcribe | 原生音频, STT |
| Qwen/阿里 | qwen3-omni-flash | 全模态(语音+视觉) |
| SiliconFlow | Fish-Speech, CosyVoice, SenseVoice-Small | TTS, ASR(托管) |
| Doubao | Seed 2.0-lite | 19语种STT, 情绪/环境声 |

### 视频生成参数

| 供应商 | 模型 | 单价 |
|--------|------|------|
| SiliconFlow | Wan2.1/Wan2.2 | $0.21/视频起 |
| Doubao/火山 | Seedance 2.0 | ¥10-15/15秒视频 |

## 附录 C: OCR 引擎与文档解析模型

> 调研时间: 2026-05-30 | 来源: 各模型官方文档 + GitHub + HuggingFace

### OCR 引擎能力矩阵

| 引擎 | 参数量 | 架构 | 语言 | 输出格式 | 许可证 | GPU需求 |
|------|--------|------|------|----------|--------|---------|
| **PaddleOCR v5** | ~100MB/server | 检测+识别流水线 | 111 | 文本+坐标 | Apache 2.0 | CPU可用 |
| **PaddleOCR-VL** | 0.9B | VLM端到端 | 109 | Markdown/JSON | Apache 2.0 | 8GB+ |
| **HunyuanOCR** | 1B | 原生多模态VLM | 100+ | Markdown/HTML/LaTeX/JSON/Mermaid | 自定义 | 10GB+ |
| **DeepSeek-OCR 2** | 3B | MoE VLM | 100+ | Markdown/LaTeX/JSON | Apache 2.0 | 20GB+ |
| **Surya OCR 2** | 6.5B | VLM | 91 | HTML/JSON/Markdown | Apache 2.0 | 16GB+ |
| **MinerU** | pipeline/VLM/hybrid | 流水线 | 109+ | MD/JSON/HTML/ZIP | Apache 2.0 | CPU/GPU |
| **GOT-OCR 2** | 500M-1.2B | VLM | 多语言 | 文本/坐标 | MIT | 8GB+ |
| **Tesseract 5** | - | LSTM CNN | 116 | 文本+坐标 | Apache 2.0 | CPU |
| **EasyOCR** | - | CRAFT+pipeline | 80+ | 文本+坐标 | Apache 2.0 | CPU/GPU |

### MinerU 输入输出模型参数

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `backend` | enum | `hybrid-auto-engine` | pipeline / vlm-transformers / vlm-sglang-engine / vlm-vllm-engine / hybrid-auto-engine |
| `parse_method` | enum | `auto` | auto / txt / ocr |
| `lang_list` | List[str] | `["ch"]` | ch/en/korean/japan/chinese_cht/ar/th/auto |
| `formula_enable` | bool | `true` | 公式解析开关 |
| `table_enable` | bool | `true` | 表格解析开关 |
| `return_md` | bool | `true` | 返回Markdown |
| `return_middle_json` | bool | `false` | 返回中间JSON |
| `return_content_list` | bool | `false` | 返回结构化内容列表(含bbox) |
| `return_images` | bool | `false` | 返回图片base64 |
| `response_format_zip` | bool | `false` | true=zip包, false=JSON直返 |
| `start_page_id` | int | `0` | 起始页码 |
| `end_page_id` | int | `99999` | 结束页码 |

**输入格式**: PDF / PNG / JPG / WEBP / BMP / DOCX / PPTX / XLSX / HTML / TXT / URL  
**输出 type 字段**: text / image / table / list / header / footer / discarded / title / formula

### PaddleOCR 流水线模型参数

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `lang` | str | `ch` | ch/en/japan/korean/french/german/arabic等 |
| `text_detection_model_name` | str | `PP-OCRv5_server_det` | 检测模型 |
| `text_recognition_model_name` | str | `PP-OCRv5_server_rec` | 识别模型 |
| `use_doc_orientation_classify` | bool | `false` | 文档方向分类 |
| `use_doc_unwarping` | bool | `false` | 文档矫正 |
| `use_textline_orientation` | bool | `true` | 文本行方向 |
| `device` | str | 自动 | cpu/gpu:0/gpu:0,1/npu:0/xpu:0 |
| `engine` | str | `paddle_static` | paddle/paddle_static/paddle_dynamic/transformers |
| `enable_hpi` | bool | `false` | 高性能推理 |
| `use_tensorrt` | bool | `false` | TensorRT加速 |
| `precision` | str | `fp32` | fp32/fp16 |
| `enable_mkldnn` | bool | `true` | MKL-DNN CPU加速 |
| `cpu_threads` | int | `10` | CPU线程数 |

**输出**: JSON含 dt_polys(检测框) + rec_texts(识别文本) + rec_scores(置信度)

### Surya OCR 模型参数

| 参数 | 模块 | 说明 |
|------|------|------|
| `langs` | OCR | 语言代码列表, 支持90+ |
| `task_name` | OCR | ocr_with_boxes / ocr_without_boxes / block_without_boxes |
| `DETECTOR_BATCH_SIZE` | 检测 | 默认36 (16GB VRAM) |
| `RECOGNITION_BATCH_SIZE` | OCR | 默认512 (20GB VRAM) |
| `LAYOUT_BATCH_SIZE` | 版面 | 默认32 (7GB VRAM) |
| `TABLE_REC_BATCH_SIZE` | 表格 | 默认64 (10GB VRAM) |
| `COMPILE_RECOGNITION` | 编译 | true=JIT编译加速 |

**输出**: JSON含 bboxes(检测框) + text_lines/text/chars/words(多级文本) + layout 15标签

### DeepSeek-OCR 2 参数

| 参数 | 值 | 说明 |
|------|-----|------|
| 视觉Token数 | 256-1,120 | 10x压缩=256(97%准确率), 4x=1024(99%+) |
| 编码器V2 | SAM-base 80M + Qwen2-0.5B | 替代CLIP, 因果流注意力 |
| 解码器 | DeepSeek3B-MoE-A570M | 活跃参数~570M |
| 推理速度 | ~2,500 tok/s (A100) | 日均20万页 |
| 部署 | vLLM / Transformers | Vertex AI托管 |

### HunyuanOCR 参数

| 参数 | 值 | 说明 |
|------|-----|------|
| 动态分辨率 | (0-6)×768×768 + 1×1024×1024 | 自适应patching |
| 视觉Token | (0-6)×144 + 256 | 动态编码 |
| max_tokens | 16384 | 单次推理上限 |
| temperature | 0 | 推荐确定性输出 |
| 部署 | vLLM ≥0.12.0 | --gpu-memory-utilization 0.2 |

### OCR 精度对比 (OmniDocBench v1.5)

| 模型 | 参数量 | 总分 | 文本 | 公式 | 表格 |
|------|--------|------|------|------|------|
| **HunyuanOCR** | 1B | **94.10** | 94.73 | 91.81 | 85.21 |
| PaddleOCR-VL | 0.9B | 92.86 | 91.22 | 90.89 | 72.19 |
| DeepSeek-OCR 2 | 3B | 91.09 | - | +6.17% | +2.5% |
| Qwen3-VL-235B | 235B | 89.15 | 88.14 | 86.21 | 79.69 |
| Gemini-2.5-pro | - | 88.03 | 85.92 | 85.71 | 80.59 |

### 选型建议

| 场景 | 推荐 |
|------|------|
| 轻量CPU部署 | PaddleOCR v5 (PP-OCRv5) |
| 轻量GPU VLM | PaddleOCR-VL (0.9B) / HunyuanOCR (1B) |
| 高精度文档解析 | HunyuanOCR (1B, SOTA总分) |
| 高压缩比RAG | DeepSeek-OCR 2 (10x压缩97%精度) |
| 英文场景 | Surya OCR 2 (91语言, 87.2%) |
| 多语言+公式+表格 | MinerU (hybrid 引擎) |

## 附录 D: 官方平台 vs 聚合平台 API 差异

> 调研时间: 2026-05-30 | 核心发现: 聚合平台通过 OpenAI Chat Completions 统一接口掩盖了底层差异

### 1. PaddleOCR: 官方 vs SiliconFlow

| 维度 | 官方 PaddleOCR | SiliconFlow 托管 |
|------|---------------|-----------------|
| **API 端点** | `POST /predict/ocr_system` | `POST /v1/chat/completions` |
| **协议格式** | 自定义 JSON `{"images": ["base64..."]}` | OpenAI Vision Chat Completions |
| **图片编码** | 必须 **Base64 字符串** (纯文本 JSON) | Base64 data URI 或 HTTP URL |
| **请求结构** | `{"images": ["base64_str"]}` | `{"model":"...","messages":[{"role":"user","content":[{"type":"image_url",...}]}]}` |
| **文本提示** | ❌ 不支持 | ✅ `"text": "<image>\nOCR this image."` |
| **输出格式** | `{"results":[{"text":"...","confidence":0.98,"text_region":[[x,y],...]}]}` | `{"choices":[{"message":{"content":"markdown_text..."}}]}` |
| **检测框坐标** | ✅ 每条文本有 `text_region` 四边形坐标 | ❌ 仅输出纯文本/Markdown (独立于检测) |
| **置信度分数** | ✅ 每条文本有 `confidence` | ❌ 通常不返回 |
| **参数** | `lang`, `cls`, `det`, `rec`, `use_gpu` | `max_tokens`, `temperature`, `top_p`, `stream` |
| **管道控制** | 独立控制检测/识别/分类模块 | ❌ 端到端，无法控制中间步骤 |
| **部署依赖** | PaddleHub Serving / FastAPI Docker | 无需部署，API Key 即可 |

**核心差异**: 官方 API 返回结构化结果(文本+坐标+置信度)，适合精确位置标注场景；SiliconFlow 统一为 OpenAI Vision 格式，输出 Markdown 文本，适合 RAG/文档理解场景。

### 2. HunyuanOCR: 官方 vs SiliconFlow

| 维度 | 官方 vLLM 服务 | SiliconFlow 托管 |
|------|---------------|-----------------|
| **API 端点** | `POST /v1/chat/completions` | `POST /v1/chat/completions` |
| **协议格式** | OpenAI Chat Completions ✅ | OpenAI Chat Completions ✅ |
| **图片编码** | Base64 data URI / URL | Base64 data URI / URL |
| **max_tokens** | 最高 16384 | 平台限制 (通常 4096-8192) |
| **temperature** | 推荐 0 (确定性输出) | 推荐 0 |
| **max_tokens 默认** | 无默认(需显式设置) | 取决于模型配置 |
| **输出格式** | `choices[0].message.content` (Markdown/HTML/JSON/Mermaid) | `choices[0].message.content` (通常仅 Markdown) |
| **多任务 Prompt** | 支持 (检测/解析/提取/翻译) | 支持 (但可能功能受限) |
| **坐标输出** | ✅ 文字检测模式输出坐标 | ⚠️ 取决于 Prompt |
| **硬件** | GPU≥10GB (FP16) | 无需关注 |

**核心差异**: 官方和 SiliconFlow 使用**相同的 OpenAI 协议**，主要差异在于 max_tokens 限制和输出能力。HunyuanOCR 的 Prompt 高度可编程，SiliconFlow 版本可能对某些 Prompt 模板支持不完整。

### 3. DeepSeek-OCR 2: 官方 vs SiliconFlow

| 维度 | 官方 vLLM 服务 | SiliconFlow 托管 |
|------|---------------|-----------------|
| **API 端点** | `POST /v1/chat/completions` | `POST /v1/chat/completions` |
| **协议格式** | OpenAI Chat Completions ✅ | OpenAI Chat Completions ✅ |
| **Vision Token** | 256-1120 (可控) | 平台固定 (通常 1024) |
| **Prompt 模板** | `<image>\n<|grounding|>Convert to markdown.` | 相同 |
| **PDF 输入** | ✅ Base64 data URI | ✅ Base64 data URI |
| **stream** | ✅ 支持 | ✅ 支持 |
| **max_tokens** | 无限制 (模型自动) | 平台限制 |
| **价格** | 免费 (开源) | 限时免费 |

**核心差异**: 几乎无差异。DeepSeek-OCR 原生就是 OpenAI 兼容的，SiliconFlow 仅做路由和容量管理。

### 4. 通用聚合平台范式

#### SiliconFlow 统一格式 (所有模型)

```
POST https://api.siliconflow.cn/v1/chat/completions
Authorization: Bearer <API_KEY>
Content-Type: application/json

{
  "model": "deepseek-ai/DeepSeek-OCR",     // 或 tencent/HunyuanOCR, PaddleOCR-VL 等
  "messages": [{
    "role": "user",
    "content": [
      {"type": "image_url", "image_url": {"url": "https://..."}},
      {"type": "text", "text": "<prompt>"}
    ]
  }],
  "max_tokens": 4096,
  "temperature": 0,
  "stream": false
}
```

#### 官方平台各异格式

| 平台 | 端点 | 协议 | 图片格式 | 参数传输 |
|------|------|------|----------|----------|
| PaddleOCR | `/predict/ocr_det` | 自定义 JSON | Base64 字符串 | POST body JSON |
| MinerU | `/file_parse` | multipart/form-data | 文件上传 | Form params |
| HunyuanOCR | `/v1/chat/completions` | OpenAI Chat | Base64/URL | JSON body |
| DeepSeek-OCR | `/v1/chat/completions` | OpenAI Chat | Base64/URL | JSON body |
| Surya | Python SDK only | Python API | PIL Image/Path | Function args |
| vLLM serve | `/v1/chat/completions` | OpenAI Chat | Base64/URL | JSON body |

### 5. 参数映射表 (官方 → 聚合)

#### PaddleOCR 官方 → SiliconFlow
```
官方 lang='ch'          → 无法映射 (端到端, Prompt中隐含)
官方 cls=True/det=True  → 无法映射 (管道控制丢失)
官方 use_gpu/gpu_id     → 无需关注 (云端托管)
保留 capability          → max_tokens, temperature, stream
```

#### PaddleOCR 官方输出 → SiliconFlow 输出
```
官方 results[i].text                → choices[0].message.content (合并为Markdown)
官方 results[i].confidence          → ❌ 丢失
官方 results[i].text_region         → ❌ 丢失
官方 results[i].text_region[4点坐标] → ❌ 丢失
```

### 6. 适配器设计影响

#### 参数标准化层
```rust
struct OcrRequest {
    // 通用参数 (所有平台)
    image: ImageInput,       // URL | base64 | Path
    prompt: Option<String>,  // 任务提示
    
    // 聚合平台参数 (OpenAI 兼容)
    max_tokens: Option<u32>,
    temperature: Option<f32>,
    stream: Option<bool>,
    
    // 本地部署参数 (PaddleOCR 专用)
    lang: Option<String>,
    use_angle_cls: Option<bool>,
    use_gpu: Option<bool>,
    
    // 输出偏好
    output_format: OcrOutputFormat, // TextOnly | WithBoxes | Markdown | JSON
}
```

#### 输出标准化
```rust
struct OcrResult {
    text: String,                    // 纯文本 (所有平台)
    confidence: Option<f32>,         // 官方的置信度
    blocks: Vec<OcrTextBlock>,      // 结构化块 (含坐标)
    markdown: Option<String>,        // 聚合平台的Markdown输出
}

struct OcrTextBlock {
    text: String,
    bbox: Option<[f32; 4]>,         // [x1,y1,x2,y2]
    polygon: Option<Vec<[f32;2]>>,  // 四边形坐标
    confidence: Option<f32>,
    block_type: Option<String>,      // text/table/formula/figure
}
```

### 7. 选型建议

| 场景 | 推荐 | 理由 |
|------|------|------|
| 需要精确坐标定位 | PaddleOCR 官方 | 唯一返回 4 边形坐标 + 逐字置信度 |
| RAG/文档理解 | SiliconFlow (HunyuanOCR/DeepSeek-OCR) | 端到端 Markdown 输出，零部署成本 |
| 混合流水线 (检测+识别+版面+表格) | PaddleOCR v5 + PP-StructureV3 | CPU 可用，完整模块控制 |
| 开源最低部署成本 | HunyuanOCR vLLM 1B | 单卡 RTX 4090 即可，SOTA 精度 |
| 多格式输入 (PDF/Office) | MinerU | 9 种输入格式，hybrid 引擎 |
| 统一 API (多模型切换) | SiliconFlow | OpenAI 兼容，一键切换模型 |

