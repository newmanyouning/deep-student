## PR3 — PDF 管道深度修复 + 聊天滚动 UX

**17 提交（基于 PR2）· 14 文件 · +426/-122 行**

依赖 PR2，展示在 PR2 基础上的增量变更：

### 1. PDF 管道深度修复 ✅（本次会话审计）
- **OCR 进度键匹配** — FileContentView 用 `node.sourceId` 替代 `node.id` 查询 statusMap
- **PDF.js 内存泄漏** — 文件切换时 `destroy()` 旧 PDFDocumentProxy，cleanup effect 依赖改为 `[file]`
- **force_ocr 标记泄漏** — ForceOcrGuard (RAII) 自动清理，防止重复 OCR 浪费 API 调用
- **连接池饥饿** — VfsPooledConnection 在 `.await` 前释放，避免 r2d2 池耗尽
- **OCR 错误静默吞没** — 错误消息通过红色横幅 + WarningCircle 图标展示给用户

### 2. 聊天滚动 UX 修复 ✅（本次会话修复）
- **流式文字重叠** — 单向 latch 滚动 + 程序化锁防止 syncScrollState 覆盖用户意图
- **sticky 思考摘要** — 流式期间禁用 sticky，避免渐变遮罩覆盖下方新内容
- **历史消息堆叠** — `useLayoutEffect` 在 paint 前完成 virtualizer 测量
- **打开会话自动滚底** — 初始数据加载完成后 scrollTop=scrollHeight
- **发送消息跳顶修复** — 移除 `scrollIntoView`（与 rAF 循环冲突导致 latch 错误触发）
- **click-to-bottom 可靠性** — `scrollTop` 直接赋值替代 `scrollTo({top, behavior})`

### 3. OpenAI Responses API 兼容
- 第三方端点（ai98pro.xyz 等）自动降级 Chat Completions

---

🤖 Generated with [Claude Code](https://claude.com/claude-code)
