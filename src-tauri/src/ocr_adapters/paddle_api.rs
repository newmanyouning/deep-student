//! PaddleOCR REST API 适配器
//!
//! 直接调用 PaddleOCR AI Studio REST API 进行 OCR 解析。
//! 与 `paddle.rs` (VLM 适配器) 不同，本适配器使用独立的 job-based REST API，
//! 支持全部 4 种模型：PaddleOCR-VL-1.6/1.5, PP-OCRv5, PP-StructureV3。
//!
//! ## 工作流程
//!
//! 1. 提交文件/URL 创建 OCR job
//! 2. 轮询 job 状态直到完成
//! 3. 下载 JSONL 结果
//! 4. 解析 JSONL 为 `OcrPageResult`
//!
//! ## 与 VLM 适配器的区别
//!
//! - `paddle.rs` (PaddleOcrVlAdapter): 通过 OpenAI 兼容 API 调用 PaddleOCR-VL 模型，
//!   使用 `build_prompt()` + `parse_response()` 的标准 VLM 流程
//! - `paddle_api.rs` (PaddleOcrApiAdapter): 直接调用 AI Studio REST API，
//!   使用 job-based 异步流程，支持更多模型变体

use super::{OcrAdapter, OcrEngineType, OcrError, OcrMode, OcrPageResult, OcrRegion};
use crate::paddleocr_api;
use async_trait::async_trait;

/// PaddleOCR REST API 适配器
///
/// 使用 `PaddleOcrApiClient` 直接调用 AI Studio REST API。
/// 此适配器的 `build_prompt()` 返回空字符串（REST API 不需要 prompt），
/// 实际的 OCR 调用通过 `call_api()` 方法完成。
///
/// ## 配置
///
/// - API Token: 通过 `set_api_token()` 设置（对应 AI Studio 的 bearer token）
pub struct PaddleOcrApiAdapter {
    engine: OcrEngineType,
    api_token: Option<String>,
}

impl PaddleOcrApiAdapter {
    pub fn new() -> Self {
        Self {
            engine: OcrEngineType::PaddleOcrApi,
            api_token: None,
        }
    }

    /// 设置 API token（AI Studio bearer token）
    pub fn with_token(token: String) -> Self {
        Self {
            engine: OcrEngineType::PaddleOcrApi,
            api_token: Some(token),
        }
    }

    /// 获取 API token
    pub fn api_token(&self) -> Option<&str> {
        self.api_token.as_deref()
    }

    /// 设置 API token（用于在创建后配置）
    pub fn set_api_token(&mut self, token: String) {
        self.api_token = Some(token);
    }

    /// 使用 REST API 执行 OCR（文件路径模式）
    ///
    /// 上传文件到 AI Studio，提交 OCR job，轮询完成后解析结果。
    pub async fn call_api_file(
        &self,
        file_path: &str,
        model: &str,
    ) -> Result<PaddleOcrApiFileResult, OcrError> {
        let token = self
            .api_token
            .as_ref()
            .ok_or_else(|| OcrError::Configuration("PaddleOCR API token not set".to_string()))?;

        let client = paddleocr_api::PaddleOcrApiClient::new(token.clone());
        let result = client
            .ocr_file(file_path, model)
            .await
            .map_err(|e| OcrError::Api {
                status: 0,
                message: format!("PaddleOCR API 调用失败: {}", e),
            })?;

        Ok(PaddleOcrApiFileResult {
            pages: result.pages,
            total_pages: result.total_pages,
            model: result.model,
        })
    }

    /// 使用 REST API 执行 OCR（URL 模式）
    ///
    /// 提交在线文件 URL 到 AI Studio，不需要上传文件。
    pub async fn call_api_url(
        &self,
        file_url: &str,
        model: &str,
    ) -> Result<PaddleOcrApiFileResult, OcrError> {
        let token = self
            .api_token
            .as_ref()
            .ok_or_else(|| OcrError::Configuration("PaddleOCR API token not set".to_string()))?;

        let client = paddleocr_api::PaddleOcrApiClient::new(token.clone());
        let result = client
            .ocr_url(file_url, model)
            .await
            .map_err(|e| OcrError::Api {
                status: 0,
                message: format!("PaddleOCR API 调用失败: {}", e),
            })?;

        Ok(PaddleOcrApiFileResult {
            pages: result.pages,
            total_pages: result.total_pages,
            model: result.model,
        })
    }
}

impl Default for PaddleOcrApiAdapter {
    fn default() -> Self {
        Self::new()
    }
}

/// REST API 返回的轻量结果（避免依赖内部类型）
pub struct PaddleOcrApiFileResult {
    pub pages: Vec<paddleocr_api::PaddleOcrPage>,
    pub total_pages: u32,
    pub model: String,
}

impl PaddleOcrApiFileResult {
    /// 转换为 OCR 适配器通用的 `OcrPageResult` 列表
    pub fn into_ocr_page_results(self, mode: OcrMode) -> Vec<OcrPageResult> {
        self.pages
            .into_iter()
            .map(|page| PaddleOcrApiAdapter::convert_page(page, mode))
            .collect()
    }

    /// 提取所有页面的文本内容
    pub fn extract_text(&self) -> String {
        let mut texts = Vec::new();
        for page in &self.pages {
            let text = page.markdown_text.trim();
            if !text.is_empty() {
                texts.push(text.to_string());
            }
        }
        texts.join("\n\n---\n\n")
    }

    /// 转换为 (text, regions) 元组（与 `test_ocr_with_engine` 兼容）
    pub fn into_text_and_regions(self, mode: OcrMode) -> (String, Vec<OcrRegion>) {
        let mut all_text = String::new();
        let mut regions = Vec::new();

        for page in self.pages {
            let text = page.markdown_text.trim();
            if !text.is_empty() {
                if !all_text.is_empty() {
                    all_text.push_str("\n\n");
                }
                all_text.push_str(text);
            }

            for img in &page.images {
                regions.push(OcrRegion {
                    label: format!("image_{}", img.name),
                    text: String::new(),
                    bbox_normalized: None,
                    bbox_pixels: None,
                    confidence: None,
                    raw_output: Some(img.url.clone()),
                });
            }
        }

        if regions.is_empty() {
            regions.push(OcrRegion {
                label: "document".to_string(),
                text: all_text.clone(),
                bbox_normalized: None,
                bbox_pixels: None,
                confidence: None,
                raw_output: None,
            });
        }

        if all_text.is_empty() {
            all_text = format!("[PaddleOCR API] {} pages processed", self.total_pages);
        }

        (all_text, regions)
    }
}

#[async_trait]
impl OcrAdapter for PaddleOcrApiAdapter {
    fn engine_type(&self) -> OcrEngineType {
        self.engine
    }

    fn supports_mode(&self, mode: OcrMode) -> bool {
        // REST API 不支持 grounding 坐标输出
        matches!(mode, OcrMode::FreeOcr)
    }

    fn build_prompt(&self, _mode: OcrMode) -> String {
        // REST API 不需要 prompt，返回空字符串
        String::new()
    }

    fn recommended_max_tokens(&self, _mode: OcrMode) -> u32 {
        // REST API 不限制 token
        0
    }

    fn parse_response(
        &self,
        response: &str,
        _image_width: u32,
        _image_height: u32,
        page_index: usize,
        _image_path: &str,
        _mode: OcrMode,
    ) -> Result<OcrPageResult, OcrError> {
        // 解析 JSONL 格式响应（单行）
        if response.trim().is_empty() {
            return Ok(OcrPageResult {
                page_index,
                image_path: String::new(),
                image_width: 0,
                image_height: 0,
                regions: vec![],
                markdown_text: None,
                engine: self.engine,
                mode: OcrMode::FreeOcr,
                processing_time_ms: None,
            });
        }

        // 尝试解析为 JSONL line
        // JSONL 格式: {"result": {"layoutParsingResults": [...], "ocrResults": [...]}}
        let is_vl = response.contains("layoutParsingResults");

        if is_vl {
            // VL 系列响应处理
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(response) {
                let mut markdown_text = String::new();
                let mut regions = Vec::new();

                if let Some(result) = parsed.get("result") {
                    if let Some(lpr) = result.get("layoutParsingResults").and_then(|v| v.as_array()) {
                        for (i, item) in lpr.iter().enumerate() {
                            if let Some(md) = item.get("markdown") {
                                if let Some(text) = md.get("text").and_then(|v| v.as_str()) {
                                    if !markdown_text.is_empty() {
                                        markdown_text.push_str("\n\n");
                                    }
                                    markdown_text.push_str(text);
                                }
                                if let Some(images) = md.get("images").and_then(|v| v.as_object()) {
                                    for (name, url) in images {
                                        regions.push(OcrRegion {
                                            label: format!("image_{}", name),
                                            text: String::new(),
                                            bbox_normalized: None,
                                            bbox_pixels: None,
                                            confidence: None,
                                            raw_output: url.as_str().map(|s| s.to_string()),
                                        });
                                    }
                                }
                            }
                        }
                    }
                }

                if regions.is_empty() && !markdown_text.is_empty() {
                    regions.push(OcrRegion {
                        label: "document".to_string(),
                        text: markdown_text.clone(),
                        bbox_normalized: None,
                        bbox_pixels: None,
                        confidence: None,
                        raw_output: None,
                    });
                }

                return Ok(OcrPageResult {
                    page_index,
                    image_path: String::new(),
                    image_width: 0,
                    image_height: 0,
                    regions,
                    markdown_text: Some(markdown_text),
                    engine: self.engine,
                    mode: OcrMode::FreeOcr,
                    processing_time_ms: None,
                });
            }
        } else if response.contains("ocrResults") || response.contains("ocrImage") {
            // PP-OCRv5 响应处理
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(response) {
                let mut regions = Vec::new();
                if let Some(result) = parsed.get("result") {
                    if let Some(ocr_results) = result.get("ocrResults").and_then(|v| v.as_array()) {
                        for item in ocr_results {
                            if let Some(url) = item.get("ocrImage").and_then(|v| v.as_str()) {
                                regions.push(OcrRegion {
                                    label: "ocr_image".to_string(),
                                    text: String::new(),
                                    bbox_normalized: None,
                                    bbox_pixels: None,
                                    confidence: None,
                                    raw_output: Some(url.to_string()),
                                });
                            }
                        }
                    }
                }
                return Ok(OcrPageResult {
                    page_index,
                    image_path: String::new(),
                    image_width: 0,
                    image_height: 0,
                    regions,
                    markdown_text: None,
                    engine: self.engine,
                    mode: OcrMode::FreeOcr,
                    processing_time_ms: None,
                });
            }
        }

        // 回退：作为纯文本处理
        Ok(OcrPageResult {
            page_index,
            image_path: String::new(),
            image_width: 0,
            image_height: 0,
            regions: vec![OcrRegion {
                label: "document".to_string(),
                text: response.trim().to_string(),
                bbox_normalized: None,
                bbox_pixels: None,
                confidence: None,
                raw_output: Some(response.to_string()),
            }],
            markdown_text: Some(response.trim().to_string()),
            engine: self.engine,
            mode: OcrMode::FreeOcr,
            processing_time_ms: None,
        })
    }
}

impl PaddleOcrApiAdapter {
    /// 将 `PaddleOcrPage` 转换为 `OcrPageResult`
    pub fn convert_page(page: paddleocr_api::PaddleOcrPage, mode: OcrMode) -> OcrPageResult {
        let regions: Vec<OcrRegion> = page
            .images
            .iter()
            .map(|img| OcrRegion {
                label: format!("page_image_{}", img.name),
                text: String::new(),
                bbox_normalized: None,
                bbox_pixels: None,
                confidence: None,
                raw_output: Some(img.url.clone()),
            })
            .collect();

        OcrPageResult {
            page_index: page.page_index as usize,
            image_path: String::new(),
            image_width: 0,
            image_height: 0,
            regions,
            markdown_text: Some(page.markdown_text),
            engine: OcrEngineType::PaddleOcrApi,
            mode,
            processing_time_ms: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adapter_engine_type() {
        let adapter = PaddleOcrApiAdapter::new();
        assert_eq!(adapter.engine_type(), OcrEngineType::PaddleOcrApi);
    }

    #[test]
    fn test_supports_only_free_ocr() {
        let adapter = PaddleOcrApiAdapter::new();
        assert!(adapter.supports_mode(OcrMode::FreeOcr));
        assert!(!adapter.supports_mode(OcrMode::Grounding));
        assert!(!adapter.supports_mode(OcrMode::Formula));
        assert!(!adapter.supports_mode(OcrMode::Table));
        assert!(!adapter.supports_mode(OcrMode::Chart));
    }

    #[test]
    fn test_build_prompt_empty() {
        let adapter = PaddleOcrApiAdapter::new();
        assert!(adapter.build_prompt(OcrMode::FreeOcr).is_empty());
    }

    #[test]
    fn test_parse_empty_response() {
        let adapter = PaddleOcrApiAdapter::new();
        let result = adapter
            .parse_response("", 100, 100, 0, "", OcrMode::FreeOcr)
            .unwrap();
        assert_eq!(result.page_index, 0);
    }

    #[test]
    fn test_parse_vl_jsonl_response() {
        let adapter = PaddleOcrApiAdapter::new();
        let jsonl = r#"{"result":{"layoutParsingResults":[{"markdown":{"text":"# Hello\n\nWorld","images":{"fig1":"https://example.com/img1.png"}}}]}}"#;
        let result = adapter
            .parse_response(jsonl, 100, 100, 0, "", OcrMode::FreeOcr)
            .unwrap();
        assert!(result.markdown_text.as_deref().unwrap_or("").contains("Hello"));
        assert_eq!(result.regions.len(), 1);
    }

    #[test]
    fn test_set_api_token() {
        let mut adapter = PaddleOcrApiAdapter::new();
        assert!(adapter.api_token().is_none());
        adapter.set_api_token("test-token".to_string());
        assert_eq!(adapter.api_token(), Some("test-token"));
    }

    #[test]
    fn test_with_token() {
        let adapter = PaddleOcrApiAdapter::with_token("my-token".to_string());
        assert_eq!(adapter.api_token(), Some("my-token"));
    }
}
