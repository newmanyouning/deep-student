# Round 20: 后端入口与命令路由诊断

**层级**: 4.1 — 后端模块（Rust）
**预计文件数**: 10-18
**状态**: ⏳ 待执行

## 目标

梳理 Tauri 后端的入口文件、命令注册、应用初始化流程。

## 扫描文件清单

| # | 文件路径 | 关注点 |
|---|---------|--------|
| 1 | `src-tauri/src/main.rs` | 二进制入口 |
| 2 | `src-tauri/src/lib.rs` | 库入口、Tauri Plugin 注册 |
| 3 | `src-tauri/Cargo.toml` | 依赖和 features |
| 4 | `src-tauri/tauri.conf.json` | Tauri 配置 |
| 5 | `src-tauri/src/cmd/` 全部 .rs 文件 | Tauri 命令处理器 |
| 6 | `src-tauri/src/data/` 全部 .rs 文件 | 数据模型/常量 |

## 诊断要点

1. **命令注册**: 所有 `#[tauri::command]` 函数清单
2. **Plugin 注册**: 启用的 Tauri Plugin 列表
3. **初始化流程**: `setup()` hook 中的初始化逻辑
4. **依赖分析**: Cargo.toml 中的主要依赖和 features
5. **错误处理**: 全局错误类型定义

## 输出格式

产出 `round-20-backend-entry.md`

```markdown
# Round 20: 后端入口与命令路由 — 诊断报告

**日期**: YYYY-MM-DD

## 应用初始化
- main.rs 流程
- lib.rs 模块结构
- Tauri Plugin 清单

## 命令清单

| 命令名 | 文件 | 所属模块 | 参数 | 返回 |
|--------|------|---------|------|------|
| ... | ... | ... | ... | ... |

## 依赖分析
- 主要 Cargo 依赖
- Feature flags

## 发现的问题
- [ ] ...
```
