//! # qbank 业务写命令的同步写门检查封装（[写门-接线] 2.3c）
//!
//! 云同步 apply 期间 `AppState.sync_write_guard` 指向正在同步的 db，
//! 业务写命令入口应调用 [`check_qbank_write_gate`] 检查，命中返回
//! `AppError::unknown("数据正在同步 (db=vfs), 请稍后重试")`（可重试语义）。
//!
//! ## 依赖方向说明
//!
//! `check_write_gate` / `SyncGuardSlot` 定义在 `data_governance` 模块，
//! 而该模块是可选 feature（`#[cfg(feature = "data_governance")]`）——
//! qbank 命令（commands.rs / question_sync_service.rs / qbank_grading）必须能在
//! 未启用该 feature 时独立编译。因此本封装做最小适配：
//!
//! - feature 启用时：转发 `data_governance::sync::permit::check_write_gate`，
//!   `SyncError` 映射为 `AppError::unknown`（消息保留"可重试"语义）；
//! - feature 未启用时：无同步写门，恒放行（`Ok(())`）。
//!
//! 原则：**写门只挡业务写，不挡同步自身** — 同步内部写不经过命令层，不受此检查影响。
//!
//! ## 数据库名决策
//!
//! qbank 写的是 **vfs.db**：`QuestionBankService` 持有 `vfs_db: Arc<VfsDatabase>`，
//! 题目/作答/统计/同步状态全部落在 vfs.db 的 questions 系列表，
//! 与 vfs/dstu 模块约定一致，统一使用数据库名 `"vfs"`。
//!
//! ## 错误映射说明
//!
//! qbank 命令的错误类型是 `AppError`（非模块化错误枚举）。为降低风险**不新增
//! AppError 变体**：`SyncError` → `AppError::unknown`，消息为
//! "数据正在同步 (db=vfs), 请稍后重试"（含"可重试"语义）。
//! 前端暂未针对 SYNC_IN_PROGRESS 码做专门处理，后续批次可加。

use crate::commands::AppState;
use crate::models::AppError;

/// 业务写命令入口的同步写门检查（qbank 侧最小适配封装）。
///
/// ## 用法
///
/// 所有会写 vfs 库的 Tauri 命令在入口处调用（`state: State<'_, AppState>`
/// 或 `app: AppHandle` 均取 `AppState.sync_write_guard`）：
///
/// ```rust,ignore
/// check_qbank_write_gate(&state)?; // state: State<'_, AppState>
/// // 或
/// check_qbank_write_gate(&app.state::<crate::commands::AppState>())?;
/// ```
///
/// 返回语义：
/// - 写门空闲 → `Ok(())`，业务写继续；
/// - 同步进行中（写门被占）→ `AppError::unknown("数据正在同步 (db=vfs), 请稍后重试")` — 可重试；
/// - 写门锁被瞬时占用 → `AppError::unknown`（消息含"可重试"）— 可重试。
#[cfg(feature = "data_governance")]
pub fn check_qbank_write_gate(state: &AppState) -> Result<(), AppError> {
    use crate::data_governance::sync::permit::check_write_gate;
    // qbank 写的是 vfs.db（QuestionBankService 持有 vfs_db），与 vfs/dstu 模块一致
    check_write_gate(&state.sync_write_guard, "vfs")
        .map_err(|_| AppError::unknown("数据正在同步 (db=vfs), 请稍后重试"))
}

/// 未启用 data_governance feature 时：无同步写门，恒放行。
#[cfg(not(feature = "data_governance"))]
pub fn check_qbank_write_gate(_state: &AppState) -> Result<(), AppError> {
    Ok(())
}
