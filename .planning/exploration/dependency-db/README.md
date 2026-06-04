# 项目依赖关系数据库 — 实施计划

**创建日期**: 2026-05-29

---

## 目标

建立一个小型 SQLite 数据库，追踪：
- 每个文件的导出（提供了什么）
- 每个文件的导入（依赖了什么）
- 函数/类型的调用关系
- 未被引用的死代码
- 废弃模块的残留引用

## 项目规模

| 语言 | 文件数 | 导入语句 | 导出语句 |
|------|--------|---------|---------|
| TypeScript | 1,485 | ~8,073 | ~6,635 |
| Rust | 396 | ~2,385 (use) | ~3,415 (pub) |
| CSS | 78 | @import | — |

**无法手工完成**。需要脚本辅助 + 数据库存储。

---

## 数据库 Schema 设计

```sql
-- 核心表: 文件清单
CREATE TABLE files (
    id INTEGER PRIMARY KEY,
    path TEXT UNIQUE NOT NULL,       -- src/components/ui/NotionButton.tsx
    language TEXT NOT NULL,          -- typescript | rust | css
    category TEXT,                   -- feature | shared | store | api | ...
    lines INTEGER,
    is_dead INTEGER DEFAULT 0,       -- 0=活跃, 1=确认废弃, 2=可疑
    notes TEXT                       -- 来自诊断报告的备注
);

-- 导出表: 每个文件导出了什么
CREATE TABLE exports (
    id INTEGER PRIMARY KEY,
    file_id INTEGER REFERENCES files(id),
    name TEXT NOT NULL,              -- 导出名称 (NotionButton / useUIStore)
    kind TEXT,                       -- function | class | type | const | enum | default
    is_default INTEGER DEFAULT 0,
    UNIQUE(file_id, name)
);

-- 导入表: 每个文件从哪导入了什么
CREATE TABLE imports (
    id INTEGER PRIMARY KEY,
    importer_file_id INTEGER REFERENCES files(id),   -- 谁在导入
    imported_file_path TEXT NOT NULL,                 -- 从哪导入 (解析后的路径)
    imported_name TEXT,                                -- 导入了什么 (null = 全部/默认)
    is_type_only INTEGER DEFAULT 0,                   -- TypeScript type import
    UNIQUE(importer_file_id, imported_file_path, imported_name)
);

-- 调用关系: 函数/组件级别的引用
CREATE TABLE calls (
    id INTEGER PRIMARY KEY,
    caller_file_id INTEGER REFERENCES files(id),
    callee_name TEXT NOT NULL,        -- 被调用的名称
    callee_file_id INTEGER,           -- 如果解析到了目标文件
    call_site_line INTEGER,           -- 调用发生的行号
    kind TEXT                         -- function_call | jsx_component | hook | type_use
);

-- 诊断发现: 来自探究报告的发现
CREATE TABLE findings (
    id INTEGER PRIMARY KEY,
    finding_id TEXT UNIQUE,           -- P1-01, P2-03 等
    severity TEXT,                    -- P1 | P2 | P3 | P4
    title TEXT,
    description TEXT,
    affected_files TEXT,              -- JSON array of file paths
    round TEXT                        -- 发现轮次
);

-- 视图: 死代码候选
CREATE VIEW dead_code_candidates AS
SELECT f.path, f.language, f.category
FROM files f
LEFT JOIN imports i ON i.imported_file_path LIKE '%' || REPLACE(f.path, 'src/', '') || '%'
WHERE i.id IS NULL
  AND f.category NOT IN ('entry', 'config', 'test');
```

---

## 分阶段执行计划

### Phase 1: 文件清单建立 (1 轮)

**目标**: 填充 `files` 表，建立所有源文件的基础记录。

**方法**: 脚本遍历 + 已有诊断报告的元数据。

| 步骤 | 内容 |
|------|------|
| 1.1 | 遍历 `src/` 和 `src-tauri/src/` 收集所有文件路径 |
| 1.2 | 统计每个文件的行数 |
| 1.3 | 按目录自动分类 (feature/shared/store/api/...) |
| 1.4 | 导入诊断报告中的已知 God File 和死代码标记 |

**产出**: `files` 表完整填充 (~1,881 行)

### Phase 2: 导入关系提取 (2 轮)

**目标**: 填充 `imports` 表，记录每个文件的依赖。

#### Phase 2a: TypeScript 导入提取

**方法**: 使用正则或 ts-morph 解析 import 语句。

| 步骤 | 内容 |
|------|------|
| 2a.1 | 编写 Node.js 脚本，遍历所有 `.ts/.tsx` 文件 |
| 2a.2 | 解析 `import { X, Y } from 'path'` 语句 |
| 2a.3 | 解析 `import X from 'path'` 默认导入 |
| 2a.4 | 解析 `import type { X } from 'path'` 类型导入 |
| 2a.5 | 处理路径别名 (`@/` → `src/`, 相对路径解析) |
| 2a.6 | 处理动态 `import()` 和 `React.lazy(() => import(...))` |

**产出**: ~8,073 条 import 记录

#### Phase 2b: Rust 导入提取

**方法**: 正则解析 `use` 语句。

| 步骤 | 内容 |
|------|------|
| 2b.1 | 解析 `use crate::module::Type;` 语句 |
| 2b.2 | 解析 `use std::...` 和外部 crate (标记为 external) |
| 2b.3 | 处理 `pub use` re-export |

**产出**: ~2,385 条 use 记录

### Phase 3: 导出关系提取 (1 轮)

**目标**: 填充 `exports` 表，记录每个文件提供了什么。

| 步骤 | 内容 |
|------|------|
| 3.1 | TypeScript: 解析 `export const/function/class/type/interface/enum` |
| 3.2 | TypeScript: 解析 `export default` |
| 3.3 | TypeScript: 解析 barrel re-export (`export * from`, `export { X } from`) |
| 3.4 | Rust: 解析 `pub fn/struct/enum/trait/mod/type` |

**产出**: ~6,635 (TS) + ~3,415 (Rust) 条 export 记录

### Phase 4: 交叉分析与死代码检测 (2 轮)

**目标**: 通过导入/导出数据的交叉分析回答问题。

#### Phase 4a: 文件级依赖分析

| 分析 | SQL 查询 |
|------|---------|
| 被依赖最多的文件 | `SELECT imported_file_path, COUNT(*) FROM imports GROUP BY 1 ORDER BY 2 DESC` |
| 无被依赖的文件 | `LEFT JOIN imports WHERE NULL → 死代码候选` |
| 循环依赖检测 | 构建有向图 → Tarjan SCC 算法 |
| 模块边界违规 | `imports WHERE importer IS feature AND imported IS NOT (shared/lib/tokens)` |

#### Phase 4b: 与诊断报告交叉验证

| 验证项 | 方法 |
|--------|------|
| 确认 MistakeItem 的 4 个引用方 | 从 `exports` 找到 `MistakeItem` → 从 `imports` 追溯到导入方 |
| 确认 cn() 75:2 比例 | 从 `imports` 统计 `@/lib/utils` vs `@/utils/cn` 的导入方 |
| 确认空壳 feature 目录 | 从 `imports` 验证 `features/practice/` 和 `features/template-management/` 是否被导入 |
| 确认 API 层绕过 | 检查 Store 文件是否直接导入 `invoke` 而非通过 `api/` |

### Phase 5: 函数级调用图 (3 轮)

**目标**: 最深层的分析 — 哪些函数调用了哪些函数。

**方法**: 这个需要更精细的解析:
- TypeScript: ts-morph AST 或 tree-sitter
- Rust: syn crate 或简单的正则

| 步骤 | 内容 |
|------|------|
| 5.1 | 对关键模块 (Chat V2, VFS, LLM Manager) 优先建立调用图 |
| 5.2 | 追踪 TauriAdapter 4104 行中的函数调用链 |
| 5.3 | 追踪 Chat V2 Store 的 18 个文件间调用 |
| 5.4 | 建立关键数据流: send_message → pipeline → tool_loop → executor |

### Phase 6: 查询接口与报告 (1 轮)

**目标**: 提供可用的查询界面。

| 步骤 | 内容 |
|------|------|
| 6.1 | 编写常用查询脚本 (死代码列表、循环依赖、违规引用) |
| 6.2 | 生成 Markdown 格式的依赖报告 |
| 6.3 | 与 .planning/exploration/reports/ 目录对接 |

---

## 技术方案选择

### TypeScript 导入解析

```javascript
// 方案 A: 正则 (简单但不完美)
const importRegex = /import\s+(?:type\s+)?(?:\{[^}]*\}|[\w*]+)\s+from\s+['"]([^'"]+)['"]/g;

// 方案 B: ts-morph (精确但重)
import { Project } from 'ts-morph';
const project = new Project({ tsConfigFilePath: 'tsconfig.json' });

// 方案 C: madge (中间方案, 推荐先尝试)
// npx madge --image graph.png src/
```

### 数据库选择

**SQLite** — 项目已在用 (rusqlite)，轻量，无需额外服务。
- 脚本用 Node.js (`better-sqlite3`) 写入
- 查询直接 SQL，可通过命令行或简单 HTML 页面浏览

---

## 执行策略

考虑到上下文限制，建议：

1. **Phase 1** — 手动 + 脚本混合，利用已有扫描数据快速建立 `files` 表
2. **Phase 2-3** — 编写独立 Node.js 脚本 (不在主项目 package.json 中)，分次执行
3. **Phase 4** — SQL 分析，可在数据库中直接查询
4. **Phase 5** — 按需深入，先覆盖已识别的高风险模块

**数据库位置**: `.planning/dependency-db/deps.db`
**脚本位置**: `.planning/dependency-db/scripts/`

---

## 预期产出

| 产出 | 描述 |
|------|------|
| `deps.db` | SQLite 数据库 |
| `reports/dead-code.md` | 死代码清单 |
| `reports/dependency-graph.md` | 模块间依赖关系 |
| `reports/circular-deps.md` | 循环依赖检测 |
| `reports/barrel-analysis.md` | Barrel export 完整性 |
| `reports/unused-exports.md` | 导出但未被引用的符号 |
| `scripts/` | 可复用的提取/分析脚本 |

---

## 预计轮次: 10 轮

| Phase | 轮次 | 内容 |
|-------|------|------|
| P1 | 1 | 文件清单 + files 表 |
| P2a | 1 | TS 导入提取 |
| P2b | 1 | Rust 导入提取 |
| P3 | 1 | 导出提取 |
| P4a | 1 | 文件级交叉分析 |
| P4b | 1 | 与诊断报告交叉验证 |
| P5 | 3 | 关键模块函数调用图 |
| P6 | 1 | 查询接口与报告 |

---

*下一步: 确认方案后开始 Phase 1。*
