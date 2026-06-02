//! VFS 路径缓存 Tauri 命令处理器（handlers 子模块）
//!
//! 提供资源路径缓存相关命令，用于引用模式上下文注入。
//!
//! ## 命令
//! - `vfs_get_resource_path`: 获取资源的当前路径
//! - `vfs_update_path_cache`: 批量更新路径缓存

use std::sync::Arc;

use rusqlite::OptionalExtension;
use tauri::State;

use crate::vfs::database::VfsDatabase;
use crate::vfs::error::{VfsError, VfsResult};
use crate::vfs::repos::VfsFolderRepo;

// ============================================================================
// 路径缓存命令
// ============================================================================

/// 获取资源的当前路径
///
/// 优先返回缓存路径（folder_items.cached_path），若未缓存则实时计算并更新缓存。
///
/// ## 参数
/// - `source_id`: 业务 ID（note_xxx, tb_xxx）
///
/// ## 返回
/// - `Ok(String)`: 资源的完整路径，如 "/高考复习/函数/笔记标题"
/// - `Err(String)`: 资源不存在或数据库错误
///
/// ## 约束
/// - 路径计算深度限制 10 层（契约 D）
/// - 路径最大长度 1000 字符
#[tauri::command]
pub async fn vfs_get_resource_path(
    source_id: String,
    vfs_db: State<'_, Arc<VfsDatabase>>,
) -> VfsResult<String> {
    log::info!(
        "[VFS::handlers] vfs_get_resource_path: source_id={}",
        source_id
    );

    // 获取数据库连接
    let conn = vfs_db.get_conn_safe()?;

    // 1. 先查 cached_path
    let cached_path: Option<String> = conn
        .query_row(
            r#"
            SELECT cached_path FROM folder_items
            WHERE item_id = ?1 AND cached_path IS NOT NULL
            LIMIT 1
            "#,
            rusqlite::params![&source_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| format!("Query cached_path failed: {}", e))?;

    if let Some(path) = cached_path {
        log::debug!(
            "[VFS::handlers] Returning cached path for {}: {}",
            source_id,
            path
        );
        return Ok(path);
    }

    // 2. 未缓存则实时计算
    // 先查找 folder_item
    let folder_item_opt: Option<(String, Option<String>, String)> = conn
        .query_row(
            r#"
            SELECT id, folder_id, item_type FROM folder_items
            WHERE item_id = ?1
            LIMIT 1
            "#,
            rusqlite::params![&source_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .optional()
        .map_err(|e| format!("Query folder_item failed: {}", e))?;

    let (fi_id, folder_id, _item_type) = match folder_item_opt {
        Some(fi) => fi,
        None => {
            // 资源不在 folder_items 中，返回资源名称作为路径
            let title = get_resource_title_with_conn(&conn, &source_id)?;
            return Ok(format!("/{}", title));
        }
    };

    // 计算路径
    let path = compute_path_with_conn(&conn, folder_id.as_deref(), &source_id)?;

    // 3. 更新缓存
    if path.len() <= 1000 {
        conn.execute(
            "UPDATE folder_items SET cached_path = ?1 WHERE id = ?2",
            rusqlite::params![&path, &fi_id],
        )
        .map_err(|e| format!("Update cached_path failed: {}", e))?;
        log::debug!(
            "[VFS::handlers] Updated cached_path for {}: {}",
            source_id,
            path
        );
    } else {
        log::warn!(
            "[VFS::handlers] Path too long ({}), not caching: {}",
            path.len(),
            source_id
        );
    }

    Ok(path)
}

/// 批量更新路径缓存（文件夹移动后调用）
///
/// 递归更新指定文件夹及其所有子文件夹下资源的 cached_path。
///
/// ## 参数
/// - `folder_id`: 被移动的文件夹 ID
///
/// ## 返回
/// - `Ok(u32)`: 更新的项数
/// - `Err(String)`: 数据库错误
///
/// ## 约束
/// - 使用事务保证一致性
/// - 路径计算深度限制 10 层
#[tauri::command]
pub async fn vfs_update_path_cache(
    folder_id: String,
    vfs_db: State<'_, Arc<VfsDatabase>>,
) -> VfsResult<u32> {
    log::info!(
        "[VFS::handlers] vfs_update_path_cache: folder_id={}",
        folder_id
    );

    // 验证文件夹 ID 格式
    if !folder_id.starts_with("fld_") {
        return Err(VfsError::InvalidArgument {
            param: "folder_id".to_string(),
            reason: format!("Invalid folder ID format: {}", folder_id),
        });
    }

    // 获取数据库连接
    let conn = vfs_db.get_conn_safe()?;

    // 使用事务
    conn.execute("BEGIN TRANSACTION", [])?;

    let result = update_path_cache_internal(&conn, &folder_id);

    match result {
        Ok(count) => {
            conn.execute("COMMIT", [])?;
            log::info!(
                "[VFS::handlers] Updated {} path caches for folder {}",
                count,
                folder_id
            );
            Ok(count)
        }
        Err(e) => {
            conn.execute("ROLLBACK", []).ok();
            log::error!("[VFS::handlers] Failed to update path cache: {}", e);
            Err(e)
        }
    }
}

/// 内部函数：批量更新路径缓存
fn update_path_cache_internal(conn: &rusqlite::Connection, folder_id: &str) -> VfsResult<u32> {
    // 1. 获取文件夹及其所有子文件夹的 ID
    let folder_ids = VfsFolderRepo::get_folder_ids_recursive_with_conn(conn, folder_id)?;

    if folder_ids.is_empty() {
        return Ok(0);
    }

    // 2. 获取这些文件夹下的所有 folder_items
    let items = VfsFolderRepo::get_items_by_folders_with_conn(conn, &folder_ids)?;

    let mut updated = 0u32;

    // 3. 逐个计算并更新路径
    for item in &items {
        let path = compute_path_with_conn(conn, item.folder_id.as_deref(), &item.item_id)?;

        // 路径长度检查
        if path.len() > 1000 {
            log::warn!(
                "[VFS::handlers] Path too long for item {}, skipping cache update",
                item.item_id
            );
            continue;
        }

        conn.execute(
            "UPDATE folder_items SET cached_path = ?1 WHERE id = ?2",
            rusqlite::params![&path, &item.id],
        )
        .map_err(|e| format!("Update cached_path failed: {}", e))?;

        updated += 1;
    }

    Ok(updated)
}

/// 计算资源的完整路径
fn compute_path_with_conn(
    conn: &rusqlite::Connection,
    folder_id: Option<&str>,
    source_id: &str,
) -> VfsResult<String> {
    // 获取资源标题
    let title = get_resource_title_with_conn(conn, source_id)?;

    // 如果没有文件夹，直接返回标题
    let Some(fid) = folder_id else {
        return Ok(format!("/{}", title));
    };

    // 获取文件夹路径（使用 CTE 递归查询）
    let folder_path =
        VfsFolderRepo::build_folder_path_with_conn(conn, fid)?;

    Ok(format!("/{}/{}", folder_path, title))
}

/// 获取资源标题
fn get_resource_title_with_conn(
    conn: &rusqlite::Connection,
    source_id: &str,
) -> VfsResult<String> {
    // 根据 source_id 前缀判断类型并查询标题
    let prefix = source_id.split('_').next().unwrap_or("");

    let title: Option<String> = match prefix {
        "note" => conn
            .query_row(
                "SELECT title FROM notes WHERE id = ?1",
                rusqlite::params![source_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| format!("Query note title failed: {}", e))?,
        "tb" => conn
            .query_row(
                "SELECT file_name FROM files WHERE id = ?1",
                rusqlite::params![source_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| format!("Query textbook title failed: {}", e))?,
        "exam" => conn
            .query_row(
                "SELECT COALESCE(exam_name, id) FROM exam_sheets WHERE id = ?1",
                rusqlite::params![source_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| format!("Query exam title failed: {}", e))?,
        "tr" => conn
            .query_row(
                "SELECT id FROM translations WHERE id = ?1",
                rusqlite::params![source_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| format!("Query translation title failed: {}", e))?,
        "essay" => conn
            .query_row(
                "SELECT COALESCE(title, id) FROM essays WHERE id = ?1",
                rusqlite::params![source_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| format!("Query essay title failed: {}", e))?,
        _ => None,
    };

    Ok(title.unwrap_or_else(|| source_id.to_string()))
}
