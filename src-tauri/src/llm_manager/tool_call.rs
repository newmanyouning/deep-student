use serde_json::Value;

impl super::LLMManager {
    /// 将 OpenAI 格式的工具调用转换为内部 ToolCall 格式
    pub(crate) fn convert_openai_tool_call(
        tool_call_value: &Value,
    ) -> std::result::Result<crate::models::ToolCall, String> {
        if let Ok(tc) = serde_json::from_value::<crate::models::ToolCall>(tool_call_value.clone()) {
            return Ok(tc);
        }

        let id = tool_call_value
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'id' field")?
            .to_string();

        let function = tool_call_value
            .get("function")
            .ok_or("Missing 'function' field")?;

        let tool_name = function
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'function.name' field")?
            .to_string();

        let arguments_value = function
            .get("arguments")
            .ok_or("Missing 'function.arguments' field")?;

        if !arguments_value.is_string() {
            if arguments_value.is_object() || arguments_value.is_array() {
                log::debug!(
                    "[llm_manager] convert_openai_tool_call: arguments 已是 JSON 值 (tool={})",
                    tool_name
                );
                return Ok(crate::models::ToolCall {
                    id,
                    tool_name,
                    args_json: arguments_value.clone(),
                });
            }
            return Ok(crate::models::ToolCall {
                id,
                tool_name,
                args_json: Value::Object(serde_json::Map::new()),
            });
        }

        let arguments_str = arguments_value.as_str().unwrap_or("{}");

        if arguments_str.trim().is_empty() {
            return Ok(crate::models::ToolCall {
                id,
                tool_name,
                args_json: Value::Object(serde_json::Map::new()),
            });
        }

        let args_json: Value = match serde_json::from_str(arguments_str) {
            Ok(v) => v,
            Err(e) => {
                let err_msg = e.to_string();
                if err_msg.contains("EOF")
                    || err_msg.contains("unexpected end")
                    || err_msg.contains("trailing")
                {
                    log::warn!(
                        "[llm_manager] 工具调用 JSON 疑似被截断 (len={}), 尝试修复...",
                        arguments_str.len()
                    );
                    match Self::try_repair_truncated_json(arguments_str) {
                        Some(repaired) => {
                            log::info!(
                                "[llm_manager] 截断 JSON 修复成功: tool={}, original_len={}, repaired_len={}",
                                tool_name,
                                arguments_str.len(),
                                repaired.to_string().len()
                            );
                            repaired
                        }
                        None => {
                            return Err(format!(
                                "Failed to parse arguments JSON (truncated, repair failed): {}",
                                e
                            ));
                        }
                    }
                } else {
                    return Err(format!("Failed to parse arguments JSON: {}", e));
                }
            }
        };

        Ok(crate::models::ToolCall {
            id,
            tool_name,
            args_json,
        })
    }

    /// 尝试修复被截断的工具调用 JSON
    fn try_repair_truncated_json(truncated: &str) -> Option<Value> {
        let s = truncated.trim();
        if s.is_empty() {
            return None;
        }

        if let Some(repaired) = Self::repair_by_bracket_completion(s) {
            return Some(repaired);
        }

        if let Some(repaired) = Self::repair_by_truncation_rollback(s) {
            return Some(repaired);
        }

        log::warn!(
            "[llm_manager] 截断 JSON 修复失败，所有策略均未成功 (len={})",
            s.len()
        );
        None
    }

    fn repair_by_bracket_completion(s: &str) -> Option<Value> {
        let mut stack: Vec<char> = Vec::new();
        let mut in_string = false;
        let mut escape_next = false;

        for ch in s.chars() {
            if escape_next {
                escape_next = false;
                continue;
            }
            if ch == '\\' && in_string {
                escape_next = true;
                continue;
            }
            if ch == '"' {
                in_string = !in_string;
                continue;
            }
            if in_string {
                continue;
            }
            match ch {
                '{' => stack.push('{'),
                '[' => stack.push('['),
                '}' => {
                    if stack.last() == Some(&'{') {
                        stack.pop();
                    }
                }
                ']' => {
                    if stack.last() == Some(&'[') {
                        stack.pop();
                    }
                }
                _ => {}
            }
        }

        if stack.is_empty() {
            return serde_json::from_str(s).ok();
        }

        let mut repaired = s.to_string();
        if in_string {
            repaired.push('"');
        }

        let trimmed = repaired.trim_end();
        let last_char = trimmed.chars().last().unwrap_or(' ');
        if last_char == ',' || last_char == ':' {
            repaired = trimmed[..trimmed.len() - 1].to_string();
        }

        for &bracket in stack.iter().rev() {
            match bracket {
                '{' => repaired.push('}'),
                '[' => repaired.push(']'),
                _ => {}
            }
        }

        match serde_json::from_str::<Value>(&repaired) {
            Ok(v) => {
                log::debug!(
                    "[llm_manager] 截断 JSON 修复成功（策略1：补全括号）, stack_depth={}",
                    stack.len()
                );
                Some(v)
            }
            Err(_) => None,
        }
    }

    fn repair_by_truncation_rollback(s: &str) -> Option<Value> {
        let rollback_targets = [',', '}', ']', '\n'];

        for &target in &rollback_targets {
            if let Some(pos) = s.rfind(target) {
                let candidate = if target == ',' {
                    &s[..pos]
                } else {
                    &s[..=pos]
                };

                if let Some(repaired) = Self::repair_by_bracket_completion(candidate) {
                    log::debug!(
                        "[llm_manager] 截断 JSON 修复成功（策略2：回退到 '{}' pos={}）",
                        target,
                        pos
                    );
                    return Some(repaired);
                }
            }
        }

        None
    }
}
