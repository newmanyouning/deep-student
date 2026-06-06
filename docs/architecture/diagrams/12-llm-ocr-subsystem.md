# LLM 管理与 OCR 子系统 — 内部架构图

> 最后更新: 2026-06-06 | 源码路径: `src-tauri/src/llm_manager/`, `src-tauri/src/ocr_adapters/`, `src-tauri/src/vfs/pdf_processing_service.rs`

## 概述

LLM Manager 负责统一管理多供应商 LLM API 调用，OCR 适配器系统提供可扩展的多引擎 OCR 能力，两者共同支撑 PDF 预处理流水线。

---

## 图 1: LLM Manager 架构 (classDiagram)

```mermaid
classDiagram
    class LLMManager {
        - client: Client
        - db: Arc~Database~
        - file_manager: Arc~FileManager~
        - crypto_service: CryptoService
        - cancel_registry: Arc~Mutex~HashSet~String~~~
        - cancel_channels: Arc~Mutex~HashMap~String, watch::Sender~bool~~~
        - mcp_tool_cache: Arc~RwLock~Option~McpToolCache~~~
        - hooks_registry: Arc~Mutex~HashMap~String, Arc~dyn LLMStreamHooks~~~~
        + new(db, file_manager) Result~Self~
        + stream_chat_completion(config, messages, hooks, cancel_id) Result~()~
        + call_llm_api(config, messages, system_prompt) Result~Value~
        + get_api_configs() Vec~ApiConfig~
        + get_model2_config() ApiConfig
        + get_model_assignments() ModelAssignments
        + encrypt_api_key(key) Result~String~
        + decrypt_api_key(encrypted) Result~String~
        + cancel_stream(cancel_id)
        + user_preference_prompt() Option~String~
        + get_http_client() Client
    }

    class ApiConfig {
        + id: String
        + name: String
        + model: String
        + provider_type: Option~String~
        + provider_scope: Option~String~
        + model_adapter: String
        + api_base: Option~String~
        + api_key: String (encrypted)
        + enabled: bool
        + is_multimodal: bool
        + is_embedding: bool
        + is_reranker: bool
        + supports_tools: bool
        + supports_reasoning: bool
        + context_window: u32
        + max_output: u32
        + temperature: f32
        + top_p: f32
        + presence_penalty: f32
        + frequency_penalty: f32
        + reasoning_effort: Option~String~
    }

    class VendorConfig {
        + id: String
        + name: String
        + provider_type: String
        + api_base: String
        + is_cloud: bool
        + models: Vec~VendorModelInfo~
    }

    class ModelProfile {
        + id: String
        + name: String
        + model: String
        + provider_type: String
        + provider_scope: String
        + model_adapter: String
        + capabilities: CapabilityFlags
        + context_window: u32
        + max_output: u32
        + is_builtin: bool
    }

    class LLMStreamHooks {
        <<trait>>
        + on_stream_event(event: StreamEvent)
        + on_complete(result: StreamResult)
        + on_error(err: StreamError)
    }

    class ProviderAdapter {
        <<trait>>
        + format_request(messages, config, tools) Value
        + parse_response(response) StreamResult
        + supports_provider(provider_type) bool
        + apply_reasoning_config(body, config, enable_thinking)
        + should_remove_sampling_params(config) bool
        + apply_common_params(body, config)
    }

    class Model2Pipeline {
        + execute(config, messages, ctx) Result~StreamResult~
        - resolve_model_config() ResolvedModelConfig
        - build_request() Value
        - parse_stream_chunk() Option~String~
    }

    class ExamEngine {
        + segment_exam_images(images, config) Result~Vec~ExamSegmentationOutput~~
        + extract_text_from_segments(segments, config) Result~String~
    }

    class RAGExtension {
        + build_rag_context(query, top_k, rerank) String
        + search_and_format(query, config) Result~String~
    }

    LLMManager --> ApiConfig : 管理多个
    LLMManager --> LLMStreamHooks : 注册钩子
    LLMManager --> Model2Pipeline : 委托
    LLMManager --> ExamEngine : 题目集分割
    LLMManager --> RAGExtension : RAG 检索

    Model2Pipeline --> ProviderAdapter : 使用适配器
    ModelProfile --> ApiConfig : 预置参考
    VendorConfig --> ModelProfile : 包含模型列表

    note for LLMManager "核心入口点\n管理所有 LLM API 调用"
    note for ApiConfig "加密存储 API Key\n支持 50+ 模型配置"
    note for ProviderAdapter "适配不同供应商协议\nOpenAI / DeepSeek / Anthropic / Ollama / ..."
```

**关键源码引用**:
| 类型 | 文件 |
|------|------|
| `LLMManager` | `src-tauri/src/llm_manager/mod.rs` |
| `ApiConfig` | `src-tauri/src/llm_manager/config_types.rs` |
| `LLMStreamHooks` | `src-tauri/src/llm_manager/streaming.rs` |
| `Model2Pipeline` | `src-tauri/src/llm_manager/model2_pipeline.rs` |
| `ProviderAdapter` | `src-tauri/src/llm_manager/adapters/` |
| `VendorConfig` | `src-tauri/src/llm_manager/vendor_config_service.rs` |
| `ModelProfile` | `src-tauri/src/llm_manager/model_profile_service.rs` |
| `ExamEngine` | `src-tauri/src/llm_manager/exam_engine.rs` |

---

## 图 2: OCR 适配器插件系统 (classDiagram)

```mermaid
classDiagram
    class OcrAdapter {
        <<interface>>
        + engine_type() OcrEngineType
        + display_name() &'static str
        + supports_mode(mode: OcrMode) bool
        + build_prompt(mode: OcrMode) String
        + build_custom_prompt(custom_prompt, mode) String
        + parse_response(response, width, height, page_idx, img_path, mode) Result~OcrPageResult, OcrError~
        + recommended_max_tokens(mode) u32
        + recommended_temperature() f32
        + requires_high_detail() bool
        + get_extra_request_params() Option~Value~
        + recommended_repetition_penalty() Option~f64~
    }

    class OcrAdapterFactory {
        + create(engine_type: OcrEngineType) Arc~dyn OcrAdapter~
        + create_from_str(engine_type: &str) Arc~dyn OcrAdapter~
        + available_engines() Vec~OcrEngineType~
        + engine_info_list() Vec~OcrEngineInfo~
        + validate_model_for_engine(model, engine_type) bool
        + infer_engine_from_model(model) OcrEngineType
    }

    class OcrEngineType {
        <<enumeration>>
        DeepSeekOcr
        PaddleOcrVl
        PaddleOcrVlV1
        PaddleOcrApi
        Glm4vOcr
        GenericVlm
        SystemOcr
    }

    class OcrMode {
        <<enumeration>>
        Grounding
        FreeOcr
        Formula
        Table
        Chart
    }

    class DeepSeekOcrAdapter {
        + engine_type() DeepSeekOcr
        + build_prompt() grounding prompt with coordinate format
        + parse_response() 0-999 normalized coordinates
    }

    class PaddleOcrVlAdapter {
        + engine_type() PaddleOcrVl / PaddleOcrVlV1
        + build_prompt() standard OCR prompt
        + parse_response() pixel coordinates + markdown
    }

    class PaddleOcrApiAdapter {
        + engine_type() PaddleOcrApi
        + build_prompt() REST API formatter
        + parse_response() async job result
    }

    class Glm4vOcrAdapter {
        + engine_type() Glm4vOcr
        + build_prompt() bbox_2d prompt
        + parse_response() GLM format regions
        + recommended_max_tokens() 8192
    }

    class GenericVlmAdapter {
        + engine_type() GenericVlm
        + supports_mode() false for Grounding
        + build_prompt() simple text-only
    }

    class SystemOcrAdapter {
        + engine_type() SystemOcr
        + build_prompt() platform OCR
        + parse_response() OS native result
    }

    class OcrPageResult {
        + page_index: usize
        + image_path: String
        + image_width: u32
        + image_height: u32
        + regions: Vec~OcrRegion~
        + markdown_text: Option~String~
        + engine: OcrEngineType
        + mode: OcrMode
        + processing_time_ms: Option~u64~
    }

    class OcrRegion {
        + label: String
        + text: String
        + bbox_normalized: Option~[f64; 4]~
        + bbox_pixels: Option~[u32; 4]~
        + confidence: Option~f64~
        + raw_output: Option~String~
    }

    class OcrError {
        <<enumeration>>
        EngineError(String)
        ParseError(String)
        Timeout(String)
        UnsupportedMode
        NetworkError(String)
    }

    OcrAdapterFactory --> OcrAdapter : 创建
    OcrAdapterFactory --> OcrEngineType : 映射
    OcrAdapter --> OcrEngineType : 标识引擎
    OcrAdapter --> OcrMode : 支持的模式
    OcrAdapter --> OcrPageResult : 输出
    OcrAdapter --> OcrError : 错误

    DeepSeekOcrAdapter --|> OcrAdapter : 实现
    PaddleOcrVlAdapter --|> OcrAdapter : 实现
    PaddleOcrApiAdapter --|> OcrAdapter : 实现
    Glm4vOcrAdapter --|> OcrAdapter : 实现
    GenericVlmAdapter --|> OcrAdapter : 实现
    SystemOcrAdapter --|> OcrAdapter : 实现

    OcrPageResult --> OcrRegion : 包含
    OcrPageResult --> OcrEngineType : 引擎标识
    OcrPageResult --> OcrMode : 模式标识

    note for DeepSeekOcrAdapter "专业 OCR 模型\n支持 Grounding 坐标\n适合题目集识别"
    note for PaddleOcrVlAdapter "百度开源 OCR-VL\n109 种语言\n精度 94.5%\n完全免费"
    note for Glm4vOcrAdapter "智谱 GLM-4.6V\n106B MoE\n支持 bbox_2d 坐标"
```

**适配器支持矩阵**:

| 适配器 | Grounding | FreeOcr | Formula | Table | Chart | 免费 | 专用 OCR |
|--------|-----------|---------|---------|-------|-------|------|---------|
| `DeepSeekOcrAdapter` | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ |
| `PaddleOcrVlAdapter` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| `PaddleOcrApiAdapter` | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| `Glm4vOcrAdapter` | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ |
| `GenericVlmAdapter` | ❌ | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ |
| `SystemOcrAdapter` | ❌ | ✅ | ❌ | ❌ | ❌ | ✅ | ✅ |

**源码引用**: `src-tauri/src/ocr_adapters/mod.rs`, `factory.rs`, `deepseek.rs`, `paddle.rs`, `paddle_api.rs`

---

## 图 3: OCR 流水线生命周期 (stateDiagram)

```mermaid
stateDiagram-v2
    [*] --> Pending : 文件上传

    state PDF {
        Pending --> TextExtraction : PDF 文档
        TextExtraction --> PageRendering : 文本提取完成
        PageRendering --> PageCompression : 页面渲染完成
        PageCompression --> OcrProcessing : 页面压缩完成
    }

    state Image {
        Pending --> ImageCompression : 图片上传
        ImageCompression --> OcrProcessing : 图片压缩完成
    }

    OcrProcessing --> VectorIndexing : OCR 完成
    VectorIndexing --> Completed : 向量索引完成
    VectorIndexing --> CompletedWithIssues : 部分索引失败

    OcrProcessing --> Error : OCR 失败
    VectorIndexing --> Error : 索引失败
    PageRendering --> Error : 渲染失败
    PageCompression --> Error : 压缩失败

    Error --> Pending : 重试 (max_retries=3)
    CompletedWithIssues --> Pending : 重试失败阶段

    Completed --> [*]
    CompletedWithIssues --> [*]

    state Pending {
        [*] --> Queued : 入队
        Queued --> Processing : 开始处理
        Processing --> [*] : 取消/完成
    }

    state OcrProcessing {
        [*] --> InvokeOcr : 选择 OCR 引擎
        InvokeOcr --> ParseResult : API 返回
        ParseResult --> SaveToDb : 解析成功
        SaveToDb --> [*] : 保存完成
        InvocOcr --> RetryDelay : API 失败
        RetryDelay --> InvokeOcr : 重试 (max_retries=3)
        RetryDelay --> ErrorState : 超过重试次数
    }

    note right of OcrProcessing
        支持的引擎顺序:
        1. DeepSeekOcr (专业)
        2. PaddleOcrVl (免费)
        3. Glm4vOcr (通用)
        4. GenericVlm (回退)
    end note
```

**阶段详情**:

| 阶段 | 说明 | 源码方法 | 事件 |
|------|------|----------|------|
| `TextExtraction` | PDF 文本提取 (pdfium) | `stage_text_extraction()` | `PdfProcessingProgressEvent` |
| `PageRendering` | PDF 页面渲染为图片 | `stage_page_rendering()` | 同上 |
| `PageCompression` | 渲染图片 JPEG 压缩 | `stage_page_compression()` | 同上 |
| `ImageCompression` | 图片压缩 (阈值 > 1MB) | `stage_image_compression()` | 同上 |
| `OcrProcessing` | OCR 识别 (多引擎可选) | `stage_ocr_processing()` | `PdfProcessingProgressEvent` |
| `VectorIndexing` | 向量化 + LanceDB 存储 | `stage_vector_indexing()` | 同上 |
| `Completed` | 处理成功完成 | — | `PdfProcessingCompletedEvent` |
| `Error` | 处理失败 | — | `PdfProcessingErrorEvent` |

**Checkpoint/Resume 机制**:
- 每个阶段完成后写数据库 `processing_stage` 字段
- 应用启动时 `resume_pending_jobs()` 扫描未完成的处理任务
- 使用 `CancellationToken` 支持取消正在进行的任务
- 失败阶段记录 `ProcessingIssue`，支持定向重试
- 图片复用：PDF 的 OCR 阶段直接使用 PageRendering 阶段生成的图片

**源码引用**:
- 流水线主逻辑: `src-tauri/src/vfs/pdf_processing_service.rs`
- 事件类型: 同上 (32000+ 行)
- 检查点恢复: `resume_pending_jobs()` 方法
- 进度事件: `PdfProcessingProgressEvent`, `PdfProcessingCompletedEvent`, `PdfProcessingErrorEvent`

---

## 文件索引

| 文件 | 说明 |
|------|------|
| `src-tauri/src/llm_manager/mod.rs` | `LLMManager` — LLM 管理器主结构 |
| `src-tauri/src/llm_manager/config_types.rs` | `ApiConfig`, `VendorConfig`, `ModelProfile` 等类型 |
| `src-tauri/src/llm_manager/model2_pipeline.rs` | `Model2Pipeline` — 新一代流式管线 |
| `src-tauri/src/llm_manager/streaming.rs` | `LLMStreamHooks` 流式钩子 |
| `src-tauri/src/llm_manager/adapters/` | `ProviderAdapter` trait 及各供应商实现 |
| `src-tauri/src/llm_manager/builtin_vendors.rs` | 内置供应商配置 |
| `src-tauri/src/llm_manager/vendor_config_service.rs` | 供应商配置服务 |
| `src-tauri/src/llm_manager/model_profile_service.rs` | 模型配置文件服务 |
| `src-tauri/src/llm_manager/exam_engine.rs` | 题目集图片分割引擎 |
| `src-tauri/src/llm_manager/rag_extension.rs` | RAG 上下文构建扩展 |
| `src-tauri/src/llm_manager/parser.rs` | API 响应流解析 |
| `src-tauri/src/llm_manager/tool_call.rs` | 工具调用格式转换 |
| `src-tauri/src/ocr_adapters/mod.rs` | `OcrAdapter` trait + `Glm4vOcrAdapter` + `GenericVlmAdapter` |
| `src-tauri/src/ocr_adapters/factory.rs` | `OcrAdapterFactory` + `OcrEngineInfo` |
| `src-tauri/src/ocr_adapters/deepseek.rs` | `DeepSeekOcrAdapter` |
| `src-tauri/src/ocr_adapters/paddle.rs` | `PaddleOcrVlAdapter` |
| `src-tauri/src/ocr_adapters/paddle_api.rs` | `PaddleOcrApiAdapter` |
| `src-tauri/src/ocr_adapters/system_ocr/` | `SystemOcrAdapter` |
| `src-tauri/src/ocr_adapters/types.rs` | `OcrEngineType`, `OcrMode`, `OcrPageResult`, `OcrRegion`, `OcrError` |
| `src-tauri/src/vfs/pdf_processing_service.rs` | PDF/图片预处理流水线 |
