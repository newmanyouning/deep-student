# 重构后编译指南

> 生成: 2026-05-30 12:10 CST | 适用: DeepStudent v0.9.40 重构后

## 前置条件

### 环境要求
- **Node.js**: >=22 (`npm` 自带)
- **Rust**: >=1.96.0 (`rustup` 安装)
- **Tauri CLI**: `npm install -g @tauri-apps/cli` 或 `cargo install tauri-cli`
- **平台**: Windows (MSVC), macOS, Linux

### 无新增外部依赖
重构未引入任何新的 npm 包或 Rust crate。所有变更使用已有依赖。

## 编译步骤

### 步骤 1: 安装前端依赖
```bash
cd C:/deep-student
npm install --legacy-peer-deps
```
> `--legacy-peer-deps`: 处理 React 18 依赖冲突

### 步骤 2: TypeScript 类型检查 (定位 17 个遗留错误)
```bash
npx tsc --noEmit -p tsconfig.json
```
**预期**: 部分类型错误 (REF-018 待修复)。记下错误列表供后续修复。

### 步骤 3: Rust 后端编译检查
```bash
cd src-tauri
cargo check
```
**预期**: 编译通过。主要变更:
- `models.rs`: AppError/AppErrorType 序列化字段重命名
- `data_governance/error.rs`: 新增模块 (已在 mod.rs 注册)
- `chat_v2/handlers/`: resource_handlers.rs 已删除 (已在 mod.rs 移除)
- 所有 `-> Result<T, String>` → `-> ModuleResult<T>` 已完成

常见编译错误及修复:
| 错误 | 原因 | 修复 |
|------|------|------|
| `use of undeclared type DataGovernanceResult` | error.rs 未在 mod.rs 注册 | 验证 `pub mod error;` 存在 |
| `cannot find module resource_handlers` | 残留引用 | 搜索 `resource_handlers` 确认已完全移除 |
| `AppError.field` 不存在 | 旧代码引用 `error_type` 字段 | 改为 `error_type` (字段名未变仅 serde rename) |

### 步骤 4: 完整构建
```bash
cd C:/deep-student  # 回到项目根目录
npm run build
```
`prebuild` 脚本会自动运行 `typecheck` (可能失败, 不影响 build), 然后 `vite build`。

### 步骤 5: Tauri 打包 (可选)
```bash
npm run tauri build
# 或
cargo tauri build
```

## 关键变更清单 (编译前确认)

### TypeScript 前端
- [x] `src/types/api.ts` — 已删除 (零消费者 re-export)
- [x] `src/types/ui.ts` — 已删除
- [x] `src/types/hooks.ts` — 已删除
- [x] `src/store/ResourceStateManager.ts` — 已删除 (P2-07)
- [x] `src/lib/utils.ts` — 改为从 `@/utils/cn` 重导出 (REF-004)
- [x] `src/features/chat/core/store/createChatStore.ts` — generateId 改为重导出 (REF-005)
- [x] `src/stores/ankiQueueStore.ts` — 使用 tauriPersistStorage (REF-007)
- [x] 59 个 Tauri 命令重命名 → 前后端同步 (REF-012)
- [x] `src/utils/tauriPersistStorage.ts` — 新增文件 (REF-007)
- [x] `SiliconFlowSection.tsx` — 存储路径修复 (BUGFIX)

### Rust 后端
- [x] `models.rs` — 16 死类型移除, AppError serde 修复
- [x] `data_governance/error.rs` — 新增 DataGovernanceError
- [x] `chat_v2/handlers/resource_handlers.rs` — 已删除
- [x] `chat_v2/handlers/mod.rs` — resource_handlers 声明已移除
- [x] `lib.rs` — resource_handlers 命令注册已移除 (7 命令)
- [x] `backup_job_manager.rs` — String→AppError (7 函数)
- [x] `cloud_storage/config.rs` — String→AppError
- [x] `anki_connect_service.rs` — String→AppError (7 函数)
- [x] `commands.rs` — 7 死命令标记 DEPRECATED (REF-016)
- [x] 所有 Tauri handler 文件 → `Result<T, String>` → `ModuleResult<T>`

### 诊断报告
- [x] `cumulative-issues.md` — 7 个问题已标记解决
- [x] `round-03-types-shared.md` — 文件删除同步
- [x] `round-04-stores.md` — store/ 移除同步
- [x] 新报告: refactor-progress-summary, master-guide, master-checklist, 10 task files

## 编译后验证

### UI 功能检查
1. 打开设置 → 输入 API Key → 保存按钮点亮 → 保存后显示已保存
2. 硅基流动: 保存按钮和清除按钮正常显示
3. 其他供应商: 同上
4. 发送一条测试消息 → 确认对话正常

### 日志检查
```bash
# Windows
type %APPDATA%\deep-student\logs\*.log | findstr "安全存储"
```
- 搜索 "安全存储" 确认无错误
- 搜索 "get_secret" 确认密钥读取成功
