/// 带指数退避的异步重试工具
///
/// 对可重试的网络操作（如上传/下载清单和变更）进行最多 `max_retries` 次尝试，
/// 每次失败后以指数退避等待（500ms, 1s, 2s, ...）。
///
/// [P3 Fix] 注意：底层传输层（WebDAV/S3）可能有自己的重试机制（通常 3 次）。
/// 调用方应使用较低的 max_retries（建议 2）以避免叠加过多重试。
use super::SyncError;

#[cfg(feature = "data_governance")]
pub(crate) async fn retry_async<F, Fut, T>(op_name: &str, max_retries: u32, f: F) -> Result<T, SyncError>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, SyncError>>,
{
    let base_ms: u64 = 500;
    let mut last_err = SyncError::Network(format!("{}: 未知错误", op_name));
    for attempt in 0..max_retries {
        match f().await {
            Ok(v) => return Ok(v),
            Err(e) => {
                last_err = e;
                if attempt + 1 < max_retries {
                    let delay = base_ms * (1u64 << attempt);
                    tracing::warn!(
                        "[Sync] {} 重试 {}/{}: {}（等待 {}ms）",
                        op_name,
                        attempt + 1,
                        max_retries,
                        last_err,
                        delay
                    );
                    tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
                }
            }
        }
    }
    Err(last_err)
}
