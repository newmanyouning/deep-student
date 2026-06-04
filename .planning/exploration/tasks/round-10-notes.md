# Round 10: 笔记系统诊断

**层级**: 3.3 — 功能模块（前端）
**预计文件数**: 10-20
**状态**: ⏳ 待执行

## 目标

梳理笔记系统前端实现，重点关注 Milkdown 编辑器集成。

## 扫描文件清单

| # | 文件路径 | 关注点 |
|---|---------|--------|
| 1 | `src/features/notes/` 全部 .ts/.tsx 文件 | 笔记功能 UI |
| 2 | `src/components/crepe/` 全部 .ts/.tsx 文件 | Milkdown Crepe 编辑器包装 |

## 诊断要点

1. **编辑器架构**: Milkdown 插件体系、自定义插件
2. **笔记格式**: 支持的富文本格式
3. **编辑器与笔记模块的关系**: crepe 组件和 notes feature 的交互

## 输出格式

产出 `round-10-notes.md`
