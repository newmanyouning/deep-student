//! PaddleOCR API 大 PDF 自动分片模块
//!
//! 当提交的 PDF 超过阈值（文件大小 20MB / 页数 50）时，自动按页切分为多个
//! 分片 PDF 文件，并发提交到 AI Studio 作业 API，最后按页序合并结果并校正
//! `page_index` 偏移。对调用方完全透明：返回类型与原始路径一致（`PaddleOcrResult`）。
//!
//! 设计文档：`docs/analysis/PDF_AUTO_SPLIT_DESIGN.md`（状态：Final — Implementation Ready）
//!
//! ## 预留（设计文档 §11 未来考虑，暂不实现）
//! - 智能分片（按章节边界 / 每页大小动态调整分片粒度）
//! - 分片结果缓存（复用 pdf_ocr_service 的 SHA-256 缓存体系）
//! - 流式渐进上传（当前 API 为 job 制，不支持增量提交）
//! - 阈值运行时可配置（当前为编译期常量）
//! - 非 PDF 多页文档格式分片（TIFF / DJVU）

use crate::paddleocr_api::{PaddleOcrApiClient, PaddleOcrApiError, PaddleOcrPage, PaddleOcrResult};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

// --- 分片配置常量（编译期默认值，§11.4 预留为运行时配置） ---

/// 默认触发分片的文件大小上限（20 MB）
const DEFAULT_MAX_FILE_SIZE: u64 = 20 * 1024 * 1024;

/// 默认每分片最大页数（50 页）
const DEFAULT_MAX_PAGES_PER_CHUNK: usize = 50;

/// 默认并发分片任务数（信号量限制，避免触发 API 限流）
const DEFAULT_MAX_CONCURRENT_CHUNKS: usize = 2;

/// 默认分片任务最大尝试次数（含首次提交）
const DEFAULT_MAX_CHUNK_RETRIES: usize = 2;

// --- 分片判定 ---

/// 分片判定结果
#[derive(Debug, PartialEq, Eq)]
enum SplitDecision {
    /// 无需分片，走原始上传路径
    NotNeeded,
    /// 需要分片：每片页数 + 总页数
    Needed { chunk_pages: usize, total_pages: usize },
}

/// 判定是否需要分片：文件大小 > 20MB 或页数 > 50 时触发
///
/// 每片页数规则（保证两个约束同时满足）：
/// - 页数超限 → 直接用页数上限（50）
/// - 仅文件大小超限 → 按大小比例估算：`ceil(总页数 * 20MB / 文件大小)`，
///   再夹取到 [1, 50]（保守起见不小于 1 页）
fn needs_split(file_bytes: &[u8], total_pages: usize) -> SplitDecision {
    let file_size = file_bytes.len() as u64;
    let needs_size_split = file_size > DEFAULT_MAX_FILE_SIZE;
    let needs_page_split = total_pages > DEFAULT_MAX_PAGES_PER_CHUNK;

    if !needs_size_split && !needs_page_split {
        return SplitDecision::NotNeeded;
    }

    let chunk_pages = if needs_page_split {
        DEFAULT_MAX_PAGES_PER_CHUNK
    } else {
        // 仅大小超限：按比例估算每片页数（file_size > 20MB 保证 ratio < 1，无除零风险）
        let ratio = DEFAULT_MAX_FILE_SIZE as f64 / file_size as f64;
        let estimated = (total_pages as f64 * ratio).ceil() as usize;
        std::cmp::max(1, std::cmp::min(estimated, DEFAULT_MAX_PAGES_PER_CHUNK))
    };

    SplitDecision::Needed { chunk_pages, total_pages }
}

// --- 页数估算 ---

/// 估算 PDF 页数（pdfium 懒加载元数据，不做页面渲染，速度快）
///
/// 先做魔数检查（`%PDF-`）拦截非 PDF 内容，避免无谓的库调用。
fn estimate_pdf_page_count(bytes: &[u8]) -> Result<usize, PaddleOcrApiError> {
    if !bytes.starts_with(b"%PDF-") {
        return Err(PaddleOcrApiError::Api("不是 PDF 文件".to_string()));
    }

    let pdfium = crate::pdfium_utils::load_pdfium()
        .map_err(|e| PaddleOcrApiError::Api(format!("pdfium 加载失败: {}", e)))?;

    let document = pdfium
        .load_pdf_from_byte_slice(bytes, None)
        .map_err(|e| PaddleOcrApiError::Api(format!("PDF 加载失败: {:?}", e)))?;

    Ok(document.pages().len() as usize)
}

// --- 分片切分 ---

/// 将 PDF 字节切分为多个临时 PDF 文件，每个包含 `chunk_pages` 页（末片可能不足）
///
/// 复用全局 pdfium 实例（`pdfium_utils::load_pdfium`），不新增外部依赖。
/// pdfium-render 0.8.37 源码核实 API：`create_new_pdf()` + `copy_page_range_from_document()`
/// + `save_to_file()`（设计文档中的 `new_pdf()` / `copy_page()` 为旧 API 名，已按实际版本适配）。
fn split_pdf_into_chunks(
    pdf_bytes: &[u8],
    chunk_pages: usize,
    total_pages: usize,
    temp_dir: &Path,
) -> Result<Vec<PathBuf>, PaddleOcrApiError> {
    let pdfium = crate::pdfium_utils::load_pdfium()
        .map_err(|e| PaddleOcrApiError::Api(format!("pdfium 加载失败: {}", e)))?;

    let document = pdfium
        .load_pdf_from_byte_slice(pdf_bytes, None)
        .map_err(|e| PaddleOcrApiError::Api(format!("PDF 加载失败: {:?}", e)))?;

    let chunk_count = total_pages.div_ceil(chunk_pages);
    let mut chunk_paths = Vec::with_capacity(chunk_count);

    for chunk_idx in 0..chunk_count {
        let start_page = chunk_idx * chunk_pages;
        let end_page = std::cmp::min(start_page + chunk_pages, total_pages); // 不含 end
        if start_page >= end_page {
            break; // 防御：正常流程不会出现空分片
        }

        // 创建空文档，从源文档一次性复制整段页区间
        let mut chunk_doc = pdfium
            .create_new_pdf()
            .map_err(|e| PaddleOcrApiError::Api(format!("创建分片文档失败: {:?}", e)))?;

        chunk_doc
            .pages_mut()
            .copy_page_range_from_document(
                &document,
                (start_page as u16)..=((end_page - 1) as u16),
                0,
            )
            .map_err(|e| {
                PaddleOcrApiError::Api(format!("复制页 {}-{} 失败: {:?}", start_page, end_page - 1, e))
            })?;

        // 保存分片到临时文件
        let chunk_path = temp_dir.join(format!("chunk_{:04}.pdf", chunk_idx));
        chunk_doc
            .save_to_file(&chunk_path)
            .map_err(|e| PaddleOcrApiError::Api(format!("保存分片 {} 失败: {:?}", chunk_idx, e)))?;

        tracing::debug!(
            "[PaddleOCR-Split] 已生成分片 {}: 页 {}-{}, 路径: {:?}",
            chunk_idx,
            start_page,
            end_page - 1,
            chunk_path
        );
        chunk_paths.push(chunk_path);
    }

    Ok(chunk_paths)
}

// --- 结果合并 ---

/// 按分片序号排序合并各分片结果，并校正 `page_index` 偏移
///
/// 分片内 `page_index` 以 0 起始（JSONL 行号），合并时需加 `idx * chunk_pages`。
/// 纯函数，便于单元测试。
fn merge_chunk_results(
    mut chunks: Vec<(usize, Vec<PaddleOcrPage>)>,
    chunk_pages: usize,
) -> Vec<PaddleOcrPage> {
    // 按分片序号排序，保证最终页序与原始文档一致
    chunks.sort_by_key(|(idx, _)| *idx);
    chunks
        .into_iter()
        .flat_map(|(idx, pages)| {
            let offset = (idx * chunk_pages) as u32;
            pages.into_iter().map(move |mut page| {
                page.page_index += offset;
                page
            })
        })
        .collect()
}

// --- 并发提交与编排 ---

/// 并发提交所有分片、收集结果并按页序合并
///
/// 错误处理（设计文档 §7）：
/// - 单个分片瞬时失败 → 指数退避重试（2s / 4s，`JobFailed` 为服务端明确判定，不重试）
/// - 部分分片失败 → 返回部分结果 + 告警日志（缺失页静默省略）
/// - 全部分片失败 → 返回聚合错误
async fn split_ocr_impl(
    client: &PaddleOcrApiClient,
    file_bytes: &[u8],
    file_name: &str,
    model: &str,
    chunk_pages: usize,
    total_pages: usize,
    temp_dir: &Path,
) -> Result<PaddleOcrResult, PaddleOcrApiError> {
    let chunk_count = total_pages.div_ceil(chunk_pages);
    tracing::info!(
        "[PaddleOCR-Split] 开始分片 OCR: 共 {} 页, 每片 {} 页, 共 {} 片",
        total_pages,
        chunk_pages,
        chunk_count
    );

    // 1. 按页切分为临时 PDF 文件
    let chunk_paths = split_pdf_into_chunks(file_bytes, chunk_pages, total_pages, temp_dir)?;

    // 2. 并发提交（信号量限制并发数，避免触发 API 限流）
    let semaphore = Arc::new(tokio::sync::Semaphore::new(DEFAULT_MAX_CONCURRENT_CHUNKS));
    let mut join_set = tokio::task::JoinSet::new();

    // 去除 .pdf 扩展名，用于生成分片文件名
    let base_name = file_name.strip_suffix(".pdf").unwrap_or(file_name);

    for (idx, chunk_path) in chunk_paths.iter().enumerate() {
        let sem = semaphore.clone();
        let model = model.to_string();
        let chunk_name = format!("{}_chunk_{:04}.pdf", base_name, idx);
        let path_str = chunk_path.to_string_lossy().to_string();
        let chunk_client = client.clone(); // PaddleOcrApiClient 无状态，克隆安全

        join_set.spawn(async move {
            // 等待并发许可（等待而非拒绝）
            let _permit = sem.acquire().await.expect("分片信号量被关闭");

            // 分片已满足大小/页数限制，直接走原始上传路径（不再进入分片判定）
            let bytes = match std::fs::read(&path_str) {
                Ok(b) => b,
                Err(e) => return (idx, Err(PaddleOcrApiError::Io(e)), path_str),
            };

            let mut attempt = 0;
            loop {
                match chunk_client.ocr_bytes_inner(&bytes, &chunk_name, &model, None).await {
                    Ok(result) => return (idx, Ok(result), path_str),
                    Err(e) => {
                        attempt += 1;
                        // 服务端明确判定的失败（如文件格式不支持）重试无意义
                        let is_job_failure = matches!(&e, PaddleOcrApiError::JobFailed(_));
                        if is_job_failure || attempt >= DEFAULT_MAX_CHUNK_RETRIES {
                            tracing::warn!("[PaddleOCR-Split] 分片 {} 提交失败: {}", idx, e);
                            return (idx, Err(e), path_str);
                        }
                        // 指数退避: 2s, 4s, ...
                        tokio::time::sleep(Duration::from_secs(2u64.pow(attempt as u32))).await;
                    }
                }
            }
        });
    }

    // 3. 收集结果（每个分片文件处理完毕即删除）
    let mut results: Vec<(usize, Vec<PaddleOcrPage>)> = Vec::new();
    let mut errors: Vec<String> = Vec::new();

    while let Some(joined) = join_set.join_next().await {
        match joined {
            Ok((idx, Ok(chunk_result), path)) => {
                let _ = std::fs::remove_file(&path);
                results.push((idx, chunk_result.pages));
            }
            Ok((idx, Err(e), path)) => {
                let _ = std::fs::remove_file(&path);
                errors.push(format!("分片 {} 失败: {}", idx, e));
            }
            Err(e) => {
                errors.push(format!("分片任务异常: {}", e));
            }
        }
    }

    // 4. 全部失败 → 聚合报错；部分失败 → 返回部分结果并告警
    if results.is_empty() {
        return Err(PaddleOcrApiError::Api(format!(
            "全部 {} 个 PDF 分片均失败: {}",
            chunk_count,
            errors.join("; ")
        )));
    }
    if !errors.is_empty() {
        tracing::warn!(
            "[PaddleOCR-Split] {}/{} 分片失败，返回部分结果: {}",
            errors.len(),
            chunk_count,
            errors.join("; ")
        );
    }

    // 5. 按页序合并（偏移校正），total_pages 取成功分片的页数（部分结果契约）
    let all_pages = merge_chunk_results(results, chunk_pages);

    Ok(PaddleOcrResult {
        total_pages: all_pages.len() as u32,
        pages: all_pages,
        model: model.to_string(),
    })
}

// --- 对外入口 ---

/// 自动分片流程的出口判定（由调用方决定是否回退到各自的原始路径）
pub(crate) enum SplitOutcome {
    /// 无需分片 / 分片基础设施不可用：调用方走原始路径（零行为变化）
    FallbackToOriginal,
    /// 分片流程已完整处理（可能为部分结果）
    Completed(PaddleOcrResult),
}

/// 主入口：判定是否需要分片，需要则切分 → 并发提交 → 合并结果
///
/// 降级链（设计文档 §12）：
/// 1. 非 PDF 内容 / pdfium 不可用 / 页数估算失败 → 回退原始路径
/// 2. 无需分片 → 回退原始路径（由调用方选择原始实现）
/// 3. 切分失败 → 返回错误（PDF 解析失败）
/// 4. 并发提交部分失败 → 部分结果 + 告警；全部失败 → 聚合错误
pub(crate) async fn maybe_split_and_ocr(
    client: &PaddleOcrApiClient,
    file_bytes: &[u8],
    file_name: &str,
    model: &str,
    temp_dir: &Path,
) -> Result<SplitOutcome, PaddleOcrApiError> {
    // 快速魔数检查：非 PDF 内容直接回退（在 pdfium 库调用之前拦截，零开销）
    if !file_bytes.starts_with(b"%PDF-") {
        tracing::warn!(
            "[PaddleOCR-Split] 文件内容不是 PDF（{}），回退原始上传路径",
            file_name
        );
        return Ok(SplitOutcome::FallbackToOriginal);
    }

    // 估算页数；pdfium 不可用或 PDF 无法解析 → 回退原始路径（保持旧行为，服务端自行判断）
    let total_pages = match estimate_pdf_page_count(file_bytes) {
        Ok(pages) => pages,
        Err(e) => {
            tracing::warn!("[PaddleOCR-Split] 页数估算失败，回退原始上传路径: {}", e);
            return Ok(SplitOutcome::FallbackToOriginal);
        }
    };

    match needs_split(file_bytes, total_pages) {
        SplitDecision::NotNeeded => Ok(SplitOutcome::FallbackToOriginal),
        SplitDecision::Needed { chunk_pages, .. } => {
            let result = split_ocr_impl(
                client,
                file_bytes,
                file_name,
                model,
                chunk_pages,
                total_pages,
                temp_dir,
            )
            .await?;
            Ok(SplitOutcome::Completed(result))
        }
    }
}

// --- 临时目录管理 ---

/// 创建分片临时目录：`{系统临时目录}/paddleocr_split/{uuid}/`
pub(crate) fn create_temp_dir() -> Result<PathBuf, PaddleOcrApiError> {
    let base = std::env::temp_dir().join("paddleocr_split");
    let dir = base.join(uuid::Uuid::new_v4().to_string());
    std::fs::create_dir_all(&dir).map_err(PaddleOcrApiError::Io)?;
    Ok(dir)
}

/// 清理超过 1 小时的遗留分片临时目录（应用启动时调用，尽力而为）
///
/// 正常流程中分片文件与会话目录都会被删除，此函数仅兜底异常退出
/// （崩溃 / 断电）遗留的目录，避免磁盘空间耗尽。
pub(crate) fn cleanup_orphaned_temp_dirs() {
    let base = std::env::temp_dir().join("paddleocr_split");
    if !base.exists() {
        return;
    }

    let now_secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or_default();
    let cutoff = now_secs.saturating_sub(3600); // 1 小时前

    if let Ok(entries) = std::fs::read_dir(&base) {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                if let Ok(created) = metadata.created() {
                    let created_secs = created
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_secs())
                        .unwrap_or_default();
                    if created_secs < cutoff {
                        let _ = std::fs::remove_dir_all(entry.path());
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 构造测试用页面（page_index 按分片内 0 起始）
    fn page(page_index: u32) -> PaddleOcrPage {
        PaddleOcrPage {
            page_index,
            markdown_text: format!("page {}", page_index),
            images: vec![],
        }
    }

    // --- 分片判定（阈值边界） ---

    #[test]
    fn test_needs_split_small_pdf_not_needed() {
        // 10 页 / 5MB → 不触发
        let bytes = vec![0u8; 5 * 1024 * 1024];
        assert_eq!(needs_split(&bytes, 10), SplitDecision::NotNeeded);
    }

    #[test]
    fn test_needs_split_page_boundary_exactly_50_not_needed() {
        // 恰好 50 页（页数上限边界）→ 不触发
        let bytes = vec![0u8; 5 * 1024 * 1024];
        assert_eq!(needs_split(&bytes, 50), SplitDecision::NotNeeded);
    }

    #[test]
    fn test_needs_split_page_limit_exceeded() {
        // 51 页 → 触发，每片 50 页
        let bytes = vec![0u8; 5 * 1024 * 1024];
        assert_eq!(
            needs_split(&bytes, 51),
            SplitDecision::Needed { chunk_pages: 50, total_pages: 51 }
        );
    }

    #[test]
    fn test_needs_split_size_boundary_exactly_20mb_not_needed() {
        // 恰好 20MB（大小上限边界）→ 不触发
        let bytes = vec![0u8; 20 * 1024 * 1024];
        assert_eq!(needs_split(&bytes, 40), SplitDecision::NotNeeded);
    }

    #[test]
    fn test_needs_split_size_limit_exceeded() {
        // 20MB + 1 字节 → 触发；比例估算每片 40 页（ceil(40 * 20MB/20MB+1) = 40）
        let bytes = vec![0u8; 20 * 1024 * 1024 + 1];
        assert_eq!(
            needs_split(&bytes, 40),
            SplitDecision::Needed { chunk_pages: 40, total_pages: 40 }
        );
    }

    #[test]
    fn test_needs_split_size_only_ratio_estimate() {
        // 100MB / 10 页 → 比例估算每片 2 页（ceil(10 * 20/100) = 2）
        let bytes = vec![0u8; 100 * 1024 * 1024];
        assert_eq!(
            needs_split(&bytes, 10),
            SplitDecision::Needed { chunk_pages: 2, total_pages: 10 }
        );
    }

    #[test]
    fn test_needs_split_size_only_ratio_floor_one() {
        // 200MB / 10 页 → 每片 1 页（ceil(10 * 20/200) = 1，夹取下限 1）
        let bytes = vec![0u8; 200 * 1024 * 1024];
        assert_eq!(
            needs_split(&bytes, 10),
            SplitDecision::Needed { chunk_pages: 1, total_pages: 10 }
        );
    }

    #[test]
    fn test_needs_split_both_limits_capped_at_50() {
        // 200 页 / 200MB → 页数分支优先，每片 50 页
        let bytes = vec![0u8; 200 * 1024 * 1024];
        assert_eq!(
            needs_split(&bytes, 200),
            SplitDecision::Needed { chunk_pages: 50, total_pages: 200 }
        );
    }

    #[test]
    fn test_needs_split_empty_file_not_needed() {
        // 空文件（0 字节 / 0 页）→ 不触发
        assert_eq!(needs_split(&[], 0), SplitDecision::NotNeeded);
    }

    // --- 页序合并（偏移校正） ---

    #[test]
    fn test_merge_chunk_results_page_index_offset() {
        // 两个分片（每片 3 页）：分片 1 的页索引应从 3 开始
        let chunks = vec![
            (0usize, vec![page(0), page(1), page(2)]),
            (1usize, vec![page(0), page(1), page(2)]),
        ];
        let merged = merge_chunk_results(chunks, 3);
        let indexes: Vec<u32> = merged.iter().map(|p| p.page_index).collect();
        assert_eq!(indexes, vec![0, 1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_merge_chunk_results_order_by_chunk_index() {
        // 输入乱序，合并后仍按分片序号排序
        let chunks = vec![(1usize, vec![page(0)]), (0usize, vec![page(0)])];
        let merged = merge_chunk_results(chunks, 1);
        let indexes: Vec<u32> = merged.iter().map(|p| p.page_index).collect();
        assert_eq!(indexes, vec![0, 1]);
    }

    #[test]
    fn test_merge_chunk_results_last_chunk_short() {
        // 3 片（5, 5, 2）：末片偏移 = 2 * 5 = 10
        let chunks = vec![
            (2usize, vec![page(0), page(1)]),
            (0usize, vec![page(0), page(1), page(2), page(3), page(4)]),
            (1usize, vec![page(0), page(1), page(2), page(3), page(4)]),
        ];
        let merged = merge_chunk_results(chunks, 5);
        let indexes: Vec<u32> = merged.iter().map(|p| p.page_index).collect();
        assert_eq!(indexes, vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);
    }

    #[test]
    fn test_merge_chunk_results_partial_result() {
        // 仅分片 0 成功 → 返回其页（部分结果契约：缺失页静默省略）
        let chunks = vec![(0usize, vec![page(0), page(1), page(2)])];
        let merged = merge_chunk_results(chunks, 3);
        assert_eq!(merged.len(), 3);
        assert_eq!(merged[0].page_index, 0);
        assert_eq!(merged[2].page_index, 2);
    }

    #[test]
    fn test_merge_chunk_results_preserves_content() {
        // 合并只改 page_index，不丢内容
        let chunks = vec![(0usize, vec![page(0), page(1)])];
        let merged = merge_chunk_results(chunks, 2);
        assert_eq!(merged[0].markdown_text, "page 0");
        assert_eq!(merged[1].markdown_text, "page 1");
    }

    #[test]
    fn test_merge_chunk_results_empty() {
        assert!(merge_chunk_results(vec![], 50).is_empty());
    }
}
