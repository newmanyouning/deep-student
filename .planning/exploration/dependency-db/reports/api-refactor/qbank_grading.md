# API 重构: QBank Grading — 题库评分

**日期**: 2026-05-29 | **命令数**: 2 | **对应诊断**: round-20~26

---

## 当前问题

2 个命令

## 当前参数模式

| 参数类型 | 出现次数 |
|---------|--------|
| `State<commands::AppState>` | 2 |
| `QbankGradingRequest` | 1 |
| `Window` | 1 |
| `String` | 1 |

## 当前返回类型

| 返回类型 | 出现次数 |
|---------|--------|
| `Option<QbankGradingResponse>` | 1 |
| `()` | 1 |

## 命令清单与变更

| 当前命令 | 改为 | 参数变更 | 返回变更 |
|---------|------|---------|--------|
| `qbank_ai_grade` | *(保持)* | — | — |
| `qbank_cancel_grading` | *(保持)* | — | — |

## 改进操作

统一错误类型

## 统一错误类型

`QbankGradingError` — 替换当前使用的 `String` / `AppError`

---
*此报告由 deps.db 数据自动生成，对应模块原始数据见 `_data/qbank_grading.json`*
