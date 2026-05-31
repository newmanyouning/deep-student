# Deep-Student 完全重扫描执行计划

> 制定: 2026-05-30 14:30 CST | 预计总耗时: 120-150min (7 批次)
> 项目规模: ~3030 文件 (1682 TS + 401 Rust + 947 其他)

## 扫描策略

**核心理念**: 将项目视作全新未知系统，重建完整上下文依赖地图。扫描重点为 **Rust 后端模块** 和 **TS 前端关键入口**，静态资源/第三方库/缓存/测试文件跳过或仅统计。

**CLAUDE.md 风格**: 中英混合、固定章节、Markdown 表格、`✅` 状态标记、`P0-P4` 优先级、`S/M/L/XL` 工作量、CST 时间戳。

## 批次划分

### Batch 0: 根配置与项目骨架 (15 文件, 15min)
**扫描对象**: 根目录配置、Cargo.toml、package.json、构建脚本入口

| 文件 | 用途 | 优先级 |
|------|------|--------|
| `package.json` | 前端依赖与脚本 | P0 |
| `tsconfig.json` / `tsconfig.node.json` | TS 编译配置 | P0 |
| `vite.config.ts` | 构建配置 | P0 |
| `src-tauri/Cargo.toml` | Rust 依赖 | P0 |
| `src-tauri/tauri.conf.json` | Tauri 配置 | P0 |
| `src-tauri/build.rs` | Rust 构建脚本 | P1 |
| `.github/workflows/ci.yml` | CI 流水线 | P1 |
| `tailwind.config.js` | CSS 框架配置 | P2 |
| `eslint.config.js` | 代码检查 | P2 |
| `postcss.config.js` | CSS 处理器 | P2 |
| `index.html` | 入口 HTML | P2 |
| `src/main.tsx` | React 入口 | P0 |
| `src/App.tsx` | 根组件 | P0 |
| `src-tauri/src/main.rs` | Rust 入口 | P0 |
| `src-tauri/src/lib.rs` | 模块注册 | P0 |

**产出**: `docs/re-scan/batch-0-root-config.md`

### Batch 1: Rust 后端 — 类型/错误/常量层 (30 文件, 20min)
**扫描对象**: 所有 `error.rs`, `types.rs`, `models.rs`, 以及模块 `mod.rs` 声明文件

| 子批次 | 文件 | 关键检查 |
|--------|------|----------|
| 1a: Error 类型 | `chat_v2/error.rs`, `vfs/error.rs`, `dstu/error.rs`, `memory/error.rs`, `essay_grading/error.rs`, `data_governance/error.rs`, `models.rs::AppError` | 错误类型命名一致性、From 转换链完整性 |
| 1b: 核心类型 | `chat_v2/types.rs`, `vfs/types.rs`, `dstu/types.rs`, `memory/types.rs`, `essay_grading/types.rs`, `data_governance/commands_types.rs` | 类型命名与模块前缀一致性 |
| 1c: 模块声明 | 所有 `mod.rs` 文件 (18个) | module 名称与文件路径一致性、pub 可见性正确性 |

**产出**: `docs/re-scan/batch-1-rust-types.md`

### Batch 2: Rust 后端 — 核心业务模块 (80 文件, 25min)
**扫描对象**: Chat V2、VFS、DSTU 三大核心模块

| 子批次 | 目录 | 文件数 | 关注点 |
|--------|------|--------|--------|
| 2a | `chat_v2/handlers/` (14) + `chat_v2/pipeline/` (7) + `chat_v2/tools/` (10) | 31 | 命令命名模式、Handler 间依赖 |
| 2b | `vfs/` 全部 (repos/, handlers/, services/) | 30 | VFS Handler 返回类型一致性 |
| 2c | `dstu/` 全部 (handlers/, export/) | 10 | DSTU 命名模式 |

**产出**: `docs/re-scan/batch-2-core-modules.md`

### Batch 3: Rust 后端 — 辅助模块与服务 (40 文件, 20min)
**扫描对象**: LLM Manager、Data Governance、Memory、Essay Grading、Providers

| 子批次 | 目录 | 关注点 |
|--------|------|--------|
| 3a | `llm_manager/` (adapters/, builtin_vendors) | 适配器注册、Provider 命名 |
| 3b | `data_governance/` (backup/, sync/, migration/, commands_*.rs) | 命令文件间依赖、错误类型使用 |
| 3c | `memory/`, `essay_grading/`, `providers/` | 服务层命名 |
| 3d | `cmd/` 子目录 (notes, web_search, ocr, mcp) | 命令命名规范 |
| 3e | `commands.rs` 遗留文件 | 死命令标记、迁移状态 |

**产出**: `docs/re-scan/batch-3-aux-modules.md`

### Batch 4: TypeScript 前端 — 类型/配置/基础设施 (50 文件, 20min)
**扫描对象**: types/, stores/, utils/, config/, hooks/

| 子批次 | 目录 | 关注点 |
|--------|------|--------|
| 4a | `src/types/` (16 文件) | TS 类型与 Rust 结构体对应关系 |
| 4b | `src/stores/` (14 文件) | Store 命名、持久化策略 |
| 4c | `src/utils/` (41 文件) | 工具函数命名、重复检测 |
| 4d | `src/config/`, `src/hooks/`, `src/events/` | 配置管理、事件命名 |

**产出**: `docs/re-scan/batch-4-ts-infra.md`

### Batch 5: TypeScript 前端 — API 层与功能入口 (50 文件, 15min)
**扫描对象**: api/, services/, features/ 入口文件

| 子批次 | 目录 | 关注点 |
|--------|------|--------|
| 5a | `src/api/` | API 封装与 Rust 命令对应 |
| 5b | `src/services/` | 服务层架构 |
| 5c | `src/features/` 各模块入口 (12 dirs) | 功能模块入口 |

**产出**: `docs/re-scan/batch-5-ts-api.md`

### Batch 6: 文档/配置/脚本/资源 (30 文件, 15min)
**扫描对象**: docs/, scripts/, migrations/, 构建配置

| 子批次 | 目录 | 关注点 |
|--------|------|--------|
| 6a | `.planning/`, `docs/` 重构相关 | 文档一致性、引用路径有效性 |
| 6b | `src-tauri/migrations/` (4 库) | 迁移文件命名、版本一致性 |
| 6c | `scripts/` | 构建脚本依赖 |

**产出**: `docs/re-scan/batch-6-docs-config.md`

### Batch 7: 汇总 — 全局模块地图与冲突矩阵 (整合, 20min)
**扫描对象**: 前 6 批的产出物汇总

- 全局模块定义表 (所有模块名称/别名/职责/文件)
- 完整依赖关系图
- 冲突跟踪表 (命名冲突/职责不符/临时命名)
- 统一命名对照表
- 后续重构行动建议

**产出**: `docs/re-scan/FINAL-RESCAN-REPORT.md`

## 依赖关系

```
Batch 0 (根配置) ─────────────────────────────────────────┐
    ↓                                                       │
Batch 1 (类型/错误层) ── 依赖 Batch 0 ─────────────────────┤
    ↓                                                       │
Batch 2 (核心模块) ──── 依赖 Batch 1 ──────────────────────┤
    ↓                                                       │
Batch 3 (辅助模块) ──── 依赖 Batch 1 ──────────────────────┤
    ↓                                                       │
Batch 4 (TS 基础设施) ── 依赖 Batch 0 ─────────────────────┤
    ↓                                                       │
Batch 5 (TS API/Feature) ─ 依赖 Batch 4 ───────────────────┤
    ↓                                                       │
Batch 6 (文档/配置) ──── 独立 ──────────────────────────────┤
    ↓                                                       │
Batch 7 (汇总) ──────── 依赖 Batch 1-6 ────────────────────┘
```

## 产出物清单

| 批次 | 报告文件 | 预估行数 |
|------|----------|----------|
| 0 | `batch-0-root-config.md` | ~200 |
| 1 | `batch-1-rust-types.md` | ~400 |
| 2 | `batch-2-core-modules.md` | ~600 |
| 3 | `batch-3-aux-modules.md` | ~400 |
| 4 | `batch-4-ts-infra.md` | ~400 |
| 5 | `batch-5-ts-api.md` | ~300 |
| 6 | `batch-6-docs-config.md` | ~200 |
| 7 | `FINAL-RESCAN-REPORT.md` | ~1000 |
| **总计** | | **~3500** |

## 依赖数据库核心表

| 表名 | 字段 | 说明 |
|------|------|------|
| `module_definitions` | name, aliases, file_path, layer, responsibility | 模块正式名称与别名 |
| `dependency_edges` | source_file, target_module, reference_type, line_number | 文件间引用关系 |
| `conflict_tracker` | conflict_id, location_a, location_b, name_diff, suggestion, status | 命名冲突矩阵 |

## 执行约束

- **禁止编译验证**（沿用项目规则）
- 每批次完成后必须产出报告，不中断跳跃
- 时间戳使用 `date +"%Y-%m-%d %H:%M CST"` 获取
- 每个文件必须记录路径、引用关系、自身定义
