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
//!
//! ## 认证头大小写
//! PaddleOCR API 要求 Authorization header 使用小写 "bearer"（而非 RFC 6750 推荐的 "Bearer"）。
//! 当前 `reqwest::Client::bearer_auth()` 使用大写 "Bearer"，如果 API 返回 401 请确认此问题。
//! 若需要，可改用 `header("Authorization", "bearer <token>")` 手动设置。

use serde::{Deserialize, Serialize};
use tokio::sync::watch;

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

fn is_false(b: &bool) -> bool { !*b }

#[derive(Serialize)]
struct OcrOptionalPayload {
    #[serde(rename = "useDocOrientationClassify")]
    use_doc_orientation_classify: bool,
    #[serde(rename = "useDocUnwarping")]
    use_doc_unwarping: bool,
    #[serde(rename = "useChartRecognition", skip_serializing_if = "is_false")]
    use_chart_recognition: bool,
    #[serde(rename = "useTextlineOrientation", skip_serializing_if = "is_false")]
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

impl JobStatusData {
    fn total_pages(&self) -> u32 {
        self.extract_progress.as_ref().map(|p| p.total_pages).unwrap_or(0)
    }

    fn extracted_pages(&self) -> u32 {
        self.extract_progress.as_ref().map(|p| p.extracted_pages).unwrap_or(0)
    }
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
        self.ocr_file_cancellable(file_path, model, None).await
    }

    /// 提交文件进行 OCR 解析（本地文件路径，multipart 上传）— 可取消
    pub async fn ocr_file_cancellable(
        &self,
        file_path: &str,
        model: &str,
        cancel_rx: Option<watch::Receiver<bool>>,
    ) -> Result<PaddleOcrResult, PaddleOcrApiError> {
        let file_bytes = std::fs::read(file_path)?;
        let file_name = std::path::Path::new(file_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("document.pdf");
        self.ocr_bytes_cancellable(&file_bytes, file_name, model, cancel_rx).await
    }

    /// 提交在线文件 URL 进行 OCR 解析（URL 模式，JSON 请求）
    pub async fn ocr_url(&self, file_url: &str, model: &str) -> Result<PaddleOcrResult, PaddleOcrApiError> {
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
            .header("Authorization", format!("bearer {}", &self.token))
            .json(&request)
            .send()
            .await?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(PaddleOcrApiError::Api(format!(
                "URL mode job submission failed ({}): {}",
                status,
                body
            )));
        }

        let submit: SubmitJobResponse = resp.json().await.map_err(|e| {
            PaddleOcrApiError::Parse(format!("解析提交响应失败: {}", e))
        })?;

        let job_id = submit.data.job_id;
        tracing::info!("[PaddleOCR] URL job submitted: {} ({})", job_id, file_url);
        tracing::info!(
            "[Pipeline::JobSubmitted] job_id={} model={} file_url={}",
            job_id, model, file_url
        );

        let result_url = self.poll_job(&job_id, None).await?;
        let pages = self.download_and_parse(&result_url, model).await?;

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
        self.ocr_bytes_cancellable(file_bytes, file_name, model, None).await
    }

    /// 提交文件字节进行 OCR 解析（multipart 上传）— 可取消
    pub async fn ocr_bytes_cancellable(
        &self,
        file_bytes: &[u8],
        file_name: &str,
        model: &str,
        cancel_rx: Option<watch::Receiver<bool>>,
    ) -> Result<PaddleOcrResult, PaddleOcrApiError> {
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
            .header("Authorization", format!("bearer {}", &self.token))
            .multipart(form)
            .send()
            .await?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(PaddleOcrApiError::Api(format!(
                "Job submission failed ({}): {}",
                status,
                body
            )));
        }

        let submit: SubmitJobResponse = resp.json().await.map_err(|e| {
            PaddleOcrApiError::Parse(format!("解析提交响应失败: {}", e))
        })?;

        let job_id = submit.data.job_id;
        tracing::info!("[PaddleOCR] Job submitted: {}", job_id);
        tracing::info!(
            "[Pipeline::JobSubmitted] job_id={} model={} file_size={} file_name={}",
            job_id, model, file_bytes.len(), file_name
        );

        // Poll until completion
        let result_url = self.poll_job(&job_id, cancel_rx).await?;

        // Download and parse results
        let pages = self.download_and_parse(&result_url, model).await?;

        Ok(PaddleOcrResult {
            total_pages: pages.len() as u32,
            model: model.to_string(),
            pages,
        })
    }

    async fn poll_job(&self, job_id: &str, cancel_rx: Option<watch::Receiver<bool>>) -> Result<String, PaddleOcrApiError> {
        let url = format!("{}/ocr/jobs/{}", PADDLEOCR_API_BASE, job_id);
        let _poll_timer = std::time::Instant::now();

        for attempt in 0..MAX_POLL_ATTEMPTS {
            let resp = self
                .client
                .get(&url)
                .header("Authorization", format!("bearer {}", &self.token))
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
                    let completed_pages = status.data.extract_progress
                        .map(|p| p.extracted_pages).unwrap_or(0);
                    tracing::info!(
                        "[PaddleOCR] Job {} completed: {} pages",
                        job_id, completed_pages,
                    );
                    tracing::info!(
                        "[Pipeline::JobCompleted] job_id={} pages={} elapsed_ms={}",
                        job_id, completed_pages, _poll_timer.elapsed().as_millis()
                    );
                    return Ok(json_url);
                }
                "failed" => {
                    let msg = status.data.error_msg.unwrap_or_else(|| "未知错误".to_string());
                    return Err(PaddleOcrApiError::JobFailed(msg));
                }
                "running" => {
                    let progress = status.data.extract_progress;
                    let extracted = progress.as_ref().map(|p| p.extracted_pages).unwrap_or(0);
                    let total = progress.map(|p| p.total_pages).unwrap_or(0);
                    tracing::debug!(
                        "[PaddleOCR] Job {} running: {}/{} pages",
                        job_id, extracted, total,
                    );
                    tracing::info!(
                        "[Pipeline::JobPolling] job_id={} state=running pages_done={} pages_total={} attempt={} elapsed_ms={}",
                        job_id, extracted, total, attempt, _poll_timer.elapsed().as_millis()
                    );
                }
                _ => { /* pending */ }
            }

            tokio::time::sleep(std::time::Duration::from_secs(POLL_INTERVAL_SECS)).await;

            // Check for cancellation
            if let Some(ref rx) = cancel_rx {
                if *rx.borrow() {
                    return Err(PaddleOcrApiError::Api("Cancelled".into()));
                }
            }

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
    ) -> Result<Vec<PaddleOcrPage>, PaddleOcrApiError> {
        let resp = self.client.get(jsonl_url).send().await?;
        let body = resp.text().await?;

        tracing::info!(
            "[Pipeline::JsonlDownloaded] url={} size_bytes={} lines={}",
            jsonl_url, body.len(), body.lines().count()
        );

        let mut pages = Vec::new();

        for (line_num, line) in body.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            let parsed: JsonlLine = serde_json::from_str(line).map_err(|e| {
                PaddleOcrApiError::Parse(format!("第 {} 行 JSON 解析失败: {}", line_num + 1, e))
            })?;

            if !parsed.result.layout_parsing_results.is_empty() {
                // layoutParsingResults → markdown (VL 系列 / StructureV3)
                for result in &parsed.result.layout_parsing_results {
                    pages.push(PaddleOcrPage {
                        page_index: line_num as u32,
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
            } else if !parsed.result.ocr_results.is_empty() {
                // ocrResults → ocrImage (PP-OCRv5)
                for result in &parsed.result.ocr_results {
                    pages.push(PaddleOcrPage {
                        page_index: line_num as u32,
                        markdown_text: String::new(), // v5 没有文本输出
                        images: vec![PaddleOcrImage {
                            name: format!("ocr_page_{}", pages.len()),
                            url: result.ocr_image.clone(),
                        }],
                    });
                }
            } else {
                tracing::warn!(
                    "[PaddleOCR] 第 {} 行没有可识别的结果字段",
                    line_num + 1
                );
            }
        }

        let total_text_chars: usize = pages.iter().map(|p| p.markdown_text.len()).sum();
        tracing::info!(
            "[PaddleOCR] Parsed {} pages from JSONL (model: {})",
            pages.len(),
            model
        );
        tracing::info!(
            "[Pipeline::JsonlParsed] pages={} text_chars={} model={}",
            pages.len(), total_text_chars, model
        );
        Ok(pages)
    }

    // ----- Connectivity check -----

    /// 轻量级连接性检查
    ///
    /// 仅验证 API 端点是否可达、TLS 握手是否成功。
    /// 不创建 OCR job，不消耗配额。
    ///
    /// # 返回
    /// - `Ok(())` — API 可达
    /// - `Err(PaddleOcrApiError)` — 连接失败（含原因：DNS / TCP / TLS / HTTP 状态码）
    pub async fn check_connectivity(&self) -> Result<(), PaddleOcrApiError> {
        // ★ 修复：原代码使用 GET /ocr/jobs 探测连通性，期望返回 401。
        // 但 PaddleOCR API 已变更为 POST-only 端点，GET 返回 405 Method Not Allowed。
        // 改用 POST 空请求（不带 fileUrl），预期返回 400 Bad Request，
        // 这同样证明 DNS 可达 + API 在线。
        let url = format!("{}/ocr/jobs", PADDLEOCR_API_BASE);

        let resp = self.client
            .post(&url)
            .header("Authorization", format!("bearer {}", &self.token))
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({"model": "PaddleOCR-VL-1.6"}))
            .timeout(std::time::Duration::from_secs(15))
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    PaddleOcrApiError::Api(format!("连接超时 (15s): {}", PADDLEOCR_API_BASE))
                } else if e.is_connect() {
                    PaddleOcrApiError::Api(format!("连接失败/DNS解析: {} — 请检查网络、DNS 和防火墙", PADDLEOCR_API_BASE))
                } else {
                    PaddleOcrApiError::Api(format!("网络错误: {}", e))
                }
            })?;

        let status = resp.status();
        let status_code = status.as_u16();

        // 400/401/403/405 都证明 API 可达（各种拒绝原因，但服务器在响应）
        if status_code == 400 || status_code == 401 || status_code == 403 || status_code == 405 {
            tracing::info!("[PaddleOCR] Connectivity check passed (HTTP {}, server reachable)", status_code);
            Ok(())
        } else if status_code == 200 {
            tracing::warn!("[PaddleOCR] Connectivity check: unexpected 200 response");
            Ok(())
        } else {
            Err(PaddleOcrApiError::Api(format!(
                "API 返回异常状态码: HTTP {} — body: {}",
                status_code,
                resp.text().await.unwrap_or_default()
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_submit_job_request_serialization() {
        let req = SubmitJobRequest {
            model: "PaddleOCR-VL-1.6",
            file_url: Some("https://example.com/doc.pdf".to_string()),
            optional_payload: Some(OcrOptionalPayload {
                use_doc_orientation_classify: false,
                use_doc_unwarping: false,
                use_chart_recognition: false,
                use_textline_orientation: false,
            }),
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"model\":\"PaddleOCR-VL-1.6\""));
        assert!(json.contains("\"fileUrl\":\"https://example.com/doc.pdf\""));
        assert!(json.contains("\"optionalPayload\""));
    }

    #[test]
    fn test_submit_job_request_no_file_url() {
        let req = SubmitJobRequest {
            model: "PP-OCRv5",
            file_url: None,
            optional_payload: None,
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"model\":\"PP-OCRv5\""));
        assert!(!json.contains("fileUrl"), "fileUrl should be omitted when None");
        assert!(!json.contains("optionalPayload"), "optionalPayload should be omitted when None");
    }

    #[test]
    fn test_job_status_response_deserialization_done() {
        let raw = r#"{
            "data": {
                "state": "done",
                "extractProgress": {
                    "totalPages": 5,
                    "extractedPages": 5,
                    "startTime": "2025-01-01T00:00:00Z",
                    "endTime": "2025-01-01T00:01:00Z"
                },
                "resultUrl": {
                    "jsonUrl": "https://storage.example.com/result.jsonl"
                }
            }
        }"#;
        let resp: JobStatusResponse = serde_json::from_str(raw).unwrap();
        assert_eq!(resp.data.state, "done");
        assert_eq!(resp.data.total_pages(), 5);
        assert_eq!(resp.data.extracted_pages(), 5);
        assert_eq!(
            resp.data.result_url.as_ref().unwrap().json_url,
            "https://storage.example.com/result.jsonl"
        );
    }

    #[test]
    fn test_job_status_response_deserialization_failed() {
        let raw = r#"{
            "data": {
                "state": "failed",
                "errorMsg": "图片解析失败：不支持的文件格式"
            }
        }"#;
        let resp: JobStatusResponse = serde_json::from_str(raw).unwrap();
        assert_eq!(resp.data.state, "failed");
        assert_eq!(
            resp.data.error_msg.as_deref(),
            Some("图片解析失败：不支持的文件格式")
        );
    }

    #[test]
    fn test_job_status_response_deserialization_running() {
        let raw = r#"{
            "data": {
                "state": "running",
                "extractProgress": {
                    "totalPages": 10,
                    "extractedPages": 3
                }
            }
        }"#;
        let resp: JobStatusResponse = serde_json::from_str(raw).unwrap();
        assert_eq!(resp.data.state, "running");
        assert_eq!(resp.data.extracted_pages(), 3);
    }

    #[test]
    fn test_job_status_response_deserialization_pending() {
        let raw = r#"{"data": {"state": "pending"}}"#;
        let resp: JobStatusResponse = serde_json::from_str(raw).unwrap();
        assert_eq!(resp.data.state, "pending");
        assert!(resp.data.error_msg.is_none());
        assert!(resp.data.result_url.is_none());
        assert!(resp.data.extract_progress.is_none());
    }

    #[test]
    fn test_jsonl_line_deserialization_vl() {
        let raw = "{\n            \"result\": {\n                \"layoutParsingResults\": [\n                    {\n                        \"markdown\": {\n                            \"text\": \"# Page 1\\n\\nHello World\",\n                            \"images\": {\n                                \"fig1\": \"https://storage.example.com/fig1.png\"\n                            }\n                        }\n                    }\n                ]\n            }\n        }";
        let line: JsonlLine = serde_json::from_str(raw).unwrap();
        assert_eq!(line.result.layout_parsing_results.len(), 1);
        assert_eq!(
            line.result.layout_parsing_results[0].markdown.text,
            "# Page 1\n\nHello World"
        );
        assert_eq!(
            line.result.layout_parsing_results[0]
                .markdown
                .images
                .get("fig1")
                .unwrap(),
            "https://storage.example.com/fig1.png"
        );
    }

    #[test]
    fn test_jsonl_line_deserialization_v5() {
        let raw = r#"{
            "result": {
                "ocrResults": [
                    {"ocrImage": "https://storage.example.com/page_0.png"},
                    {"ocrImage": "https://storage.example.com/page_1.png"}
                ]
            }
        }"#;
        let line: JsonlLine = serde_json::from_str(raw).unwrap();
        assert_eq!(line.result.ocr_results.len(), 2);
        assert_eq!(
            line.result.ocr_results[0].ocr_image,
            "https://storage.example.com/page_0.png"
        );
    }

    #[test]
    fn test_jsonl_line_both_fields_empty() {
        let raw = r#"{"result": {}}"#;
        let line: JsonlLine = serde_json::from_str(raw).unwrap();
        assert!(line.result.layout_parsing_results.is_empty());
        assert!(line.result.ocr_results.is_empty());
    }

    #[test]
    fn test_submit_job_response_deserialization() {
        let raw = r#"{"data": {"jobId": "job_abc123"}}"#;
        let resp: SubmitJobResponse = serde_json::from_str(raw).unwrap();
        assert_eq!(resp.data.job_id, "job_abc123");
    }

    #[test]
    fn test_paddle_ocr_result_construction() {
        let result = PaddleOcrResult {
            total_pages: 2,
            model: "PaddleOCR-VL-1.6".to_string(),
            pages: vec![
                PaddleOcrPage {
                    page_index: 0,
                    markdown_text: "# Page 1".to_string(),
                    images: vec![],
                },
                PaddleOcrPage {
                    page_index: 1,
                    markdown_text: "# Page 2".to_string(),
                    images: vec![PaddleOcrImage {
                        name: "fig1".to_string(),
                        url: "https://example.com/fig1.png".to_string(),
                    }],
                },
            ],
        };
        assert_eq!(result.total_pages, 2);
        assert_eq!(result.pages[0].markdown_text, "# Page 1");
        assert_eq!(result.pages[1].images[0].name, "fig1");
    }

    #[test]
    fn test_optional_payload_default() {
        let payload = OcrOptionalPayload::default();
        assert!(!payload.use_doc_orientation_classify);
        assert!(!payload.use_doc_unwarping);
        assert!(!payload.use_chart_recognition);
        assert!(!payload.use_textline_orientation);

        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("\"useDocOrientationClassify\":false"));
        assert!(json.contains("\"useDocUnwarping\":false"));
        assert!(!json.contains("useChartRecognition"));
        assert!(!json.contains("useTextlineOrientation"));
    }

    #[test]
    fn test_optional_payload_v5() {
        let payload = OcrOptionalPayload {
            use_doc_orientation_classify: false,
            use_doc_unwarping: false,
            use_chart_recognition: false,
            use_textline_orientation: true,
        };
        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("\"useTextlineOrientation\":true"));
    }

    #[test]
    fn test_optional_payload_full() {
        let payload = OcrOptionalPayload {
            use_doc_orientation_classify: true,
            use_doc_unwarping: true,
            use_chart_recognition: true,
            use_textline_orientation: true,
        };
        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("\"useDocOrientationClassify\":true"));
        assert!(json.contains("\"useDocUnwarping\":true"));
        assert!(json.contains("\"useChartRecognition\":true"));
        assert!(json.contains("\"useTextlineOrientation\":true"));
    }

    #[test]
    fn test_paddle_ocr_api_error_display() {
        let err = PaddleOcrApiError::Api("rate limited".to_string());
        assert!(err.to_string().contains("rate limited"));

        let err = PaddleOcrApiError::Timeout(120);
        assert!(err.to_string().contains("120"));

        let err = PaddleOcrApiError::JobFailed("unsupported format".to_string());
        assert!(err.to_string().contains("unsupported format"));
    }

    #[test]
    fn test_paddle_ocr_page_defaults() {
        let page = PaddleOcrPage {
            page_index: 0,
            markdown_text: String::new(),
            images: vec![],
        };
        assert!(page.markdown_text.is_empty());
        assert!(page.images.is_empty());
    }
}
