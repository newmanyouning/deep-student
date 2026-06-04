# API 重构: Review Plan — 复习计划

**日期**: 2026-05-29 | **命令数**: 17 | **对应诊断**: round-20~26

---

## 当前问题

17 个命令，命名一致（review_plan_ 前缀），但返回类型用 String。每个命令都需要 VfsDatabase State。

## 当前参数模式

| 参数类型 | 出现次数 |
|---------|--------|
| `State<VfsDatabase>` | 17 |
| `String` | 14 |
| `Option<String>` | 8 |
| `Option<u32>` | 4 |
| `Vec<String>` | 1 |
| `DueReviewsFilter` | 1 |

## 当前返回类型

| 返回类型 | 出现次数 |
|---------|--------|
| `ReviewPlan` | 4 |
| `DueReviewsResult` | 3 |
| `BatchCreateResult` | 2 |
| `Option<ReviewPlan>` | 2 |
| `ReviewStats` | 2 |
| `()` | 1 |

## 命令清单与变更

| 当前命令 | 改为 | 参数变更 | 返回变更 |
|---------|------|---------|--------|
| `review_plan_batch_create` | *(保持)* | — | Result<T, ReviewPlanError> |
| `review_plan_create` | *(保持)* | — | Result<T, ReviewPlanError> |
| `review_plan_create_for_exam` | *(保持)* | — | Result<T, ReviewPlanError> |
| `review_plan_delete` | *(保持)* | — | Result<T, ReviewPlanError> |
| `review_plan_get` | *(保持)* | — | Result<T, ReviewPlanError> |
| `review_plan_get_by_question` | *(保持)* | — | Result<T, ReviewPlanError> |
| `review_plan_get_calendar_data` | *(保持)* | → Input struct | Result<T, ReviewPlanError> |
| `review_plan_get_due` | *(保持)* | — | Result<T, ReviewPlanError> |
| `review_plan_get_due_with_filter` | *(保持)* | — | Result<T, ReviewPlanError> |
| `review_plan_get_history` | *(保持)* | — | Result<T, ReviewPlanError> |
| `review_plan_get_or_create` | *(保持)* | — | Result<T, ReviewPlanError> |
| `review_plan_get_stats` | *(保持)* | — | Result<T, ReviewPlanError> |
| `review_plan_list_by_exam` | *(保持)* | → Input struct | Result<T, ReviewPlanError> |
| `review_plan_process` | *(保持)* | → Input struct | Result<T, ReviewPlanError> |
| `review_plan_refresh_stats` | *(保持)* | — | Result<T, ReviewPlanError> |
| `review_plan_resume` | *(保持)* | — | Result<T, ReviewPlanError> |
| `review_plan_suspend` | *(保持)* | — | Result<T, ReviewPlanError> |

## 改进操作

统一错误类型为 ReviewPlanError

## 统一错误类型

`ReviewPlanError` — 替换当前使用的 `String` / `AppError`

---
*此报告由 deps.db 数据自动生成，对应模块原始数据见 `_data/review_plan_service.json`*
