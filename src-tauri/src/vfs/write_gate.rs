//! # VFS 业务写命令的同步写门检查封装（[写门-接线] 2.3c）
//!
//! 云同步 apply 期间 `AppState.sync_write_guard` 指向正在同步的 db，
//! 业务写命令入口应调用 [`check_vfs_write_gate`] 检查，命中返回
//! [`VfsError::SyncInProgress`]（可重试，前端提示"数据正在同步，请稍后重试"）。
//!
//! ## 依赖方向说明
//!
//! `check_write_gate` / `SyncGuardSlot` 定义在 `data_governance` 模块，
//! 而该模块是可选 feature（`#[cfg(feature = "data_governance")]`）——
//! vfs 必须能在未启用该 feature 时独立编译。因此本封装在 vfs 侧做最小适配：
//!
//! - feature 启用时：转发 `data_governance::sync::permit::check_write_gate`，
//!   `SyncError` 经 `From` 转换链变成 `VfsError`（见 `error.rs`）；
//! - feature 未启用时：无同步写门，恒放行（`Ok(())`）。
//!
//! 原则：**写门只挡业务写，不挡同步自身** — 同步内部写不经过命令层，不受此检查影响。
//! vfs 命令统一使用数据库名 `"vfs"`（`DatabaseId::Vfs::as_str()`）。

use crate::commands::AppState;
use crate::vfs::error::{VfsError, VfsResult};

/// 业务写命令入口的同步写门检查（vfs 侧最小适配封装）。
///
/// ## 用法
///
/// 所有会写 vfs 库的 Tauri 命令在入口处调用（`app: AppHandle` 或 `state: State<AppState>` 均取
/// `AppState.sync_write_guard`）：
///
/// ```rust,ignore
/// check_vfs_write_gate(&app.state::<crate::commands::AppState>())?;
/// // 或
/// check_vfs_write_gate(&state)?; // state: State<'_, AppState>
/// ```
///
/// 返回语义：
/// - 写门空闲 → `Ok(())`，业务写继续；
/// - 同步进行中（写门被占）→ `VfsError::SyncInProgress { db: 持有者 }` — 可重试；
/// - 写门锁被瞬时占用 → `VfsError::Other`（消息含"可重试"）— 可重试。
#[cfg(feature = "data_governance")]
pub fn check_vfs_write_gate(state: &AppState) -> VfsResult<()> {
    use crate::data_governance::sync::permit::check_write_gate;
    // vfs 命令统一使用 "vfs" 库名（DatabaseId::Vfs）
    check_write_gate(&state.sync_write_guard, "vfs").map_err(VfsError::from)
}

/// 未启用 data_governance feature 时：无同步写门，恒放行。
#[cfg(not(feature = "data_governance"))]
pub fn check_vfs_write_gate(_state: &AppState) -> VfsResult<()> {
    Ok(())
}
