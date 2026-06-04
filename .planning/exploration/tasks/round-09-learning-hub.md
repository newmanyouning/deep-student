# Round 09: 学习资源中心诊断

**层级**: 3.2 — 功能模块（前端）
**预计文件数**: 10-20
**状态**: ⏳ 待执行

## 目标

梳理学习资源中心（Learning Hub）的前端实现。

## 扫描文件清单

| # | 文件路径 | 关注点 |
|---|---------|--------|
| 1 | `src/features/learning-hub/` 全部 .ts/.tsx 文件 | 资源管理 UI |
| 2 | `src/components/previews/` 全部 .tsx 文件 | 文件预览组件 |
| 3 | `src/components/LearningHeatmap/` 全部文件 | 学习热力图 |
| 4 | `src/components/BatchOperationToolbar/` 全部文件 | 批量操作工具栏 |

## 诊断要点

1. **资源类型**: 支持哪些资源格式
2. **导入流程**: 拖拽 → OCR → 分块 → 嵌入 → 索引的 UI 反馈
3. **文件预览**: PDF/DOCX 预览实现方式
4. **批量操作**: 支持的批量操作类型

## 输出格式

产出 `round-09-learning-hub.md`
