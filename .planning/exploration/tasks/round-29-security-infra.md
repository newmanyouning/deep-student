# Round 29: 安全与基础设施诊断

**层级**: 4.10 — 后端模块（Rust）
**预计文件数**: 12-25
**状态**: ⏳ 待执行

## 目标

梳理加密、多模态处理、OCR 适配、MCP 后端、DSTU 后端。

## 扫描文件清单

| # | 文件路径 | 关注点 |
|---|---------|--------|
| 1 | `src-tauri/src/crypto/` 全部 .rs 文件 | AES-256-GCM 加密 |
| 2 | `src-tauri/src/multimodal/` 全部 .rs 文件 | 多模态处理 |
| 3 | `src-tauri/src/ocr_adapters/` 全部 .rs 文件 | OCR 适配器 (6引擎) |
| 4 | `src-tauri/src/mcp/` 全部 .rs 文件 | MCP 协议后端 |
| 5 | `src-tauri/src/dstu/` 全部 .rs 文件 | DSTU 协议后端 |
| 6 | `src-tauri/src/vendors/` 全部 .rs 文件 | 供应商特定代码 |

## 诊断要点

1. **加密方案**: A/B 双槽位切换机制
2. **OCR 引擎**: 6 种 OCR 引擎的适配状态
3. **MCP 实现**: 服务器端协议实现程度
4. **多模态**: 支持的模态和处理管线
5. **DSTU 后端**: 资源协议的 Rust 实现

## 输出格式

产出 `round-29-security-infra.md`
