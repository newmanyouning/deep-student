# Round 23: LLM 管理与适配诊断

**层级**: 4.4 — 后端模块（Rust）
**预计文件数**: 15-25
**状态**: ⏳ 待执行

## 目标

梳理 LLM Manager：多模型供应商适配、使用量追踪。

## 扫描文件清单

| # | 文件路径 | 关注点 |
|---|---------|--------|
| 1 | `src-tauri/src/llm_manager/` 全部 .rs 文件 | LLM 管理核心 |
| 2 | `src-tauri/src/llm_manager/adapters/` 全部 .rs 文件 | 各供应商适配器 |
| 3 | `src-tauri/src/llm_usage/` 全部 .rs 文件 | Token 用量追踪 |
| 4 | `src-tauri/src/adapters/` 全部 .rs 文件 | 通用适配器 |
| 5 | `src-tauri/src/providers/` 全部 .rs 文件 | 供应商定义 |

## 诊断要点

1. **供应商清单**: 确认内置的 9 家供应商及其适配状态
2. **适配器模式**: 统一的 Adapter trait 设计
3. **模型能力检测**: 自动检测机制
4. **用量追踪**: 记录维度和存储方式
5. **OpenAI 兼容**: 自定义 endpoint 支持

## 输出格式

产出 `round-23-llm-manager.md`
