// ==================== 备份/恢复共享函数和类型 ====================
//
// 本文件包含 commands_backup 和 commands_restore 之间共享的函数和类型，
// 用于打破两个模块之间的循环依赖。
//
// 共享内容包括：
// - get_app_data_dir, get_backup_dir: 路径解析
// - sanitize_path_for_user, validate_backup_id, ensure_existing_path_within_backup_dir: 路径验证
// - acquire_backup_global_permit: 全局互斥锁
// - BackupJobStartResponse: 任务启动响应
// - execute_backup_with_progress_resumable, execute_zip_import_with_progress_resumable: 可恢复的执行函数

use std::path::{Path, PathBuf};
use tauri::Manager;
use tracing::{error, info};

use super::{DataGovernanceError, DataGovernanceResult};

#[cfg(feature = "data_governance")]
use super::audit::{AuditLog, AuditOperation};
use crate::backup_common::BACKUP_GLOBAL_LIMITER;
use crate::backup_job_manager::{
    BackupJobContext, BackupJobParams, BackupJobPhase, BackupJobResultPayload,
};

#[cfg(feature = "data_governance")]
use super::commands::try_save_audit_log;

// ==================== 路径解析 ====================

/// 获取应用数据基础目录（Tauri app_data_dir）
///
/// 注意：此目录是基础目录，**不是**运行时数据库/资产的实际存储位置。
/// 运行时存储位置请使用 `get_active_data_dir`。
pub(super) fn get_app_data_dir(app: &tauri::AppHandle) -> DataGovernanceResult<PathBuf> {
    app.path()
        .app_data_dir()
        .map_err(|e| DataGovernanceError::from(format!("获取应用数据目录失败: {}", e)))
}

/// 获取备份目录
pub(super) fn get_backup_dir(app_data_dir: &PathBuf) -> PathBuf {
    app_data_dir.join("backups")
}

// ==================== 路径安全 ====================

/// 将路径中用户主目录替换为 "~/"，避免在面向用户的错误信息中泄露完整文件系统路径
pub(super) fn sanitize_path_for_user(path: &Path) -> String {
    let path_str = path.to_string_lossy();
    if let Some(home) = dirs::home_dir() {
        let home_str = home.to_string_lossy();
        if path_str.starts_with(home_str.as_ref()) {
            return format!("~/{}", &path_str[home_str.len()..].trim_start_matches('/'));
        }
    }
    // 如果无法获取 home 目录，至少只保留最后两级路径
    let components: Vec<&str> = path_str.split('/').filter(|s| !s.is_empty()).collect();
    if components.len() > 2 {
        format!(".../{}", components[components.len() - 2..].join("/"))
    } else {
        path_str.to_string()
    }
}

/// 验证用户提供的备份 ID
///
/// 确保 backup_id 不包含路径遍历、URL 编码、非法字符或路径分隔符。
/// 最大长度为 128 个 ASCII 字符。
pub(super) fn validate_backup_id(raw_backup_id: &str) -> DataGovernanceResult<String> {
    let trimmed = raw_backup_id.trim();
    if trimmed.is_empty() {
        return Err(DataGovernanceError::from("backup_id 不能为空".to_string()));
    }

    let decoded = urlencoding::decode(trimmed)
        .map_err(|e| DataGovernanceError::from(format!("backup_id 编码非法: {}", e)))?
        .into_owned();

    if decoded != trimmed {
        return Err(DataGovernanceError::from("backup_id 不允许包含 URL 编码".to_string()));
    }

    if decoded.len() > 128 {
        return Err(DataGovernanceError::from("backup_id 长度超限（最大 128）".to_string()));
    }

    if decoded.contains('/')
        || decoded.contains('\\')
        || decoded.contains("..")
        || decoded.starts_with('.')
    {
        return Err(DataGovernanceError::from("backup_id 包含非法路径片段".to_string()));
    }

    if !decoded
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.'))
    {
        return Err(DataGovernanceError::from("backup_id 包含非法字符".to_string()));
    }

    Ok(decoded)
}

/// 验证路径在备份目录范围内（防止路径遍历越界）
pub(super) fn ensure_existing_path_within_backup_dir(
    path: &std::path::Path,
    backup_dir: &std::path::Path,
) -> DataGovernanceResult<()> {
    let canonical_backup_dir = std::fs::canonicalize(backup_dir)?;
    let canonical_path = std::fs::canonicalize(path)?;

    if !canonical_path.starts_with(&canonical_backup_dir) {
        return Err(DataGovernanceError::from(format!(
            "备份路径越界: {}。请确认路径在备份目录内，或前往「设置 > 数据治理」重新选择备份目录",
            sanitize_path_for_user(&canonical_path)
        )));
    }

    Ok(())
}

// ==================== 全局互斥锁 ====================

/// 获取全局备份互斥锁（取消友好）
///
/// 背景：备份/恢复/ZIP 导入导出都会读写同一套备份目录和数据库文件。
/// 若并发执行，容易导致：
/// - 备份目录写入覆盖（尤其是历史上秒级时间戳目录名）
/// - restore 与备份/导出并发，造成一致性风险或 Windows 文件锁问题
///
/// 这里统一使用 `backup_common::BACKUP_GLOBAL_LIMITER` 串行化所有相关任务。
pub(super) async fn acquire_backup_global_permit(
    job_ctx: &BackupJobContext,
    waiting_message: &str,
) -> Option<tokio::sync::OwnedSemaphorePermit> {
    // 向前端暴露"正在等待"状态（不阻塞 UI）
    job_ctx.mark_running(
        BackupJobPhase::Queued,
        0.0,
        Some(waiting_message.to_string()),
        0,
        0,
    );

    let fut = BACKUP_GLOBAL_LIMITER.clone().acquire_owned();
    tokio::pin!(fut);

    loop {
        if job_ctx.is_cancelled() {
            job_ctx.cancelled(Some("用户取消任务".to_string()));
            return None;
        }

        tokio::select! {
            permit = &mut fut => {
                return match permit {
                    Ok(p) => Some(p),
                    Err(e) => {
                        job_ctx.fail(format!("获取全局备份锁失败: {}", e));
                        None
                    }
                };
            }
            _ = tokio::time::sleep(std::time::Duration::from_millis(200)) => {}
        }
    }
}

// ==================== 共享响应类型 ====================

/// 后台备份任务启动响应
#[derive(Debug, Clone, serde::Serialize)]
pub struct BackupJobStartResponse {
    /// 任务 ID，用于查询状态和取消
    pub job_id: String,
    /// 任务类型
    pub kind: String,
    /// 初始状态
    pub status: String,
    /// 提示消息
    pub message: String,
}

// ==================== 可恢复的执行函数 ====================

/// 执行可恢复的备份（支持从失败中重新开始）
///
/// 与 execute_backup_with_progress 类似，但会：
/// 1. 设置任务参数供持久化（用于失败后重新启动）
/// 2. 初始化检查点追踪
/// 3. 在处理每个数据库后更新检查点（用于进度记录）
///
/// 注意：由于 BackupManager 的备份方法是原子操作（一次性备份所有数据库），
/// 恢复实际上是使用相同参数重新执行完整备份，而非从中断点继续。
/// 检查点信息仅用于进度显示和日志追踪。
pub(super) async fn execute_backup_with_progress_resumable(
    app: tauri::AppHandle,
    job_ctx: BackupJobContext,
    backup_type: String,
    base_version: Option<String>,
    include_assets: bool,
    asset_types: Option<Vec<String>>,
) {
    use super::backup::{AssetBackupConfig, AssetType, BackupManager};
    use std::time::Instant;

    let start = Instant::now();

    // 全局互斥：避免备份/恢复/ZIP 导入导出并发
    let _global_permit =
        match acquire_backup_global_permit(&job_ctx, "正在等待其他备份/恢复任务完成...").await
        {
            Some(p) => p,
            None => return,
        };

    // 设置任务参数（用于持久化和恢复）
    job_ctx.set_params(BackupJobParams {
        backup_type: Some(backup_type.clone()),
        base_version: base_version.clone(),
        include_assets,
        asset_types: asset_types.clone(),
        ..Default::default()
    });

    // 获取应用数据目录
    let app_data_dir = match get_app_data_dir(&app) {
        Ok(dir) => dir,
        Err(e) => {
            job_ctx.fail(format!("获取应用数据目录失败: {}", e));
            return;
        }
    };
    let backup_dir = get_backup_dir(&app_data_dir);

    // 确保备份目录存在
    if !backup_dir.exists() {
        if let Err(e) = std::fs::create_dir_all(&backup_dir) {
            job_ctx.fail(format!("创建备份目录失败: {}", e));
            return;
        }
    }

    // 检查是否从失败任务恢复（备份操作是原子的，恢复 = 重新执行）
    let previous_items = job_ctx.get_processed_items();
    let is_retrying = !previous_items.is_empty();

    if is_retrying {
        info!("[data_governance] 从失败任务重新执行备份（原子操作，重新开始）");
    }

    // 阶段 1: 准备中
    job_ctx.mark_running(
        BackupJobPhase::Scan,
        5.0,
        Some(if is_retrying {
            "重新执行备份，正在准备...".to_string()
        } else {
            "正在准备备份...".to_string()
        }),
        0,
        4, // 总共 4 个数据库
    );

    // 初始化检查点（始终重新初始化，因为备份是原子操作）
    job_ctx.init_checkpoint(4); // 4 个数据库

    // 检查取消
    if job_ctx.is_cancelled() {
        job_ctx.cancelled(Some("用户取消备份".to_string()));
        return;
    }

    // 创建备份管理器
    let mut manager = BackupManager::new(backup_dir);
    manager.set_app_data_dir(app_data_dir.clone());
    manager.set_app_version(env!("CARGO_PKG_VERSION").to_string());

    // 阶段 2: 执行 checkpoint
    job_ctx.mark_running(
        BackupJobPhase::Checkpoint,
        10.0,
        Some("正在执行数据库 checkpoint...".to_string()),
        0,
        4,
    );

    if job_ctx.is_cancelled() {
        job_ctx.cancelled(Some("用户取消备份".to_string()));
        return;
    }

    // 执行备份（原子操作：一次性备份所有数据库）
    let result = match backup_type.as_str() {
        "incremental" => {
            let base = match base_version {
                Some(v) => v,
                None => {
                    job_ctx.fail("增量备份需要指定 base_version 参数".to_string());
                    return;
                }
            };

            job_ctx.mark_running(
                BackupJobPhase::Compress,
                30.0,
                Some("正在执行增量备份...".to_string()),
                0,
                4,
            );

            manager.backup_incremental(&base)
        }
        _ => {
            if include_assets {
                let asset_config = if let Some(types) = asset_types {
                    let parsed_types: Vec<AssetType> = types
                        .iter()
                        .filter_map(|s| AssetType::from_str(s))
                        .collect();
                    if parsed_types.is_empty() {
                        AssetBackupConfig::default()
                    } else {
                        AssetBackupConfig {
                            asset_types: parsed_types,
                            ..Default::default()
                        }
                    }
                } else {
                    AssetBackupConfig::default()
                };

                job_ctx.mark_running(
                    BackupJobPhase::Compress,
                    30.0,
                    Some("正在备份数据库和资产文件...".to_string()),
                    0,
                    4,
                );

                manager.backup_with_assets(Some(asset_config))
            } else {
                job_ctx.mark_running(
                    BackupJobPhase::Compress,
                    30.0,
                    Some("正在备份数据库...".to_string()),
                    0,
                    4,
                );

                manager.backup_full()
            }
        }
    };

    if job_ctx.is_cancelled() {
        job_ctx.cancelled(Some("用户取消备份".to_string()));
        return;
    }

    // 阶段 4: 验证
    job_ctx.mark_running(
        BackupJobPhase::Verify,
        80.0,
        Some("正在验证备份...".to_string()),
        3,
        4,
    );

    let duration_ms = start.elapsed().as_millis() as u64;

    match result {
        Ok(manifest) => {
            // 标记所有数据库为已处理
            for file in &manifest.files {
                if let Some(db_id) = &file.database_id {
                    job_ctx.update_checkpoint(db_id);
                }
            }

            let db_size: u64 = manifest.files.iter().map(|f| f.size).sum();
            let asset_size: u64 = manifest.assets.as_ref().map(|a| a.total_size).unwrap_or(0);
            let backup_size = db_size + asset_size;

            let databases_backed_up: Vec<String> = manifest
                .files
                .iter()
                .filter_map(|f| f.database_id.clone())
                .collect();

            info!(
                "[data_governance] 后台备份成功: id={}, files={}, size={}, duration={}ms, retried={}",
                manifest.backup_id,
                manifest.files.len(),
                backup_size,
                duration_ms,
                is_retrying
            );

            let result_payload = BackupJobResultPayload {
                success: true,
                output_path: Some(manifest.backup_id.clone()),
                resolved_path: None,
                message: Some(format!(
                    "备份完成: {} 个数据库, {} 字节{}",
                    databases_backed_up.len(),
                    backup_size,
                    if is_retrying { " (重新执行)" } else { "" }
                )),
                error: None,
                duration_ms: Some(duration_ms),
                stats: Some(serde_json::json!({
                    "databases_backed_up": databases_backed_up,
                    "backup_size": backup_size,
                    "db_files": manifest.files.len(),
                    "asset_files": manifest.assets.as_ref().map(|a| a.total_files).unwrap_or(0),
                    "retried_from_failure": is_retrying,
                })),
                requires_restart: false,
                checkpoint_path: None,
                resumable_job_id: None,
            };

            job_ctx.complete(
                Some(format!("备份完成: {}", manifest.backup_id)),
                databases_backed_up.len() as u64,
                databases_backed_up.len() as u64,
                result_payload,
            );
        }
        Err(e) => {
            error!("[data_governance] 后台备份失败: {}", e);
            job_ctx.fail(format!("备份失败: {}", e));
        }
    }
}

/// 执行可恢复的 ZIP 导入（带断点续传支持）
///
/// 与 execute_zip_import_with_progress 类似，但会：
/// 1. 设置任务参数供持久化
/// 2. 初始化检查点
/// 3. 断点续传：跳过目标目录中已存在且大小匹配的文件
pub(super) async fn execute_zip_import_with_progress_resumable(
    app: tauri::AppHandle,
    job_ctx: BackupJobContext,
    zip_file_path: PathBuf,
    backup_id: Option<String>,
) {
    use super::backup::zip_export::{import_backup_from_zip_resumable, ZipImportPhase};
    use std::time::Instant;

    let start = Instant::now();

    // 全局互斥：避免备份/恢复/ZIP 导入导出并发
    let _global_permit =
        match acquire_backup_global_permit(&job_ctx, "正在等待其他备份/恢复任务完成...").await
        {
            Some(p) => p,
            None => return,
        };

    // 设置任务参数（用于持久化和恢复）
    job_ctx.set_params(BackupJobParams {
        zip_path: Some(zip_file_path.to_string_lossy().to_string()),
        backup_id: backup_id.clone(),
        ..Default::default()
    });

    // 获取应用数据目录
    let app_data_dir = match get_app_data_dir(&app) {
        Ok(dir) => dir,
        Err(e) => {
            job_ctx.fail(format!("获取应用数据目录失败: {}", e));
            return;
        }
    };
    let backup_dir = get_backup_dir(&app_data_dir);

    // 确保备份目录存在
    if !backup_dir.exists() {
        if let Err(e) = std::fs::create_dir_all(&backup_dir) {
            job_ctx.fail(format!("创建备份目录失败: {}", e));
            return;
        }
    }

    // 获取已处理的项目列表（用于断点续传）
    let processed_items = job_ctx.get_processed_items();
    let is_resuming = !processed_items.is_empty();

    if is_resuming {
        info!(
            "[data_governance] 从检查点恢复 ZIP 导入任务，已处理 {} 个文件",
            processed_items.len()
        );
    }

    // 确定备份 ID
    let generated_backup_id = backup_id.unwrap_or_else(|| {
        use uuid::Uuid;
        let now = chrono::Utc::now();
        let timestamp = now.format("%Y%m%d_%H%M%S").to_string();
        let millis = now.timestamp_subsec_millis();
        let rand8 = &Uuid::new_v4().simple().to_string()[..8];
        format!("{}_{}_{:03}_imported", timestamp, rand8, millis)
    });

    let target_backup_id = match validate_backup_id(&generated_backup_id) {
        Ok(id) => id,
        Err(e) => {
            job_ctx.fail(format!("backup_id 非法: {}", e));
            return;
        }
    };

    let target_dir = backup_dir.join(&target_backup_id);

    // 如果是恢复，目标目录可能已经存在（部分解压）
    if target_dir.exists() && !is_resuming {
        if let Err(e) = ensure_existing_path_within_backup_dir(&target_dir, &backup_dir) {
            job_ctx.fail(format!("备份路径校验失败: {}", e));
            return;
        }
        job_ctx.fail(format!("备份已存在: {}", target_backup_id));
        return;
    }

    // 阶段 1: 扫描
    job_ctx.mark_running(
        BackupJobPhase::Scan,
        0.0,
        Some(if is_resuming {
            "从检查点恢复，正在验证 ZIP 文件...".to_string()
        } else {
            "正在验证 ZIP 文件...".to_string()
        }),
        processed_items.len() as u64,
        0,
    );

    // 检查取消
    if job_ctx.is_cancelled() {
        job_ctx.cancelled(Some("用户取消导入".to_string()));
        return;
    }

    // 使用带进度的导入函数
    let job_ctx_for_progress = job_ctx.clone();
    let job_ctx_for_cancel = job_ctx.clone();

    // 断点续传：使用 import_backup_from_zip_resumable，
    // 自动跳过目标目录中已存在且大小匹配的文件
    let result = import_backup_from_zip_resumable(
        &zip_file_path,
        &target_dir,
        |progress| {
            let phase = match progress.phase {
                ZipImportPhase::Scan => BackupJobPhase::Scan,
                ZipImportPhase::Extract => BackupJobPhase::Extract,
                ZipImportPhase::Verify => BackupJobPhase::Verify,
                ZipImportPhase::Completed => BackupJobPhase::Completed,
            };

            job_ctx_for_progress.mark_running(
                phase,
                progress.progress,
                Some(
                    if is_resuming && progress.phase == ZipImportPhase::Extract {
                        format!("(断点续传) {}", progress.message)
                    } else {
                        progress.message
                    },
                ),
                progress.processed_files as u64,
                progress.total_files as u64,
            );

            // 更新检查点
            if let Some(ref file_name) = progress.current_file {
                job_ctx_for_progress.update_checkpoint(file_name);
            }
        },
        || job_ctx_for_cancel.is_cancelled(),
    );

    match result {
        Ok(file_count) => {
            let duration_ms = start.elapsed().as_millis() as u64;

            // 阶段 4: 清理（90% - 100%）
            job_ctx.mark_running(
                BackupJobPhase::Cleanup,
                95.0,
                Some("正在清理临时文件...".to_string()),
                file_count as u64,
                file_count as u64,
            );

            // 完成
            let result_payload = BackupJobResultPayload {
                success: true,
                output_path: Some(target_backup_id.clone()),
                resolved_path: Some(target_dir.to_string_lossy().to_string()),
                message: Some(format!(
                    "ZIP 导入完成: {} 个文件, 耗时 {}ms{}",
                    file_count,
                    duration_ms,
                    if is_resuming {
                        " (从检查点恢复)"
                    } else {
                        ""
                    }
                )),
                error: None,
                duration_ms: Some(duration_ms),
                stats: Some(serde_json::json!({
                    "backup_id": target_backup_id,
                    "file_count": file_count,
                    "zip_path": zip_file_path.to_string_lossy(),
                    "resumed_from_checkpoint": is_resuming,
                })),
                requires_restart: false,
                checkpoint_path: None,
                resumable_job_id: None,
            };

            #[cfg(feature = "data_governance")]
            {
                try_save_audit_log(
                    &app,
                    AuditLog::new(
                        AuditOperation::Backup {
                            backup_type: super::audit::BackupType::Full,
                            file_count,
                            total_size: 0,
                        },
                        format!("zip_import/{}", target_backup_id),
                    )
                    .complete(duration_ms)
                    .with_details(serde_json::json!({
                        "job_id": job_ctx.job_id.clone(),
                        "zip_path": zip_file_path.to_string_lossy(),
                        "backup_id": target_backup_id,
                        "backup_path": target_dir.to_string_lossy(),
                        "file_count": file_count,
                        "resumed_from_checkpoint": is_resuming,
                        "subtype": "zip_import_resumable",
                    })),
                );
            }

            job_ctx.complete(
                Some(format!("ZIP 导入完成: {}", target_backup_id)),
                file_count as u64,
                file_count as u64,
                result_payload,
            );
        }
        Err(e) => {
            let error_msg = e.to_string();
            if error_msg.contains("用户取消") || error_msg.contains("Interrupted") {
                job_ctx.cancelled(Some("用户取消导入".to_string()));
            } else {
                error!("[data_governance] ZIP 导入失败: {}", e);
                job_ctx.fail(format!("ZIP 导入失败: {}", e));
            }

            #[cfg(feature = "data_governance")]
            {
                try_save_audit_log(
                    &app,
                    AuditLog::new(
                        AuditOperation::Backup {
                            backup_type: super::audit::BackupType::Full,
                            file_count: 0,
                            total_size: 0,
                        },
                        format!("zip_import/{}", target_backup_id),
                    )
                    .fail(error_msg.clone())
                    .with_details(serde_json::json!({
                        "job_id": job_ctx.job_id.clone(),
                        "zip_path": zip_file_path.to_string_lossy(),
                        "backup_id": target_backup_id,
                        "backup_path": target_dir.to_string_lossy(),
                        "resumed_from_checkpoint": is_resuming,
                        "subtype": "zip_import_resumable",
                    })),
                );
            }
        }
    }
}
