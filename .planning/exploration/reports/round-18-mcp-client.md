# Round 18: MCP 客户端 — 诊断报告

**日期**: 2026-05-29
**状态**: ✅ 完成

---

## 模块规模: 19 文件, ~4,300 行

```
src/mcp/ (6 文件)
├── mcpService.ts           1916 行 🔴 — MCP 服务核心
├── builtinMcpServer.ts     1748 行 🔴 — 内置 MCP 服务器定义
├── tauriStdioTransport.ts  293 行 — Tauri stdio 传输层
├── mcpFrontendTester.ts    153 行 — 前端测试工具
├── presetMcpServers.ts     109 行 — 预设服务器配置
└── searchEngineAvailability.ts 58行 — 搜索引擎可用性

src/mcp-debug/ (13 文件) — 在生产构建中被排除
├── core/ (7 文件 — actionRecorder, errorCapture, networkMonitor...)
├── bridge.ts / registerStores.ts / types.ts
```

---

## 关键发现

- [ ] **P2** — `mcpService.ts` 1916 行 — MCP 连接管理/工具列表/状态同步全在一个文件
- [ ] **P2** — `builtinMcpServer.ts` 1748 行 — 内置工具定义过大。12 个内置技能的工具定义应拆分为独立文件
- [ ] **P2** — MCP 代码分散在 4 个位置: `src/mcp/` (核心) + `src/mcp-debug/` (调试) + `src/features/settings/components/McpToolsSection.tsx` (2247行 UI) + `src/features/settings/components/McpEditorSection.tsx` (1878行 UI)
- [ ] **P3** — `mcp-debug/` 13 文件在生产构建中被排除（[R01] vite exclude-mcp-debug 插件），但它们引用 `mcpService` — 排除后可能产生未使用的导入

---

## 建议优先处理

1. 拆分 `mcpService.ts` (1916行) — 连接管理/工具发现/事件处理分离
2. 拆分 `builtinMcpServer.ts` (1748行) — 每个技能一个工具定义文件
3. 统一 MCP 代码位置: 核心 (`mcp/`) + UI (`features/settings/components/mcp*`) → `features/mcp-management/` 或保持现状但明确边界
