# API 重构: cmd::ocr — OCR 引擎

**日期**: 2026-05-29 | **命令数**: 14 | **对应诊断**: round-20~26

---

## 当前问题

14 个命令，命名一致

## 当前参数模式

| 参数类型 | 出现次数 |
|---------|--------|
| `String` | 10 |
| `State<AppState>` | 10 |
| `Option<String>` | 1 |
| `Vec<SaveOcrModelRequest>` | 1 |
| `bool` | 1 |
| `OcrTestRequest` | 1 |

## 当前返回类型

| 返回类型 | 出现次数 |
|---------|--------|
| `Result<bool>` | 7 |
| `Result<String>` | 3 |
| `Result<Vec<AvailableOcrModelResponse>>` | 1 |
| `Result<Vec<OcrEngineInfoResponse>>` | 1 |
| `Result<OcrTestResponse>` | 1 |
| `Result<ValidateOcrModelResponse>` | 1 |

## 命令清单与变更

| 当前命令 | 改为 | 参数变更 | 返回变更 |
|---------|------|---------|--------|
| `add_ocr_engine` | *(保持)* | → Input struct | — |
| `get_available_ocr_models` | *(保持)* | — | — |
| `get_ocr_engine_type` | *(保持)* | — | — |
| `get_ocr_engines` | *(保持)* | — | — |
| `get_ocr_prompt_template` | *(保持)* | — | — |
| `get_ocr_thinking_enabled` | *(保持)* | — | — |
| `infer_ocr_engine_from_model` | *(保持)* | — | — |
| `remove_ocr_engine` | *(保持)* | — | — |
| `save_available_ocr_models` | *(保持)* | — | — |
| `set_ocr_engine_type` | *(保持)* | — | — |
| `set_ocr_thinking_enabled` | *(保持)* | — | — |
| `test_ocr_engine` | *(保持)* | — | — |
| `update_ocr_engine_priority` | *(保持)* | — | — |
| `validate_ocr_model` | *(保持)* | — | — |

## 改进操作

统一错误类型

## 统一错误类型

`OcrError` — 替换当前使用的 `String` / `AppError`

---
*此报告由 deps.db 数据自动生成，对应模块原始数据见 `_data/cmd__ocr.json`*
