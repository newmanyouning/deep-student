# API 重构: Essay Grading — 作文批改

**日期**: 2026-05-29 | **命令数**: 20 | **对应诊断**: round-20~26

---

## 当前问题

20 个命令，返回类型直接用具体类型而非 Result 包裹。

## 当前参数模式

| 参数类型 | 出现次数 |
|---------|--------|
| `State<commands::AppState>` | 19 |
| `String` | 13 |
| `Option<String>` | 2 |
| `Option<u32>` | 2 |
| `custom_modes::CreateModeInput` | 1 |
| `i32` | 1 |

## 当前返回类型

| 返回类型 | 出现次数 |
|---------|--------|
| `types::GradingMode` | 4 |
| `()` | 2 |
| `Vec<types::GradingMode>` | 2 |
| `bool` | 2 |
| `VfsEssaySession` | 1 |
| `usize` | 1 |

## 命令清单与变更

| 当前命令 | 改为 | 参数变更 | 返回变更 |
|---------|------|---------|--------|
| `essay_grading_create_custom_mode` | *(保持)* | — | — |
| `essay_grading_create_session` | *(保持)* | → Input struct | — |
| `essay_grading_delete_custom_mode` | *(保持)* | — | — |
| `essay_grading_delete_session` | *(保持)* | — | — |
| `essay_grading_get_latest_round_number` | *(保持)* | — | — |
| `essay_grading_get_mode` | *(保持)* | — | — |
| `essay_grading_get_models` | *(保持)* | — | — |
| `essay_grading_get_modes` | *(保持)* | — | — |
| `essay_grading_get_round` | *(保持)* | — | — |
| `essay_grading_get_rounds` | *(保持)* | — | — |
| `essay_grading_get_session` | *(保持)* | — | — |
| `essay_grading_has_builtin_override` | *(保持)* | — | — |
| `essay_grading_list_custom_modes` | *(保持)* | — | — |
| `essay_grading_list_sessions` | *(保持)* | → Input struct | — |
| `essay_grading_reset_builtin_mode` | *(保持)* | — | — |
| `essay_grading_save_builtin_override` | *(保持)* | — | — |
| `essay_grading_stream` | *(保持)* | — | — |
| `essay_grading_toggle_favorite` | *(保持)* | — | — |
| `essay_grading_update_custom_mode` | *(保持)* | — | — |
| `essay_grading_update_session` | *(保持)* | — | — |

## 改进操作

全部包装为 Result<T, EssayGradingError>

## 统一错误类型

`EssayGradingError` — 替换当前使用的 `String` / `AppError`

---
*此报告由 deps.db 数据自动生成，对应模块原始数据见 `_data/essay_grading.json`*
