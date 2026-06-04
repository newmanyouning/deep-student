# Round 16: 作文批改及其他功能诊断

**层级**: 3.9 — 功能模块（前端）
**预计文件数**: 10-20
**状态**: ⏳ 待执行

## 目标

梳理作文批改、番茄钟、待办、沙箱、语音输入等较小功能模块。

## 扫描文件清单

| # | 文件路径 | 关注点 |
|---|---------|--------|
| 1 | `src/essay-grading/` 全部 .ts/.tsx 文件 | 作文批改 |
| 2 | `src/components/essay-grading/` 全部 .tsx 文件 | 作文批改 UI 组件 |
| 3 | `src/features/pomodoro/` 全部 .ts/.tsx 文件 | 番茄钟 |
| 4 | `src/features/todo/` 全部 .ts/.tsx 文件 | 待办事项 |
| 5 | `src/features/sandbox/` 全部 .ts/.tsx 文件 | 沙箱环境 |
| 6 | `src/voice-input/` 全部 .ts/.tsx 文件 | 语音输入 |
| 7 | `src/features/voice-input/` 全部文件 | 语音输入 feature |

## 诊断要点

1. **作文评分维度**: 支持的评分维度和场景
2. **各模块成熟度**: 区分完整/实验性/未完成的功能
3. **代码重用**: 是否有跨模块共享的评分/渲染逻辑

## 输出格式

产出 `round-16-essay-others.md`
