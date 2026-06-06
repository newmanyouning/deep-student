# 前后端架构全面诊断报告

## 一、总体健康度评分

| 维度 | 分数 | 说明 |
|------|------|------|
| **架构健康度** | **74/100** | 模块划分清晰但存在致命通信断裂点 |
| **数据连通性** | **65/100** | ~200+ 后端命令在前端无调用，3 处静默失败路径 |
| **错误处理** | **60/100** | phantom 命令被 .catch() 吞没，错误不可见 |
| **代码质量** | **70/100** | 事件命名不一致、参数风格二象性、巨型文件 |

---

## 二、关键发现 (Critical)

### C1. WORKSPACE_CLOSED 事件名称不匹配

**严重程度: 高** -- 事件监听器静默永不触发。

| 侧 | 位置 | 事件名 |
|----|------|--------|
| Rust 发射端 | `src-tauri/src/chat_v2/workspace/emitter.rs:17` | `"chat_v2_workspace_closed"` |
| TypeScript 监听端 | `src/features/chat/workspace/events.ts:39` | `"workspace_closed"` |

**影响范围**: 工作区生命周期 UI 完全失效（关闭检测、资源清理、导航跳转）。用户关闭工作区后前端无任何反应。

**根因**: 常量定义不一致。Rust 的 `pub const WORKSPACE_CLOSED` 值与 TS 端的 `WORKSPACE_EVENTS.WORKSPACE_CLOSED` 值不同步。

**修复建议**: 统一事件名。推荐将 Rust 端 `emitter.rs:17` 改为 `pub const WORKSPACE_CLOSED: &str = "workspace_closed";`，或更改 TS 常量值（需检查是否有其他监听者也用此常量）。

---

### C2. `test_web_search_connectivity` 声明但未注册

**严重程度: 高** -- 每次调用都会在运行时抛出 "command not found"。

**证据链**:
- `#[tauri::command]` 声明于 `src-tauri/src/cmd/web_search.rs:17`
- 通过 `pub use crate::cmd::web_search::*` 在 `commands.rs:53` 重导出
- 前端在 `src/utils/settingsApi.ts:263` 调用
- **但不在 `lib.rs` 的 `invoke_handler` 列表中** -- grep 确认 `test_web_search` 在 `lib.rs` 中零匹配

**影响范围**: 前端设置页面中的"测试网络搜索连通性"功能对用户完全不可用，但 UI 上不会报错（catch 处理）。

**修复建议**: 在 `lib.rs` 的 `invoke_handler![]` 宏中加入 `test_web_search_connectivity`。

---

### C3. 19 个幽灵命令 (graphApi.ts) -- 无 Rust 实现

**严重程度: 高** -- 全部 19 个命令会在运行时产生 "command not found"，错误被 `.catch()` 吞没，用户无感知。

**文件**: `src/utils/graphApi.ts`

| # | 命令 | 行号 |
|---|------|------|
| 1 | `continue_unified_chat_stream` | 208 |
| 2 | `unified_get_card_tags` | 41 |
| 3 | `unified_toggle_card_tag` | 46 |
| 4 | `unified_add_card` | 52 |
| 5 | `unified_update_card` | 58 |
| 6 | `unified_batch_update_cards` | 64 |
| 7 | `unified_get_all_card_tags` | 71 |
| 8 | `unified_get_card_detail` | 77 |
| 9 | `unified_get_view_cards` | 84 |
| 10 | `unified_switch_view` | 93 |
| 11 | `unified_switch_card` | 99 |
| 12 | `unified_delete_card` | 105 |
| 13 | `unified_statistics` | 113 |
| 14 | `unified_grading_summary` | 120 |
| 15 | `unified_grading_sessions` | 127 |
| 16 | `unified_question_bank_tree` | 138 |
| 17 | `unified_set_question_selection` | 148 |
| 18 | `unified_upload_question_images` | 158 |
| 19 | `begin_chat_v2_session` | 188 |

**影响范围**: 所有这 19 次调用都是静默失败。`graphApi.ts` 可能是被废弃的旧模块（原"统一"架构的残留），但未被清理。任何代码路径如果调用这些函数都会产生无声错误。

**修复建议**: 两个选择之一：
1. (推荐) 删除 `graphApi.ts` 及所有引用它的代码，如果该模块已不再使用
2. (如需保留) 在 Rust 端实现全部 19 个命令并注册到 `lib.rs`

---

## 三、警告项 (Warning)

### W1. ~200+ 后端命令在前端无 `invoke()` 调用

**证据**: Rust 端在 `lib.rs` 中注册了约 580+ 命令，但 TypeScript 端的 `invoke()` 调用仅覆盖约 380 个。约 200+ 命令无前端消费。

**主要未使用命令群组**:

| 群组 | 示例 | 估计数量 |
|------|------|---------|
| PDF/考试会话 | `process_pdf_ocr`, `init_pdf_ocr_session`, `upload_pdf_ocr_page` | ~20 |
| 题库导入 | `import_question_bank`, `import_question_bank_stream`, `qbank_get_source_images` | ~10 |
| CSV 导入导出 | `import_questions_csv`, `export_questions_csv`, `get_csv_preview` | 4 |
| 语音/调试 | `voice_input_transcribe`, `get_debug_logs_info`, `clear_debug_logs` | ~8 |
| 安全/功能开关 | `get_security_status`, `get_feature_flags`, `is_feature_enabled` | ~12 |
| LLM 配置 CRUD | `get_api_configurations`, `save_api_configurations`, `test_api_connection` | ~15 |
| OCR 引擎配置 | `get_ocr_engines`, `set_ocr_engine_type`, `validate_ocr_model` | ~14 |
| Anki Connect | `anki_connect_check_status`, `anki_connect_get_deck_names`, `anki_connect_create_deck` | ~10 |

**影响**: 大量"死代码"增加维护成本，增加二进制体积，且表明存在已实现但从未上线的功能路径。部分命令可能是未来功能预留，但缺乏标记。

**建议**: 标记为 `#[doc = "UNUSED: ..."]` 或移入 `_unused` 模块，未来正式接入时再恢复。

---

### W2. snake_case + camelCase 双参数传递模式

**严重程度: 中** -- 设计上有意为之，但增加代码复杂度。

**文件**: `src/utils/shared.ts:185-188` 中存在辅助函数，同时传递 snake_case 和 camelCase 参数键。这是一个在代码库中有意采用的模式。

**影响范围**: 所有使用了该辅助函数的 API 调用。模式通过辅助函数分发，但导致了参数命名的二象性，增加了接口约定的复杂度。

**建议**: 建立统一标准。如果 Tauri `invoke` 调用必须使用 snake_case，则全部统一；反之亦然。移除双参数传递的辅助函数，减少混乱。

---

### W3. `essay_grading_list_sessions` 返回类型不匹配

**证据**: 分析报告标记为 **不匹配**，经核实确认。

- **Rust 后端** (`src-tauri/src/essay_grading/mod.rs:172-184`): 返回 `EssayGradingResult<Vec<VfsEssaySession>>`（扁平 JSON 数组）
- **前端** (`src/essay-grading/essayGradingApi.ts:134`): 期望一个带 `{ sessions: ... }` 包装的对象结构

**影响**: 前端获取到的数据结构与预期不符，可能导致渲染错误或数据字段遗漏。

**建议**: 统一契约。要么后端加包装层，要么前端改解构方式。建议以后端为锚点调整前端解构。

---

### W4. 分析报告数据不一致

| 报告声明 | 实际验证 | 
|---------|---------|
| "37 DSTU commands total" | 实际 grep `#[tauri::command]` 在 DSTU 模块中为 36 个 + `pub use` 重导出约 40 个 |
| "DSTU 源文件总计 ~21,286 lines" | 实际加总为 16,181 行（差异达 5,105 行） |

**影响**: 虽然不影响运行时代码，但表明文档/报告生成流程中存在数据提取错误，可能导致后续决策失误。

**建议**: 审查报告生成脚本的数据提取逻辑。

---

### W5. 两个超大源文件

| 文件 | 行数 |
|------|------|
| `src-tauri/src/vfs/pdf_processing_service.rs` | 4,042 |
| `src-tauri/src/vfs/repos/attachment_repo.rs` | 2,896 |

单文件 4,000+ 行表示模块内聚性不足。建议按职责拆分。

---

## 四、信息项 (Info)

### I1. 模块架构合理

系统划分为 23 个顶级模块 (+ 65 个独立文件)，总计 432 个 Rust 源文件。模块划分整体清晰：
- `chat_v2` (97) -- 最大的功能模块，采用 Block 架构
- `vfs` (60) -- 统一虚拟文件系统
- `data_governance` (42) -- 数据治理
- `dstu` (36) -- Finder 协议层

各模块职责单一，依赖关系较清晰，没有发现循环依赖。

### I2. 前端 API 层设计统一

所有 13 个 API 文件统一通过 `@tauri-apps/api/core` 的 `invoke()` 进行 IPC 通信，没有使用 `fetch()`。这是一致的架构决策，有利于安全性和类型安全。

### I3. Two Large Files Verified Accurate

`pdf_processing_service.rs` (4,042 行) 和 `attachment_repo.rs` (2,896 行) 经核实行数准确，文件路径正确。这是代码库中行数最多的两个文件。

### I4. Chat V2 工具子系统结构完整

ChatV2 Tools 目录结构包含 5 个子目录、约 150 个工具函数，划分清晰：
- `standard/` -- 标准工具
- `context_collect/` -- 上下文采集
- `delegate/` -- 委托工具
- `file_operations/` -- 文件操作
- `teacher_tools/` -- 教学工具

### I5. 错误类型层次完整

错误类型转换链完整，覆盖了从底层 `VfsError` 到上层 `ToolError` 的完整链路。包含 4 个自定义错误类型、16 个 `From` 实现、~637 个函数已完成标准化。

---

## 五、架构亮点

1. **统一的错误处理体系**: 通过 `From` 转换链实现层级式错误传播（VfsError -> MemoryError -> ...），避免了 `anyhow::Error` 的滥用。4 个自定义错误类型覆盖全栈。

2. **前后端 IPC 模式一致**: 全部 13 个前端 API 文件统一使用 Tauri `invoke()`，未混入 `fetch()`/`axios()`，确保安全模型一致。

3. **模块职责划分清晰**: 23 个顶级模块各有独立职责，`vfs` 作为统一存储层、`chat_v2` 作为对话引擎、`data_governance` 管理数据生命周期，整体层次分明。

4. **大重构工程已完成**: ~637 个函数已完成错误类型标准化，P0-P2 重构任务全部完成，Rust 阶段语法扫描 (20/44 批次) 全部通过。

5. **前端 API 文件按业务聚合**: 13 个 API 文件按后端模块前缀命名(`chatV2Api.ts`, `memoryApi.ts`, `vfsFileApi.ts` 等)，便于维护和定位。

---

## 六、子系统健康度矩阵

| 子系统 | 后端完整性 | 前端完整性 | 数据连通 | 错误处理 | 总分 |
|--------|-----------|-----------|---------|---------|------|
| **Chat V2** | 95% | 90% | 85% | 80% | **88** |
| **Memory** | 90% | 85% | 85% | 80% | **85** |
| **VFS** | 90% | 85% | 80% | 75% | **83** |
| **Essay Grading** | 85% | 80% | 80% (有类型不匹配) | 75% | **80** |
| **Data Governance** | 85% | 80% | 75% | 75% | **79** |
| **DSTU** | 85% | 75% | 70% | 75% | **76** |
| **Translation** | 80% | 75% | 75% | 75% | **76** |
| **MCP** | 75% | 70% | 65% | 70% | **70** |
| **LLM Manager** | 80% | 65% | 60% (大量命令未接) | 70% | **69** |
| **Cloud Storage** | 70% | 65% | 60% | 65% | **65** |
| **OCR** | 75% | 60% | 55% (~14 命令未接) | 65% | **64** |

---

## 七、改进路线图

### P0 -- 立即修复（影响功能性）

1. **修复 WORKSPACE_CLOSED 事件名** (C1)
   - 统一 Rust `emitter.rs` 和 TS `events.ts` 中的事件名字符串
   - 预期工时: 15 分钟

2. **注册 `test_web_search_connectivity`** (C2)
   - 在 `lib.rs` 的 `invoke_handler![]` 中加入该命令
   - 预期工时: 5 分钟

3. **处理 graphApi.ts 19 个幽灵命令** (C3)
   - 方案 A: 删除 `graphApi.ts` 及所有引用（推荐，如果已废弃）
   - 方案 B: 在 Rust 端实现并注册所有命令
   - 预期工时: 1-2 小时

### P1 -- 近期修复（影响维护性）

4. **清理 200+ 未使用命令** (W1)
   - 标记或移除无前端调用的后端命令
   - 预期工时: 3-4 小时

5. **拆分超大源文件** (W5)
   - `pdf_processing_service.rs` (4,042 行) -> 按流程阶段拆分
   - `attachment_repo.rs` (2,896 行) -> 按操作类型拆分
   - 预期工时: 4 小时

6. **修复 `essay_grading_list_sessions` 类型不匹配** (W3)
   - 统一前后端数据契约
   - 预期工时: 30 分钟

### P2 -- 中期改进（影响质量）

7. **消除 snake_case/camelCase 二象性** (W2)
   - 选择一种命名风格统一所有 Tauri 命令参数
   - 移除 `shared.ts` 中的双参数辅助函数
   - 预期工时: 2-3 小时

8. **修复报告生成脚本** (W4)
   - 修复 DSTU 命令计数和行数统计逻辑
   - 预期工时: 1 小时

9. **替换 phantom 命令的 `.catch()` 吞没模式**
   - 在 `graphApi.ts` (如保留) 和其他静默错误处添加日志或用户通知
   - 预期工时: 2 小时

### P3 -- 长期优化

10. **建立前后端契约自动化检查**
    - 引入 Tauri 命令注册表与前端 `invoke()` 调用的交叉验证 CI 步骤
    - 防止新的幽灵命令/未注册命令

11. **第二阶段语法扫描 (TypeScript)**
    - 按计划推进 T1-T21 批次扫描
    - 当前 Rust 阶段已完成 (20/44 批次, 45.5%)

12. **交叉检查 (C1-C3)**
    - Rust mod vs 文件系统一致性
    - TS import vs 文件系统一致性
    - Tauri command 注册 vs 前端调用一致性

---

**报告生成时间**: 2026-06-06 13:00 CST | **数据来源**: 6 轮独立架构侦察 + 交叉验证审计