//! Anki 统一服务层
//!
//! 本模块是 Anki 卡片生成、导出、同步的统一入口。
//! 整合了原分散在 4 个文件中的 Anki 功能。
//!
//! ## 子模块
//! - 编排层: `crate::enhanced_anki_service::EnhancedAnkiService`
//! - 流式生成: `crate::streaming_anki_service::StreamingAnkiService`
//! - AnkiConnect: `crate::anki_connect_service`
//! - APKG 导出: `crate::apkg_exporter_service`

// 重新导出公共服务类型和函数

// orchestration (EnhancedAnkiService)
pub use crate::enhanced_anki_service::{
    DocumentStateDto, DocumentTaskCountsDto, EnhancedAnkiService,
};

// generation (StreamingAnkiService)
pub use crate::streaming_anki_service::StreamingAnkiService;

// connect (AnkiConnect HTTP client)
pub use crate::anki_connect_service::{
    check_anki_connect_availability,
    get_deck_names,
    get_model_names,
    get_model_field_names,
    create_deck_if_not_exists,
    add_notes_to_anki,
    add_notes_to_anki_with_card_models,
    import_apkg,
    AnkiConnectError,
    AnkiConnectResult,
};

// export (APKG)
pub use crate::apkg_exporter_service::{
    export_cards_to_apkg_with_template,
    export_cards_to_apkg_with_full_template,
    anki_connect_export_multi_apkg,
};
