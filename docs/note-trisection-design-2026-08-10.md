# 笔记三分域设计: OCR 识别 / 记忆 / 普通笔记 的 md 区分规则

> 日期: 2026-08-10 13:41 CST | 分支: pr/3-pdf-reading
> 背景: 三种 md 笔记混在"全部笔记"一个分类。实测用户库: 1726 条笔记中 1577 条 (91.6%) 是 OCR 逐页笔记 ("第 N 页"), 真笔记仅 ~141 条。

---

## 1. 区分规则 (数据级判定)

| 类别 | 判定规则 | 依据 |
|------|----------|------|
| **OCR 笔记** | `notes.tags` JSON 数组含 `"ocr"` | 唯一创建点 `pdf_processing_service.rs:3930-3940`: 逐页笔记固定写入 `["ocr","auto-generated","source:{file_id}","page:{N}","folder:{fld}"]`; 同时挂载到 `<文件名>.pdf (OCR)` 文件夹 |
| **记忆笔记** | 笔记挂载在记忆根文件夹子树内 | `memory_config.memory_root_folder_id` 指定根, `folders.parent_id` 递归子树 + `folder_items.item_type='note'` |
| **普通笔记** | 以上两者之外 | 排除法 |

### 规则稳定性论证

- OCR 标签由后端唯一点写入, 不依赖标题模式 (兼容未来标题格式变化);
- 记忆判定走文件夹归属 (记忆系统的既有契约), 不依赖标题/标签约定;
- 两规则正交: OCR 笔记不可能在记忆子树 (创建时固定挂到 PDF 专属文件夹), 记忆笔记无 ocr 标签。

### SQL 实现

- OCR: `n.tags LIKE '%"ocr"%'` (带引号精确匹配数组成员, 不误伤 `"ocrfoo"`)
- 记忆子树: `WITH RECURSIVE sub(id) AS (root UNION folders.parent_id 递归) → folder_items` 收 note id 集
- 普通: `NOT LIKE '%"ocr"%' AND id NOT IN (记忆集)`

## 2. 接口设计

`DstuListOptions` 新增 `noteScope` (前后端同步):

| 值 | 语义 | 用途 |
|----|------|------|
| `'normal'` | 普通笔记 (排除 OCR + 记忆) | "全部笔记" 分类默认 |
| `'ocr'` | 仅 OCR 页笔记 | 新增 "OCR 识别" 分类 |
| 缺省 | 全部 (兼容旧行为) | 第三方/旧调用不受影响 |

- 后端: `dstu/types.rs` `note_scope` 字段 + `VfsNoteRepo::list_notes_scoped(_with_conn)` (新函数, 旧 `list_notes` 不动)
- 分发点: `dstu/handler_utils/list_helpers.rs` Note 臂 (智能文件夹模式)

## 3. 导航设计

```
资源类型
├── 全部笔记   → typeFilter=note, noteScope=normal   (不再混入 OCR/记忆)
├── OCR 识别   → typeFilter=note, noteScope=ocr      (★ 新增, 粉色扫描框图标)
├── 全部教材 ...
系统
└── 记忆管理   → 既有入口 (记忆根文件夹/MemoryView)
```

- `QuickAccessType` 新增 `'ocrNotes'`; `QUICK_ACCESS_TARGETS`/`getQuickAccessTypeFromPath`/别名表同步
- `FinderPath.noteScope` 持久化进导航历史; `getFinderPathDisplayPath` 对 ocr 视图返回 `/@notes/ocr` (normal 保持 `/` 旧值), 保证加载 effect 在两个分类间切换时重新触发
- 移动端启动器 (DstuAppLauncher) 同步新增

## 4. 影响面与兼容

| 场景 | 行为变化 |
|------|----------|
| 全部笔记分类 | 只显示普通笔记 (**用户的 1706 → ~141**) |
| OCR 识别分类 | 显示 1577+ 页笔记, 按更新时间排序 |
| 聊天右侧资源面板 (canvas) | 同走 notes=normal, 注入选择器不再被 OCR 页淹没 |
| 文件夹视图 | 不变 (OCR 页仍在各自 PDF 文件夹内可见) |
| 搜索 | dstu.search 未加 scope, 全局搜索仍覆盖全部笔记 |
| 收藏/回收站 | 不变 |

## 5. 遗留事项

- `LearningHubSidebarV2.tsx` 确认零引用 (死代码), 未删除, 建议下批清理
- 页头标题在 OCR 视图可能回退显示"全部笔记" (装饰性, t() 有默认值)
- OCR 页笔记打开后仍走笔记编辑器 — 如需只读/定位源 PDF, 另开任务

## 6. 复核发现的规则脆弱点 (2026-08-10 18:46 CST)

- **"ocr" 标签是用户可写数据**: 笔记编辑器暴露标签编辑 (`NoteContentView.tsx:327 handleTagsChange` / `NoteTagsEditor`)。用户给普通笔记加 `ocr` 标签 → 该笔记从"全部笔记"消失、出现在"OCR 识别"; 用户清理 OCR 页笔记的自动标签 → 页笔记回流"全部笔记"。分类标记放在用户可编辑字段上, 长期看需要系统级标记 (如 notes 表加 `origin` 列) 才能彻底防逃逸
- **"全部笔记"标签语义漂移**: 该分类现在只显示 normal 笔记, 名不副实, 后续宜改名"笔记"/"普通笔记"
- **记忆子树规则的副作用**: 用户手动把笔记移入记忆文件夹子树 → 从"全部笔记"静默消失 (契约内行为, 但需知晓)
