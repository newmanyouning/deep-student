# API 重构: cmd::mcp — MCP 协议

**日期**: 2026-05-29 | **命令数**: 13 | **对应诊断**: round-20~26

---

## 当前问题

13 个命令

## 当前参数模式

| 参数类型 | 出现次数 |
|---------|--------|
| `String` | 11 |
| `State<AppState>` | 9 |
| `Option<HashMap<String, String>>` | 5 |
| `Option<String>` | 5 |
| `Window` | 2 |
| `Vec<String>` | 2 |

## 当前返回类型

| 返回类型 | 出现次数 |
|---------|--------|
| `Result<serde_json::Value>` | 8 |
| `Result<()>` | 2 |
| `Result<Vec<serde_json::Value>>` | 1 |
| `Result<String>` | 1 |
| `Result<bool>` | 1 |

## 命令清单与变更

| 当前命令 | 改为 | 参数变更 | 返回变更 |
|---------|------|---------|--------|
| `get_mcp_status` | *(保持)* | — | — |
| `get_mcp_tools` | *(保持)* | — | — |
| `mcp_stdio_close` | *(保持)* | — | — |
| `mcp_stdio_send` | *(保持)* | — | — |
| `mcp_stdio_start` | *(保持)* | → Input struct | — |
| `preheat_mcp_tools` | *(保持)* | — | — |
| `reload_mcp_client` | *(保持)* | — | — |
| `save_mcp_config` | *(保持)* | — | — |
| `test_mcp_connection` | *(保持)* | → Input struct | — |
| `test_mcp_http` | *(保持)* | → Input struct | — |
| `test_mcp_sse` | *(保持)* | → Input struct | — |
| `test_mcp_websocket` | *(保持)* | — | — |
| `test_rmcp_streamable_http` | *(保持)* | — | — |

## 改进操作

统一错误类型

## 统一错误类型

`McpError` — 替换当前使用的 `String` / `AppError`

---
*此报告由 deps.db 数据自动生成，对应模块原始数据见 `_data/cmd__mcp.json`*
