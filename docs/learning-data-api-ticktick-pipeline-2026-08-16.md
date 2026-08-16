# DeepStudent 学习数据接口 + 日程规划管道 + 滴答清单集成 设计文档

> 状态: 待实施 (P0 未开工) | 创建: 2026-08-16 11:59 CST | 分支: pr/3-pdf-reading
> 前置决策: 用户已确认产品方向 —— DeepStudent 转型"学习数据中台", 通用日程外包滴答清单(用户已有会员)

---

## 1. 背景与定位

### 1.1 产品判断(已确认)

DeepStudent 内置日程(todo/番茄钟/复习计划)是学习闭环的附属品; 滴答清单是专业 GTD 产品,
多端同步/提醒/日历视图能力远超自建价值。正确分工:

- **DeepStudent 做深**: 错题→遗忘曲线复习、记忆库语义检索、学习资源定位、专注统计
  (依赖 VFS/memory/mistakes 三个库, 滴答永远做不了)
- **滴答清单承接**: 任务呈现、多端提醒、日历视图、跨设备同步(纯通用能力, 不自研)

### 1.2 本文档范围

设计"DeepStudent 只读数据接口 → deepseek-harness 规划 → 滴答清单写入"完整管道,
含接口规范、安全模型、状态回流协议、阶段计划。**P0 只读, 写接口(除 P3 回流)一律不做。**

---

## 2. 目标 / 非目标

### 目标

1. DeepStudent 内置 localhost 只读 HTTP JSON API, 暴露学习素材(记忆/错题/资源/计划/日程占用)
2. deepseek-harness 可配置 HTTP 工具直接调用, 无需改 harness 源码
3. 规划结果经滴答清单 Open API 写入, 幂等防重复
4. 预留状态回流协议: 滴答任务完成状态可回映射 DeepStudent 复习记录

### 非目标(明确排除)

- ❌ 不做 DeepStudent 日程模块的功能扩充(保留为"最小可用兜底")
- ❌ P0 不做任何写接口(模型不能改 DeepStudent 数据)
- ❌ 不做 MCP Server(P2 再评估, HTTP 已够用)
- ❌ 不做滴答清单客户端功能(不写滴答 UI)

---

## 3. 总体架构

```
┌─────────────────────────────────────────────────────────┐
│ DeepStudent (Tauri, 本机常驻)                            │
│  ┌───────────────────────────────────────────────────┐  │
│  │ LocalApiServer  127.0.0.1:59322  (hyper, 只读)     │  │
│  │  ├─ memory::service::search      (语义搜学习记忆)   │  │
│  │  ├─ dstu mistakes repo           (错题查询)        │  │
│  │  ├─ vfs file/folder/note_repo    (学习资源)        │  │
│  │  ├─ review_plan_service          (复习计划)        │  │
│  │  └─ vfs todo_repo/pomodoro       (日程占用)        │  │
│  └───────────────────────────────────────────────────┘  │
└──────────────────────▲──────────────────────────────────┘
                       │ HTTP + Bearer token (仅 loopback)
┌──────────────────────┴──────────────────────────────────┐
│ deepseek-harness (规划器)                                │
│  1. 调 DeepStudent API 抽学习素材                        │
│  2. 模型生成日程规划(含优先级/时长/依据)                 │
│  3. 调滴答 Open API 写入任务                             │
└──────────────────────┬──────────────────────────────────┘
                       │ HTTPS
              ┌────────▼────────┐
              │  滴答清单云服务  │
              └────────┬────────┘
                       │ (P3) 定时回拉已完成任务
┌──────────────────────▼──────────────────────────────────┐
│ DeepStudent 回流同步器: 完成状态 → 复习记录/统计          │
└─────────────────────────────────────────────────────────┘
```

---

## 4. P0: DeepStudent 只读数据 API

### 4.1 服务形态

- **实现**: 新建 `src-tauri/src/local_api/mod.rs`, 复用 `hyper` + `tokio`(均已 vendored),
  服务模式照抄 `metrics_server.rs`(127.0.0.1 绑定守卫、端口管理、启动/停止)
- **绑定**: 仅 `127.0.0.1`, 端口 `59322`(与 metrics 59321 相邻); 拒绝非 loopback 地址
- **生命周期**: 随 app 启动(设置项开启时), 不单独驻留
- **设置项**: 设置 → 数据治理/集成 新增 "本地数据接口" 开关(默认**关**),
  开启后显示 token(可复制)与端点列表

### 4.2 认证

- 首次开启时生成 32 字节随机 token, 写入 `<app_data>/local-api-token`(权限 0600),
  同步存入 secure_store
- 所有请求(除 `/health`)要求 `Authorization: Bearer <token>`, 失败返回 401
- token 可在设置页"重新生成"(旧 token 立即失效)

### 4.3 全局守卫

每个请求处理前执行:

1. `is_in_maintenance_mode()` → true 返回 `503 {"error":{"code":"MAINTENANCE"}}`
   (备份/恢复期间 DB 切内存态, 绝不返回假数据)
2. 只读: 仅注册 GET 路由, 其他方法返回 405
3. 统一超时 8s, 统一响应 JSON

### 4.4 端点规范

通用错误模型:
```json
{ "error": { "code": "STRING_CODE", "message": "人类可读描述" } }
```
错误码: `MAINTENANCE`(503) / `UNAUTHORIZED`(401) / `INVALID_PARAMS`(400) /
`NOT_FOUND`(404) / `TIMEOUT`(504) / `INTERNAL`(500)

#### 4.4.1 `GET /api/v1/health`

探活(免 token, 供 harness 启动检测)。
```json
{ "ok": true, "version": "0.9.40", "maintenance": false, "time": "2026-08-16T12:00:00+08:00" }
```

#### 4.4.2 `GET /api/v1/memory/search`

语义搜索学习记忆。
- 参数: `q`(必填, ≤500字), `top_k`(默认 8, ≤20), `category`(可选)
- 底层: `memory::service::search_with_rerank`(service.rs:1825)
- 响应:
```json
{ "results": [ { "id": "...", "content": "...", "category": "...", "score": 0.83,
                 "updated_at": "...", "source": "..." } ], "took_ms": 320 }
```
- 超时 5s(embedding 重排序在大数据量下可能秒级, 超限返回已得部分 + `"truncated": true`)

#### 4.4.3 `GET /api/v1/mistakes`

错题查询(规划"待巩固"素材的核心源)。
- 参数: `subject`(可选), `since`(可选, ISO 日期), `until`(可选),
  `limit`(默认 20, ≤50), `offset`(默认 0), `order`(默认 `recent`, 可选 `review_due`)
- 底层: dstu mistakes 查询(mistakes.db, 已有热查询索引 `V20260208__add_hot_query_indexes`)
- 响应:
```json
{ "items": [ { "id": "...", "subject": "数学", "title": "...", "summary": "...",
               "tags": ["极限"], "created_at": "...", "review_state": "...",
               "next_review_at": "..." } ],
  "total": 137, "limit": 20, "offset": 0 }
```

#### 4.4.4 `GET /api/v1/vfs/resources`

学习资源列表/检索(教材/笔记/文件)。
- 参数: `folder_id` 或 `path`(可选, 默认根), `q`(可选名称搜索),
  `type`(可选 file/note/folder), `limit`(默认 50, ≤100)
- 底层: `vfs::repos::{file_repo, folder_repo, note_repo}`
- 响应:
```json
{ "items": [ { "id": "...", "name": "数据结构C语言版.pdf", "type": "file",
               "path": "/教材/数据结构", "size": 13505804, "mime": "application/pdf",
               "updated_at": "..." } ] }
```
- 不返回文件内容(体积与安全), 仅元数据; 内容提取后续单独立项

#### 4.4.5 `GET /api/v1/review/plans`

现有复习计划(查重与衔接)。
- 参数: `status`(可选 active/done), `limit`(默认 20)
- 底层: `review_plan_service`(17 个命令层) + `vfs::repos::review_plan_repo`
- 响应:
```json
{ "plans": [ { "id": "...", "title": "...", "exam_id": "...", "status": "active",
               "start_date": "...", "end_date": "...", "daily_load": 30,
               "progress": 0.45 } ] }
```

#### 4.4.6 `GET /api/v1/schedule/busy`

日程占用(避免与已有安排冲突)。
- 参数: `date`(必填, YYYY-MM-DD) 或 `from`/`to`
- 底层: `vfs::repos::todo_repo`(todo 清单) + 番茄钟记录
- 响应:
```json
{ "date": "2026-08-17",
  "busy": [ { "kind": "todo", "title": "...", "start": "09:00", "end": "10:00", "source_list": "..." } ],
  "free_hours_estimate": 5.5 }
```

### 4.5 配置与注册

- `lib.rs` setup 中: `#[cfg(feature = "local_api")]`(新 feature, 默认启用)
  读取设置 → 开启则 `local_api::start(app_handle, state)`
- invoke 命令: `local_api_get_status` / `local_api_set_enabled` /
  `local_api_regenerate_token`(供设置页调用)

---

## 5. harness 集成(deepseek-harness)

### 5.1 HTTP 工具配置(示例)

```yaml
tools:
  - name: deepstudent
    type: http
    base_url: http://127.0.0.1:59322/api/v1
    headers:
      Authorization: "Bearer <从设置页复制的 token>"
    endpoints:
      - { name: search_memory, path: /memory/search, params: [q, top_k] }
      - { name: list_mistakes, path: /mistakes, params: [subject, since, limit] }
      - { name: list_resources, path: /vfs/resources, params: [path, q] }
      - { name: list_plans, path: /review/plans }
      - { name: get_busy, path: /schedule/busy, params: [date] }
```

### 5.2 规划 Prompt 模板(要点)

1. **角色**: 学习计划编排器, 输出必须是机器可解析 JSON(禁止散文)
2. **输入槽位**: search_memory(近期薄弱点) + list_mistakes(待巩固) +
   list_plans(已有计划查重) + get_busy(未来 7 天空闲)
3. **规划规则**: 错题优先(遗忘曲线到期者优先级最高)、单日学习负荷上限、
   资源必须引用真实存在的 vfs 资源 id(防幻觉)
4. **输出 schema**:
```json
{ "tasks": [ { "title": "复习: 极限错题 5 道", "date": "2026-08-17",
               "start": "19:00", "duration_min": 40, "priority": 3,
               "source": { "kind": "mistake", "ids": ["..."] },
               "dedup_key": "mistake:2026-08-17:极限" } ] }
```

---

## 6. 滴答清单写入

- **API**: 官方 Open API `https://api.dida365.com/open/v1`(会员账号已就绪)
- **授权**: OAuth2 Authorization Code; token 存 harness 侧(不归 DeepStudent 管)
- **建任务**: `POST /open/v1/task`
  - 字段映射: title/date/duration/priority(0/1/3/5) ← 规划输出
  - **幂等**: 以 `dedup_key` 写入任务 `desc` 首部标记 `[ds:dedup_key]`,
    写入前先 `GET /open/v1/project/{id}/data` 查重, 存在同 key 未完成任务则跳过/更新
- **限流**: 官方有频率限制; harness 侧串行 + 429 退避(≤10 任务/批)
- **清单归属**: 固定写入"学习规划"清单(首次自动创建), 不污染收件箱

---

## 7. 状态回流协议(P3 预留, 现在定稿)

**问题**: 用户在滴答完成任务, 但 DeepStudent 复习记录没更新 → 学习统计失真。

**协议**:
1. 规划输出阶段即为每个任务生成 `dedup_key`(含素材 ids), 写入滴答任务 desc
2. 回流同步器(harness 侧定时任务, 或 DeepStudent 后台 loop):
   每 30 分钟 `GET /open/v1/project/{id}/data`, 筛出近 24h 完成且含 `[ds:...]` 标记的任务
3. 映射: `dedup_key` 解析素材 ids → 调 DeepStudent(此时需要 P3 写接口):
   - `POST /api/v1/review/record`(错题标记已复习 + 更新遗忘曲线)
   - 冲突策略: 已存在当天复习记录则合并(不累加时长), 以 DeepStudent 记录为准
4. 回流状态记入审计日志(data_governance audit)

---

## 8. 安全与隐私

| 项 | 措施 |
|----|------|
| 网络面 | 仅 127.0.0.1; 拒绝非 loopback 绑定; 不开端口转发 |
| 认证 | Bearer token(0600 文件 + secure_store); 可重新生成 |
| 权限 | P0 仅 GET; 写接口 P3 单独评审 |
| 数据出境 | 抽出的学习记录会发给云端模型 —— 设置页开关旁明示, 默认关闭 |
| 日志 | API 访问日志不落查询内容(只记端点/耗时/状态码), 防学习记录泄漏到日志 |
| 限流 | 每端点 10 req/s 令牌桶, 防失控循环调用 |

---

## 9. 阶段计划与验收

| 阶段 | 内容 | 验收标准 | 工作量 |
|------|------|----------|--------|
| P0 | local_api 模块 + 6 端点 + token + 设置页开关 | curl 冒烟 6/6 通过; maintenance 态返回 503; 非 loopback 不可达 | 1.5-2 天 |
| P1 | harness 配置打通 + 规划 prompt + 滴答 OAuth + 幂等写入 | 端到端: 抽素材→生成 JSON→滴答出现任务; 重复执行不产生重复任务 | 0.5-1 天 |
| P2 | (评估) 包装 MCP Server | harness 原生 tools 调用成功 | 1-2 天(可选) |
| P3 | 回流同步器 + 写接口(评审) + 审计 | 滴答完成任务 30 分钟内反映到复习记录; 无重复累计 | 1 天+评审 |

---

## 10. 测试计划

- **单元**: 每端点参数校验(缺 q/非法 date/超限 limit)、maintenance 503、401
- **集成**: 启动 app → curl 6 端点 → 校验 schema; 大库(用户现状: 数千记忆/错题)下
  memory/search ≤5s
- **端到端(P1)**: 固定素材快照 → 规划输出 JSON schema 校验 → 滴答沙箱清单写入/查重
- **回归**: 桌面打包后设置页开关/ token 重新生成生效; Android/iOS 上 API 可同构启用
  (移动端仅 loopback, 同样安全)

---

## 11. 风险与缓解

| 风险 | 缓解 |
|------|------|
| embedding 搜索慢查询拖住规划 | top_k≤20 + 5s 超时 + truncated 标记 |
| maintenance 态返回假数据 | 全局守卫强制 503 |
| 滴答 API 限额/429 | 串行写入 + 退避 + 批量≤10 |
| 模型幻觉编造不存在的资源 id | prompt 强制引用 API 返回的 id; harness 侧校验 id 存在性(回调 vfs/resources 验证) |
| 重复任务污染滴答 | dedup_key + 写前查重(强制) |
| 端口冲突(59322 被占) | 启动时探测, 冲突则顺延至 59323-59331 并写入状态 |

---

## 12. 待决策问题(开工前确认)

1. ✅ ~~产品方向~~(已确认: 学习中台 + 滴答外包)
2. token 展示方式: 设置页**仅复制按钮**还是明文显示?(建议: 复制按钮 + "已复制"提示, 不明文)
3. 规划输出的资源引用粒度: 到文件级还是笔记/章节级?(建议 P0 文件级, 后续细化)
4. 滴答清单名: 固定"学习规划"还是可配置?(建议: 可配置, 默认"学习规划")
5. P2 MCP Server 是否做: 取决于 harness 对 MCP tools 的原生支持程度(开工前验证一次)
