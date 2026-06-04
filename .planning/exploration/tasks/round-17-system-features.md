# Round 17: 系统功能诊断（设置/命令面板/技能管理/调试面板）

**层级**: 3.10 — 功能模块（前端）
**预计文件数**: 15-30
**状态**: ⏳ 待执行

## 目标

梳理系统级功能：设置、命令面板、技能管理UI、调试面板。

## 扫描文件清单

| # | 文件路径 | 关注点 |
|---|---------|--------|
| 1 | `src/features/settings/` 全部 .ts/.tsx 文件 | 设置页 |
| 2 | `src/command-palette/` 全部 .ts/.tsx 文件 | 命令面板 (cmdk) |
| 3 | `src/features/command-palette/` 全部文件 | 命令面板 feature |
| 4 | `src/features/skills-management/` 全部文件 | 技能管理 UI |
| 5 | `src/components/skills-management/` 全部 .tsx 文件 | 技能管理组件 |
| 6 | `src/components/llm-usage/` 全部 .tsx 文件 | LLM 用量展示 |
| 7 | `src/components/stats/` 全部 .tsx 文件 | 统计面板 |
| 8 | `src/components/style-lab/` 全部 .tsx 文件 | 样式实验室 |
| 9 | `src/debug-panel/` 全部 .ts/.tsx 文件 | 调试面板 |

## 诊断要点

1. **设置结构**: 设置项分类和存储方式
2. **命令面板**: 命令注册机制、搜索算法、快捷键绑定
3. **技能管理**: 技能 CRUD、加载配置
4. **调试面板**: 可用的调试工具和插件

## 输出格式

产出 `round-17-system-features.md`
