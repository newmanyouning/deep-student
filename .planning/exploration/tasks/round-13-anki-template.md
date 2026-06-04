# Round 13: Anki 闪卡与模板管理诊断

**层级**: 3.6 — 功能模块（前端）
**预计文件数**: 10-20
**状态**: ⏳ 待执行

## 目标

梳理 Anki 闪卡制作和模板管理的前端实现。

## 扫描文件清单

| # | 文件路径 | 关注点 |
|---|---------|--------|
| 1 | `src/components/anki/` 全部 .ts/.tsx 文件 | Anki 制卡 UI |
| 2 | `src/features/template-management/` 全部 .ts/.tsx 文件 | 模板管理 |
| 3 | `src/components/RealTimeTemplateEditor/` 全部文件 | 模板编辑器 |
| 4 | `src/data/anki/` 全部 .ts 文件 | Anki 数据层 |

## 诊断要点

1. **制卡流程**: 从对话到制卡的触发机制
2. **模板引擎**: Mustache 模板的加载和渲染
3. **3D 预览**: 卡片翻转预览的实现
4. **Anki 同步**: 与 Anki 生态的对接方式

## 输出格式

产出 `round-13-anki-template.md`
