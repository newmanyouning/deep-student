# API 重构: Secure Store — 安全存储

**日期**: 2026-05-29 | **命令数**: 4 | **对应诊断**: round-20~26

---

## 当前问题

4 个命令

## 当前参数模式

| 参数类型 | 出现次数 |
|---------|--------|
| `CloudStorageCredentials` | 1 |

## 当前返回类型

| 返回类型 | 出现次数 |
|---------|--------|
| `()` | 2 |
| `Option<CloudStorageCredentials>` | 1 |
| `bool` | 1 |

## 命令清单与变更

| 当前命令 | 改为 | 参数变更 | 返回变更 |
|---------|------|---------|--------|
| `secure_delete_cloud_credentials` | *(保持)* | — | — |
| `secure_get_cloud_credentials` | *(保持)* | — | — |
| `secure_save_cloud_credentials` | *(保持)* | — | — |
| `secure_store_is_available` | *(保持)* | — | Result<bool, AppError> |

## 改进操作

保持现状

## 统一错误类型

`AppError` — 替换当前使用的 `String` / `AppError`

---
*此报告由 deps.db 数据自动生成，对应模块原始数据见 `_data/secure_store.json`*
