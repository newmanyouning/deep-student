# API 重构: cmd::anki_cards — Anki 卡片

**日期**: 2026-05-29 | **命令数**: 3 | **对应诊断**: round-20~26

---

## 当前问题

3 个命令

## 当前参数模式

| 参数类型 | 出现次数 |
|---------|--------|
| `State<AppState>` | 3 |
| `Vec<serde_json::Value>` | 1 |

## 当前返回类型

| 返回类型 | 出现次数 |
|---------|--------|
| `Result<Vec<serde_json::Value>>` | 2 |
| `Result<()>` | 1 |

## 命令清单与变更

| 当前命令 | 改为 | 参数变更 | 返回变更 |
|---------|------|---------|--------|
| `get_model_adapter_options` | *(保持)* | — | — |
| `reset_model_adapter_options` | *(保持)* | — | — |
| `save_model_adapter_options` | *(保持)* | — | — |

## 改进操作

合并到 enhanced_anki

## 统一错误类型

`AppError` — 替换当前使用的 `String` / `AppError`

---
*此报告由 deps.db 数据自动生成，对应模块原始数据见 `_data/cmd__anki_cards.json`*
