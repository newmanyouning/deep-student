# REF-009: Extract LLM Adapter Interface to Reduce Coupling

## Meta
- **Layer**: Layer 3 -- Core Business Logic
- **Priority**: P2
- **Est. effort**: XL
- **Predecessors**: None
- **Scope**: `src-tauri/src/llm_manager/`, `src-tauri/src/providers/`, `src-tauri/src/chat_v2/pipeline.rs`
- **Related reports**: `.planning/exploration/reports/round-24-26-backend-merged.md` (LLM Manager section)

## Problem Description

The LLM integration layer has **tight coupling between adapter configuration and provider-specific implementations**:

**Current architecture:**
```
LLMManager
  ├── get_api_configs() -> Vec<ApiConfig>  // Flat list, all vendors mixed
  ├── get_vendor_configs() -> Vec<VendorConfig>
  ├── get_model_profiles() -> Vec<ModelProfile>
  ├── test_api_connection()  
  └── should_use_openai_responses_for_config()
```

**Coupling problems:**
1. **Monolithic LLMManager**: Handles all vendors (OpenAI, Anthropic, SiliconFlow, DeepSeek, local models) with switch-case logic
2. **Adapter selection scattered**: `should_use_openai_responses_for_config` logic is in `commands.rs` line 9 (`crate::llm_manager::should_use_openai_responses_for_config`) and is imported across multiple modules
3. **Vendor-specific logic in pipeline**: `chat_v2/pipeline.rs` has vendor-specific message formatting and streaming logic rather than going through a uniform adapter interface
4. **`providers/` module** has overlapping responsibilities with `llm_manager/` -- unclear separation
5. **Adding a new vendor** requires changes to: LLMManager, pipeline, types, commands.rs, and potentially the frontend

**Impact:**
- High cognitive load for understanding the LLM integration
- New vendor support is risky (touches many modules)
- Testing requires mocking the entire LLMManager

## Target State

- Clean **LLM Adapter trait** interface:
  ```rust
  trait LLMAdapter {
      fn vendor_name(&self) -> &str;
      fn is_multimodal(&self) -> bool;
      fn format_messages(&self, messages: &[ChatMessage]) -> Result<Vec<AdapterMessage>>;
      fn stream_chat(&self, request: ChatRequest) -> BoxStream<Result<ChatResponseChunk>>;
      // ... additional common methods
  }
  ```
- Each vendor implements the trait (OpenAIAdapter, AnthropicAdapter, etc.)
- LLMManager becomes a registry/factory that returns the right adapter for a given config
- Pipeline depends only on the `LLMAdapter` trait, not on vendor-specific logic
- Adding a new vendor = implementing the trait + registering the adapter, no pipeline changes

## Steps

1. Design the `LLMAdapter` trait interface (analyze all current LLM usage patterns across the codebase)
2. Extract common adapter types to `src-tauri/src/llm_manager/adapter_types.rs`
3. Implement adapters for each vendor, extracting vendor-specific logic from pipeline
4. Refactor `LLMManager` into a registry pattern
5. Update `chat_v2/pipeline.rs` to depend only on `dyn LLMAdapter`
6. Update all call sites that import `should_use_openai_responses_for_config` and similar vendor-specific functions
7. Move vendor-specific formatting out of `commands.rs` into respective adapters
8. Clean up the `providers/` module -- either consolidate into adapters or clarify its role
9. Add tests for the adapter trait and each implementation

## Files Affected

| Path | Change | Description |
|------|--------|-------------|
| `src-tauri/src/llm_manager/mod.rs` | Refactor | Registry/factory pattern instead of monolithic |
| `src-tauri/src/llm_manager/adapter_trait.rs` | Create | LLMAdapter trait definition |
| `src-tauri/src/llm_manager/adapters/` | Create | Vendor-specific adapter implementations |
| `src-tauri/src/llm_manager/adapter_types.rs` | Create | Shared types for adapter interface |
| `src-tauri/src/chat_v2/pipeline.rs` | Refactor | Use LLMAdapter trait instead of LLMManager directly |
| `src-tauri/src/commands.rs` | Modify | Remove vendor-specific LLM logic, delegate to adapters |
| `src-tauri/src/providers/` | Review | Clarify role, possibly merge into adapters |
| `src-tauri/src/llm_manager/vendor_config.rs` | Modify | Simplify to adapter registration data |

## Interface Changes

| Symbol | Type | Old | New |
|--------|------|-----|-----|
| `LLMManager` | Struct | Monolithic, vendor logic mixed | Registry/factory returning `Box<dyn LLMAdapter>` |
| `should_use_openai_responses_for_config` | Function | Direct import from llm_manager | Method on OpenAIAdapter::supports_responses_api() |
| Pipeline LLM calls | Logic | Vendor-specific format/stream | Adapter trait methods |

## Static Verification

- [ ] Run `cargo check` before and after
- [ ] Run `cargo clippy --no-deps` before and after
- [ ] Run `cargo test` (unit + integration)
- [ ] Ensure no performance regressions in streaming (no extra boxing/allocation in hot path)
- [ ] Manual review: verify all vendor-specific code paths are covered by adapters

## Completion Criteria

- [ ] `LLMAdapter` trait defined with all necessary methods
- [ ] Each vendor (OpenAI, Anthropic, SiliconFlow, DeepSeek, local) has an adapter implementation
- [ ] Pipeline.rs has zero vendor-specific `if/else` logic
- [ ] `LLMManager` is a clean registry (100 lines or less)
- [ ] `commands.rs` has no vendor-specific LLM formatting
- [ ] Adding a new vendor = 1 new file (adapter) + 1 registration line
- [ ] All existing tests pass
