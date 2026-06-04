# API 重构: Translation — 翻译

**日期**: 2026-05-29 | **命令数**: 3 | **对应诊断**: round-20~26

---

## 当前问题

3 个命令

## 当前参数模式

| 参数类型 | 出现次数 |
|---------|--------|
| `Window` | 3 |
| `ChatTranslationRequest` | 2 |
| `State<AppState>` | 2 |
| `TranslationRequest` | 1 |
| `State<commands::AppState>` | 1 |

## 当前返回类型

| 返回类型 | 出现次数 |
|---------|--------|
| `()` | 2 |
| `Option<TranslationResponse>` | 1 |

## 命令清单与变更

| 当前命令 | 改为 | 参数变更 | 返回变更 |
|---------|------|---------|--------|
| `stream_chat_translation_aligned` | *(保持)* | — | — |
| `stream_chat_translation_plain` | *(保持)* | — | — |
| `translate_text_stream` | *(保持)* | — | — |

## 改进操作

统一错误类型

## 统一错误类型

`TranslationError` — 替换当前使用的 `String` / `AppError`

---
*此报告由 deps.db 数据自动生成，对应模块原始数据见 `_data/translation.json`*
