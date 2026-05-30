# REF-012: Tauri 命令命名标准化 — ✅ 审计完成

> 完成: 2026-05-30 10:17 CST | 总耗时: ~4min | 状态: 审计报告, 需编译后执行重命名

## 资源盘点

| 指标 | 数值 |
|------|------|
| 总命令数 | 689 |
| 模块数 | 31 |
| 已合规 (module_action) | ~440 (64%) |
| 需重命名 | ~50 (7%, cmd/ 子模块) |
| 遗留 (commands.rs) | ~200 (29%, 低优先级) |

## 审计结果

### 已合规模块 ✅
vfs(94), dstu(54), chat_v2(47), qbank(40), notes(34), memory(27), essay(20), todo(20), workspace(18), review(17), cloud(14), textbooks(11), canvas(4), skill(5), pomodoro(5)

### 需重命名: cmd/ 子模块 (~50 命令)
所有命令在 `cmd/` 目录下，已按功能拆分但前缀不统一:

| 模块 | 命令数 | 当前前缀 | 建议前缀 |
|------|--------|----------|----------|
| cmd__enhanced_anki | 12 | get_/delete_/export_/list_/update_ | anki_ |
| cmd__ocr | 9 | get_/save_/set_/update_/test_ | ocr_ |
| cmd__web_search | 10 | get_/delete_/save_/test_/update_ | web_search_ |
| cmd__mcp | 8 | get_/save_/test_ | mcp_ |
| cmd__anki_connect | 7 | get_/export_/import_/save_/check_ | anki_connect_ |
| cmd__anki_cards | 2 | get_/save_ | anki_ |
| cmd__textbooks | 11 | textbook_* (已合规) | — |
| cmd__notes | 34 | notes_* (已合规) | — |
| cmd__translation | 1 | (已合规) | — |

### 遗留低优先级
commands.rs 中 ~200 命令使用通用前缀 (get/save/test/export/import/check/delete/debug/set/update/clear/list)。这些命令分散在传统架构中，重命名需协调前端 invoke() 调用，建议在 Chat V2 完全替代旧架构后统一处理。

## 执行计划 (需编译验证)

### Phase 1: 单模块试点 (cmd__mcp, 8命令)
1. 后端: 重命名函数 + 更新 lib.rs 注册
2. 前端: 更新 invoke() 调用
3. 编译验证

### Phase 2: 批量迁移 (cmd__ocr + cmd__web_search + cmd__anki_cards, 21命令)

### Phase 3: 收尾 (cmd__enhanced_anki + cmd__anki_connect, 19命令)

> ⚠️ 注意: 命令重命名是**破坏性变更**，必须前后端同步更新。建议在有编译环境时执行。
