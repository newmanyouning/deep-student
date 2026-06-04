# Round 24: VFS 虚拟文件系统诊断

**层级**: 4.5 — 后端模块（Rust）
**预计文件数**: 12-20
**状态**: ⏳ 待执行

## 目标

梳理虚拟文件系统（VFS）的后端实现。

## 扫描文件清单

| # | 文件路径 | 关注点 |
|---|---------|--------|
| 1 | `src-tauri/src/vfs/` 全部 .rs 文件 | VFS 核心 |
| 2 | `src-tauri/src/vfs/repos/` 全部 .rs 文件 | 仓储层 |
| 3 | `src-tauri/src/vfs/unit_builder/` 全部 .rs 文件 | 单元构建器 |

## 诊断要点

1. **存储架构**: SQLite 元数据 + LanceDB 向量 + Blob 文件
2. **向量化 Pipeline**: OCR → 分块 → Embedding → 索引
3. **文件格式支持**: 支持导入的文件类型
4. **API 设计**: 对外暴露的 VFS 操作

## 输出格式

产出 `round-24-vfs.md`
