# Round 27: 数据治理与云同步诊断

**层级**: 4.8 — 后端模块（Rust）
**预计文件数**: 15-25
**状态**: ⏳ 待执行

## 目标

梳理数据治理（备份/审计/迁移）和云同步的后端实现。

## 扫描文件清单

| # | 文件路径 | 关注点 |
|---|---------|--------|
| 1 | `src-tauri/src/data_governance/` 全部 .rs 文件 | 数据治理核心 |
| 2 | `src-tauri/src/data_governance/audit/` 全部 .rs 文件 | 审计日志 |
| 3 | `src-tauri/src/data_governance/backup/` 全部 .rs 文件 | 备份恢复 |
| 4 | `src-tauri/src/data_governance/sync/` 全部 .rs 文件 | 增量同步 |
| 5 | `src-tauri/src/data_governance/migration/` 全部 .rs 文件 | 数据迁移 |
| 6 | `src-tauri/src/data_governance/dto/` 全部 .rs 文件 | 数据传输对象 |
| 7 | `src-tauri/src/cloud_storage/` 全部 .rs 文件 | 云存储 (S3/WebDAV) |

## 诊断要点

1. **同步范围**: 确认云同步分析文档中描述的 "13张表" 是哪些
2. **冲突解决**: conflict_resolver 的策略
3. **备份粒度**: 全量/增量备份
4. **审计追溯**: 审计日志的记录维度
5. **已知局限**: 对照 2026-05-23 的分析报告确认风险点

## 输出格式

产出 `round-27-data-governance.md`
