# Round 31: 构建与脚本诊断

**层级**: 5.2 — 横切关注点
**预计文件数**: 10-20
**状态**: ⏳ 待执行

## 目标

梳理构建脚本、工具脚本和模型注册表。

## 扫描文件清单

| # | 路径 | 关注点 |
|---|------|--------|
| 1 | `scripts/build_all.sh` | 全平台构建 |
| 2 | `scripts/build_android.sh` | Android 构建 |
| 3 | `scripts/build_mac.sh` | macOS 构建 |
| 4 | `scripts/build_windows.sh` | Windows 构建 |
| 5 | `scripts/build_ios.sh` | iOS 构建 |
| 6 | `scripts/build_linux_all.sh` | Linux 构建 |
| 7 | `scripts/check-i18n.mjs` | i18n 完整性检查 |
| 8 | `scripts/check-missing-translations.mjs` | 翻译键缺失检查 |
| 9 | `scripts/check-licenses.mjs` | 许可证检查 |
| 10 | `scripts/lifecycle-score-gate.mjs` | 生命周期评分门禁 |
| 11 | `scripts/sync-model-registry.mjs` | 模型注册表同步 |
| 12 | `scripts/model-capability-registry.json` | 模型能力注册表 |
| 13 | `scripts/gemini-model-registry.json` | Gemini 模型注册表 |
| 14 | `scripts/provider-protocol-registry.json` | 供应商协议注册表 |
| 15 | `scripts/scan-component-usage.mjs` | 组件使用扫描 |
| 16 | `scripts/dev/README.md` | 开发脚本说明 |

## 诊断要点

1. **构建矩阵**: 各平台的构建入口和参数
2. **代码质量门禁**: lint、类型检查、i18n 检查
3. **模型注册表**: 模型信息的维护方式
4. **脚本组织**: 脚本的命名规范和文档完整度

## 输出格式

产出 `round-31-build-scripts.md`
