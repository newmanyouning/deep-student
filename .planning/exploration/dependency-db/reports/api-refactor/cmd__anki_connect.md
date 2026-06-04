# API 重构: cmd::anki_connect — Anki Connect

**日期**: 2026-05-29 | **命令数**: 13 | **对应诊断**: round-20~26

---

## 当前问题

13 个命令

## 当前参数模式

| 参数类型 | 出现次数 |
|---------|--------|
| `String` | 12 |
| `State<AppState>` | 6 |
| `Vec<models::AnkiCard>` | 4 |
| `Option<String>` | 2 |
| `Vec<BatchExportNote>` | 1 |
| `BatchExportOptions` | 1 |

## 当前返回类型

| 返回类型 | 出现次数 |
|---------|--------|
| `Result<String>` | 5 |
| `Result<Vec<String>>` | 3 |
| `Result<bool>` | 2 |
| `Result<Vec<Option<u64>>>` | 1 |
| `Result<()>` | 1 |
| `Result<SaveAnkiCardsResponse>` | 1 |

## 命令清单与变更

| 当前命令 | 改为 | 参数变更 | 返回变更 |
|---------|------|---------|--------|
| `add_cards_to_anki_connect` | *(保持)* | → Input struct | — |
| `anki_get_deck_names` | *(保持)* | — | — |
| `batch_export_cards` | *(保持)* | → Input struct | — |
| `check_anki_connect_status` | *(保持)* | — | — |
| `create_anki_deck` | *(保持)* | — | — |
| `export_cards_as_apkg` | *(保持)* | → Input struct | — |
| `export_cards_as_apkg_with_template` | *(保持)* | → Input struct | — |
| `export_multi_template_apkg` | *(保持)* | → Input struct | — |
| `get_anki_deck_names` | *(保持)* | — | — |
| `get_anki_model_names` | *(保持)* | — | — |
| `import_anki_package` | *(保持)* | — | — |
| `save_anki_cards` | *(保持)* | — | — |
| `save_json_file` | *(保持)* | — | — |

## 改进操作

统一错误类型

## 统一错误类型

`AnkiConnectError` — 替换当前使用的 `String` / `AppError`

---
*此报告由 deps.db 数据自动生成，对应模块原始数据见 `_data/cmd__anki_connect.json`*
