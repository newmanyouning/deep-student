# API 重构: cmd::enhanced_anki — Anki 制卡

**日期**: 2026-05-29 | **命令数**: 22 | **对应诊断**: round-20~26

---

## 当前问题

22 个命令，命名不一致（delete_anki_card vs generate_anki_cards_from_document）。

## 当前参数模式

| 参数类型 | 出现次数 |
|---------|--------|
| `State<AppState>` | 22 |
| `String` | 16 |
| `Window` | 4 |
| `Option<Vec<String>>` | 2 |
| `AnkiGenerationOptions` | 2 |
| `models::ExportAnkiCardsRequest` | 1 |

## 当前返回类型

| 返回类型 | 出现次数 |
|---------|--------|
| `Result<bool>` | 5 |
| `Result<u32>` | 3 |
| `Result<String>` | 2 |
| `Result<Vec<crate::models::AnkiCard>>` | 2 |
| `Result<()>` | 2 |
| `Result<crate::models::ExportAnkiCardsResponse>` | 1 |

## 命令清单与变更

| 当前命令 | 改为 | 参数变更 | 返回变更 |
|---------|------|---------|--------|
| `delete_anki_card` | *(保持)* | — | — |
| `delete_document_session` | *(保持)* | — | — |
| `delete_document_task` | *(保持)* | — | — |
| `dismiss_pending_memory_candidates` | *(保持)* | — | — |
| `export_anki_cards` | *(保持)* | — | — |
| `export_apkg_for_selection` | *(保持)* | → Input struct | — |
| `get_anki_stats` | *(保持)* | — | — |
| `get_document_cards` | *(保持)* | — | — |
| `get_document_processing_state` | *(保持)* | — | — |
| `get_document_task_counts` | *(保持)* | — | — |
| `get_document_tasks` | *(保持)* | — | — |
| `get_pending_memory_candidates` | *(保持)* | — | — |
| `get_task_cards` | *(保持)* | — | — |
| `list_anki_library_cards` | *(保持)* | — | — |
| `list_document_sessions` | *(保持)* | — | — |
| `mark_pending_memory_candidates_saved` | *(保持)* | — | — |
| `pause_document_processing` | *(保持)* | — | — |
| `recover_stuck_document_tasks` | *(保持)* | — | — |
| `resume_document_processing` | *(保持)* | — | — |
| `start_enhanced_document_processing` | *(保持)* | → Input struct | — |
| `trigger_task_processing` | *(保持)* | — | — |
| `update_anki_card` | *(保持)* | — | — |

## 改进操作

统一为 anki_ 前缀，统一错误类型

## 统一错误类型

`AnkiError` — 替换当前使用的 `String` / `AppError`

---
*此报告由 deps.db 数据自动生成，对应模块原始数据见 `_data/cmd__enhanced_anki.json`*
