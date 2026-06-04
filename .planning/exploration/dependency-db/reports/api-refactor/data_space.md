# API 重构: Data Space — 数据空间 (A/B 槽位)

**日期**: 2026-05-29 | **命令数**: 10 | **对应诊断**: round-20~26

---

## 当前问题

10 个命令

## 当前参数模式

| 参数类型 | 出现次数 |
|---------|--------|
| `String` | 3 |
| `AppHandle` | 1 |

## 当前返回类型

| 返回类型 | 出现次数 |
|---------|--------|
| `String` | 4 |
| `DataSpaceInfo` | 1 |
| `SlotSizeInfo` | 1 |
| `TestSlotInfo` | 1 |
| `()` | 1 |
| `Vec<SlotIntegrityReport>` | 1 |

## 命令清单与变更

| 当前命令 | 改为 | 参数变更 | 返回变更 |
|---------|------|---------|--------|
| `check_switch_disk_space` | *(保持)* | — | — |
| `clear_test_slots` | *(保持)* | — | — |
| `get_data_space_info` | *(保持)* | — | — |
| `get_slot_directory` | *(保持)* | — | — |
| `get_slot_size` | *(保持)* | — | — |
| `get_test_slot_info` | *(保持)* | — | — |
| `mark_data_space_pending_switch_to_inactive` | *(保持)* | — | — |
| `restart_app` | *(保持)* | — | — |
| `verify_all_slots_integrity` | *(保持)* | — | — |
| `verify_slot_integrity` | *(保持)* | — | — |

## 改进操作

统一错误类型

## 统一错误类型

`DataSpaceError` — 替换当前使用的 `String` / `AppError`

---
*此报告由 deps.db 数据自动生成，对应模块原始数据见 `_data/data_space.json`*
