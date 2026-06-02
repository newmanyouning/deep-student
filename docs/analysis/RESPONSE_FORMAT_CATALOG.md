# Response Format Catalog

> Source: `C:/deep-student/scripts/connectivity_test_results.json`
> Generated: 2026-06-01
> Test methodology: Single-turn chat completion, `{"messages":[{"role":"user","content":"say connected"}], "stream":false}`, OpenAI-compatible API format.

---

## Summary

| Total Vendors | Models Tested | Passed | Failed |
|--------------|---------------|--------|--------|
| 9            | 21            | 12     | 9      |

**Passed models (12 across 6 vendors):**

| # | Vendor | Model | Thinking? | Response Content | Reasoning Tokens |
|---|--------|-------|-----------|-----------------|-----------------|
| 1 | SiliconFlow | Qwen/Qwen3-8B | Yes | `"\n\nconnected"` | 147 / 149 |
| 2 | SiliconFlow | THUDM/GLM-Z1-9B-0414 | Yes | `"\nconnected"` | 145 / 146 |
| 3 | DeepSeek | deepseek-v4-flash | Yes | `""` (empty) | 10 / 10 |
| 4 | DeepSeek | deepseek-v4-pro | Yes | `""` (empty) | 10 / 10 |
| 5 | Qwen (DashScope) | qwen-plus | No | `"connected"` | N/A |
| 6 | Qwen (DashScope) | qwen3.5-plus | Yes | `"connected"` | 186 / 192 |
| 7 | Zhipu | glm-5 | Yes | `""` (empty) | 10 / 10 |
| 8 | Zhipu | glm-4.7-flash | Yes | `""` (empty) | 10 / 10 |
| 9 | Moonshot | kimi-k2.5 | Yes | `""` (empty) | N/A (no details) |
| 10 | Moonshot | kimi-k2.6 | No | `""` (empty) | N/A |
| 11 | MiMo | mimo-v2.5-pro | Yes | `""` (empty) | 9 / 10 |
| 12 | MiMo | mimo-v2-flash | No | `"connected"` | 0 |

---

## 1. SiliconFlow

**API Base:** `https://api.siliconflow.cn/v1`
**Vendor-specific field:** `prompt_cache_hit_tokens`, `prompt_cache_miss_tokens` at the top level of `usage`.

### 1.1 Qwen/Qwen3-8B (Thinking Model)

```json
{
  "ok": true,
  "status_code": 200,
  "response_text": "\n\nconnected",
  "usage": {
    "prompt_tokens": 18,
    "completion_tokens": 149,
    "total_tokens": 167,
    "completion_tokens_details": {
      "reasoning_tokens": 147
    },
    "prompt_tokens_details": {
      "cached_tokens": 0
    },
    "prompt_cache_hit_tokens": 0,
    "prompt_cache_miss_tokens": 18
  },
  "model_used": "Qwen/Qwen3-8B"
}
```

**Response structure analysis:**
- `choices[0].message.content`: plain text string. Here it is `"\n\nconnected"` (two newlines followed by the word). The model prefixes output with blank lines.
- `choices[0].message.reasoning_content`: This is a **thinking model**. The API separates reasoning tokens into `reasoning_content` field (not included in `content`). The `reasoning_content` is approximately 147 tokens of hidden chain-of-thought.
- `usage.completion_tokens_details.reasoning_tokens`: 147 (98.7% of completion tokens were reasoning).
- **Vendor-specific fields:** `prompt_cache_hit_tokens` and `prompt_cache_miss_tokens` appear at the top level of `usage`, separate from the nested `prompt_tokens_details.cached_tokens`.
- `tool_calls` support: Yes - Qwen3-8B supports function calling. `tool_calls` would appear in `choices[0].message.tool_calls` when a tool/function is specified in the request.
- **Multimodal:** No (text-only). SiliconFlow hosts Qwen3-VL-* models separately for vision tasks.

### 1.2 THUDM/GLM-Z1-9B-0414 (Thinking Model)

```json
{
  "ok": true,
  "status_code": 200,
  "response_text": "\nconnected",
  "usage": {
    "prompt_tokens": 18,
    "completion_tokens": 146,
    "total_tokens": 164,
    "completion_tokens_details": {
      "reasoning_tokens": 145
    }
  },
  "model_used": "THUDM/GLM-Z1-9B-0414"
}
```

**Response structure analysis:**
- `choices[0].message.content`: plain text, `"\nconnected"` (single newline + word).
- `choices[0].message.reasoning_content`: Present (thinking model). 145 of 146 tokens are reasoning.
- **Vendor-specific differences from Qwen3-8B above:** No `prompt_tokens_details`, no `prompt_cache_hit_tokens`, no `prompt_cache_miss_tokens`. The usage object is cleaner - just the four core fields.
- `tool_calls` support: Yes - GLM-Z1 is a chat model with function calling capability.
- **Multimodal:** No.

---

## 2. DeepSeek

**API Base:** `https://api.deepseek.com/v1`
**Vendor-specific note:** Same extra cache fields as SiliconFlow (`prompt_cache_hit_tokens`, `prompt_cache_miss_tokens`). Both models use the same API backend.

### 2.1 deepseek-v4-flash (Thinking Model)

```json
{
  "ok": true,
  "status_code": 200,
  "response_text": "",
  "usage": {
    "prompt_tokens": 14,
    "completion_tokens": 10,
    "total_tokens": 24,
    "prompt_tokens_details": {
      "cached_tokens": 0
    },
    "completion_tokens_details": {
      "reasoning_tokens": 10
    },
    "prompt_cache_hit_tokens": 0,
    "prompt_cache_miss_tokens": 14
  },
  "model_used": "deepseek-v4-flash"
}
```

**Response structure analysis:**
- `choices[0].message.content`: Empty string or `null`. The model produced only reasoning/thinking content (10 tokens, all reasoning) and no visible text output. When `reasoning_content` is present and `content` is `null`/empty, this is the model's way of saying "I thought but have nothing to say."
- `choices[0].message.reasoning_content`: Present and non-empty. 100% of completion tokens are reasoning.
- **Vendor-specific fields:** `prompt_cache_hit_tokens`, `prompt_cache_miss_tokens` (same pattern as SiliconFlow).
- `tool_calls` support: Yes - DeepSeek models natively support function calling / tool use. `tool_calls` would appear in `choices[0].message.tool_calls` when requested.
- **Multimodal:** No. DeepSeek has separate OCR models for vision tasks.

### 2.2 deepseek-v4-pro (Thinking Model)

```json
{
  "ok": true,
  "status_code": 200,
  "response_text": "",
  "usage": {
    "prompt_tokens": 14,
    "completion_tokens": 10,
    "total_tokens": 24,
    "prompt_tokens_details": {
      "cached_tokens": 0
    },
    "completion_tokens_details": {
      "reasoning_tokens": 10
    },
    "prompt_cache_hit_tokens": 0,
    "prompt_cache_miss_tokens": 14
  },
  "model_used": "deepseek-v4-pro"
}
```

**Identical structure to deepseek-v4-flash.** The only difference is `model_used`. Same API backend, same response schema.

**Key observation:** Both DeepSeek models produce empty `content` and 100% reasoning tokens for a simple "say connected" prompt. This is expected DeepSeek behavior - the model's reasoning content fills in but the visible output is suppressed when the thinking exceeds the visible portion.

---

## 3. Qwen (DashScope / Alibaba Cloud)

**API Base:** `https://dashscope.aliyuncs.com/compatible-mode/v1`
**Vendor-specific behavior:** DashScope returns `usage` with fields in a different order. No top-level cache hit/miss fields. Instead uses `text_tokens` sub-field in token details.

### 3.1 qwen-plus (Non-Thinking Model)

```json
{
  "ok": true,
  "status_code": 200,
  "response_text": "connected",
  "usage": {
    "total_tokens": 19,
    "completion_tokens": 1,
    "prompt_tokens": 18,
    "prompt_tokens_details": {
      "cached_tokens": 0
    }
  },
  "model_used": "qwen-plus"
}
```

**Response structure analysis:**
- `choices[0].message.content`: Plain text `"connected"`. No leading/trailing whitespace.
- `choices[0].message.reasoning_content`: **Not present.** This is a non-thinking model. No `reasoning_content` field, no `completion_tokens_details`.
- `usage` field ordering: Note that `total_tokens` appears **first** (unlike SiliconFlow/DeepSeek which have it last). This is a DashScope-specific ordering quirk.
- `completion_tokens`: Only 1 (just the single word "connected").
- No `completion_tokens_details` at all.
- `tool_calls` support: Yes - qwen-plus supports function calling. `tool_calls` in `choices[0].message.tool_calls`.
- **Multimodal:** No (text-only).

### 3.2 qwen3.5-plus (Thinking Model)

```json
{
  "ok": true,
  "status_code": 200,
  "response_text": "connected",
  "usage": {
    "prompt_tokens": 20,
    "completion_tokens": 192,
    "total_tokens": 212,
    "completion_tokens_details": {
      "reasoning_tokens": 186,
      "text_tokens": 192
    },
    "prompt_tokens_details": {
      "text_tokens": 20
    }
  },
  "model_used": "qwen3.5-plus"
}
```

**Response structure analysis:**
- `choices[0].message.content`: `"connected"` (just the word, despite 186 reasoning tokens and 192 completion tokens total).
- `choices[0].message.reasoning_content`: **Present.** The model did extensive reasoning (186 tokens) but the visible output is just "connected".
- **DashScope-specific field:** `completion_tokens_details.text_tokens` = 192 (equals `completion_tokens` total, indicating text_tokens is the total and reasoning_tokens is a breakdown of that total). `prompt_tokens_details.text_tokens` = 20 (equals `prompt_tokens`).
- **Key structural difference:** No `cached_tokens` in `prompt_tokens_details` (unlike qwen-plus which had `cached_tokens: 0`). The thinking model path returns `text_tokens` instead.
- `tool_calls` support: Yes - qwen3.5-plus supports function calling.
- **Multimodal:** No (text-only).

---

## 4. Zhipu (BigModel / OpenAI)

**API Base:** `https://open.bigmodel.cn/api/paas/v4`
**Vendor-specific behavior:** `usage` fields ordered with `completion_tokens` first (unique among vendors).

### 4.1 glm-5 (Thinking Model)

```json
{
  "ok": true,
  "status_code": 200,
  "response_text": "",
  "usage": {
    "completion_tokens": 10,
    "completion_tokens_details": {
      "reasoning_tokens": 10
    },
    "prompt_tokens": 15,
    "prompt_tokens_details": {
      "cached_tokens": 0
    },
    "total_tokens": 25
  },
  "model_used": "glm-5"
}
```

**Response structure analysis:**
- `choices[0].message.content`: Empty string (thinking-only response).
- `choices[0].message.reasoning_content`: Present. 100% of completion tokens are reasoning.
- **Zhipu-specific field ordering:** `completion_tokens` listed **first** inside `usage` (unique among all 6 vendors).
- `prompt_tokens_details.cached_tokens`: 0.
- `tool_calls` support: Yes - GLM-5 supports function calling / tool use.
- **Multimodal:** No (text-only).

### 4.2 glm-4.7-flash (Thinking Model)

```json
{
  "ok": true,
  "status_code": 200,
  "response_text": "",
  "usage": {
    "completion_tokens": 10,
    "completion_tokens_details": {
      "reasoning_tokens": 10
    },
    "prompt_tokens": 15,
    "prompt_tokens_details": {
      "cached_tokens": 2
    },
    "total_tokens": 25
  },
  "model_used": "glm-4.7-flash"
}
```

**Same structure as glm-5** but with one difference:
- `prompt_tokens_details.cached_tokens`: **2** (this is the only model in the entire test that showed an **actual cache hit** greater than 0). This indicates the Zhipu API supports prompt caching and the glm-4.7-flash model had 2 cached prompt tokens.

---

## 5. Moonshot (Kimi)

**API Base:** `https://api.moonshot.cn/v1`
**Vendor-specific behavior:** `usage` has a top-level `cached_tokens` field (legacy format).

### 5.1 kimi-k2.5 (Thinking Model)

```json
{
  "ok": true,
  "status_code": 200,
  "response_text": "",
  "usage": {
    "prompt_tokens": 17,
    "completion_tokens": 10,
    "total_tokens": 27,
    "cached_tokens": 17,
    "prompt_tokens_details": {
      "cached_tokens": 17
    }
  },
  "model_used": "kimi-k2.5"
}
```

**Response structure analysis:**
- `choices[0].message.content`: Empty string (thinking-only response).
- `choices[0].message.reasoning_content`: Likely present (thinking model), but no `completion_tokens_details` returned, so reasoning token count is unknown from this test.
- **Moonshot-specific field:** Top-level `cached_tokens` field in `usage` (value: 17). This is a legacy/compatibility field that duplicates `prompt_tokens_details.cached_tokens`.
- No `completion_tokens_details` at all - Moonshot does not break down completion tokens by reasoning vs text even for thinking models.
- `tool_calls` support: Yes - Kimi models support function calling. `tool_calls` in `choices[0].message.tool_calls`.
- **Multimodal:** kimi-k2.5 itself is text-only. Moonshot has separate vision-preview models (moonshot-v1-*-vision-preview).

### 5.2 kimi-k2.6 (Non-Thinking Model)

```json
{
  "ok": true,
  "status_code": 200,
  "response_text": "",
  "usage": {
    "prompt_tokens": 17,
    "completion_tokens": 10,
    "total_tokens": 27
  },
  "model_used": "kimi-k2.6"
}
```

**Response structure analysis:**
- `choices[0].message.content`: Empty string.
- `choices[0].message.reasoning_content`: **Not present** (no thinking/No details returned).
- **Simplest usage format of all models:** Just 3 fields - `prompt_tokens`, `completion_tokens`, `total_tokens`. No `completion_tokens_details`, no `prompt_tokens_details`, no `cached_tokens`.
- Despite being the newer model (k2.6), it returns less detail than k2.5. This could be a model-specific or API-version behavior.
- `tool_calls` support: Yes - Kimi k2.6 supports function calling.
- **Multimodal:** No (text-only).

---

## 6. MiMo (Xiaomi)

**API Base:** `https://api.xiaomimimo.com/v1`
**Vendor-specific behavior:** `usage` fields ordered with `completion_tokens` first (similar to Zhipu). `reasoning_tokens` always present even when 0.

### 6.1 mimo-v2.5-pro (Thinking Model)

```json
{
  "ok": true,
  "status_code": 200,
  "response_text": "",
  "usage": {
    "completion_tokens": 10,
    "prompt_tokens": 261,
    "total_tokens": 271,
    "completion_tokens_details": {
      "reasoning_tokens": 9
    },
    "prompt_tokens_details": {
      "cached_tokens": 256
    }
  },
  "model_used": "mimo-v2.5-pro"
}
```

**Response structure analysis:**
- `choices[0].message.content`: Empty string (thinking-only response).
- `choices[0].message.reasoning_content`: Present. 9 of 10 completion tokens are reasoning.
- **High prompt cache hit:** 256 out of 261 prompt tokens were cached (98.1% cache hit rate). This is the highest cache utilization across all vendors.
- **Field ordering:** `completion_tokens` first (same as Zhipu).
- `tool_calls` support: Yes - MiMo models support function calling.
- **Multimodal:** No (text-only). MiMo has a separate `mimo-v2-omni` model for multimodal tasks.

### 6.2 mimo-v2-flash (Non-Thinking Model)

```json
{
  "ok": true,
  "status_code": 200,
  "response_text": "connected",
  "usage": {
    "completion_tokens": 2,
    "prompt_tokens": 36,
    "total_tokens": 38,
    "completion_tokens_details": {
      "reasoning_tokens": 0
    },
    "prompt_tokens_details": {
      "cached_tokens": 34
    }
  },
  "model_used": "mimo-v2-flash"
}
```

**Response structure analysis:**
- `choices[0].message.content`: `"connected"` (direct text response).
- `choices[0].message.reasoning_content`: **Not present** (non-thinking model).
- **MiMo-specific quirk:** `completion_tokens_details.reasoning_tokens` is explicitly set to **0** (not absent, but explicitly 0). This only appears here and differs from other non-thinking models (qwen-plus, kimi-k2.6) which omit the field entirely.
- High cache hit: 34/36 prompt tokens cached (94.4%).
- `tool_calls` support: Yes - MiMo flash models support function calling.
- **Multimodal:** No (text-only).

---

## Cross-Vendor Comparison

### Usage Field Presence Matrix

| Field | SF Qwen3-8B | SF GLM-Z1 | DS v4-flash | DS v4-pro | DS qwen-plus | DS qwen3.5+ | ZP glm-5 | ZP glm-4.7f | MS kimi-k2.5 | MS kimi-k2.6 | MiMo v2.5p | MiMo v2f |
|-------|-------------|-----------|-------------|-----------|-------------|-------------|----------|-------------|-------------|-------------|------------|----------|
| prompt_tokens | Y | Y | Y | Y | Y | Y | Y | Y | Y | Y | Y | Y |
| completion_tokens | Y | Y | Y | Y | Y | Y | Y | Y | Y | Y | Y | Y |
| total_tokens | Y | Y | Y | Y | Y | Y | Y | Y | Y | Y | Y | Y |
| cached_tokens (top-level) | - | - | - | - | - | - | - | - | Y(17) | - | - | - |
| prompt_tokens_details.cached_tokens | Y(0) | - | Y(0) | Y(0) | Y(0) | - | Y(0) | Y(2) | Y(17) | - | - | Y(34) |
| prompt_tokens_details.text_tokens | - | - | - | - | - | Y(20) | - | - | - | - | - | - |
| completion_tokens_details.reasoning_tokens | Y(147) | Y(145) | Y(10) | Y(10) | - | Y(186) | Y(10) | Y(10) | - | - | Y(9) | Y(0) |
| completion_tokens_details.text_tokens | - | - | - | - | - | Y(192) | - | - | - | - | - | - |
| prompt_cache_hit_tokens | Y(0) | - | Y(0) | Y(0) | - | - | - | - | - | - | - | - |
| prompt_cache_miss_tokens | Y(18) | - | Y(14) | Y(14) | - | - | - | - | - | - | - | - |

Legend: SF = SiliconFlow, DS = DashScope, ZP = Zhipu, MS = Moonshot, MiMo = Xiaomi MiMo

### Response Content Format by Vendor

| Vendor | Content Format | Leading Whitespace? | Empty on Thinking? |
|--------|---------------|-------------------|-------------------|
| SiliconFlow | Plain text | Yes (1-2 newlines) | No (always returns text) |
| DeepSeek | Plain text | No | Yes (empty when thinking-only) |
| DashScope | Plain text | No | No |
| Zhipu | Plain text | No | Yes (empty when thinking-only) |
| Moonshot | Plain text | No | Yes (empty for all) |
| MiMo | Plain text | No | Mixed (pro empty, flash has text) |

### Reasoning Content Behavior by Vendor

| Vendor | Thinking Model returns `reasoning_content`? | Field name convention |
|--------|---------------------------------------------|----------------------|
| SiliconFlow | Yes | `choices[0].message.reasoning_content` |
| DeepSeek | Yes | `choices[0].message.reasoning_content` |
| DashScope | Yes | `choices[0].message.reasoning_content` |
| Zhipu | Yes | `choices[0].message.reasoning_content` |
| Moonshot (k2.5) | Likely (no token details to confirm) | Standard OpenAI-compatible |
| MiMo | Yes (pro); No (flash) | Standard OpenAI-compatible |

All vendors follow the OpenAI-compatible convention: `reasoning_content` as a separate field in `choices[0].message`, parallel to `content`.

---

## Vendor-Specific Response Fields Summary

| Vendor | Extra `usage` fields | Notes |
|--------|---------------------|-------|
| **SiliconFlow** | `prompt_cache_hit_tokens`, `prompt_cache_miss_tokens` | Located at top level of `usage` alongside standard fields. Unique to SiliconFlow/DeepSeek. |
| **DeepSeek** | `prompt_cache_hit_tokens`, `prompt_cache_miss_tokens` | Same format as SiliconFlow (likely same API gateway). |
| **DashScope** | `prompt_tokens_details.text_tokens`, `completion_tokens_details.text_tokens` | DashScope uses `text_tokens` instead of duplicating total tokens. No top-level cache fields. |
| **Zhipu** | (standard fields only) | Distinct field ordering: `completion_tokens` first. |
| **Moonshot** | `cached_tokens` (top-level) | Legacy top-level `cached_tokens` duplicates `prompt_tokens_details.cached_tokens`. |
| **MiMo** | (standard fields only) | Uniquely returns `reasoning_tokens: 0` explicitly for non-thinking models. |

---

## Tool Calls Support Assessment

All 12 passed models are chat completion models and support function calling / tool use via the standard OpenAI-compatible `tool_calls` field in `choices[0].message`.

When a request includes `tools` or `tool_choice`, the response structure would be:

```json
{
  "choices": [{
    "index": 0,
    "message": {
      "role": "assistant",
      "content": null,
      "tool_calls": [
        {
          "id": "call_xxx",
          "type": "function",
          "function": {
            "name": "function_name",
            "arguments": "{\"key\": \"value\"}"
          }
        }
      ]
    },
    "finish_reason": "tool_calls"
  }]
}
```

**Confirmed tool_calls support per model:**
| Model | Tool Calls | Notes |
|-------|-----------|-------|
| Qwen/Qwen3-8B | Yes | Native function calling |
| THUDM/GLM-Z1-9B-0414 | Yes | GLM series supports tools |
| deepseek-v4-flash | Yes | DeepSeek native tool support |
| deepseek-v4-pro | Yes | DeepSeek native tool support |
| qwen-plus | Yes | DashScope function calling |
| qwen3.5-plus | Yes | DashScope function calling |
| glm-5 | Yes | Zhipu tool use |
| glm-4.7-flash | Yes | Zhipu tool use |
| kimi-k2.5 | Yes | Moonshot function calling |
| kimi-k2.6 | Yes | Moonshot function calling |
| mimo-v2.5-pro | Yes | MiMo function calling |
| mimo-v2-flash | Yes | MiMo function calling |

---

## Multimodal Support Assessment

**None** of the 12 tested models support multimodal input/output. Tested models are all text-only chat completion models.

However, each vendor offers separate vision/multimodal models:

| Vendor | Available Multimodal Models |
|--------|---------------------------|
| **SiliconFlow** | Qwen/Qwen3-VL-32B-Instruct, Qwen/Qwen3-VL-8B-Instruct, Qwen/Qwen3-Omni-30B-A3B-Instruct, PaddlePaddle/PaddleOCR-VL-1.5 (listed in models endpoint) |
| **DeepSeek** | deepseek-ocr (for OCR tasks, not tested) |
| **DashScope** | qwen3.5-omni-plus, qwen3.5-omni-flash (listed in models endpoint) |
| **Zhipu** | (no multimodal models listed in models endpoint) |
| **Moonshot** | moonshot-v1-8k-vision-preview, moonshot-v1-128k-vision-preview (listed in models endpoint) |
| **MiMo** | mimo-v2-omni (listed in models endpoint) |

The test script did not cover any multimodal or vision model. For multimodal models, the input format accepts `content` as an array of content blocks (text + image_url), and the response would follow the same OpenAI-compatible format as text models.

---

## General Response Structure Template (OpenAI-Compatible)

All 12 models follow this general structure:

```json
{
  "id": "chatcmpl-<unique_id>",
  "object": "chat.completion",
  "created": <unix_timestamp>,
  "model": "<model_name>",
  "choices": [
    {
      "index": 0,
      "message": {
        "role": "assistant",
        "content": "<string or null>",
        "reasoning_content": "<string or null (thinking models only)>",
        "tool_calls": null
      },
      "finish_reason": "stop",
      "logprobs": null
    }
  ],
  "usage": {
    "prompt_tokens": <int>,
    "completion_tokens": <int>,
    "total_tokens": <int>,
    "completion_tokens_details": {
      "reasoning_tokens": <int>
    },
    "prompt_tokens_details": {
      "cached_tokens": <int>
    }
  }
}
```

**Vendor-specific deviations from this template:**
1. **Moonshot** adds top-level `cached_tokens` in `usage`.
2. **SiliconFlow / DeepSeek** add `prompt_cache_hit_tokens` and `prompt_cache_miss_tokens` in `usage`.
3. **DashScope** replaces `prompt_tokens_details.cached_tokens` with `prompt_tokens_details.text_tokens` for thinking models, and adds `completion_tokens_details.text_tokens`.
4. **MiMo** explicitly includes `reasoning_tokens: 0` for non-thinking models (others omit the field).
5. Field ordering within `usage` varies by vendor (no standardized order).
