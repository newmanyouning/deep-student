# Round 23: VFS 虚拟文件系统 — 诊断报告

**日期**: 2026-05-29
**状态**: ✅ 完成

## 规模: 42 文件, 55,542 行

```
src-tauri/src/vfs/
├── handlers.rs           7431 行 🔴🔴 最大文件
├── indexing.rs           4632 行 🔴 向量化索引
├── types.rs              3147 行 🟡
├── pdf_processing_service.rs 3421 行 🟡
├── ref_handlers.rs       2543 行
├── repos/
│   ├── folder_repo.rs    3072 行
│   └── attachment_repo.rs 2704 行
└── unit_builder/         — 向量单元构建
```

## 发现

- **P1**: `handlers.rs` **7431 行** — 仅次于 chatanki_executor (5758行) 的项目 #2 Rust 文件
- **P2**: `indexing.rs` 4632 行 — PDF OCR → 分块 → Embedding → LanceDB 全流程
- **P2**: Folder 和 Attachment repo 各超 2500 行，repo 层偏重
