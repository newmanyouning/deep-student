# API 重构: cmd::textbooks — 教材

**日期**: 2026-05-29 | **命令数**: 11 | **对应诊断**: round-20~26

---

## 当前问题

11 个命令

## 当前参数模式

| 参数类型 | 出现次数 |
|---------|--------|
| `State<AppState>` | 11 |
| `String` | 7 |
| `Window` | 5 |
| `State<PdfProcessingService>` | 2 |
| `Vec<String>` | 2 |
| `Option<String>` | 2 |

## 当前返回类型

| 返回类型 | 出现次数 |
|---------|--------|
| `Result<bool>` | 7 |
| `Result<Vec<TextbookDto>>` | 3 |
| `Result<serde_json::Value>` | 1 |

## 命令清单与变更

| 当前命令 | 改为 | 参数变更 | 返回变更 |
|---------|------|---------|--------|
| `textbooks_add` | *(保持)* | → Input struct | — |
| `textbooks_adopt` | *(保持)* | → Input struct | — |
| `textbooks_delete_permanent` | *(保持)* | → Input struct | — |
| `textbooks_list` | *(保持)* | — | — |
| `textbooks_purge_trash` | *(保持)* | — | — |
| `textbooks_recover` | *(保持)* | — | — |
| `textbooks_remove` | *(保持)* | — | — |
| `textbooks_set_favorite` | *(保持)* | — | — |
| `textbooks_update_bookmarks` | *(保持)* | — | — |
| `textbooks_update_page_count` | *(保持)* | — | — |
| `textbooks_update_reading_progress` | *(保持)* | — | — |

## 改进操作

统一错误类型

## 统一错误类型

`TextbookError` — 替换当前使用的 `String` / `AppError`

---
*此报告由 deps.db 数据自动生成，对应模块原始数据见 `_data/cmd__textbooks.json`*
