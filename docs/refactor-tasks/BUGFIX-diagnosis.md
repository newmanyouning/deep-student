# API Key 存储诊断报告

> 2026-05-30 12:10 CST | 综合诊断

## 诊断结论

### 已确认的修复 (✅)
1. **硅基流动 (SiliconFlowSection.tsx)**: 已从 `TauriAPI.saveSetting()` 改为 `saveVendorConfigs()`，与其他内置供应商统一
2. **所有供应商**: 前端保存路径统一为: `configApi.saveVendorConfigs()` → `invoke('save_vendor_configs')` → `LLMManager::save_vendor_configs()`

### 已验证正确 (✅)
3. 后端存储: `LLMManager::save_vendor_configs()` → `SecureStore::save_secret()` → 加密存储
4. 后端读取: `LLMManager::get_vendor_configs()` → `SecureStore::get_secret()` → 读取真实 Key
5. 失败回退: `SecureStore` → `web_search_get_setting` / `web_search_save_setting` 两级回退

### 仍可能出现问题的情况 (⚠️)
如果编译后仍出现所有供应商 API Key 为空:

#### 情况A: SecureStore 加密文件损坏
- 位置: `{app_data_dir}/.secure/` 
- 原因: AES 密钥种子文件损坏或权限不足
- 症状: 所有内置供应商 Key 为空
- 诊断: 检查应用日志中是否有 "安全存储失败，回退到明文存储" 警告
- 解决: 删除 `.secure/` 目录，重新配置 API Key

#### 情况B: 旧数据未迁移
- 如果使用旧版应用保存过 Key
- 旧路径: `TauriAPI.saveSetting("builtin-siliconflow.api_key")` → settings 表
- 新路径: `saveVendorConfigs()` → SecureStore
- 新版本首次启动: `get_vendor_configs()` 读取 SecureStore → 空 → 回退 settings → 找到旧 Key → 自动迁移 ✅

#### 情况C: 前端未触发重新加载
- 保存后需要 window 事件广播触发 `loadAll()`
- 事件: `api_configurations_changed` / `siliconflow-apikey-changed`
- 验证: 保存后刷新页面观察按钮是否点亮

### 验证步骤
1. 编译后运行应用
2. 打开 `{app_data_dir}/debug.log` 或控制台
3. 搜索 "安全存储" / "secure_store" / "get_secret" / "save_secret"
4. 确认日志中无错误或警告
5. 在设置页面输入 API Key → 点击保存 → 等待 2 秒 → 观察按钮状态
6. 关闭并重新打开设置页面 → 确认 Key 仍在
7. 尝试发送一条测试消息 → 确认模型配置正确
