# Batch 3: Rust 辅助模块 — 重新扫描报告

> 扫描时间: 2026-05-30 15:48 CST | 40 文件 | 状态: ✅ 完成

## 3.1 LLM Manager Adapters — 14 实现

| Adapter | 文件 | Pattern |
|---------|------|---------|
| `GenericOpenAIAdapter` | `generic_openai.rs` | 通用默认 |
| `DeepSeekAdapter` | `deepseek.rs` | 版本感知 + 平台检测 |
| `QwenAdapter` | `qwen.rs` | 思考模式兼容 |
| `ZhipuAdapter` | `zhipu.rs` | 智谱 AI |
| `MiniMaxAdapter` | `minimax.rs` | 坚决移除不支持参数 |
| `MoonshotAdapter` | `moonshot.rs` | Kimi 适配 |
| `DoubaoAdapter` | `doubao.rs` | 火山引擎 |
| `AnthropicAdapter` | `anthropic.rs` | Claude 格式 |
| `GeminiAdapter` | `gemini.rs` | Google 格式 |
| `GrokAdapter` | `grok.rs` | xAI |
| `MistralAdapter` | `mistral.rs` | 欧洲供应商 |
| `ErnieAdapter` | `ernie.rs` | 百度文心 |
| `MimoAdapter` | `mimo.rs` | 小米 |

**22 注册条目**（含 9 个别名: `kimi→moonshot`, `baidu→ernie`, `claude→anthropic` 等）

| ID | 问题 |
|----|------|
| N3-01 | `pub mod zhipu` 与其他 `private` 模块不一致 |

## 3.2 cmd/ 目录 — 命令前缀分析

| 子模块 | 命令数 | 前缀一致性 |
|--------|--------|-----------|
| `notes.rs` | 34 | ✅ `notes_*` |
| `textbooks.rs` | 11 | ✅ `textbooks_*` |
| `ocr.rs` | 16 | ✅ `ocr_*` |
| `anki_cards.rs` | 3 | ✅ |
| `mcp.rs` | 17 | ✅ `mcp_*` |
| `enhanced_anki.rs` | 21 | ❌ 无统一前缀 |
| `anki_connect.rs` | 14 | ❌ 3 种风格混用 |
| `web_search.rs` | 9 | ❌ 5/7 命令与搜索无关 |

### 命令错放 (CRITICAL)

| 文件 | 命令 | 应归属 |
|------|------|--------|
| `translation.rs` | `ocr_extract_text` | `ocr.rs` |
| `web_search.rs` | `get_security_status` | 独立安全模块 |
| `web_search.rs` | `get_cn_whitelist_config` | 配置模块 |
| `web_search.rs` | `detect_tool_conflicts` | `mcp.rs` |

## 3.3 Data Governance — 黄金标准

所有 45 命令使用统一的 `data_governance_*` 前缀。6 个 commands 文件全部一致。

## 3.4 OCR Adapters

| Adapter | 命名模式 |
|---------|---------|
| `DeepSeekOcrAdapter` | `NameOcrAdapter` ✅ |
| `PaddleOcrVlAdapter` | `NameAdapter` (非标准) |
| `Glm4vOcrAdapter` | `NameOcrAdapter` ✅ |
| `GenericVlmAdapter` | `NameAdapter` (非标准) |
| `SystemOcrAdapter` | `NameOcrAdapter` ✅ |

| ID | 问题 |
|----|------|
| N3-02 | OCR 适配器命名不统一: `OcrAdapter` vs `Adapter` vs 内联 `Vl` |

---

*Batch 3 完成。文件: 40 | 冲突: 2 (N3-01, N3-02) + 4 命令错放*
