# OCR 全链路审计

> 2026-05-30 12:20 CST | 追踪: 前端 → 后端 → 存储

## 1. 调用入口

### 入口 A: 分析模式 OCR (standalone)
```
前端: analysis.ts::performOcr(images: string[])
  → invoke('chat_v2_perform_ocr', { request: { images: base64[] } })
    → 后端: chat_v2::handlers::ocr::chat_v2_perform_ocr()
      → 系统原生 OCR 或 VLM 云端 OCR
      → 返回 OcrResponse { ocr_text, tags, mistake_type }
```

**文件处理**: 图片仅解码到内存，处理后丢弃。**无永久存储**。注释明确: "只执行 OCR，不创建会话或保存图片"。

### 入口 B: PDF OCR 处理
```
前端: 上传 PDF → invoke('init_pdf_ocr_session')
  → 后端: pdf_ocr_service.rs
    → PDF → 分页渲染为 JPG → 缓存到 {app_data}/pdf_ocr_cache/
    → 逐页 OCR → 返回文本
    → load_cached_blocks() 检测缓存避免重复处理
```

**文件存储**: `{app_data_dir}/pdf_ocr_cache/` — 页面图像缓存，有 `enforce_cache_budget` 自动清理。

### 入口 C: VFS OCR (resource attachment)
```
前端: 上传图片到聊天 → invoke('vfs_resource_*')
  → 后端: VFS handlers → VfsBlobRepo
    → 图片存入 VFS blob 存储: {app_data}/vfs/blobs/
    → 内容哈希去重: 相同文件不会重复存储
```

**文件存储**: VFS blob 目录，用 SHA256 内容哈希去重。

## 2. 存储总结

| OCR 类型 | 输入文件存储 | OCR 结果存储 | 重复检测 |
|----------|------------|-------------|---------|
| 分析模式 | ❌ 内存临时 | ❌ 仅返回前端 | ❌ 无 |
| PDF OCR | 页面缓存 (磁盘) | ❌ 不达文本 | ✅ `load_cached_blocks` |
| VFS 附件 | ✅ blob 存储 | ✅ 数据库 | ✅ SHA256 哈希 |

## 3. 重复上传分析

### 分析模式: ⚠️ 存在重复
- 每次调用都重新 base64→解码→OCR
- 相同图片多次上传会重复处理
- **建议**: 添加 base64 内容哈希缓存

### PDF OCR: ✅ 已有缓存
- `load_cached_blocks(cache_dir, page_index)` 检测已处理页面
- `enforce_cache_budget` 清理旧缓存

### VFS 附件: ✅ 已有去重
- `VfsBlobRepo` 使用 SHA256 哈希
- 相同内容只存储一次

## 4. 数据流图

```
 ┌─────────────────────────────────────────────────────┐
 │  前端分析模式 (analysis.ts)                          │
 │    .selectFiles() → 本地文件                          │
 │    .toBase64() → 编码                                 │
 │    .performOcr([base64]) → invoke()                  │
 ├─────────────────────────────────────────────────────┤
 │  后端 OCR handler (chat_v2::handlers::ocr.rs)       │
 │    parse_base64_image() → 内存解码                    │
 │    ┌─ 系统原生 → SystemOcrAdapter                    │
 │    └─ VLM 云端 → call_ocr_model_raw_prompt()         │
 │    OcrResponse { ocr_text } → 返回前端               │
 │    ⚠️ 图片丢弃, 无缓存                                │
 ├─────────────────────────────────────────────────────┤
 │  PDF OCR (pdf_ocr_service.rs)                       │
 │    PDF → 分页渲染 JPG → {app_data}/pdf_ocr_cache/   │
 │    ✅ load_cached_blocks → 避免重复 OCR               │
 ├─────────────────────────────────────────────────────┤
 │  VFS 附件 (vfs/handlers.rs)                         │
 │    图片 → SHA256 哈希 → blob 存储                     │
 │    ✅ 内容哈希去重                                    │
 └─────────────────────────────────────────────────────┘
```

## 5. 建议修复

### 问题 1: 分析模式无缓存
相同图片重复上传会重复 OCR 处理。

**修复方案**: 在 `chat_v2_perform_ocr` 中添加 base64 内容哈希缓存:
```
base64_hash = sha256(base64_data)
if cache.get(base64_hash):
    return cached_result
result = perform_ocr(base64_data)
cache.set(base64_hash, result)
return result
```

### 问题 2: OCR 结果无持久化
分析模式的 OCR 结果仅保存在前端内存中，刷新后丢失。

**现有解决**: 分析模式将 OCR 结果嵌入聊天消息 content 中，消息本身保存到 ChatV2 数据库。
