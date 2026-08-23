//! # SyncEnvelope — v3 云端变更载体
//!
//! v3 格式是对 `SyncChangesPayload` (v2) 的升级：
//!
//! 1. **信封字段**：`format_version = 3` + `schema_version` + `device_id` / `client_id`
//!    双 ID + `op_id` 幂等键 + `created_at_hlc` (HLC 时间戳，与
//!    `parse_flexible_timestamp` 兼容) + `base_checkpoint` 增量基线
//! 2. **单条变更扩展**：`EnvelopeChange` 在 v2 的 table/record/operation/payload
//!    基础上增加 `fields` (修改字段集)、`record_schema_version` (单记录级版本)、
//!    `business_unique_keys` (业务唯一键)、`causal_deps` (CRDT 依赖)
//! 3. **大文件引用**：`object_refs` (内容寻址，预留)
//!
//! ## 兼容性约定
//!
//! - **所有字段带 `#[serde(default)]`**：老 JSON / 未来缺字段均可解析，字段增减不破坏格式
//! - **读取链写入顺序 v3 → v2 → v1**：`orchestrator::download_changes` 在 v2 解析失败后、
//!   v1 之前尝试本格式；`format_version >= 3` 才视为合法 v3 信封
//! - **上传仍为 v2**：本模块只提供类型与转换，不参与上传构造（老设备兼容）
//! - **CRDT 合并逻辑未实现**：`causal_deps` 仅存储，预留

use super::manifest::{ChangeOperation, SyncChangeWithData};
use serde::{Deserialize, Serialize};

/// v3 云端变更载荷信封（与设计文档 §3.4 对齐）
///
/// 替代 `SyncChangesPayload` 作为主载体的候选格式。
/// 读取端要求 `format_version >= 3` 才按本格式处理，
/// 否则视为解析失败并落回 v1/v2 尝试（防止缺字段的老 JSON 被误识别）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SyncEnvelope {
    /// 格式版本号（= 3；读取端以 `>= 3` 作为判别条件，未来 v4 兼容）
    #[serde(default)]
    pub format_version: u32,
    /// 业务 schema 版本（与各库 `schema_versions` 对齐，预留）
    #[serde(default)]
    pub schema_version: u32,
    /// 上传设备 ID
    #[serde(default)]
    pub device_id: String,
    /// 客户端 ID
    #[serde(default)]
    pub client_id: String,
    /// 操作幂等键（重试不重复应用，预留）
    #[serde(default)]
    pub op_id: String,
    /// HLC 时间戳（字符串，与 `parse_flexible_timestamp` 兼容，参与排序归一）
    #[serde(default)]
    pub created_at_hlc: String,
    /// 增量基线（prune 后用于断层检测，预留）
    #[serde(default)]
    pub base_checkpoint: Option<String>,
    /// 变更列表
    #[serde(default)]
    pub changes: Vec<EnvelopeChange>,
    /// 大文件引用（内容寻址，预留；为空时序列化省略）
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub object_refs: Vec<ObjectRef>,
}

/// 单条变更 — 表名 + 主键 + 操作 + 修改字段集 + 载荷
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnvelopeChange {
    /// 表名
    #[serde(default)]
    pub table_name: String,
    /// 记录 ID
    #[serde(default)]
    pub record_id: String,
    /// 操作类型（复用 manifest.rs 的 ChangeOperation）
    ///
    /// ChangeOperation 未实现 Default，缺失时取 Update
    /// （与 `ChangeLogEntry::from_row` 的 unwrap_or(Update) 先例一致）
    #[serde(default = "default_change_operation")]
    pub operation: ChangeOperation,
    /// 修改的字段集合（源自 `__change_log.field_deltas_json`，预留；为空时序列化省略）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fields: Option<Vec<String>>,
    /// 完整行 JSON（与 `SyncChangeWithData.data` 同义；Delete 可为 null）
    #[serde(default)]
    pub payload: serde_json::Value,
    /// 单记录级版本（未来按表校验，预留；为空时序列化省略）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub record_schema_version: Option<u32>,
    /// 业务唯一键（冲突检测用，预留；为空时序列化省略）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub business_unique_keys: Option<serde_json::Value>,
    /// `(device_id, op_id)` 因果依赖（预留 CRDT；为空时序列化省略）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub causal_deps: Option<Vec<String>>,
}

impl EnvelopeChange {
    /// 从现有 `SyncChangeWithData` 转换（v2 单条 → v3 单条）
    ///
    /// - `payload` = `change.data`，无数据时为 null
    /// - `operation` 直接映射（同一枚举）
    /// - 预留字段（fields 之外）初始化为 None
    pub fn from_sync_change(change: &SyncChangeWithData, fields: Option<Vec<String>>) -> Self {
        Self {
            table_name: change.table_name.clone(),
            record_id: change.record_id.clone(),
            operation: change.operation,
            fields,
            payload: change.data.clone().unwrap_or(serde_json::Value::Null),
            record_schema_version: None,
            business_unique_keys: None,
            causal_deps: None,
        }
    }

    /// 转回 `SyncChangeWithData`（v3 单条 → 现有应用流格式）
    ///
    /// - `data` = payload，null 时转为 None（与 v2 Delete 语义一致）
    /// - `changed_at` 取信封级 `created_at_hlc`（v3 无单条时间戳）
    /// - `database_name` / `change_log_id` 为 None（v3 无对应字段，不参与按库过滤）
    pub fn to_sync_change_with_data(&self, created_at_hlc: &str) -> SyncChangeWithData {
        SyncChangeWithData {
            table_name: self.table_name.clone(),
            record_id: self.record_id.clone(),
            operation: self.operation,
            data: if self.payload.is_null() {
                None
            } else {
                Some(self.payload.clone())
            },
            changed_at: created_at_hlc.to_string(),
            change_log_id: None,
            database_name: None,
            suppress_change_log: None,
        }
    }
}

/// ChangeOperation 的 serde default（缺失时按最宽松的 Update 处理，
/// 与 ChangeLogEntry::from_row 的 unwrap_or(ChangeOperation::Update) 先例一致；
/// 格式层校验 validate_change_payload 仍会拦截无有效载荷的条目）
fn default_change_operation() -> ChangeOperation {
    ChangeOperation::Update
}

/// 大文件对象引用（内容寻址，预留）
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObjectRef {
    /// 内容寻址 key
    #[serde(default)]
    pub key: String,
    /// 内容 SHA-256
    #[serde(default)]
    pub sha256: String,
    /// 文件大小（字节）
    #[serde(default)]
    pub size: u64,
}

/// 校验单条 EnvelopeChange 的载荷是否可按目标表应用（格式层校验，不含业务规则）
///
/// 规则（至少）：
/// - `table_name` / `record_id` 非空
/// - `Insert` / `Update`：`payload` 必须是非空 JSON 对象（null / 数组 / 标量均拒绝）
/// - `Delete`：`payload` 可为 null；若携带载荷则必须是 JSON 对象
///
/// 失败时返回中文原因，调用方将其记录进 quarantine 清单并跳过该条，继续其余变更。
pub fn validate_change_payload(change: &EnvelopeChange) -> Result<(), String> {
    if change.table_name.trim().is_empty() {
        return Err("table_name 为空".to_string());
    }
    if change.record_id.trim().is_empty() {
        return Err("record_id 为空".to_string());
    }
    match change.operation {
        ChangeOperation::Insert | ChangeOperation::Update => {
            let obj = change.payload.as_object().ok_or_else(|| {
                format!(
                    "{:?} 操作的 payload 必须是 JSON 对象（当前为 null/数组/标量）",
                    change.operation
                )
            })?;
            if obj.is_empty() {
                return Err(format!(
                    "{:?} 操作的 payload 不能为空对象",
                    change.operation
                ));
            }
        }
        ChangeOperation::Delete => {
            // 删除不需要数据：允许 null 载荷；若携带载荷则必须是对象
            if !change.payload.is_null() && !change.payload.is_object() {
                return Err("Delete 操作的 payload 必须是 null 或 JSON 对象".to_string());
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 构造一条合法的 v3 信封（供各用例复用）
    fn sample_envelope() -> SyncEnvelope {
        SyncEnvelope {
            format_version: 3,
            schema_version: 1,
            device_id: "dev-a".to_string(),
            client_id: "client-1".to_string(),
            op_id: "op-100".to_string(),
            created_at_hlc: "2026-08-07T10:00:00Z".to_string(),
            base_checkpoint: None,
            changes: vec![EnvelopeChange {
                table_name: "notes".to_string(),
                record_id: "1".to_string(),
                operation: ChangeOperation::Insert,
                fields: Some(vec!["title".to_string()]),
                payload: serde_json::json!({ "id": 1, "title": "hello" }),
                record_schema_version: None,
                business_unique_keys: None,
                causal_deps: None,
            }],
            object_refs: Vec::new(),
        }
    }

    #[test]
    fn serde_roundtrip_preserves_all_fields() {
        let envelope = sample_envelope();
        let json = serde_json::to_string(&envelope).expect("序列化失败");
        let parsed: SyncEnvelope = serde_json::from_str(&json).expect("反序列化失败");
        assert_eq!(envelope, parsed);
    }

    #[test]
    fn serde_omits_empty_reserved_fields() {
        // object_refs / fields / record_schema_version 为空时应被省略
        let json = serde_json::to_string(&sample_envelope()).expect("序列化失败");
        assert!(!json.contains("object_refs"), "空 object_refs 不应序列化");
        assert!(!json.contains("record_schema_version"), "None 不应序列化");
        assert!(!json.contains("business_unique_keys"), "None 不应序列化");
        assert!(!json.contains("causal_deps"), "None 不应序列化");
        assert!(json.contains("fields"), "Some(fields) 应序列化");
    }

    #[test]
    fn serde_default_tolerates_missing_fields() {
        // 老/缺字段 JSON：所有字段都有 serde default，必须能解析
        let minimal = r#"{"format_version":3,"changes":[]}"#;
        let parsed: SyncEnvelope = serde_json::from_str(minimal).expect("缺字段 JSON 应可解析");
        assert_eq!(parsed.format_version, 3);
        assert_eq!(parsed.schema_version, 0);
        assert!(parsed.device_id.is_empty());
        assert!(parsed.client_id.is_empty());
        assert!(parsed.op_id.is_empty());
        assert!(parsed.created_at_hlc.is_empty());
        assert!(parsed.base_checkpoint.is_none());
        assert!(parsed.changes.is_empty());
        assert!(parsed.object_refs.is_empty());
    }

    #[test]
    fn serde_default_tolerates_missing_change_fields() {
        // 单条变更缺 payload 等字段：payload 默认 null，Delete 场景合法
        let json = r#"{
            "format_version": 3,
            "changes": [{"table_name":"notes","record_id":"9","operation":"Delete"}]
        }"#;
        let parsed: SyncEnvelope = serde_json::from_str(json).expect("缺字段变更应可解析");
        let change = &parsed.changes[0];
        assert_eq!(change.operation, ChangeOperation::Delete);
        assert!(change.payload.is_null());
        assert!(change.fields.is_none());
        assert!(change.causal_deps.is_none());
    }

    #[test]
    fn from_sync_change_maps_data_to_payload() {
        let change = SyncChangeWithData {
            table_name: "notes".to_string(),
            record_id: "5".to_string(),
            operation: ChangeOperation::Update,
            data: Some(serde_json::json!({ "id": 5, "title": "x" })),
            changed_at: "2026-08-07T10:00:00Z".to_string(),
            change_log_id: Some(42),
            database_name: Some("chat_v2".to_string()),
            suppress_change_log: Some(true),
        };
        let envelope_change =
            EnvelopeChange::from_sync_change(&change, Some(vec!["title".to_string()]));
        assert_eq!(envelope_change.table_name, "notes");
        assert_eq!(envelope_change.record_id, "5");
        assert_eq!(envelope_change.operation, ChangeOperation::Update);
        assert_eq!(envelope_change.payload, serde_json::json!({ "id": 5, "title": "x" }));
        assert_eq!(
            envelope_change.fields,
            Some(vec!["title".to_string()])
        );
        // 预留字段初始为 None
        assert!(envelope_change.record_schema_version.is_none());
        assert!(envelope_change.business_unique_keys.is_none());
        assert!(envelope_change.causal_deps.is_none());
    }

    #[test]
    fn from_sync_change_maps_missing_data_to_null() {
        let change = SyncChangeWithData {
            table_name: "notes".to_string(),
            record_id: "6".to_string(),
            operation: ChangeOperation::Delete,
            data: None,
            changed_at: "2026-08-07T10:00:00Z".to_string(),
            change_log_id: None,
            database_name: None,
            suppress_change_log: None,
        };
        let envelope_change = EnvelopeChange::from_sync_change(&change, None);
        assert!(envelope_change.payload.is_null());
    }

    #[test]
    fn to_sync_change_with_data_roundtrip() {
        let envelope_change = EnvelopeChange {
            table_name: "notes".to_string(),
            record_id: "7".to_string(),
            operation: ChangeOperation::Insert,
            fields: None,
            payload: serde_json::json!({ "id": 7, "title": "y" }),
            record_schema_version: Some(1),
            business_unique_keys: Some(serde_json::json!({ "title": "y" })),
            causal_deps: Some(vec!["dev-a/op-1".to_string()]),
        };
        let converted = envelope_change.to_sync_change_with_data("2026-08-07T11:00:00Z");
        assert_eq!(converted.table_name, "notes");
        assert_eq!(converted.record_id, "7");
        assert_eq!(converted.operation, ChangeOperation::Insert);
        assert_eq!(converted.data, Some(serde_json::json!({ "id": 7, "title": "y" })));
        assert_eq!(converted.changed_at, "2026-08-07T11:00:00Z");
        assert!(converted.database_name.is_none());
        assert!(converted.change_log_id.is_none());
    }

    #[test]
    fn to_sync_change_with_data_null_payload_becomes_none() {
        let envelope_change = EnvelopeChange {
            table_name: "notes".to_string(),
            record_id: "8".to_string(),
            operation: ChangeOperation::Delete,
            fields: None,
            payload: serde_json::Value::Null,
            record_schema_version: None,
            business_unique_keys: None,
            causal_deps: None,
        };
        let converted = envelope_change.to_sync_change_with_data("2026-08-07T11:00:00Z");
        assert_eq!(converted.operation, ChangeOperation::Delete);
        assert!(converted.data.is_none());
    }

    #[test]
    fn validate_accepts_insert_with_object_payload() {
        let change = EnvelopeChange {
            table_name: "notes".to_string(),
            record_id: "1".to_string(),
            operation: ChangeOperation::Insert,
            fields: None,
            payload: serde_json::json!({ "id": 1, "title": "ok" }),
            record_schema_version: None,
            business_unique_keys: None,
            causal_deps: None,
        };
        assert!(validate_change_payload(&change).is_ok());
    }

    #[test]
    fn validate_rejects_insert_with_null_payload() {
        let change = EnvelopeChange {
            table_name: "notes".to_string(),
            record_id: "2".to_string(),
            operation: ChangeOperation::Insert,
            fields: None,
            payload: serde_json::Value::Null,
            record_schema_version: None,
            business_unique_keys: None,
            causal_deps: None,
        };
        let err = validate_change_payload(&change).expect_err("Insert + null 应拒绝");
        assert!(err.contains("payload 必须是 JSON 对象"), "错误信息: {}", err);
    }

    #[test]
    fn validate_rejects_insert_with_empty_object() {
        let change = EnvelopeChange {
            table_name: "notes".to_string(),
            record_id: "3".to_string(),
            operation: ChangeOperation::Insert,
            fields: None,
            payload: serde_json::json!({}),
            record_schema_version: None,
            business_unique_keys: None,
            causal_deps: None,
        };
        let err = validate_change_payload(&change).expect_err("Insert + 空对象应拒绝");
        assert!(err.contains("不能为空对象"), "错误信息: {}", err);
    }

    #[test]
    fn validate_rejects_scalar_and_array_payload() {
        // 标量
        let scalar = EnvelopeChange {
            table_name: "notes".to_string(),
            record_id: "4".to_string(),
            operation: ChangeOperation::Update,
            fields: None,
            payload: serde_json::json!(42),
            record_schema_version: None,
            business_unique_keys: None,
            causal_deps: None,
        };
        assert!(validate_change_payload(&scalar).is_err(), "标量 payload 应拒绝");

        // 数组
        let array = EnvelopeChange {
            table_name: "notes".to_string(),
            record_id: "4".to_string(),
            operation: ChangeOperation::Update,
            fields: None,
            payload: serde_json::json!([1, 2, 3]),
            record_schema_version: None,
            business_unique_keys: None,
            causal_deps: None,
        };
        assert!(validate_change_payload(&array).is_err(), "数组 payload 应拒绝");
    }

    #[test]
    fn validate_accepts_update_with_object_payload() {
        let change = EnvelopeChange {
            table_name: "notes".to_string(),
            record_id: "5".to_string(),
            operation: ChangeOperation::Update,
            fields: None,
            payload: serde_json::json!({ "title": "updated" }),
            record_schema_version: None,
            business_unique_keys: None,
            causal_deps: None,
        };
        assert!(validate_change_payload(&change).is_ok());
    }

    #[test]
    fn validate_accepts_delete_with_null_payload() {
        let change = EnvelopeChange {
            table_name: "notes".to_string(),
            record_id: "6".to_string(),
            operation: ChangeOperation::Delete,
            fields: None,
            payload: serde_json::Value::Null,
            record_schema_version: None,
            business_unique_keys: None,
            causal_deps: None,
        };
        assert!(validate_change_payload(&change).is_ok());
    }

    #[test]
    fn validate_accepts_delete_with_object_payload() {
        let change = EnvelopeChange {
            table_name: "notes".to_string(),
            record_id: "7".to_string(),
            operation: ChangeOperation::Delete,
            fields: None,
            payload: serde_json::json!({ "id": 7 }),
            record_schema_version: None,
            business_unique_keys: None,
            causal_deps: None,
        };
        assert!(validate_change_payload(&change).is_ok());
    }

    #[test]
    fn validate_rejects_delete_with_scalar_payload() {
        let change = EnvelopeChange {
            table_name: "notes".to_string(),
            record_id: "8".to_string(),
            operation: ChangeOperation::Delete,
            fields: None,
            payload: serde_json::json!("just-a-string"),
            record_schema_version: None,
            business_unique_keys: None,
            causal_deps: None,
        };
        let err = validate_change_payload(&change).expect_err("Delete + 标量应拒绝");
        assert!(err.contains("Delete 操作的 payload 必须是 null 或 JSON 对象"), "错误信息: {}", err);
    }

    #[test]
    fn validate_rejects_empty_table_name_and_record_id() {
        let no_table = EnvelopeChange {
            table_name: String::new(),
            record_id: "1".to_string(),
            operation: ChangeOperation::Insert,
            fields: None,
            payload: serde_json::json!({ "a": 1 }),
            record_schema_version: None,
            business_unique_keys: None,
            causal_deps: None,
        };
        let err = validate_change_payload(&no_table).expect_err("空 table_name 应拒绝");
        assert!(err.contains("table_name 为空"), "错误信息: {}", err);

        let no_record = EnvelopeChange {
            table_name: "notes".to_string(),
            record_id: "  ".to_string(),
            operation: ChangeOperation::Insert,
            fields: None,
            payload: serde_json::json!({ "a": 1 }),
            record_schema_version: None,
            business_unique_keys: None,
            causal_deps: None,
        };
        let err = validate_change_payload(&no_record).expect_err("空 record_id 应拒绝");
        assert!(err.contains("record_id 为空"), "错误信息: {}", err);
    }
}
