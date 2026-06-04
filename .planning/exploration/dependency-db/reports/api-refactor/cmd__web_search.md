# API 重构: cmd::web_search — 搜索与设置

**日期**: 2026-05-29 | **命令数**: 17 | **对应诊断**: round-20~26

---

## 当前问题

17 个命令，混合了搜索引擎命令和通用设置命令（get_setting, delete_setting）。

## 当前参数模式

| 参数类型 | 出现次数 |
|---------|--------|
| `State<AppState>` | 17 |
| `String` | 10 |
| `Option<String>` | 2 |
| `tools::web_search::ProviderStrategies` | 1 |
| `Option<serde_json::Value>` | 1 |

## 当前返回类型

| 返回类型 | 出现次数 |
|---------|--------|
| `Result<serde_json::Value>` | 9 |
| `Result<bool>` | 4 |
| `Result<usize>` | 1 |
| `Result<Vec<ToolConflict>>` | 1 |
| `Result<Option<String>>` | 1 |
| `Vec<(String` | 1 |

## 命令清单与变更

| 当前命令 | 改为 | 参数变更 | 返回变更 |
|---------|------|---------|--------|
| `delete_setting` | *(保持)* | — | — |
| `delete_settings_by_prefix` | *(保持)* | — | — |
| `detect_tool_conflicts` | *(保持)* | — | — |
| `get_cn_whitelist_config` | *(保持)* | — | — |
| `get_feature_flags` | *(保持)* | — | — |
| `get_provider_strategies_config` | *(保持)* | — | — |
| `get_security_status` | *(保持)* | — | — |
| `get_setting` | *(保持)* | — | — |
| `get_settings_by_prefix` | *(保持)* | — | — |
| `get_tools_namespace_config` | *(保持)* | — | — |
| `is_feature_enabled` | *(保持)* | — | — |
| `save_provider_strategies_config` | *(保持)* | — | — |
| `save_setting` | *(保持)* | — | — |
| `test_all_search_engines` | *(保持)* | — | — |
| `test_search_engine` | *(保持)* | — | — |
| `test_web_search_connectivity` | *(保持)* | — | — |
| `update_feature_flag` | *(保持)* | → Input struct | — |

## 改进操作

拆分：搜索命令保留，通用设置命令移到独立 settings 模块

## 统一错误类型

`SearchError` — 替换当前使用的 `String` / `AppError`

---
*此报告由 deps.db 数据自动生成，对应模块原始数据见 `_data/cmd__web_search.json`*
