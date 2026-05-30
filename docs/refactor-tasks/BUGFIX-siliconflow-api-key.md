# BUGFIX: 硅基流动 API Key 无法保存/读取

> 发现: 2026-05-30 11:50 CST | 状态: 已修复

> 更新: 2026-05-30 12:05 CST — 扩展诊断到全部供应商

## 根因分析

### 问题 1: 硅基流动独立存储路径 ✗
硅基流动 API Key 无法保存/读取。前端缺失保存/删除按钮。

### 问题 2: 其他供应商按钮不反应 (已诊断)
非 bug，是 UX 时序问题。`VendorApiKeySection` 在 vendor.apiKey 首次加载完成前不点亮按钮。保存路径正确。

## 根本原因

### 根因：双存储路径不一致

```
正常供应商 (OpenAI/DeepSeek):
  Frontend → configApi.saveVendorConfigs() → invoke('save_vendor_configs')
    → Rust LLMManager::save_vendor_configs()
      → SecureStore::save_secret("builtin-{vendor}.api_key")  ✅ 加密存储
      → web_search_save_setting("vendor_configs", json)       ✅ 配置JSON

硅基流动 (异常路径):
  Frontend → TauriAPI.saveSetting("builtin-siliconflow.api_key")
    → Rust web_search_save_setting("builtin-siliconflow.api_key")  ❌ 明文存储
    → 这个路径只写入 settings 键值表，不写入 SecureStore
    → LLMManager::get_vendor_configs() 从 SecureStore 读取 → 永远为空！
```

### 影响范围
- `src/features/settings/components/SiliconFlowSection.tsx` — 使用 TauriAPI.saveSetting
- `src-tauri/src/voice_input.rs:138-139` — 也从 settings 读取 (能工作，但路径不同)
- 用户体验: API Key 输入后无法保存，刷新后丢失

## 修复方案

### 已修复 (2026-05-30)

**文件1**: `SiliconFlowSection.tsx`
- 移除 `TauriAPI.saveSetting()` / `TauriAPI.getSetting()` 调用
- 改用 `configApi.saveVendorConfigs()` / `configApi.getVendorConfigs()`
- 使用 `VendorApiKeySection` 组件 (提供保存/删除按钮 + 状态显示)
- API Key 通过 `onSave`/`onClear` 回调与标准供应商路径一致

**修复原理**:
```
旧: SiliconFlow → TauriAPI.saveSetting() → settings 键值表 ❌
新: SiliconFlow → configApi.saveVendorConfigs() → LLMManager → SecureStore ✅
```

### 修复后数据流
```
SiliconFlowSection
  |-- 加载: getVendorConfigs() → LLMManager → SecureStore → 返回加密密钥
  |-- 保存: onSave(key) → saveVendorConfigs([{id:"builtin-siliconflow", apiKey:key}])
  |     → SecureStore::save_secret("builtin-siliconflow.api_key") ✅
  |     → SecureStore::save_secret("siliconflow.api_key") (兼容旧数据)
  |-- 删除: onClear() → saveVendorConfigs([{id:"builtin-siliconflow", apiKey:""}])
  |     → SecureStore::delete_secret("builtin-siliconflow.api_key")
  |
  |-- Voice Input (voice_input.rs) 也使用此路径
```

#### 修复 v2 (2026-05-30 12:15 CST) — 状态同步
追加修复: `persistApiKey` 保存成功后调用 `setApiKey(trimmed)` 同步更新
SiliconFlowSection 本地状态。此前保存成功但 VendorApiKeySection
因 prop 未更新而立即将输入框重置为空。

## 验证
- [x] 前端保存 API Key 后 LLMManager SecureStore 写入正确
- [x] 保存后本地状态同步更新, 输入框显示已保存的 Key
## 全供应商验证 (2026-05-30)

### save_vendor_configs → SecureStore ✅
```
commands.rs:1568 → LLMManager::save_vendor_configs()
  → is_builtin_vendor 判定: cfg.is_builtin || cfg.id.starts_with("builtin-")
  → ✅ 内置供应商 → SecureStore::save_secret("{vendor_id}.api_key")
  → ✅ 用户供应商 → encrypt → vendor_configs JSON
```

### get_vendor_configs → SecureStore ✅
```
commands.rs:1563 → LLMManager::get_vendor_configs()
  → builtin vendor + api_key empty → SecureStore::get_secret("{vendor_id}.api_key")
  → Failed → 回退 web_search_get_setting  ← 双重保险
```

### Tauri IPC 序列化 ✅
```
前端 camelCase: { apiKey, isBuiltin, providerType }
  → Tauri v2 自动转换
后端  snake_case: { api_key, is_builtin, provider_type }
```

### 结论
- 硅基流动: ✅ 已修复 (存储路径 + 状态同步)
- 其他供应商: ✅ 已验证 (始终使用正确路径)
- 对话管道: ✅ vendor_configs_for_runtime() 从同一 SecureStore 读取
