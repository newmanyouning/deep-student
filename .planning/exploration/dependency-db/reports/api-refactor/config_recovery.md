# API 重构: Config Recovery — 配置恢复

**日期**: 2026-05-29 | **命令数**: 2 | **对应诊断**: round-20~26

---

## 当前问题

2 个命令

## 当前参数模式

| 参数类型 | 出现次数 |
|---------|--------|
| `State<std::sync::Arc<LLMManager>>` | 2 |

## 当前返回类型

| 返回类型 | 出现次数 |
|---------|--------|
| `serde_json::Value` | 1 |
| `String` | 1 |

## 命令清单与变更

| 当前命令 | 改为 | 参数变更 | 返回变更 |
|---------|------|---------|--------|
| `check_api_config_status` | *(保持)* | — | Result<T, AppError> |
| `restore_default_api_configs` | *(保持)* | — | Result<T, AppError> |

## 改进操作

保持现状

## 统一错误类型

`AppError` — 替换当前使用的 `String` / `AppError`

---
*此报告由 deps.db 数据自动生成，对应模块原始数据见 `_data/config_recovery.json`*
