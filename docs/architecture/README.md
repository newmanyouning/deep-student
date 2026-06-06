# Deep Student 架构文档

> 最后更新: 2026-06-06 | 基于源代码静态分析生成
> 
> 所有图示使用 **Mermaid** 格式，在 GitHub、VS Code、Typora 等支持 Mermaid 的编辑器中可直接渲染。

---

## 📂 文档索引

### 🏗️ 系统总览

| 文档 | 内容 | 图表类型 |
|------|------|---------|
| [00-system-overview](diagrams/00-system-overview.md) | C4 系统上下文图 + 容器图 | C4Context, C4Container |
| [01-backend-modules](diagrams/01-backend-modules.md) | Rust 模块依赖图 + 核心类图 | Flowchart, ClassDiagram |
| [02-database-schema](diagrams/02-database-schema.md) | 5 个 SQLite 数据库的完整 ER 图 | ErDiagram |
| [03-error-propagation](diagrams/03-error-propagation.md) | 错误类型层级 + From 转换链 + 传播时序 | ClassDiagram, Flowchart, SequenceDiagram |

### 🎨 前端架构

| 文档 | 内容 | 图表类型 |
|------|------|---------|
| [04-frontend-architecture](diagrams/04-frontend-architecture.md) | React 组件树 + 路由结构 + 功能模块地图 | Flowchart |
| [05-state-management](diagrams/05-state-management.md) | Zustand Store 架构 + 数据流 | Flowchart, ClassDiagram |
| [06-api-layer](diagrams/06-api-layer.md) | API 层 + Hook → API → Command 映射 | Flowchart, SequenceDiagram |

### 🔗 前后端数据连通

| 文档 | 内容 | 图表类型 |
|------|------|---------|
| [07-tauri-command-map](diagrams/07-tauri-command-map.md) | Tauri 命令注册映射表 | Flowchart, Table |
| [08-event-system](diagrams/08-event-system.md) | 事件发射/订阅体系 | Flowchart, SequenceDiagram |
| [09-critical-data-flows](diagrams/09-critical-data-flows.md) | PDF上传→OCR、聊天消息、资源打开流程 | SequenceDiagram |

### 🧩 子系统深度分析

| 文档 | 内容 | 图表类型 |
|------|------|---------|
| [10-vfs-subsystem](diagrams/10-vfs-subsystem.md) | VFS 虚拟文件系统内部结构 | ClassDiagram, SequenceDiagram, Flowchart |
| [11-chatv2-subsystem](diagrams/11-chatv2-subsystem.md) | ChatV2 聊天系统 + 工具执行架构 | SequenceDiagram, ClassDiagram |
| [12-llm-ocr-subsystem](diagrams/12-llm-ocr-subsystem.md) | LLM 管理器 + OCR 适配器插件体系 | ClassDiagram, StateDiagram |
| [13-data-memory-essay](diagrams/13-data-memory-essay-subsystems.md) | 数据治理 + 记忆系统 + 作文批改 | ClassDiagram, SequenceDiagram |

---

## 🗺️ 系统架构速览

```
┌─────────────────────────────────────────────────────────────┐
│                    Deep Student Desktop App                   │
│  ┌──────────────────────┐  ┌──────────────────────────────┐ │
│  │   React Frontend      │  │    Tauri Rust Backend        │ │
│  │   (WebView2/WKWebView)│  │    (~397 source files)       │ │
│  │                       │  │                              │ │
│  │  ┌─────────────────┐  │  │  ┌────────────────────────┐  │ │
│  │  │ Features (~20)   │  │  │  │ Feature Modules (~15)  │  │ │
│  │  │ - learning-hub   │  │  │  │ - vfs, chat_v2, dstu   │  │ │
│  │  │ - chat           │◄─┼──┼─►│ - essay, memory, llm   │  │ │
│  │  │ - pdf            │  │  │  │ - tools, mcp, cloud    │  │ │
│  │  │ - settings       │  │  │  └────────────────────────┘  │ │
│  │  │ - dstu, essay    │  │  │                              │ │
│  │  └─────────────────┘  │  │  ┌────────────────────────┐  │ │
│  │                       │  │  │ Infrastructure          │  │ │
│  │  ┌─────────────────┐  │  │  │ - database (5 DBs)     │  │ │
│  │  │ Stores (~30)     │  │  │  │ - file_manager         │  │ │
│  │  │ Zustand + persist│  │  │  │ - crypto_service       │  │ │
│  │  └─────────────────┘  │  │  │ - config               │  │ │
│  │                       │  │  └────────────────────────┘  │ │
│  │  ┌─────────────────┐  │  │                              │ │
│  │  │ Hooks (~40)      │  │  │  ┌────────────────────────┐  │ │
│  │  │ data + UI logic  │  │  │  │ External Adapters      │  │ │
│  │  └─────────────────┘  │  │  │ - ocr_adapters          │  │ │
│  │                       │  │  │ - paddleocr_api         │  │ │
│  │  ┌─────────────────┐  │  │  │ - anki_connect_service  │  │ │
│  │  │ API Layer (~30)  │  │  │  │ - mcp servers           │  │ │
│  │  │ invoke() calls   │  │  │  └────────────────────────┘  │ │
│  │  └─────────────────┘  │  │                              │ │
│  └──────────────────────┘  └──────────────────────────────┘ │
│                                                              │
│  ┌──────────────────────────────────────────────────────┐   │
│  │                 Data Storage Layer                     │   │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ │   │
│  │  │ mistakes │ │  vfs.db  │ │ chat_v2  │ │ llm_usage│ │   │
│  │  │   .db    │ │  (~27    │ │   .db    │ │   .db    │ │   │
│  │  │          │ │  tables) │ │          │ │          │ │   │
│  │  └──────────┘ └──────────┘ └──────────┘ └──────────┘ │   │
│  │  ┌──────────┐ ┌──────────────────────────────────────┐ │   │
│  │  │ audit.db │ │         File System Blobs            │ │   │
│  │  └──────────┘ └──────────────────────────────────────┘ │   │
│  └──────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

---

## 📊 技术栈

| 层级 | 技术 | 版本 |
|------|------|------|
| 桌面框架 | Tauri v2 | 2.x |
| 后端语言 | Rust | 1.96 |
| 前端框架 | React | 18 |
| 状态管理 | Zustand | 4.x |
| PDF 渲染 | react-pdf (pdfjs-dist) | 3.x |
| 虚拟滚动 | @tanstack/react-virtual | 3.x |
| UI 组件 | Radix UI + Tailwind CSS | - |
| 数据库 | SQLite (rusqlite) | - |
| OCR 引擎 | PaddleOCR-VL, DeepSeek-OCR, System OCR | - |
| LLM 提供商 | DeepSeek, OpenAI, Anthropic, GLM, Qwen | - |

---

## 📈 规模统计

| 指标 | 数值 |
|------|------|
| Rust 源文件 | ~397 |
| TypeScript/TSX 源文件 | ~1,575 |
| Tauri 命令 | ~200+ |
| SQLite 表 | ~65+ |
| Zustand Stores | ~30 |
| 自定义 Hooks | ~40 |
| API 文件 | ~30 |
| 错误类型 | 9 |
| 功能模块 (features) | ~20 |

---

## 🔍 相关分析报告

- [功能分析报告](../analysis/) — 各子系统功能点详细分析
- [数据追踪报告](../analysis/) — 关键功能的数据流追踪
- [依赖图](../analysis/DEPENDENCY_GRAPH.md) — 模块间依赖关系
- [错误目录](../analysis/ERROR_CATALOG.md) — 完整错误码和变体
- [命令注册表](../analysis/COMMAND_REGISTRY.md) — Tauri 命令完整列表
