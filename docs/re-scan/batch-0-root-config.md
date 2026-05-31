# Batch 0: 根配置与项目骨架 — 重新扫描报告

> 扫描时间: 2026-05-30 15:39-15:45 CST | 15 文件 | 状态: ✅ 完成
> 下一批次: Batch 1 (Rust 类型/错误层)

## 0.1 模块定义表

| 模块名称 | 别名 | 文件路径 | 层级 | 职责 |
|----------|------|----------|------|------|
| `deep-student` | — | `package.json` | L5 | 前端包定义, npm scripts 入口 |
| `deep-student` (Rust) | `deep_student_lib` | `src-tauri/Cargo.toml` | L5 | Rust crate 定义, 依赖管理 |
| `TauriConfig` | — | `src-tauri/tauri.conf.json` | L5 | Tauri 窗口/安全/打包/更新配置 |
| `ViteConfig` | — | `vite.config.ts` | L5 | Vite 构建配置, 插件, 别名 |
| `TsConfig` | — | `tsconfig.json` | L5 | TypeScript 编译选项 |
| `App` (React) | `App.tsx` | `src/App.tsx` | L5 | 根组件, 路由入口 |
| `main` (TS) | `main.tsx` | `src/main.tsx` | L5 | React 挂载入口 |
| `main` (Rust) | — | `src-tauri/src/main.rs` | L5 | Rust 二进制入口 |
| `lib` (Rust) | `deep_student_lib` | `src-tauri/src/lib.rs` | L5 | Tauri 插件注册 + 命令注册 |

## 0.2 依赖关系表

| 源文件 | 目标模块 | 引用类型 | 说明 |
|--------|----------|----------|------|
| `package.json` | `vite` 6 | devDependency | 构建工具 |
| `package.json` | `react` 18 | dependency | UI 框架 |
| `package.json` | `tauri` 2 | dependency | 桌面框架 |
| `package.json` | `zustand` 5.0.6 | dependency | 状态管理 |
| `Cargo.toml` | `tauri` 2, `serde`, `rusqlite`, `reqwest` | dependency | 核心依赖 |
| `Cargo.toml` | `protoc-bin-vendored` 3.0 | build-dep | protoc 编译器 |
| `tauri.conf.json` | `npm run dev`/`npm run build` | 命令 | 开发/构建流程 |
| `vite.config.ts` | `@vitejs/plugin-react` | 插件 | React Fast Refresh |
| `vite.config.ts` | `tailwindcss`, `autoprefixer` | CSS | 样式框架 |
| `tsconfig.json` | `@/*` → `src/*` | 路径别名 | 模块导入简写 |
| `App.tsx` | `@/stores/*`, `@/features/*`, `@/components/*` | 组件导入 | 全局状态与路由 |

## 0.3 关键发现

### TS 配置问题
| ID | 问题 | 位置 | 影响 |
|----|------|------|------|
| C0-01 | `strict: false` | `tsconfig.json:22` | TypeScript 非严格模式, 类型安全降低 |
| C0-02 | `noImplicitAny: false` | `tsconfig.json:27` | 隐式 any 不报错 |

### 依赖版本
| 包 | 版本 | 备注 |
|----|------|------|
| React | ^18.3.1 | 非 React 19 |
| Tauri | 2.0.x | 稳定版 |
| TypeScript | ~5.5+ | ES2022 target |
| Rust | 1.96.0 (Cargo.toml edition 2021) | 2021 Edition |

### 构建流程
```
npm run dev          → vite --host --port 1422
npm run build        → prebuild: typecheck + version:generate → vite build
npm run tauri dev    → 启动 Tauri 开发环境
npm run tauri build  → 打包桌面应用
```

### 命名冲突检查
| ID | 发现 | 详情 |
|----|------|------|
| N0-01 | `deep-student` vs `deep_student_lib` | package.json 用连字符, Cargo.toml lib.name 用下划线 |

## 0.4 依赖数据库更新

### 新增模块定义: 9
### 新增依赖边: 15
### 发现冲突: 2 (C0-01, N0-01)

---

*Batch 0 完成。文件: 15 | 模块: 9 | 依赖边: 15 | 冲突: 2*
