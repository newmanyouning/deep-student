## PR2 — 基础设施 + 通用修复

**76 提交 · 518 文件 · +146K/-38K 行**

这是所有后续修复的**地基 PR**。包含：

### 1. 错误类型标准化（~30 提交）
- 全模块 `anyhow::Result` / `String` → 类型化错误（VfsResult, DstuResult, ChatV2Result 等）
- ~637 函数/命令转换，9 种错误类型 + 16 个 From 转换链
- 编译器验证：`cargo check` 通过，零行为变更

### 2. CI 工作流（~10 提交）
- 新增 build-test.yml 手动构建
- 修复 Rust/TS 编译错误
- CI 忽略 .md/.github 提交、YAML 语法修复

### 3. PDF 阅读恢复 ✅（~15 提交）
- PDF blob 存储（小文件和大文件分离策略）
- 403 根因修复（textbook attachment 路径）
- 大 PDF 加载（271MB+）
- PDF 错误类型分类 + 多策略重试 + 用户友好错误 UI
- PDF 渲染 + OCR 检查点/续传

### 4. 通用修复 ✅
- **API key 粘贴检测** — InputEvent 监听 + onInput 回退
- **会话加载兼容** — 分段恢复跳过不兼容块
- **弃用工具模式修正** — `builtin-anki_*` 精确匹配
- **默认搜索引擎** — zhipu 替代 google_cse
- **第三方代理降级** — 非 OpenAI 端点自动 Chat Completions
- **性能优化** — 分批 set()、LRU 适配器池、async 节流持久化
- **Anki** — 模型配置验证 + 卡片工具错误处理
- **LLM** — 错误消息含 provider/model/URL
- **架构文档** — 14 个 Mermaid UML 图 + 诊断报告

### 5. 杂项
- iPad 构建工作流、AUTHORS、gitignore、测试脚本清理

> ⚠️ 注意：OCR 按钮/横幅的改动包含在此 PR 中（提交交错无法分离），但 OCR 功能尚未完全恢复，需 PR3 配合。

---

🤖 Generated with [Claude Code](https://claude.com/claude-code)
