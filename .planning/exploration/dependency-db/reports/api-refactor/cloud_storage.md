# API 重构: Cloud Storage — 云存储

**日期**: 2026-05-29 | **命令数**: 14 | **对应诊断**: round-20~26

---

## 当前问题

14 个命令，每个命令都重复 CloudStorageConfig 参数。应封装为 State。

## 当前参数模式

| 参数类型 | 出现次数 |
|---------|--------|
| `CloudStorageConfig` | 12 |
| `String` | 9 |
| `Option<String>` | 3 |
| `AppHandle` | 2 |
| `Vec<u8>` | 1 |

## 当前返回类型

| 返回类型 | 出现次数 |
|---------|--------|
| `Result<()>` | 3 |
| `Result<bool>` | 2 |
| `Result<Option<Vec<u8>>>` | 1 |
| `bool` | 1 |
| `Result<Vec<FileInfo>>` | 1 |
| `Result<Option<FileInfo>>` | 1 |

## 命令清单与变更

| 当前命令 | 改为 | 参数变更 | 返回变更 |
|---------|------|---------|--------|
| `cloud_storage_check_connection` | *(保持)* | → State<CloudConfig> | — |
| `cloud_storage_delete` | *(保持)* | → State<CloudConfig> | — |
| `cloud_storage_exists` | *(保持)* | → State<CloudConfig> | — |
| `cloud_storage_get` | *(保持)* | → State<CloudConfig> | — |
| `cloud_storage_is_s3_enabled` | *(保持)* | — | Result<bool, CloudStorageError> |
| `cloud_storage_list` | *(保持)* | → State<CloudConfig> | — |
| `cloud_storage_put` | *(保持)* | → State<CloudConfig> | — |
| `cloud_storage_stat` | *(保持)* | → State<CloudConfig> | — |
| `cloud_sync_delete_version` | *(保持)* | → State<CloudConfig> | — |
| `cloud_sync_download` | *(保持)* | → Input struct | — |
| `cloud_sync_get_device_id` | *(保持)* | — | Result<String, CloudStorageError> |
| `cloud_sync_get_status` | *(保持)* | → State<CloudConfig> | — |
| `cloud_sync_list_versions` | *(保持)* | → State<CloudConfig> | — |
| `cloud_sync_upload` | *(保持)* | → Input struct | — |

## 改进操作

CloudStorageConfig 封装为 State，统一错误类型

## 统一错误类型

`CloudStorageError` — 替换当前使用的 `String` / `AppError`

---
*此报告由 deps.db 数据自动生成，对应模块原始数据见 `_data/cloud_storage.json`*
