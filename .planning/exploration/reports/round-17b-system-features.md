# Round 17b: 命令面板 + 技能管理 + 调试面板 — 合并诊断报告

**日期**: 2026-05-29
**状态**: ✅ 完成

---

## 命令面板 — 21+4 文件, ~5,200 行

```
src/command-palette/
├── CommandPalette.tsx         453 行
├── CommandPaletteProvider.tsx  295 行
├── registry/commandRegistry.ts 573 行 — 命令注册表
├── modules/
│   ├── chat.commands.ts       463 行
│   ├── global.commands.ts     443 行
│   ├── learning.commands.ts   430 行
│   └── ... (更多模块命令)
├── components/                 UI 组件
│   └── ShortcutSettings.tsx   400 行
└── hooks/                      useCommandEvents 等
```

✅ 基于 cmdk 库。命令注册表模式 — 类似 Chat V2 的插件注册，设计良好
✅ 模块化命令定义 (chat/global/learning 分离)

---

## 技能管理 — 3+5 文件, ~3,700 行

```
src/features/skills-management/ (3 文件, 主要是 re-export)
src/components/skills-management/ (5 文件)
├── SkillsManagementPage.tsx      953 行 — 主页面
├── SkillEditorModal.tsx          696 行 — 技能编辑弹窗
├── SkillFullscreenEditor.tsx     658 行 — 全屏编辑器
├── EmbeddedToolsEditor.tsx       415 行 — 内嵌工具编辑
└── SkillsList.tsx / SkillsSidebar.tsx
```

⚠️ 代码仍在 `components/` 而非 `features/`，与其他模块一致的不完全迁移
⚠️ 3 个编辑器组件功能有重叠 (Modal/Fullscreen/Embedded)

---

## 调试面板 — 59 文件, **28,566 行**

```
src/debug-panel/
├── DebugPanelHost.tsx         1168 行 — 调试面板主容器
├── debugMasterSwitch.ts       — 全局调试总开关
├── plugins/                   数十个调试插件
│   ├── ChatV2TimelinePlugin    — Chat V2 时间线
│   ├── DstuDebugPlugin         — DSTU 协议调试
│   ├── CrepeEditorDebugPlugin  — 编辑器调试
│   ├── ExamSheetProcessingDebugPlugin — 试卷处理调试
│   ├── MultiVariantDebugPlugin — 多变体调试
│   └── ...
├── events/                    调试事件通道
├── hooks/                     调试专用 Hooks
└── services/                  调试服务 (pageLifecycleTracker 等)
```

🔴 调试面板 **28,566 行** — 几乎相当于一个小型应用。这是否超出了"调试"的范畴？

---

## 发现的问题

- [ ] **P2** — 技能管理代码在 `components/skills-management/` 而非 `features/`，延续不完全迁移
- [ ] **P2** — 技能管理 3 个编辑器 (Modal/Fullscreen/Embedded) 功能重叠
- [ ] **P2** — 调试面板 **28,566 行** / 59 文件，规模令人担忧。应评估哪些是真正的调试工具，哪些是变相的功能代码
- [ ] **P3** — 调试面板在 production build 中被排除 (vite exclude-mcp-debug 插件)，但源代码保留在仓库中
- [ ] **P3** — 命令面板的 `features/command-palette/` (4文件) 和 `src/command-palette/` (21文件) 并存，前者可能只是 re-export

---

## 建议优先处理

1. 审计调试面板的 28,566 行代码 — 确认没有被误分类的功能代码
2. 合并 3 个技能编辑器为 1 个可配置的编辑器组件
3. 统一命令面板到 `features/command-palette/` 目录
