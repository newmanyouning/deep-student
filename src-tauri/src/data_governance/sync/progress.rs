//! 同步进度管理
//!
//! 提供同步过程中的进度跟踪、速度计算和 ETA 估算功能。

use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// 同步阶段
///
/// ## 阶段语义（设计文档 §3.5 六阶段生命周期）
/// 命令链事件序列对齐为 6 个阶段命名，前端对未知阶段名降级显示（向后兼容）：
/// - `preflight` — 预检：维护模式检查 → 全局锁(60s) → 云凭据校验（命令开头发射）
/// - `export` — 导出：读 pending changes + 整行数据快照（只读）
/// - `transfer` — 传输：上传/下载变更 + 冲突检测
/// - `apply` — 应用：下载变更本地事务回放
/// - `finalize` — 收尾：mark_synced + manifest 上传 + prune + 审计写入
/// - `ended` — 会话结束：同步会话收尾后的最终事件（终态）
///
/// ## 向后兼容保留
/// `completed` / `failed` 为保留的旧终态阶段：旧前端以 `phase === 'completed'` /
/// `phase === 'failed'` 触发完成/失败回调，事件序列末尾始终以二者收尾（最后一个事件），
/// `ended` 在二者之前发出——若 `ended` 落在末尾，旧前端会因未知终态卡在"运行中"UI。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SyncPhase {
    /// 预检阶段（命令入口）
    Preflight,
    /// 导出阶段（读 pending + 整行数据快照）
    Export,
    /// 传输阶段（上传/下载 + 冲突检测）
    Transfer,
    /// 应用阶段（下载变更本地回放）
    Apply,
    /// 收尾阶段（mark_synced + manifest 上传 + prune + 审计）
    Finalize,
    /// 会话结束（终态，在 completed/failed 之前发出）
    Ended,
    /// 已完成（终态，保留兼容：旧前端以 completed 判定完成回调）
    Completed,
    /// 失败（终态，保留兼容：旧前端以 failed 判定错误回调）
    Failed,
}

impl SyncPhase {
    /// 获取阶段的显示名称
    pub fn display_name(&self) -> &'static str {
        match self {
            SyncPhase::Preflight => "预检",
            SyncPhase::Export => "导出",
            SyncPhase::Transfer => "传输",
            SyncPhase::Apply => "应用",
            SyncPhase::Finalize => "收尾",
            SyncPhase::Ended => "会话结束",
            SyncPhase::Completed => "已完成",
            SyncPhase::Failed => "失败",
        }
    }

    /// 判断是否为终止状态
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            SyncPhase::Ended | SyncPhase::Completed | SyncPhase::Failed
        )
    }
}

/// 同步进度
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncProgress {
    /// 当前阶段
    pub phase: SyncPhase,
    /// 进度百分比 (0-100)
    pub percent: f32,
    /// 当前处理的项目数
    pub current: u64,
    /// 总项目数
    pub total: u64,
    /// 当前处理的文件/记录名
    pub current_item: Option<String>,
    /// 传输速度（字节/秒）
    pub speed_bytes_per_sec: Option<u64>,
    /// 预计剩余时间（秒）
    pub eta_seconds: Option<u64>,
    /// 错误信息（如果有）
    pub error: Option<String>,
}

impl Default for SyncProgress {
    fn default() -> Self {
        Self {
            phase: SyncPhase::Preflight,
            percent: 0.0,
            current: 0,
            total: 0,
            current_item: None,
            speed_bytes_per_sec: None,
            eta_seconds: None,
            error: None,
        }
    }
}

impl SyncProgress {
    /// 创建预检阶段（命令入口）的进度
    pub fn preflight() -> Self {
        Self {
            phase: SyncPhase::Preflight,
            percent: 0.0,
            current: 0,
            total: 0,
            current_item: None,
            speed_bytes_per_sec: None,
            eta_seconds: None,
            error: None,
        }
    }

    /// 创建导出阶段（读 pending + 整行数据快照）的进度
    pub fn export() -> Self {
        Self {
            phase: SyncPhase::Export,
            percent: 5.0,
            current: 0,
            total: 0,
            current_item: None,
            speed_bytes_per_sec: None,
            eta_seconds: None,
            error: None,
        }
    }

    /// 创建传输阶段（上传/下载 + 冲突检测）的进度
    pub fn transfer(current: u64, total: u64) -> Self {
        let percent = if total > 0 {
            10.0 + (current as f32 / total as f32) * 40.0 // 10% - 50%
        } else {
            10.0
        };

        Self {
            phase: SyncPhase::Transfer,
            percent,
            current,
            total,
            current_item: None,
            speed_bytes_per_sec: None,
            eta_seconds: None,
            error: None,
        }
    }

    /// 创建应用阶段（下载变更本地回放）的进度
    pub fn apply(current: u64, total: u64) -> Self {
        let percent = if total > 0 {
            80.0 + (current as f32 / total as f32) * 18.0 // 80% - 98%
        } else {
            80.0
        };

        Self {
            phase: SyncPhase::Apply,
            percent,
            current,
            total,
            current_item: None,
            speed_bytes_per_sec: None,
            eta_seconds: None,
            error: None,
        }
    }

    /// 创建收尾阶段（mark_synced + manifest 上传 + prune + 审计）的进度
    pub fn finalizing() -> Self {
        Self {
            phase: SyncPhase::Finalize,
            percent: 99.0,
            current: 0,
            total: 0,
            current_item: None,
            speed_bytes_per_sec: None,
            eta_seconds: None,
            error: None,
        }
    }

    /// 创建完成状态的进度
    pub fn completed() -> Self {
        Self {
            phase: SyncPhase::Completed,
            percent: 100.0,
            current: 0,
            total: 0,
            current_item: None,
            speed_bytes_per_sec: None,
            eta_seconds: None,
            error: None,
        }
    }

    /// 创建失败状态的进度
    pub fn failed(error: String) -> Self {
        Self {
            phase: SyncPhase::Failed,
            percent: 0.0,
            current: 0,
            total: 0,
            current_item: None,
            speed_bytes_per_sec: None,
            eta_seconds: None,
            error: Some(error),
        }
    }

    /// 创建会话结束状态的进度（终态，在 completed/failed 之前发出）
    pub fn ended() -> Self {
        Self {
            phase: SyncPhase::Ended,
            percent: 100.0,
            current: 0,
            total: 0,
            current_item: None,
            speed_bytes_per_sec: None,
            eta_seconds: None,
            error: None,
        }
    }

    /// 设置当前处理的项目名称
    pub fn with_current_item(mut self, item: impl Into<String>) -> Self {
        self.current_item = Some(item.into());
        self
    }

    /// 设置传输速度和 ETA
    pub fn with_speed(mut self, speed_bytes_per_sec: u64, eta_seconds: Option<u64>) -> Self {
        self.speed_bytes_per_sec = Some(speed_bytes_per_sec);
        self.eta_seconds = eta_seconds;
        self
    }

    /// 更新进度百分比
    pub fn with_percent(mut self, percent: f32) -> Self {
        self.percent = percent.clamp(0.0, 100.0);
        self
    }
}

/// 速度计算器（滑动窗口）
///
/// 使用滑动窗口计算传输速度，提供更平滑的速度估算。
pub struct SpeedCalculator {
    /// 采样数据：(时间点, 累计字节数)
    samples: Vec<(Instant, u64)>,
    /// 窗口大小
    window_size: usize,
}

impl SpeedCalculator {
    /// 创建新的速度计算器
    ///
    /// # 参数
    /// * `window_size` - 滑动窗口大小（采样数量）
    pub fn new(window_size: usize) -> Self {
        Self {
            samples: Vec::with_capacity(window_size),
            window_size: window_size.max(2), // 至少需要 2 个点计算速度
        }
    }

    /// 使用默认窗口大小创建（10 个采样点）
    pub fn default_window() -> Self {
        Self::new(10)
    }

    /// 添加采样数据
    ///
    /// # 参数
    /// * `bytes` - 当前累计传输的字节数
    pub fn add_sample(&mut self, bytes: u64) {
        let now = Instant::now();

        // 如果窗口已满，移除最旧的采样
        if self.samples.len() >= self.window_size {
            self.samples.remove(0);
        }

        self.samples.push((now, bytes));
    }

    /// 计算当前传输速度（字节/秒）
    ///
    /// 使用滑动窗口内的第一个和最后一个采样点计算平均速度。
    ///
    /// # 返回
    /// * `Some(speed)` - 计算出的速度（字节/秒）
    /// * `None` - 采样数据不足（少于 2 个点）
    pub fn calculate_speed(&self) -> Option<u64> {
        if self.samples.len() < 2 {
            return None;
        }

        let first = &self.samples[0];
        let last = &self.samples[self.samples.len() - 1];

        let duration = last.0.duration_since(first.0);
        let bytes_diff = last.1.saturating_sub(first.1);

        if duration.is_zero() {
            return None;
        }

        let speed = (bytes_diff as f64 / duration.as_secs_f64()) as u64;
        Some(speed)
    }

    /// 计算预计剩余时间（秒）
    ///
    /// # 参数
    /// * `remaining` - 剩余字节数
    ///
    /// # 返回
    /// * `Some(seconds)` - 预计剩余秒数
    /// * `None` - 无法计算（速度为零或数据不足）
    pub fn calculate_eta(&self, remaining: u64) -> Option<u64> {
        let speed = self.calculate_speed()?;

        if speed == 0 {
            return None;
        }

        Some(remaining / speed)
    }

    /// 重置计算器
    pub fn reset(&mut self) {
        self.samples.clear();
    }

    /// 获取当前采样数量
    pub fn sample_count(&self) -> usize {
        self.samples.len()
    }
}

/// 进度跟踪器
///
/// 组合进度状态和速度计算器，提供完整的进度跟踪功能。
pub struct ProgressTracker {
    /// 当前进度
    progress: SyncProgress,
    /// 速度计算器
    speed_calculator: SpeedCalculator,
    /// 开始时间
    start_time: Instant,
    /// 总字节数（用于计算 ETA）
    total_bytes: u64,
    /// 已传输字节数
    transferred_bytes: u64,
}

impl ProgressTracker {
    /// 创建新的进度跟踪器
    pub fn new() -> Self {
        Self {
            progress: SyncProgress::preflight(),
            speed_calculator: SpeedCalculator::default_window(),
            start_time: Instant::now(),
            total_bytes: 0,
            transferred_bytes: 0,
        }
    }

    /// 设置总字节数
    pub fn set_total_bytes(&mut self, total: u64) {
        self.total_bytes = total;
    }

    /// 更新传输字节数
    pub fn update_transferred(&mut self, bytes: u64) {
        self.transferred_bytes = bytes;
        self.speed_calculator.add_sample(bytes);
    }

    /// 更新阶段
    pub fn set_phase(&mut self, phase: SyncPhase) {
        self.progress.phase = phase;
    }

    /// 更新项目进度
    pub fn update_items(&mut self, current: u64, total: u64) {
        self.progress.current = current;
        self.progress.total = total;
    }

    /// 设置当前处理的项目
    pub fn set_current_item(&mut self, item: impl Into<String>) {
        self.progress.current_item = Some(item.into());
    }

    /// 获取当前进度（包含速度和 ETA）
    pub fn get_progress(&self) -> SyncProgress {
        let mut progress = self.progress.clone();

        // 计算速度和 ETA
        if let Some(speed) = self.speed_calculator.calculate_speed() {
            progress.speed_bytes_per_sec = Some(speed);

            let remaining = self.total_bytes.saturating_sub(self.transferred_bytes);
            progress.eta_seconds = self.speed_calculator.calculate_eta(remaining);
        }

        progress
    }

    /// 获取已用时间
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// 标记完成
    pub fn complete(&mut self) {
        self.progress = SyncProgress::completed();
    }

    /// 标记失败
    pub fn fail(&mut self, error: impl Into<String>) {
        self.progress = SyncProgress::failed(error.into());
    }
}

impl Default for ProgressTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_sync_phase_display_name() {
        assert_eq!(SyncPhase::Preflight.display_name(), "预检");
        assert_eq!(SyncPhase::Transfer.display_name(), "传输");
        assert_eq!(SyncPhase::Finalize.display_name(), "收尾");
        assert_eq!(SyncPhase::Completed.display_name(), "已完成");
    }

    #[test]
    fn test_sync_phase_is_terminal() {
        assert!(!SyncPhase::Preflight.is_terminal());
        assert!(!SyncPhase::Transfer.is_terminal());
        assert!(SyncPhase::Ended.is_terminal());
        assert!(SyncPhase::Completed.is_terminal());
        assert!(SyncPhase::Failed.is_terminal());
    }

    #[test]
    fn test_sync_progress_builders() {
        let preflight = SyncProgress::preflight();
        assert_eq!(preflight.phase, SyncPhase::Preflight);
        assert_eq!(preflight.percent, 0.0);

        let export = SyncProgress::export();
        assert_eq!(export.phase, SyncPhase::Export);
        assert_eq!(export.percent, 5.0);

        let transfer = SyncProgress::transfer(50, 100);
        assert_eq!(transfer.phase, SyncPhase::Transfer);
        assert_eq!(transfer.current, 50);
        assert_eq!(transfer.total, 100);
        assert!(transfer.percent >= 10.0 && transfer.percent <= 50.0);

        let apply = SyncProgress::apply(50, 100);
        assert_eq!(apply.phase, SyncPhase::Apply);
        assert!(apply.percent >= 80.0 && apply.percent <= 98.0);

        let finalizing = SyncProgress::finalizing();
        assert_eq!(finalizing.phase, SyncPhase::Finalize);
        assert_eq!(finalizing.percent, 99.0);

        let ended = SyncProgress::ended();
        assert_eq!(ended.phase, SyncPhase::Ended);
        assert_eq!(ended.percent, 100.0);

        let completed = SyncProgress::completed();
        assert_eq!(completed.phase, SyncPhase::Completed);
        assert_eq!(completed.percent, 100.0);

        let failed = SyncProgress::failed("test error".to_string());
        assert_eq!(failed.phase, SyncPhase::Failed);
        assert_eq!(failed.error, Some("test error".to_string()));
    }

    #[test]
    fn test_sync_progress_with_methods() {
        let progress = SyncProgress::transfer(10, 100)
            .with_current_item("file.txt")
            .with_speed(1024, Some(60))
            .with_percent(25.0);

        assert_eq!(progress.current_item, Some("file.txt".to_string()));
        assert_eq!(progress.speed_bytes_per_sec, Some(1024));
        assert_eq!(progress.eta_seconds, Some(60));
        assert_eq!(progress.percent, 25.0);
    }

    #[test]
    fn test_speed_calculator_insufficient_samples() {
        let calc = SpeedCalculator::new(5);
        assert!(calc.calculate_speed().is_none());

        let mut calc = SpeedCalculator::new(5);
        calc.add_sample(100);
        assert!(calc.calculate_speed().is_none());
    }

    #[test]
    fn test_speed_calculator_with_samples() {
        let mut calc = SpeedCalculator::new(5);

        calc.add_sample(0);
        thread::sleep(Duration::from_millis(100));
        calc.add_sample(1000);

        let speed = calc.calculate_speed();
        assert!(speed.is_some());
        // 约 10000 字节/秒（1000 字节 / 0.1 秒），允许一定误差
        let speed = speed.unwrap();
        assert!(speed > 5000 && speed < 15000);
    }

    #[test]
    fn test_speed_calculator_eta() {
        let mut calc = SpeedCalculator::new(5);

        calc.add_sample(0);
        thread::sleep(Duration::from_millis(100));
        calc.add_sample(1000);

        // 剩余 10000 字节，速度约 10000 字节/秒，ETA 约 1 秒
        let eta = calc.calculate_eta(10000);
        assert!(eta.is_some());
        let eta = eta.unwrap();
        assert!(eta <= 3); // 允许一定误差
    }

    #[test]
    fn test_speed_calculator_window_overflow() {
        let mut calc = SpeedCalculator::new(3);

        calc.add_sample(0);
        calc.add_sample(100);
        calc.add_sample(200);
        calc.add_sample(300); // 这会移除第一个采样

        assert_eq!(calc.sample_count(), 3);
    }

    #[test]
    fn test_progress_tracker() {
        let mut tracker = ProgressTracker::new();

        tracker.set_total_bytes(10000);
        tracker.set_phase(SyncPhase::Transfer);
        tracker.update_items(0, 10);
        tracker.set_current_item("test.txt");

        let progress = tracker.get_progress();
        assert_eq!(progress.phase, SyncPhase::Transfer);
        assert_eq!(progress.total, 10);
        assert_eq!(progress.current_item, Some("test.txt".to_string()));

        tracker.complete();
        let progress = tracker.get_progress();
        assert_eq!(progress.phase, SyncPhase::Completed);
    }

    #[test]
    fn test_progress_tracker_fail() {
        let mut tracker = ProgressTracker::new();
        tracker.fail("Network error");

        let progress = tracker.get_progress();
        assert_eq!(progress.phase, SyncPhase::Failed);
        assert_eq!(progress.error, Some("Network error".to_string()));
    }
}
