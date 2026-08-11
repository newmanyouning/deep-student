//! 进度事件发射器
//!
//! 负责将同步进度事件发送到前端，支持节流以避免过于频繁的更新。
//!
//! ## 阶段语义（六阶段生命周期，设计文档 §3.5）
//! 同步命令的事件序列对齐为 6 个阶段命名：
//! `preflight`（命令入口预检）→ `export`（导出快照）→ `transfer`（上传/下载）→
//! `apply`（本地回放）→ `finalize`（收尾）→ `ended`（会话结束）。
//! 其中 `completed` / `failed` 为保留的旧终态阶段（旧前端以此触发完成/失败回调，
//! 必须始终作为事件流的**最后一个**事件，否则旧前端会卡在"运行中"UI）；
//! `ended` 在 `finalize` 之后、旧终态阶段之前发出。前端对未知阶段名降级显示。

use super::progress::{SyncPhase, SyncProgress};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex;

/// 进度事件名称
pub const EVENT_NAME: &str = "data-governance-sync-progress";

/// 节流间隔（毫秒）
const THROTTLE_INTERVAL: Duration = Duration::from_millis(100);

/// 进度发射器
///
/// 负责向前端发送同步进度事件。
/// 使用节流机制避免过于频繁的事件发送，但阶段变化时会强制发送。
pub struct SyncProgressEmitter {
    /// Tauri AppHandle
    app: AppHandle,
    /// 上次发射时间
    last_emit: Arc<Mutex<Option<Instant>>>,
    /// 上次发射的阶段（用于检测阶段变化）
    last_phase: Arc<Mutex<Option<SyncPhase>>>,
}

impl SyncProgressEmitter {
    /// 创建新的进度发射器
    ///
    /// # 参数
    /// * `app` - Tauri AppHandle
    pub fn new(app: AppHandle) -> Self {
        Self {
            app,
            last_emit: Arc::new(Mutex::new(None)),
            last_phase: Arc::new(Mutex::new(None)),
        }
    }

    /// 发射进度事件（带节流）
    ///
    /// 正常情况下会根据节流间隔限制发送频率，但以下情况会强制发送：
    /// - 阶段发生变化
    /// - 进度达到终止状态（完成或失败）
    ///
    /// # 参数
    /// * `progress` - 当前进度
    pub async fn emit(&self, progress: SyncProgress) {
        let now = Instant::now();

        let mut last_emit_guard = self.last_emit.lock().await;
        let mut last_phase_guard = self.last_phase.lock().await;

        // 检查是否需要强制发射
        let phase_changed = last_phase_guard.map_or(true, |p| p != progress.phase);
        let is_terminal = progress.phase.is_terminal();

        // 如果阶段变化或达到终止状态，强制发射
        if phase_changed || is_terminal {
            self.do_emit(&progress);
            *last_emit_guard = Some(now);
            *last_phase_guard = Some(progress.phase);
            return;
        }

        // 检查是否满足节流条件
        let should_emit = match *last_emit_guard {
            None => true,
            Some(last) => now.duration_since(last) >= THROTTLE_INTERVAL,
        };

        if should_emit {
            self.do_emit(&progress);
            *last_emit_guard = Some(now);
            *last_phase_guard = Some(progress.phase);
        }
    }

    /// 强制发射进度事件（不节流）
    ///
    /// 无论节流状态如何，立即发送进度事件。
    ///
    /// # 参数
    /// * `progress` - 当前进度
    pub fn emit_force(&self, progress: SyncProgress) {
        self.do_emit(&progress);
    }

    /// 发射预检阶段（命令入口：维护模式检查 → 全局锁 → 云凭据校验）
    pub async fn emit_preflight(&self) {
        self.emit(SyncProgress::preflight()).await;
    }

    /// 发射导出阶段（读 pending changes + 整行数据快照）
    pub async fn emit_export(&self) {
        self.emit(SyncProgress::export()).await;
    }

    /// 发射传输阶段（上传/下载变更 + 冲突检测，覆盖原 uploading/downloading）
    ///
    /// # 参数
    /// * `current` - 当前项目数
    /// * `total` - 总项目数
    /// * `current_item` - 当前处理的项目名（可选）
    pub async fn emit_transfer(&self, current: u64, total: u64, current_item: Option<String>) {
        let mut progress = SyncProgress::transfer(current, total);
        if let Some(item) = current_item {
            progress = progress.with_current_item(item);
        }
        self.emit(progress).await;
    }

    /// 发射应用阶段（下载变更本地事务回放）
    ///
    /// # 参数
    /// * `current` - 当前项目数
    /// * `total` - 总项目数
    /// * `current_item` - 当前处理的项目名（可选）
    pub async fn emit_apply(&self, current: u64, total: u64, current_item: Option<String>) {
        let mut progress = SyncProgress::apply(current, total);
        if let Some(item) = current_item {
            progress = progress.with_current_item(item);
        }
        self.emit(progress).await;
    }

    /// 发射收尾阶段（mark_synced + manifest 上传 + prune + 审计写入）
    pub async fn emit_finalizing(&self) {
        self.emit(SyncProgress::finalizing()).await;
    }

    /// 发射会话结束事件（六阶段命名最后一个事件，在 completed/failed 之前发出）
    pub async fn emit_ended(&self) {
        self.emit(SyncProgress::ended()).await;
    }

    /// 发射完成状态（保留的旧终态阶段，旧前端以 completed 判定完成回调）
    pub async fn emit_completed(&self) {
        self.emit(SyncProgress::completed()).await;
    }

    /// 发射失败状态（保留的旧终态阶段，旧前端以 failed 判定错误回调）
    ///
    /// # 参数
    /// * `error` - 错误信息
    pub async fn emit_failed(&self, error: impl Into<String>) {
        self.emit(SyncProgress::failed(error.into())).await;
    }

    /// 发射带速度信息的进度
    ///
    /// # 参数
    /// * `progress` - 基础进度
    /// * `speed_bytes_per_sec` - 传输速度（字节/秒）
    /// * `eta_seconds` - 预计剩余时间（秒）
    pub async fn emit_with_speed(
        &self,
        progress: SyncProgress,
        speed_bytes_per_sec: u64,
        eta_seconds: Option<u64>,
    ) {
        let progress = progress.with_speed(speed_bytes_per_sec, eta_seconds);
        self.emit(progress).await;
    }

    /// 实际发射事件
    fn do_emit(&self, progress: &SyncProgress) {
        if let Err(e) = self.app.emit(EVENT_NAME, progress) {
            tracing::error!("[sync_emitter] 发送进度事件失败: {}", e);
        } else {
            tracing::trace!(
                "[sync_emitter] 进度事件: phase={:?}, percent={:.1}%, current={}/{}",
                progress.phase,
                progress.percent,
                progress.current,
                progress.total
            );
        }
    }
}

impl Clone for SyncProgressEmitter {
    fn clone(&self) -> Self {
        Self {
            app: self.app.clone(),
            last_emit: Arc::clone(&self.last_emit),
            last_phase: Arc::clone(&self.last_phase),
        }
    }
}

/// 同步进度回调 trait
///
/// 为需要接收进度回调的同步操作提供统一接口。
#[async_trait::async_trait]
pub trait SyncProgressCallback: Send + Sync {
    /// 报告进度
    async fn on_progress(&self, progress: SyncProgress);

    /// 报告完成
    async fn on_complete(&self) {
        self.on_progress(SyncProgress::completed()).await;
    }

    /// 报告失败
    async fn on_error(&self, error: String) {
        self.on_progress(SyncProgress::failed(error)).await;
    }
}

#[async_trait::async_trait]
impl SyncProgressCallback for SyncProgressEmitter {
    async fn on_progress(&self, progress: SyncProgress) {
        self.emit(progress).await;
    }
}

/// 空进度回调（用于不需要进度报告的场景）
pub struct NoopProgressCallback;

#[async_trait::async_trait]
impl SyncProgressCallback for NoopProgressCallback {
    async fn on_progress(&self, _progress: SyncProgress) {
        // 不做任何事
    }
}

/// 可选的进度发射器包装
///
/// 用于同步方法中可选地接收进度回调。
pub struct OptionalEmitter {
    emitter: Option<SyncProgressEmitter>,
}

impl OptionalEmitter {
    /// 创建有发射器的包装
    pub fn with_emitter(emitter: SyncProgressEmitter) -> Self {
        Self {
            emitter: Some(emitter),
        }
    }

    /// 创建无发射器的包装
    pub fn none() -> Self {
        Self { emitter: None }
    }

    /// 发射进度（如果有发射器）
    pub async fn emit(&self, progress: SyncProgress) {
        if let Some(ref emitter) = self.emitter {
            emitter.emit(progress).await;
        }
    }

    /// 发射预检阶段（命令入口）
    pub async fn emit_preflight(&self) {
        self.emit(SyncProgress::preflight()).await;
    }

    /// 发射导出阶段（读 pending + 整行数据快照）
    pub async fn emit_export(&self) {
        self.emit(SyncProgress::export()).await;
    }

    /// 发射传输阶段（上传/下载，覆盖原 uploading/downloading）
    pub async fn emit_transfer(&self, current: u64, total: u64, current_item: Option<String>) {
        let mut progress = SyncProgress::transfer(current, total);
        if let Some(item) = current_item {
            progress = progress.with_current_item(item);
        }
        self.emit(progress).await;
    }

    /// 发射应用阶段（下载变更本地回放）
    pub async fn emit_apply(&self, current: u64, total: u64, current_item: Option<String>) {
        let mut progress = SyncProgress::apply(current, total);
        if let Some(item) = current_item {
            progress = progress.with_current_item(item);
        }
        self.emit(progress).await;
    }

    /// 发射收尾阶段（mark_synced + manifest 上传 + prune + 审计）
    pub async fn emit_finalizing(&self) {
        self.emit(SyncProgress::finalizing()).await;
    }

    /// 发射会话结束事件（六阶段命名最后一个事件）
    pub async fn emit_ended(&self) {
        self.emit(SyncProgress::ended()).await;
    }

    /// 发射完成状态（保留的旧终态阶段）
    pub async fn emit_completed(&self) {
        self.emit(SyncProgress::completed()).await;
    }

    /// 发射失败状态（保留的旧终态阶段）
    pub async fn emit_failed(&self, error: impl Into<String>) {
        self.emit(SyncProgress::failed(error.into())).await;
    }

    /// 是否有发射器
    pub fn has_emitter(&self) -> bool {
        self.emitter.is_some()
    }

    /// 同步强制发射（不节流）—— 专供 sync 回调闭包使用
    ///
    /// 与 `emit` 不同，此方法为同步，可在非 async 上下文（如上传进度回调）中安全调用。
    pub fn emit_force_sync(&self, progress: SyncProgress) {
        if let Some(ref emitter) = self.emitter {
            emitter.emit_force(progress);
        }
    }
}

impl Clone for OptionalEmitter {
    fn clone(&self) -> Self {
        Self {
            emitter: self.emitter.clone(),
        }
    }
}

impl From<Option<SyncProgressEmitter>> for OptionalEmitter {
    fn from(emitter: Option<SyncProgressEmitter>) -> Self {
        Self { emitter }
    }
}

impl From<SyncProgressEmitter> for OptionalEmitter {
    fn from(emitter: SyncProgressEmitter) -> Self {
        Self::with_emitter(emitter)
    }
}

// ==================== 文件级同步进度汇聚点（全局 sink） ====================
//
// 背景：文件级同步（工作区数据库 / VFS blob / 资产目录）由 SyncManager 内部的
// 多个循环执行，逐文件调用 put_file/get_file，期间无任何进度事件。大文件
// （几十 MB 的工作区库、PDF 附件）在坚果云限速下单文件就要几分钟，UI 进度条
// 因此"卡在中间不动"。
//
// 这些函数签名改动会波及大量调用方（含测试与非进度同步路径），因此采用
// 全局 sink：命令层在开始同步前挂上发射器，orchestrator 内部的字节回调
// 经 `report_file_sync_progress` 上报；同步结束（无论成败）必须
// `clear_file_sync_sink`。同步本身有全局信号量串行化，不存在并发写冲突。
//
// percent 策略：总字节数事先未知（取决于云端清单与本地扫描的差集），
// 采用"已完成文件数 + 当前文件字节比例"渐进推进，固定区间 65%–92%，
// 单调不回退（put_file 重试时会从 0 重新汇报当前文件字节）。

/// 文件同步进度区间下限
const FILE_SYNC_BAND_MIN: f32 = 65.0;
/// 文件同步进度区间上限（之后留给 prune/finalize）
const FILE_SYNC_BAND_MAX: f32 = 92.0;
/// 字节回调最小发射间隔
const FILE_SYNC_EMIT_INTERVAL: Duration = Duration::from_millis(200);

struct FileSyncSink {
    emitter: SyncProgressEmitter,
    last_emit: Instant,
    files_done: u32,
    max_percent: f32,
}

static FILE_SYNC_SINK: std::sync::OnceLock<std::sync::RwLock<Option<FileSyncSink>>> =
    std::sync::OnceLock::new();

fn file_sync_sink_slot() -> &'static std::sync::RwLock<Option<FileSyncSink>> {
    FILE_SYNC_SINK.get_or_init(|| std::sync::RwLock::new(None))
}

/// 挂上文件级同步进度发射器（同步命令入口调用）
pub fn set_file_sync_sink(emitter: SyncProgressEmitter) {
    if let Ok(mut guard) = file_sync_sink_slot().write() {
        *guard = Some(FileSyncSink {
            emitter,
            // 初始化为已过期，保证第一次回调立即发射
            last_emit: Instant::now() - FILE_SYNC_EMIT_INTERVAL,
            files_done: 0,
            max_percent: FILE_SYNC_BAND_MIN,
        });
    }
}

/// 卸下文件级同步进度发射器（同步命令结束/失败时调用）
pub fn clear_file_sync_sink() {
    if let Ok(mut guard) = file_sync_sink_slot().write() {
        *guard = None;
    }
}

/// 上报文件级同步的字节级进度（由 put_file/get_file 的进度回调调用）。
///
/// # 参数
/// * `label` - 操作标签，如 "上传附件" / "下载工作区数据库"
/// * `name` - 文件名或标识
/// * `done` / `total` - 当前文件已传输/总字节数
pub fn report_file_sync_progress(label: &str, name: &str, done: u64, total: u64) {
    let mut guard = match file_sync_sink_slot().write() {
        Ok(g) => g,
        Err(_) => return,
    };
    let sink = match guard.as_mut() {
        Some(s) => s,
        None => return,
    };

    let is_final = total > 0 && done >= total;
    if !is_final && sink.last_emit.elapsed() < FILE_SYNC_EMIT_INTERVAL {
        return;
    }
    sink.last_emit = Instant::now();

    let frac = if total > 0 {
        (done as f32 / total as f32).clamp(0.0, 1.0)
    } else {
        0.0
    };
    // 每完成一个文件 +1.5%，当前文件字节比例最多再 +3%，上限封顶
    let pct = (FILE_SYNC_BAND_MIN + sink.files_done as f32 * 1.5 + frac * 3.0)
        .min(FILE_SYNC_BAND_MAX)
        .max(sink.max_percent);
    sink.max_percent = pct;

    let item = if total > 0 {
        format!(
            "{} {} ({:.1}/{:.1} MB)",
            label,
            name,
            done as f64 / 1_048_576.0,
            total as f64 / 1_048_576.0
        )
    } else {
        format!("{} {}", label, name)
    };

    sink.emitter.emit_force(SyncProgress {
        phase: SyncPhase::Transfer,
        percent: pct,
        current: done,
        total,
        current_item: Some(item),
        speed_bytes_per_sec: None,
        eta_seconds: None,
        error: None,
    });
}

/// 标记一个文件传输完成（推进文件计数，从而推进进度区间）
pub fn report_file_sync_file_done() {
    if let Ok(mut guard) = file_sync_sink_slot().write() {
        if let Some(sink) = guard.as_mut() {
            sink.files_done = sink.files_done.saturating_add(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // 注意：由于 SyncProgressEmitter 需要 AppHandle，
    // 实际的集成测试需要在 Tauri 环境中运行。
    // 这里只测试辅助结构。

    #[tokio::test]
    async fn test_optional_emitter_none() {
        let emitter = OptionalEmitter::none();
        assert!(!emitter.has_emitter());

        // 这些调用应该不会 panic
        emitter.emit_preflight().await;
        emitter
            .emit_transfer(1, 10, Some("test.txt".to_string()))
            .await;
        emitter.emit_finalizing().await;
        emitter.emit_completed().await;
        emitter.emit_ended().await;
    }

    #[test]
    fn test_optional_emitter_from_none() {
        let emitter: OptionalEmitter = None.into();
        assert!(!emitter.has_emitter());
    }

    #[tokio::test]
    async fn test_noop_callback() {
        let callback = NoopProgressCallback;
        // 这些调用应该不会 panic
        callback.on_progress(SyncProgress::preflight()).await;
        callback.on_complete().await;
        callback.on_error("test error".to_string()).await;
    }
}
