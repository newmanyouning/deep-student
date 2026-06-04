# Round 17a: 设置模块 — 诊断报告

**日期**: 2026-05-29
**状态**: ✅ 完成

---

## 模块规模: 81 文件, 33,419 行

```
src/features/settings/
├── components/              主体 (70+ 文件)
│   ├── data-governance/     8 文件 — 审计/备份/迁移/同步/概览/会话归档
│   ├── GeneralTab.tsx       通用设置 Tab
│   ├── AppearanceTab.tsx    外观设置 Tab
│   ├── ApisTab.tsx          API 配置 Tab
│   ├── ExternalSearchTab.tsx 搜索引擎 Tab
│   ├── AboutTab.tsx         关于 Tab
│   ├── Settings.tsx         1730 行 — 主设置容器
│   ├── McpToolsSection.tsx   2247 行 🔴 — MCP 工具管理 (最大文件)
│   ├── McpEditorSection.tsx  1878 行 🔴 — MCP 编辑器
│   ├── ShadApiEditModal.tsx  1827 行 🔴 — API 编辑弹窗
│   ├── DataGovernanceDashboard.tsx 1644 行 🟡
│   ├── SiliconFlowSection.tsx      1164 行 🟡
│   ├── CloudStorageSection.tsx     1092 行 🟡
│   ├── BackupTab.tsx               1129 行 🟡
│   ├── EngineSettingsSection.tsx   624 行
│   ├── useSettingsVendorState.tsx  960 行 — 供应商状态 Hook
│   ├── VoiceInputSettingsSection.tsx 899 行
│   └── ...
├── hooks/                  设置专用 Hooks
└── styles/                 设置页样式
```

### 关键指标

| 文件 | 行数 | 评级 |
|------|------|------|
| `McpToolsSection.tsx` | **2247** | 🔴 MCP 工具 CRUD 全在一个文件 |
| `McpEditorSection.tsx` | **1878** | 🔴 MCP 配置编辑器 |
| `ShadApiEditModal.tsx` | **1827** | 🔴 API 编辑弹窗 |
| `Settings.tsx` | **1730** | 🟡 主容器 |
| `DataGovernanceDashboard.tsx` | **1644** | 🟡 数据治理仪表板 |
| 其他 >1000行 | 3 个 | 🟡 |

---

## 架构

设置页面采用 **Tab 式组织**:
```
Settings (容器, 1730行)
├── GeneralTab     — 通用设置
├── AppearanceTab  — 主题/字体/字号
├── ApisTab        — API 供应商管理
├── ExternalSearchTab — 搜索引擎配置
├── AboutTab       — 关于信息
└── DataGovernanceDashboard → 数据治理子Tab
    ├── OverviewTab / BackupTab / SyncTab
    ├── AuditTab / MigrationTab
    └── RecordConflictsPanel / SyncIndicator
```

---

## 发现的问题

- [ ] **P1** — `McpToolsSection.tsx` (2247行) + `McpEditorSection.tsx` (1878行) + `ShadApiEditModal.tsx` (1827行) = **5952 行 MCP/API 管理代码**全在设置模块中。这是 MCP 管理 UI，应该放在 `features/skills-management/` 或独立的 `mcp/` 管理模块
- [ ] **P2** — 设置模块含 **6 个超过 1000 行**的文件，是 God Component 密度最高的模块
- [ ] **P2** — `useSettingsVendorState.tsx` (960行) 是 Hook 但实为 .tsx（含 JSX），命名误导
- [ ] **P3** — 数据治理的管理 UI (1644+1129+...) 放在设置模块中合理，但每个 Tab 都可以独立

---

## 建议优先处理

1. 将 MCP 管理 UI (McpToolsSection + McpEditorSection) 从设置模块移出到独立的 MCP 管理区域
2. 拆分 ShadApiEditModal (1827行) — 将表单校验/字段渲染/测试连接提取为子组件
3. `useSettingsVendorState.tsx` 重命名为 `useSettingsVendorState.ts`（如果确实无 JSX）
