//! 搜索处理器
//!
//! 包含全局搜索、文件夹内搜索和相关辅助函数

use std::sync::Arc;

use tauri::State;

use crate::vfs::{
    repos::VfsMindMapRepo, VfsDatabase, VfsExamRepo, VfsFileRepo, VfsNoteRepo,
    VfsTextbookRepo, VfsTranslationRepo,
};

use super::super::error::{DstuError, DstuResult};
use super::super::handler_utils::{
    note_to_dstu_node, textbook_to_dstu_node, translation_to_dstu_node, exam_to_dstu_node,
    file_to_dstu_node, mindmap_to_dstu_node,
    search_by_index, search_all,
};
use super::super::types::{DstuListOptions, DstuNode};

use super::is_memory_system_hidden_name;

/// 全局搜索资源
///
/// 全文搜索所有类型的资源。
#[tauri::command]
pub async fn dstu_search(
    query: String,
    options: Option<DstuListOptions>,
    vfs_db: State<'_, Arc<VfsDatabase>>,
) -> DstuResult<Vec<DstuNode>> {
    log::info!("[DSTU::handlers] dstu_search: query={}", query);

    let options = options.unwrap_or_default();
    let mut results = search_all(&vfs_db, &query, &options)?;
    // ★ 记忆系统改造：全局搜索结果也需隐藏 __*__ 系统保留笔记
    results.retain(|node| !is_memory_system_hidden_name(&node.name));
    log::info!(
        "[DSTU::handlers] dstu_search: found {} results",
        results.len()
    );
    Ok(results)
}

/// 在指定文件夹内搜索资源
#[tauri::command]
pub async fn dstu_search_in_folder(
    folder_id: Option<String>,
    query: String,
    options: Option<DstuListOptions>,
    vfs_db: State<'_, Arc<VfsDatabase>>,
) -> DstuResult<Vec<DstuNode>> {
    log::info!(
        "[DSTU::handlers] dstu_search_in_folder: folder={:?}, query={}",
        folder_id,
        query
    );

    let options = options.unwrap_or_default();

    // 如果有 folder_id，先获取文件夹内的所有项
    if let Some(ref fid) = folder_id {
        let _folder = match crate::vfs::VfsFolderRepo::get_folder(&vfs_db, fid) {
            Ok(Some(f)) => f,
            Ok(None) => {
                log::error!(
                    "[DSTU::handlers] dstu_get_nodes_in_folder: FAILED - folder not found, id={}",
                    fid
                );
                return Err(DstuError::from(format!("Folder not found: {}", fid)));
            }
            Err(e) => {
                log::error!(
                    "[DSTU::handlers] dstu_get_nodes_in_folder: FAILED - get_folder error, id={}, error={}",
                    fid,
                    e
                );
                return Err(DstuError::from(e.to_string()));
            }
        };
        let items = match crate::vfs::VfsFolderRepo::list_items_by_folder(&vfs_db, Some(fid)) {
            Ok(i) => i,
            Err(e) => {
                log::error!(
                    "[DSTU::handlers] dstu_get_nodes_in_folder: FAILED - list_items_by_folder error, folder_id={}, error={}",
                    fid,
                    e
                );
                return Err(DstuError::from(e.to_string()));
            }
        };

        let folder_item_ids: std::collections::HashSet<String> =
            items.iter().map(|item| item.item_id.clone()).collect();

        let query_lower = query.to_lowercase();
        let mut results = Vec::new();
        for item in items {
            let node = match item.item_type.as_str() {
                "note" => {
                    if let Ok(Some(note)) = VfsNoteRepo::get_note(&vfs_db, &item.item_id) {
                        if note.title.to_lowercase().contains(&query_lower) {
                            Some(note_to_dstu_node(&note))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
                "textbook" => {
                    if let Ok(Some(tb)) = VfsTextbookRepo::get_textbook(&vfs_db, &item.item_id) {
                        if tb.file_name.to_lowercase().contains(&query_lower) {
                            Some(textbook_to_dstu_node(&tb))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
                "file" => {
                    if let Ok(Some(f)) = VfsFileRepo::get_file(&vfs_db, &item.item_id) {
                        if f.file_name.to_lowercase().contains(&query_lower) {
                            Some(file_to_dstu_node(&f))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
                "translation" => {
                    if let Ok(Some(t)) = VfsTranslationRepo::get_translation(&vfs_db, &item.item_id)
                    {
                        if t.title.as_deref().unwrap_or("").to_lowercase().contains(&query_lower) {
                            Some(translation_to_dstu_node(&t))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
                "exam" => {
                    if let Ok(Some(e)) = VfsExamRepo::get_exam_sheet(&vfs_db, &item.item_id) {
                        if e.exam_name.as_deref().unwrap_or("").to_lowercase().contains(&query_lower) {
                            Some(exam_to_dstu_node(&e))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
                "mindmap" => {
                    if let Ok(Some(m)) = VfsMindMapRepo::get_mindmap(&vfs_db, &item.item_id) {
                        if m.title.to_lowercase().contains(&query_lower) {
                            Some(mindmap_to_dstu_node(&m))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
                _ => None,
            };

            if let Some(n) = node {
                if is_memory_system_hidden_name(&n.name) {
                    continue;
                }
                results.push(n);
            }
        }

        let existing_ids: std::collections::HashSet<String> =
            results.iter().map(|n| n.id.clone()).collect();
        let index_limit = options.limit.unwrap_or(50);
        if let Ok(index_results) = search_by_index(&vfs_db, &query, index_limit, &existing_ids) {
            for node in index_results {
                if folder_item_ids.contains(&node.id) {
                    if is_memory_system_hidden_name(&node.name) {
                        continue;
                    }
                    results.push(node);
                }
            }
        }

        results.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

        if let Some(limit) = options.limit {
            results.truncate(limit as usize);
        }

        return Ok(results);
    }

    // 没有指定文件夹，使用全局搜索
    dstu_search(query, Some(options), vfs_db).await
}
