# API 重构报告索引

**日期**: 2026-05-29
**关联**: 探究报告 (round-01 ~ round-supplement-complete)

---

## 目录对照

| 优先级 | 模块 | 命令数 | 重构报告 | 对应诊断 |
|--------|------|--------|---------|---------|
| 🔴 P0 | commands.rs | 137 | [commands.md](commands.md) | R20 (后端入口) |
| 🔴 P0 | VFS | 119 | [vfs.md](vfs.md) | R23 (VFS诊断) |
| 🔴 P1 | Chat V2 | 78 | [chat_v2.md](chat_v2.md) | R21 (Chat后端) |
| 🟡 P1 | DSTU | 54 | [dstu.md](dstu.md) | R19 (DSTU前端) + R24 (DSTU后端) |
| 🟡 P1 | Data Governance | 43 | [data_governance.md](data_governance.md) | R24-26 (后端合并) |
| 🟡 P1 | Memory | 27 | [memory.md](memory.md) | R26 (Memory诊断) |
| 🟡 P2 | Notes (cmd) | 39 | [cmd__notes.md](cmd__notes.md) | R10 (Notes前端) |
| 🟡 P2 | Enhanced Anki | 22 | [cmd__enhanced_anki.md](cmd__enhanced_anki.md) | R13 (Anki前端) |
| 🟢 P2 | Essay Grading | 20 | [essay_grading.md](essay_grading.md) | R16 (Essay) |
| 🟢 P2 | Review Plan | 17 | [review_plan_service.md](review_plan_service.md) | R12 (Practice) |
| 🟢 P2 | Web Search | 17 | [cmd__web_search.md](cmd__web_search.md) | R25 (搜索后端) |
| 🟢 P3 | OCR | 14 | [cmd__ocr.md](cmd__ocr.md) | R29 (OCR) |
| 🟢 P3 | Cloud Storage | 14 | [cloud_storage.md](cloud_storage.md) | R27 (云同步) |
| 🟢 P3 | MCP (cmd) | 13 | [cmd__mcp.md](cmd__mcp.md) | R18 (MCP客户端) |
| 🟢 P3 | Anki Connect | 13 | [cmd__anki_connect.md](cmd__anki_connect.md) | R13 (Anki) |
| 🟢 P3 | Textbooks | 11 | [cmd__textbooks.md](cmd__textbooks.md) | R14 (PDF) |
| ✅ P4 | Data Space | 10 | [data_space.md](data_space.md) | R27 (DataGov) |
| ✅ P4 | Debug Commands | 7 | [debug_commands.md](debug_commands.md) | R17b (调试面板) |
| ✅ P4 | Question Sync | 6 | [question_sync_service.md](question_sync_service.md) | R12 (Practice) |
| ✅ P4 | Backup Config | 5 | [backup_config.md](backup_config.md) | R27 (DataGov) |
| ✅ P4 | Secure Store | 4 | [secure_store.md](secure_store.md) | R29 (安全) |
| ✅ P4 | TTS | 3 | [tts.md](tts.md) | R16 (其他) |
| ✅ P4 | Translation | 3 | [translation.md](translation.md) | R15 (翻译) |
| ✅ P4 | Anki Cards | 3 | [cmd__anki_cards.md](cmd__anki_cards.md) | R13 (Anki) |
| ✅ P4 | QBank Grading | 2 | [qbank_grading.md](qbank_grading.md) | R12 (Practice) |
| ✅ P4 | LLM Usage | 2 | [llm_usage.md](llm_usage.md) | R22 (LLM) |
| ✅ P4 | Config Recovery | 2 | [config_recovery.md](config_recovery.md) | R29 (安全) |
| ✅ P4 | Pdfium Utils | 1 | [pdfium_utils.md](pdfium_utils.md) | R14 (PDF) |
| ✅ P4 | Debug Logger | 1 | [debug_logger.md](debug_logger.md) | R17b (调试面板) |
| ✅ P4 | Translation (cmd) | 1 | [cmd__translation.md](cmd__translation.md) | R15 (翻译) |
| ✅ P4 | Anki Connect Svc | 1 | [anki_connect_service.md](anki_connect_service.md) | R13 (Anki) |

## 全局改进统计

| 改进项 | 影响模块数 | 优先级 |
|--------|----------|--------|
| 统一错误类型 (String → ModuleError) | 15 | P1 |
| 输入封装 (参数 → Input struct) | 8 | P2 |
| State 冗余消除 (Context struct) | 2 | P2 |
| 模块拆分 (VFS → 4模块) | 1 | P0 |
| 遗留退役 (commands.rs → cmd/) | 1 | P0 |
| 合并重复模块 | 3 | P3 |
| 命名规范化 | 5 | P3 |
| dev-only 标记 | 2 | P4 |

## 原始数据

每个模块的完整命令列表（参数/返回类型/文档注释）存储在：
```
_data/
├── commands.json
├── vfs.json
├── chat_v2.json
├── dstu.json
├── ...
```

## 与探究报告的对照关系

```
探究报告 (22份)                  API重构报告 (31份)
─────────────────────────────────────────────────
round-08-chat-v2.md         →  chat_v2.md
round-09-learning-hub.md    →  (通过 DSTU/VFS 间接)
round-10-notes.md           →  cmd__notes.md
round-11-mindmap.md         →  (前端无直接后端)
round-12-practice.md        →  review_plan_service.md + qbank_grading.md
round-13-anki-template.md   →  cmd__enhanced_anki.md + cmd__anki_connect.md
round-14-pdf-docx.md        →  cmd__textbooks.md + pdfium_utils.md
round-15-translation.md     →  translation.md + cmd__translation.md
round-16-essay-others.md    →  essay_grading.md + tts.md
round-17a-settings.md       →  cmd__web_search.md (settings部分)
round-18-mcp-client.md      →  cmd__mcp.md
round-19-dstu.md            →  dstu.md
round-20-backend-entry.md   →  commands.md
round-21-chat-v2-backend.md →  chat_v2.md
round-23-vfs.md             →  vfs.md
round-24-26-backend-merged  →  data_governance + cloud_storage.md
round-supplement-complete   →  其余单文件模块
```
