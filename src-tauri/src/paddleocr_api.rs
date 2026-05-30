//! PaddleOCR REST API 集成
//!
//! 直接调用 PaddleOCR AI Studio REST API 进行 OCR 解析。
//! 支持全部 4 种模型: PaddleOCR-VL-1.6/1.5, PP-OCRv5, PP-StructureV3。
//!
//! ## API 端点
//! - 提交 job: POST https://paddleocr.aistudio-app.com/api/v2/ocr/jobs
//! - 查询状态: GET  https://paddleocr.aistudio-app.com/api/v2/ocr/jobs/{jobId}
//! - 下载结果: 从 resultUrl 下载 JSONL
//!
//! ## 模型差异
//! - VL 系列 (1.5/1.6): layoutParsingResults → markdown.text + images
//! - PP-OCRv5:            ocrResults → ocrImage (文字叠加到图片)
//! - PP-StructureV3:      layoutParsingResults → markdown.text + images

use serde::{Deserialize, Serialize};

const PADDLEOCR_API_BASE: &str = "https://paddleocr.aistudio-app.com/api/v2";
const POLL_INTERVAL_SECS: u64 = 3;
const MAX_POLL_ATTEMPTS: u32 = 120; // 6 minutes max

// --- Request / Response types ---

#[derive(Serialize)]
struct SubmitJobRequest<'a> {
    model: &'a str,
    #[serde(rename = "fileUrl", skip_serializing_if = "Option::is_none")]
    file_url: Option<String>,
    #[serde(rename = "optionalPayload", skip_serializing_if = "Option::is_none")]
    optional_payload: Option<OcrOptionalPayload>,
}

#[derive(Serialize)]
struct OcrOptionalPayload {
    #[serde(rename = "useDocOrientationClassify")]
    use_doc_orientation_classify: bool,
    #[serde(rename = "useDocUnwarping")]
    use_doc_unwarping: bool,
    #[serde(rename = "useChartRecognition", skip_serializing_if = "std::ops::Not::not")]
    use_chart_recognition: bool,
    #[serde(rename = "useTextlineOrientation", skip_serializing_if = "std::ops::Not::not")]
    use_textline_orientation: bool,
}

impl Default for OcrOptionalPayload {
    fn default() -> Self {
        Self {
            use_doc_orientation_classify: false,
            use_doc_unwarping: false,
            use_chart_recognition: false,
            use_textline_orientation: false,
        }
    }
}

#[derive(Deserialize)]
struct SubmitJobResponse {
    data: SubmitJobData,
}

#[derive(Deserialize)]
struct SubmitJobData {
    #[serde(rename = "jobId")]
    job_id: String,
}

#[derive(Debug, Deserialize)]
struct JobStatusResponse {
    data: JobStatusData,
}

#[derive(Debug, Deserialize)]
struct JobStatusData {
    state: String,
    #[serde(default)]
    #[serde(rename = "errorMsg")]
    error_msg: Option<String>,
    #[serde(rename = "extractProgress")]
    extract_progress: Option<ExtractProgress>,
    #[serde(rename = "resultUrl")]
    result_url: Option<ResultUrl>,
}

#[derive(Debug, Deserialize)]
struct ExtractProgress {
    #[serde(rename = "totalPages", default)]
    total_pages: u32,
    #[serde(rename = "extractedPages", default)]
    extracted_pages: u32,
    #[serde(rename = "startTime", default)]
    start_time: Option<String>,
    #[serde(rename = "endTime", default)]
    end_time: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ResultUrl {
    #[serde(rename = "jsonUrl")]
    json_url: String,
}

// --- JSONL result types ---

/// 单行 JSONL 结果
#[derive(Debug, Deserialize)]
struct JsonlLine {
    result: JsonlResult,
}

#[derive(Debug, Deserialize)]
struct JsonlResult {
    /// VL 系列 / StructureV3: 版面解析结果
    #[serde(default)]
    #[serde(rename = "layoutParsingResults")]
    layout_parsing_results: Vec<LayoutParsingResult>,
    /// PP-OCRv5: OCR 结果
    #[serde(default)]
    #[serde(rename = "ocrResults")]
    ocr_results: Vec<OcrImageResult>,
}

#[derive(Debug, Deserialize)]
struct LayoutParsingResult {
    markdown: MarkdownContent,
    #[serde(default)]
    #[serde(rename = "outputImages")]
    output_images: std::collections::HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
struct MarkdownContent {
    text: String,
    #[serde(default)]
    images: std::collections::HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
struct OcrImageResult {
    #[serde(rename = "ocrImage")]
    ocr_image: String,
}

// --- Public output types ---

/// PaddleOCR 解析后的单页结果
#[derive(Debug, Clone)]
pub struct PaddleOcrPage {
    pub page_index: u32,
    pub markdown_text: String,
    pub images: Vec<PaddleOcrImage>,
}

/// OCR 结果中的图片
#[derive(Debug, Clone)]
pub struct PaddleOcrImage {
    pub name: String,
    pub url: String,
}

/// 完整 OCR 解析结果
#[derive(Debug, Clone)]
pub struct PaddleOcrResult {
    pub pages: Vec<PaddleOcrPage>,
    pub total_pages: u32,
    pub model: String,
}

// --- Error type ---

#[derive(Debug, thiserror::Error)]
pub enum PaddleOcrApiError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("API error: {0}")]
    Api(String),
    #[error("Job failed: {0}")]
    JobFailed(String),
    #[error("Job timeout after {0} attempts")]
    Timeout(u32),
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

// --- Service ---

/// PaddleOCR REST API 客户端
pub struct PaddleOcrApiClient {
    client: reqwest::Client,
    token: String,
}

impl PaddleOcrApiClient {
    pub fn new(token: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            token,
        }
    }

    /// 提交文件进行 OCR 解析（本地文件路径，multipart 上传）
    pub async fn ocr_file(&self, file_path: &str, model: &str) -> Result<PaddleOcrResult, PaddleOcrApiError> {
        let file_bytes = std::fs::read(file_path)?;
        let file_name = std::path::Path::new(file_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("document.pdf");
        self.ocr_bytes(&file_bytes, file_name, model).await
    }

    /// 提交在线文件 URL 进行 OCR 解析（URL 模式，JSON 请求）
    pub async fn ocr_url(&self, file_url: &str, model: &str) -> Result<PaddleOcrResult, PaddleOcrApiError> {
        let is_vl = model.contains("PaddleOCR-VL") || model.contains("PP-StructureV3");
        let is_v5 = model.contains("PP-OCRv5");

        let optional_payload = OcrOptionalPayload {
            use_textline_orientation: is_v5,
            ..Default::default()
        };

        let request = SubmitJobRequest {
            model,
            file_url: Some(file_url.to_string()),
            optional_payload: Some(optional_payload),
        };

        let resp = self
            .client
            .post(format!("{}/ocr/jobs", PADDLEOCR_API_BASE))
            .bearer_auth(&self.token)
            .json(&request)
            .send()
            .await?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(PaddleOcrApiError::Api(format!(
                "URL mode job submission failed ({}): {}",
                resp.status(),
                body
            )));
        }

        let submit: SubmitJobResponse = resp.json().await.map_err(|e| {
            PaddleOcrApiError::Parse(format!("解析提交响应失败: {}", e))
        })?;

        let job_id = submit.data.job_id;
        tracing::info!("[PaddleOCR] URL job submitted: {} ({})", job_id, file_url);

        let result_url = self.poll_job(&job_id).await?;
        let pages = self.download_and_parse(&result_url, model, is_vl).await?;

        Ok(PaddleOcrResult {
            total_pages: pages.len() as u32,
            model: model.to_string(),
            pages,
        })
    }

    /// 提交文件字节进行 OCR 解析（multipart 上传）
    pub async fn ocr_bytes(
        &self,
        file_bytes: &[u8],
        file_name: &str,
        model: &str,
    ) -> Result<PaddleOcrResult, PaddleOcrApiError> {
        let is_vl = model.contains("PaddleOCR-VL") || model.contains("PP-StructureV3");
        let is_v5 = model.contains("PP-OCRv5");

        let optional_payload = OcrOptionalPayload {
            use_textline_orientation: is_v5,
            ..Default::default()
        };

        let form = reqwest::multipart::Form::new()
            .text("model", model.to_string())
            .text(
                "optionalPayload",
                serde_json::to_string(&optional_payload).unwrap_or_default(),
            )
            .part(
                "file",
                reqwest::multipart::Part::bytes(file_bytes.to_vec())
                    .file_name(file_name.to_string()),
            );

        let resp = self
            .client
            .post(format!("{}/ocr/jobs", PADDLEOCR_API_BASE))
            .bearer_auth(&self.token)
            .multipart(form)
            .send()
            .await?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(PaddleOcrApiError::Api(format!(
                "Job submission failed ({}): {}",
                resp.status(),
                body
            )));
        }

        let submit: SubmitJobResponse = resp.json().await.map_err(|e| {
            PaddleOcrApiError::Parse(format!("解析提交响应失败: {}", e))
        })?;

        let job_id = submit.data.job_id;
        tracing::info!("[PaddleOCR] Job submitted: {}", job_id);

        // Poll until completion
        let result_url = self.poll_job(&job_id).await?;

        // Download and parse results
        let pages = self.download_and_parse(&result_url, model, is_vl).await?;

        Ok(PaddleOcrResult {
            total_pages: pages.len() as u32,
            model: model.to_string(),
            pages,
        })
    }

    async fn poll_job(&self, job_id: &str) -> Result<String, PaddleOcrApiError> {
        let url = format!("{}/ocr/jobs/{}", PADDLEOCR_API_BASE, job_id);

        for attempt in 0..MAX_POLL_ATTEMPTS {
            let resp = self
                .client
                .get(&url)
                .bearer_auth(&self.token)
                .send()
                .await?;

            if !resp.status().is_success() {
                return Err(PaddleOcrApiError::Api(format!(
                    "Poll failed ({}): {}",
                    resp.status(),
                    resp.text().await.unwrap_or_default()
                )));
            }

            let status: JobStatusResponse = resp.json().await.map_err(|e| {
                PaddleOcrApiError::Parse(format!("解析状态响应失败: {}", e))
            })?;

            match status.data.state.as_str() {
                "done" => {
                    let json_url = status
                        .data
                        .result_url
                        .ok_or_else(|| PaddleOcrApiError::Api("结果 URL 缺失".to_string()))?
                        .json_url;
                    tracing::info!(
                        "[PaddleOCR] Job {} completed: {} pages",
                        job_id,
                        status.data.extract_progress.map(|p| p.extracted_pages).unwrap_or(0)
                    );
                    return Ok(json_url);
                }
                "failed" => {
                    let msg = status.data.error_msg.unwrap_or_else(|| "未知错误".to_string());
                    return Err(PaddleOcrApiError::JobFailed(msg));
                }
                "running" => {
                    let progress = status.data.extract_progress;
                    tracing::debug!(
                        "[PaddleOCR] Job {} running: {}/{} pages",
                        job_id,
                        progress.as_ref().map(|p| p.extracted_pages).unwrap_or(0),
                        progress.map(|p| p.total_pages).unwrap_or(0)
                    );
                }
                _ => { /* pending */ }
            }

            tokio::time::sleep(std::time::Duration::from_secs(POLL_INTERVAL_SECS)).await;

            // Only count actual wait attempts
            if attempt > 0 && attempt % 20 == 0 {
                tracing::info!(
                    "[PaddleOCR] Job {} still processing (attempt {}/{})",
                    job_id, attempt, MAX_POLL_ATTEMPTS
                );
            }
        }

        Err(PaddleOcrApiError::Timeout(MAX_POLL_ATTEMPTS))
    }

    async fn download_and_parse(
        &self,
        jsonl_url: &str,
        model: &str,
        is_vl: bool,
    ) -> Result<Vec<PaddleOcrPage>, PaddleOcrApiError> {
        let resp = self.client.get(jsonl_url).send().await?;
        let body = resp.text().await?;

        let mut pages = Vec::new();

        for (line_num, line) in body.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            let parsed: JsonlLine = serde_json::from_str(line).map_err(|e| {
                PaddleOcrApiError::Parse(format!("第 {} 行 JSON 解析失败: {}", line_num + 1, e))
            })?;

            if is_vl {
                // VL 系列 / StructureV3: layoutParsingResults → markdown
                for result in &parsed.result.layout_parsing_results {
                    pages.push(PaddleOcrPage {
                        page_index: pages.len() as u32,
                        markdown_text: result.markdown.text.clone(),
                        images: result
                            .markdown
                            .images
                            .iter()
                            .map(|(name, url)| PaddleOcrImage {
                                name: name.clone(),
                                url: url.clone(),
                            })
                            .collect(),
                    });
                }
            } else {
                // PP-OCRv5: ocrResults → ocrImage
                for result in &parsed.result.ocr_results {
                    pages.push(PaddleOcrPage {
                        page_index: pages.len() as u32,
                        markdown_text: String::new(), // v5 没有文本输出
                        images: vec![PaddleOcrImage {
                            name: format!("ocr_page_{}", pages.len()),
                            url: result.ocr_image.clone(),
                        }],
                    });
                }
            }
        }

        tracing::info!(
            "[PaddleOCR] Parsed {} pages from JSONL (model: {})",
            pages.len(),
            model
        );
        Ok(pages)
    }
}
