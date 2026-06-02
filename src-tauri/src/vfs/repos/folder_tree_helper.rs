//! VFS 文件夹树操作与路径缓存失效辅助函数
//!
//! 本模块提取 `folder_repo` 和 `path_cache_repo` 之间的交叉引用函数，
//! 作为自由函数，打破两个模块之间的循环依赖。
//!
//! ## 打破的循环
//! 原循环: `folder_repo → VfsPathCacheRepo` ⇄ `path_cache_repo → VfsFolderRepo`
//! 现结构: `folder_repo → folder_tree_helper → path_cache_repo` (单向)
//!         `path_cache_repo → folder_tree_helper` (单向)
//!
//! ## 包含的函数
//! - `get_folder_ids_recursive_with_conn`: 递归获取文件夹 ID（来自 VfsFolderRepo）
//! - `get_folder_with_conn`: 获取文件夹（来自 VfsFolderRepo）
//! - `invalidate_path_cache_by_folders_batch`: 批量失效路径缓存（来自 VfsPathCacheRepo）
//! - `ensure_path_cache_table_exists`: 确保路径缓存表存在

use std::collections::HashMap;

use rusqlite::{params, Connection, OptionalExtension};
use tracing::debug;

use crate::vfs::error::VfsResult;
use crate::vfs::types::VfsFolder;

/// 最大文件夹深度限制
pub const MAX_FOLDER_DEPTH: usize = 10;

/// 批量操作最大批次大小
const BATCH_SIZE: usize = 100;

// ============================================================================
// 路径缓存表管理
// ============================================================================

/// 确保 path_cache 表存在
///
/// 此方法在使用缓存前调用，确保表已创建（兼容迁移未执行的情况）
pub fn ensure_path_cache_table_exists(conn: &Connection) -> VfsResult<()> {
    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS path_cache (
            item_type TEXT NOT NULL,
            item_id TEXT NOT NULL,
            full_path TEXT NOT NULL,
            folder_path TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            PRIMARY KEY (item_type, item_id)
        )
        "#,
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_path_cache_path ON path_cache(full_path)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_path_cache_folder ON path_cache(folder_path)",
        [],
    )?;

    Ok(())
}

// ============================================================================
// 文件夹树查询
// ============================================================================

/// 递归获取文件夹及其所有子文件夹的 ID
///
/// 使用 CTE 递归查询，限制最大深度为 10 层。
/// 返回的列表包含起始文件夹自身。排除软删除的文件夹。
pub fn get_folder_ids_recursive_with_conn(
    conn: &Connection,
    folder_id: &str,
) -> VfsResult<Vec<String>> {
    let mut stmt = conn.prepare(
        r#"
        WITH RECURSIVE folder_tree AS (
            SELECT id, parent_id, title, 0 as depth
            FROM folders WHERE id = ?1 AND deleted_at IS NULL
            UNION ALL
            SELECT f.id, f.parent_id, f.title, ft.depth + 1
            FROM folders f JOIN folder_tree ft ON f.parent_id = ft.id
            WHERE ft.depth < ?2 AND f.deleted_at IS NULL
        )
        SELECT id FROM folder_tree
        "#,
    )?;

    let ids = stmt
        .query_map(params![folder_id, MAX_FOLDER_DEPTH], |row| row.get(0))?
        .collect::<Result<Vec<String>, _>>()?;

    debug!(
        "[VFS::FolderTreeHelper] get_folder_ids_recursive: {} -> {} folders",
        folder_id,
        ids.len()
    );

    Ok(ids)
}

/// 获取文件夹（使用现有连接）
///
/// ★ P0 修复：使用 CASE typeof() 兼容处理 updated_at/created_at 可能存储为 TEXT 的历史数据
/// 此函数可能读取已软删除的文件夹（如 restore 路径），需要兼容旧版本写入的 TEXT 类型
pub fn get_folder_with_conn(
    conn: &Connection,
    folder_id: &str,
) -> VfsResult<Option<VfsFolder>> {
    let folder = conn
        .query_row(
            r#"
            SELECT id, parent_id, title, icon, color, is_expanded, is_favorite, sort_order,
                   CASE typeof(created_at) WHEN 'text' THEN CAST(strftime('%s', created_at) AS INTEGER) * 1000 ELSE created_at END,
                   CASE typeof(updated_at) WHEN 'text' THEN CAST(strftime('%s', updated_at) AS INTEGER) * 1000 ELSE updated_at END
            FROM folders
            WHERE id = ?1
            "#,
            params![folder_id],
            |row| {
                Ok(VfsFolder {
                    id: row.get(0)?,
                    parent_id: row.get(1)?,
                    title: row.get(2)?,
                    icon: row.get(3)?,
                    color: row.get(4)?,
                    is_expanded: row.get::<_, i32>(5)? != 0,
                    is_favorite: row.get::<_, i32>(6)? != 0,
                    sort_order: row.get(7)?,
                    created_at: row.get(8)?,
                    updated_at: row.get(9)?,
                })
            },
        )
        .optional()?;

    Ok(folder)
}

// ============================================================================
// 路径缓存失效（用于 folder_repo 避免导入 path_cache_repo）
// ============================================================================

/// 批量使多个文件夹下所有路径缓存失效
///
/// 分批删除 path_cache 表中属于指定文件夹及其资源的缓存条目。
/// 包含两步操作：
/// 1. 删除文件夹自身的缓存（item_type = 'folder'）
/// 2. 查询文件夹下的所有资源并删除其缓存
///
/// ★ HIGH-R001修复：分批处理，避免SQL过长
pub fn invalidate_path_cache_by_folders_batch(
    conn: &Connection,
    folder_ids: &[String],
) -> VfsResult<usize> {
    if folder_ids.is_empty() {
        return Ok(0);
    }

    ensure_path_cache_table_exists(conn)?;

    let mut total_deleted = 0usize;

    // 1. 分批删除文件夹自身的缓存
    for chunk in folder_ids.chunks(BATCH_SIZE) {
        let placeholders: Vec<String> = (1..=chunk.len()).map(|i| format!("?{}", i)).collect();
        let in_clause = placeholders.join(", ");

        let sql = format!(
            "DELETE FROM path_cache WHERE item_type = 'folder' AND item_id IN ({})",
            in_clause
        );
        let params: Vec<&dyn rusqlite::ToSql> =
            chunk.iter().map(|id| id as &dyn rusqlite::ToSql).collect();
        let deleted = conn.execute(&sql, params.as_slice())?;
        total_deleted += deleted;
    }

    // 2. 分批获取并删除资源的缓存
    for chunk in folder_ids.chunks(BATCH_SIZE) {
        let placeholders: Vec<String> = (1..=chunk.len()).map(|i| format!("?{}", i)).collect();
        let in_clause = placeholders.join(", ");

        // 获取该批次文件夹下的所有资源
        let sql = format!(
            "SELECT item_type, item_id FROM folder_items WHERE folder_id IN ({})",
            in_clause
        );
        let params: Vec<&dyn rusqlite::ToSql> =
            chunk.iter().map(|id| id as &dyn rusqlite::ToSql).collect();

        let mut stmt = conn.prepare(&sql)?;
        let items: Vec<(String, String)> = stmt
            .query_map(params.as_slice(), |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        // 按 item_type 分组以优化删除
        let mut items_by_type: HashMap<String, Vec<String>> = HashMap::new();
        for (item_type, item_id) in items {
            items_by_type.entry(item_type).or_default().push(item_id);
        }

        // 分批删除每种类型的资源缓存
        for (item_type, item_ids) in items_by_type {
            for item_chunk in item_ids.chunks(BATCH_SIZE) {
                let placeholders: Vec<String> = (1..=item_chunk.len())
                    .map(|i| format!("?{}", i + 1)) // 从 ?2 开始，?1 是 item_type
                    .collect();
                let in_clause = placeholders.join(", ");

                let sql = format!(
                    "DELETE FROM path_cache WHERE item_type = ?1 AND item_id IN ({})",
                    in_clause
                );

                let mut params: Vec<&dyn rusqlite::ToSql> =
                    vec![&item_type as &dyn rusqlite::ToSql];
                for id in item_chunk {
                    params.push(id as &dyn rusqlite::ToSql);
                }

                let deleted = conn.execute(&sql, params.as_slice())?;
                total_deleted += deleted;
            }
        }
    }

    debug!(
        "[VFS::FolderTreeHelper] Batch invalidated {} cache entries for {} folders",
        total_deleted,
        folder_ids.len()
    );

    Ok(total_deleted)
}
