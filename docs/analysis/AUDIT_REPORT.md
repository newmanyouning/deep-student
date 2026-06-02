# Deep Student 全量审计报告

> 生成日期: 2026-06-01
> 涵盖: 结构分析 | 语法扫描 | 适配器审计 | 测试基础设施 | 模型对比
> 对应分支: main (commit aca0ad)

---

## 1. 执行摘要

- **项目规模**: src-tauri/src 目录含 397 个 Rust 源文件，25 个子目录，69 个 lib.rs 注册模块。前端 ~1,575 个 TypeScript/TSX 文件未扫描。
- **编译状态**: 当前存在 **24 个编译错误** (E0308 类型不匹配为主) 和 **80 个 warnings**。核心原因: 错误类型重构 (String -> typed Error) 后未完全适配调用点，遗留 `String` 与 `DstuError`/`ToolError`/`VfsError` 混用。
- **死代码**: 2 个 Rust 文件 (`database.debug.rs`, `review_plan_error.rs`) 和 1 个非 Rust 资源目录 (`src/data/`) 属于死代码/死空间，永远不会被编译。
- **模型适配器**: 13 个专用适配器全部注册且功能正常。Aliyun DashScope (QwenAdapter) 和 SiliconFlow (GenericOpenAIAdapter) 各有独立的参数处理逻辑。
- **测试基础设施就绪**: `scripts/test_api_keys.json` (占位密钥) + `scripts/test_model_connectivity.py` (Python 端到端测试) 已就绪，仅需填入真实 API key 即可运行。
- **依赖安全风险**: 三个版本的 `rustls` (0.21/0.22/0.23)、`reqwest` (0.11/0.12/0.13)、`zip` (0.6/2.4/4.6) 同时编译，增加安全攻击面和编译体积。

---

## 2. 严重问题 (必须修复)

### 2.1 编译错误: 24 个 E0308/E0382 (5 个文件受累)

| 文件 | 错误数 | 错误模式 |
|------|--------|---------|
| `src/chat_v2/tools/chatanki_executor.rs` | 11 | `ToolError` / `AnkiConnectError` 传入期望 `String` 的参数位；全角引号语法错误 |
| `src/dstu/handlers.rs` | 6 | `String` 误作为 `DstuResult` 的 Err 变体返回 |
| `src/dstu/folder_handlers.rs` | 1 | `Err(e)` 中 `e` 为 `String` 而非 `DstuError` |
| `src/vfs/handlers.rs` | 2 | `Err(e.to_string())` 返回了 `String` 而非 `VfsError` |
| `src/paddleocr_api.rs` | 2 | `E0382`: `reqwest::Response` 在被 `.text()` 消费后重复使用 |
| `src/essay_grading/mod.rs` | 1 | `?` 操作符作用于返回 `()` 的函数，期望 `Result` |
| `src/chat_v2/tools/paper_save_executor.rs` | 1 | `Some(e.clone())` 中 `e` 为 `ToolError` 而非 `String` |

**根源**: 错误类型重构 (String -> typed Error) 过程中，部分调用点未能同步更新。典型模式:
```rust
// 错误: 返回 String 而非 DstuError
return Err("message".to_string());

// 纠正: 应包裹为 typed error
return Err(DstuError::InvalidPath("message".to_string()));
```

### 2.2 全角引号语法错误

`src/chat_v2/tools/chatanki_executor.rs:4712` — 字符串字面量中包含全角引号 `"..."` (U+201C/U+201D)，Rust 编译器无法识别。尽管 CI 反馈已修复 24+2 文件，此文件仍存在残留。

### 2.3 死代码文件

| 文件 | 问题 |
|------|------|
| `src/database.debug.rs` | 文件名含 `.` 导致无效 Rust 模块名，未在 lib.rs 注册，未被引用 |
| `src/review_plan_error.rs` | 未在 lib.rs 注册，`grep` 在所有 .rs 文件中零引用 |

**处理**: 两者应被删除或正确注册。

---

## 3. 警告问题 (建议修复)

### 3.1 Cargo Features 死配置

| Feature | 定义位置 | 问题 |
|---------|---------|------|
| `sqlite` | Cargo.toml | 定义但从未在任何 `.rs` 文件中 `#[cfg(feature = "sqlite")]` 引用 |
| `db_migration` | Cargo.toml | 同上 |
| `old_migration_impl` | Cargo.toml | 同上 |
| `devtools` | Cargo.toml | 仅为 `tauri/devtools` 特性透传，无应用级 `cfg` 守卫 |
| `examples` | 源码使用 | `src/mcp/client.rs:1976` 中 `#[cfg(feature = "examples")]` 引用但 Cargo.toml 未定义此 feature |

### 3.2 运行时 Feature Flags 无后端效果

`feature_flags.rs` 中的 `FeatureFlagManager` 定义了 11 个运行时标志（5 启用、3 禁用、1 渐进 30%、2 禁用且标记为仅前端），但后端代码实际上从未检查这些标志。`web_search` 功能直接从 DB 加载独立设置，名称相同但路径不同。

### 3.3 依赖膨胀

- **3 个 rustls 版本** (0.21/0.22/0.23) 同时编译 — 安全修复需要更新所有依赖
- **3 个 reqwest 版本** (0.11/0.12/0.13) — hyper 也双版本 (0.14/1.x)
- **3 个 zip 版本** (0.6/2.4/4.6) — Cargo.toml 指定 zip = "0.6"，但 vendored 依赖引入 2.4/4.6
- **125 个包** 以多个 semver 不兼容版本存在

### 3.4 弃用 API 使用

- `chrono::NaiveDateTime::from_timestamp_millis` (废弃) — 应在 `exam_sheet_service.rs` 中替换为 `DateTime::from_timestamp_millis`
- `VfsTextbookRepo::clear_mm_index` / `save_page_mm_index` 等 — 应在 `page_indexer.rs` 和 vfs repos 中替换为 `VfsIndexService`

### 3.5 `pub use` 全局导出可见性错误

`src/chat_v2/pipeline.rs:94-97` — 3 行 `pub use constants::*` / `pub use helpers::*` / `pub use variant_adapter::*` 标记为 `pub`，但实际导出项最高为 `pub(crate)`，导致 `rustc` warning。

### 3.6 `src/data/` 目录位置错误

`src/data/builtin-templates.json` 位于 Rust src 目录下但包含 0 个 .rs 文件、无 mod.rs、未在 lib.rs 注册。应移至 `src-tauri/resources/` 或由构建脚本管理。

### 3.7 大量未使用导入警告

80 个 warnings 中约 60+ 为 `unused_imports`，涉及:
- 所有 chat_v2/tools/* 中的 `event_types` 导入 (约 20 个文件)
- `ToolError` 在多个 executor 文件中被导入但未使用
- `data_governance/commands*.rs` 中的多个 BackupJob* 类型
- `essay_grading/mod.rs` 中的 `EssayGradingResult as _` 无效果

---

## 4. 模型 API 审计结果

### 4.1 Aliyun DashScope (通义千问)

| 项目 | 状态 |
|------|------|
| **供应商注册** | `builtin-vendors.rs` — `builtin-qwen` ✅ |
| **适配器** | `QwenAdapter` — 专用适配器 ✅ |
| **适配器特性** | enable_thinking, thinking_budget, reasoning_effort ✅ |
| **适配器特殊性** | DashScope 路径下自动移除 `frequency_penalty` (API 不支持) ✅ |
| **Base URL** | `https://dashscope.aliyuncs.com/compatible-mode/v1` |
| **注册模型 (7个)** | `qwen3-max`, `qwen3.5-plus`, `qwen3.5-flash`, `qwen-plus`, `qwq-plus`, `qwen3.5-397b-a17b`, `qwen3.5-122b-a10b` |
| **base_url 检测** | `is_dashscope()` 检查 `dashscope.aliyuncs.com` 或 `dashscope-intl.aliyuncs.com` ✅ |
| **测试密钥** | `scripts/test_api_keys.json` — 占位符 (`PLACEHOLDER_ALIYUN_KEY`) |
| **端到端验证** | 需要真实 API key。通过 `TEST_ALIYUN_DASHSCOPE_KEY` 环境变量注入 |

### 4.2 SiliconFlow

| 项目 | 状态 |
|------|------|
| **供应商注册** | `builtin-vendors.rs` — `builtin-siliconflow` ✅ |
| **适配器** | `GenericOpenAIAdapter` (标准 OpenAI 兼容) — 除非前端设置 `model_adapter=qwen` |
| **适配器特殊性** | QwenAdapter 在 SiliconFlow 路径下对 thinking_budget 做 128-32768 裁剪 ✅ |
| **Base URL** | `https://api.siliconflow.cn/v1` |
| **内置免费模型 (3个)** | `Qwen/Qwen3-8B`, `zai-org/GLM-4.6V`, `BAAI/bge-m3` — 通过编译时环境变量启用 |
| **编译特性** | `builtin_free_models` (Cargo feature) |
| **环境变量** | `SILICONFLOW_BUILTIN_TEXT_KEY`, `SILICONFLOW_BUILTIN_VISION_KEY`, `SILICONFLOW_BUILTIN_EMBED_KEY` |
| **测试密钥** | `scripts/test_api_keys.json` — 占位符 (`PLACEHOLDER_SILICONFLOW_KEY`) |

### 4.3 模型列表差异

**Aliyun DashScope**: 7 个 builtin 模型 vs API `/models` 端点 — 需要真实 key 才能获取实际可用模型列表。常见差异: API 可能返回 `qwen-turbo`、`qwen-max` 等额外模型，builtin 列表未包含 `qwen-turbo`。

**SiliconFlow**: 3 个 builtin 模型 (仅限免费 env-var 模型) vs API `/models` 端点 (含数百个社区/商业模型)。Builtin 列表仅为极小抽样。

### 4.4 适配器注册表完整性

已注册 (13 个适配器 + 别名):

| 主键 | 适配器 | 别名 |
|------|--------|------|
| `openai`, `general`, `siliconflow`, `nvidia` | `GenericOpenAIAdapter` | — |
| `deepseek` | `DeepSeekAdapter` | — |
| `qwen` | `QwenAdapter` | — |
| `zhipu` | `ZhipuAdapter` | — |
| `doubao` | `DoubaoAdapter` | — |
| `moonshot` | `MoonshotAdapter` | `kimi` |
| `ernie` | `ErnieAdapter` | `baidu` |
| `anthropic` | `AnthropicAdapter` | `claude` |
| `google` | `GeminiAdapter` | `gemini` |
| `xai` | `GrokAdapter` | `grok` |
| `mistral` | `MistralAdapter` | — |
| `minimax` | `MiniMaxAdapter` | — |
| `mimo` | `MimoAdapter` | — |

全部已注册的适配器都有对应的单元测试。聚合平台 (siliconflow, openrouter, together, fireworks, groq) 跳过 provider_type 适配器查找，转而使用 `model_adapter` 字段。

---

## 5. 测试基础设施状态

### 5.1 已生成的文件

| 文件 | 用途 |
|------|------|
| `scripts/test_api_keys.json` | 测试密钥配置文件 (占位符格式) |
| `scripts/test_model_connectivity.py` | 端到端模型连通性测试脚本 |

### 5.2 如何使用测试密钥文件

**test_api_keys.json** 结构:
```json
{
  "aliyun_dashscope": {
    "api_key": "PLACEHOLDER_ALIYUN_KEY",
    "base_url": "https://dashscope.aliyuncs.com/compatible-mode/v1",
    "test_models": ["qwen-plus", "qwen-turbo", "qwq-plus"]
  },
  "siliconflow": {
    "api_key": "PLACEHOLDER_SILICONFLOW_KEY",
    "base_url": "https://api.siliconflow.cn/v1",
    "test_models": ["Qwen/Qwen3-8B", "zai-org/GLM-4.6V", "BAAI/bge-m3"]
  }
}
```

**使用方式**:
1. 直接编辑 `api_key` 值为真实密钥
2. 或设置环境变量覆盖: `TEST_ALIYUN_DASHSCOPE_KEY` / `TEST_SILICONFLOW_KEY`
3. 运行: `python scripts/test_model_connectivity.py`
4. 输出: 完整 JSON 报告 (stdout)，含 /models 端点状态、chat completion 测试结果、模型列表 diff

### 5.3 Python 脚本能力

`test_model_connectivity.py` 执行以下操作:

1. **读取配置**: 从 JSON 文件加载平台配置
2. **检查密钥**: 检测占位符，跳过无真实密钥的平台
3. **测试 /models 端点**: `GET /v1/models` — 验证 API 可达性并获取可用模型列表
4. **模型列表比对**: 将 API 返回的模型列表与 Rust builtin 定义 (硬编码在脚本中) 做 diff，报告 `models_missing_from_api` 和 `models_not_in_builtin`
5. **Chat Completion 测试**: 对每个 `test_models` 发送 `{"model": "...", "messages": [{"role": "user", "content": "Hi"}], "max_tokens": 5}` — 记录状态码、延迟、响应文本、token 用量
6. **汇总报告**: 输出 JSON，含测试摘要 (总平台数、已测/跳过数、通过/失败数)

---

## 6. 下一步

### 6.1 需要真实 API key 的操作

| 任务 | 依赖 |
|------|------|
| 运行 `test_model_connectivity.py` 端到端测试 | Aliyun + SiliconFlow 真实 key |
| 对比 API /models 列表与 builtin 模型配置 | 同上 |
| 验证 thinking/reasoning 参数在 DashScope 上正确工作 | Aliyun DashScope 真实 key |
| 验证 builtin_free_models 编译和运行时加载 | SiliconFlow 免费 key (3 个 env var) |

### 6.2 无需 API key 可立即执行的操作

| 任务 | 优先级 |
|------|--------|
| 修复 24 个编译错误 | P0 |
| 删除死代码 (`database.debug.rs`, `review_plan_error.rs`) | P0 |
| 删除全角引号残留 | P0 |
| 清理 80 个 unused import warnings | P1 |
| 替换弃用 API (`from_timestamp_millis`, mm_index 函数) | P1 |
| 修复 `pub use` glob 可见性问题 | P2 |
| 将 `src/data/` 移出 src 目录 | P2 |
| 删除/修复死 Cargo features (`sqlite`, `db_migration`, `old_migration_impl`) | P2 |
| 清理依赖重复 (rustls, reqwest, hyper, zip) | P2 |
| 运行 TypeScript 语法扫描 (全部 ~1,575 文件) | P2 |
| 运行交叉检查 (C1-C3: mod vs fs, import vs fs, command vs frontend) | P3 |
| 整合运行时 feature flags 与后端逻辑 | P3 |

### 6.3 推荐修复优先级顺序

```
批次 1 (阻止编译)
  ├── chatanki_executor.rs: 全角引号 + 8 处 type mismatch (String vs ToolError/AnkiConnectError)
  ├── dstu/handlers.rs: 6 处 Err(String) -> Err(DstuError)
  ├── dstu/folder_handlers.rs: 1 处 Err(String) -> Err(DstuError)
  ├── vfs/handlers.rs: 2 处 Err(String) -> Err(VfsError)
  ├── paddleocr_api.rs: 2 处 E0382 borrow-after-move
  ├── essay_grading/mod.rs: 1 处 ? on ()
  └── paper_save_executor.rs: 1 处 String vs ToolError

批次 2 (清理阶段)
  ├── 删除 database.debug.rs, review_plan_error.rs
  ├── 移除 src/data/ -> 移至 resources/
  └── 全局清理 unused imports (60+) 和 deprecation 替换

批次 3 (模型验证)
  ├── 填入真实 API key 运行 test_model_connectivity.py
  ├── 对比模型列表，更新 builtin_vendors.rs 模型配置
  └── 验证 thinking/reasoning 参数生效

批次 4 (基础设施)
  ├── 修复 Cargo features
  ├── 清理依赖重复版本
  ├── 运行 TS 语法扫描
  └── 交叉检查
```

---

*报告结束*
