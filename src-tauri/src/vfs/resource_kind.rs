//! 统一资源类型枚举
//!
//! 本模块是"三个资源枚举统一"的第一步，收敛以下枚举为单一事实源：
//! - `VfsResourceType` (vfs/types.rs) — 运行时唯一事实源
//! - `DstuNodeType` (dstu/types.rs) — 协议层视图
//! - chat_v2 `ResourceType` (chat_v2/resource_types.rs) — 尸体代码
//!
//! 三者取并集共 11 个变体。

use serde::{Deserialize, Serialize};

// ============================================================================
// 资源类型枚举
// ============================================================================

/// 统一资源类型枚举
///
/// # 持久化契约 ⚠️
/// 11 个小写字符串是持久化契约（数据库列、云端 payload 按原值读写），
/// **序列化值一个都不能改**。别名只允许进入 `from_str`，禁止出现在序列化层。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ResourceKind {
    /// 笔记
    Note,
    /// 教材
    ///
    /// ⚠️ 存储口径 (2026-08-10 调研确认): 教材在 `resources` 表以 `type='file'` 存储
    /// (files 表为 textbooks+attachments 合并表, 无教材标记列), `Textbook` 是逻辑类型,
    /// 实际区分仅靠 ID 前缀 `tb_` (见 `from_id_prefix`)。详见 textbook_repo.rs 注释。
    Textbook,
    /// 题目集识别
    Exam,
    /// 翻译
    Translation,
    /// 作文批改
    Essay,
    /// 图片
    Image,
    /// 文件附件
    File,
    /// 检索结果（RAG 知识库检索）
    Retrieval,
    /// 知识导图
    MindMap,
    /// 题目卡片快照
    Card,
    /// 文件夹（虚拟节点，用于表示科目或类型目录）
    Folder,
}

impl ResourceKind {
    /// 从字符串解析资源类型（宽容层）
    ///
    /// 合并 `DstuNodeType::from_str` 的全部现有别名语义（单数/复数/中文），
    /// 外加标准 11 个 lowercase 值。注意：
    /// - `attachment`/`attachments`/`附件` → Image（沿用 DSTU 现有映射：
    ///   附件按图片附件语义处理；文档附件以 `att_` 前缀 ID 走 File，见 `from_id_prefix`）
    /// - `img` → Image（图片的常见缩写）
    ///
    /// # 参数
    /// - `s`: 类型字符串（如 "note", "notes", "教材"）
    ///
    /// # 返回
    /// 解析成功返回 `Some(ResourceKind)`，失败返回 `None`
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            // 标准 11 个 lowercase 值
            "note" => Some(ResourceKind::Note),
            "textbook" => Some(ResourceKind::Textbook),
            "exam" => Some(ResourceKind::Exam),
            "translation" => Some(ResourceKind::Translation),
            "essay" => Some(ResourceKind::Essay),
            "image" => Some(ResourceKind::Image),
            "file" => Some(ResourceKind::File),
            "retrieval" => Some(ResourceKind::Retrieval),
            "mindmap" => Some(ResourceKind::MindMap),
            "card" => Some(ResourceKind::Card),
            "folder" => Some(ResourceKind::Folder),
            // 别名（复数 + 中文，合并自 DstuNodeType::from_str）
            "notes" | "笔记" => Some(ResourceKind::Note),
            "textbooks" | "教材" => Some(ResourceKind::Textbook),
            "exams" | "题目集" | "试卷" => Some(ResourceKind::Exam),
            "translations" | "翻译" => Some(ResourceKind::Translation),
            "essays" | "作文" | "作文批改" => Some(ResourceKind::Essay),
            "images" | "图片" | "img" => Some(ResourceKind::Image),
            "files" | "文件" => Some(ResourceKind::File),
            "retrievals" | "检索" | "检索结果" => Some(ResourceKind::Retrieval),
            "mindmaps" | "知识导图" | "导图" => Some(ResourceKind::MindMap),
            "cards" | "卡片" => Some(ResourceKind::Card),
            "folders" | "文件夹" => Some(ResourceKind::Folder),
            // 附件类型映射到 Image（沿用 DSTU 现有语义）
            "attachment" | "attachments" | "附件" => Some(ResourceKind::Image),
            _ => None,
        }
    }

    /// 转换为路径段字符串（复数形式）
    ///
    /// 用于构建 DSTU 路径，如 "/高考复习/note_123"
    pub fn to_path_segment(&self) -> &'static str {
        match self {
            ResourceKind::Note => "notes",
            ResourceKind::Textbook => "textbooks",
            ResourceKind::Exam => "exams",
            ResourceKind::Translation => "translations",
            ResourceKind::Essay => "essays",
            ResourceKind::Image => "images",
            ResourceKind::File => "files",
            ResourceKind::Retrieval => "retrievals",
            ResourceKind::MindMap => "mindmaps",
            ResourceKind::Card => "cards",
            ResourceKind::Folder => "folders",
        }
    }

    /// 转换为单数字符串（用于 type 字段）
    ///
    /// 返回资源类型的单数形式，如 "note", "textbook", "exam"。
    /// 输出值即持久化契约，与 `Display` 一致。
    pub fn to_type_string(&self) -> &'static str {
        match self {
            ResourceKind::Note => "note",
            ResourceKind::Textbook => "textbook",
            ResourceKind::Exam => "exam",
            ResourceKind::Translation => "translation",
            ResourceKind::Essay => "essay",
            ResourceKind::Image => "image",
            ResourceKind::File => "file",
            ResourceKind::Retrieval => "retrieval",
            ResourceKind::MindMap => "mindmap",
            ResourceKind::Card => "card",
            ResourceKind::Folder => "folder",
        }
    }

    /// 从资源 ID 前缀推断资源类型
    ///
    /// 合并 `RESOURCE_ID_PREFIXES` (dstu/path_types.rs) + `DstuNodeType::from_id_prefix`
    /// (dstu/types.rs) 的全部前缀，作为 ID 前缀推断的唯一规范入口。
    ///
    /// # 修复与补充
    /// - `img_` → Image（修复：现状被归为 File）
    /// - `att_` → File（attachment 是遗留 file 实体）
    /// - `essay_session_` → Essay
    /// - `es_` → Essay（补充：vfs 索引与 chat 执行器已按此前缀识别作文会话 ID）
    /// - `card_` → Card（补充：题目卡片持久化 ID 格式 `card_{nanoid}`）
    ///
    /// # 说明
    /// 检索（Retrieval）资源没有专用 ID 前缀：现有实现将其存入 VFS resources 表，
    /// 使用通用的 `res_{nanoid}` ID，类型需查库解析（`res_` 本身不携带类型信息），
    /// 因此不在此映射 `res_`，未知前缀一律返回 `None`。
    ///
    /// # 参数
    /// - `id`: 资源 ID（如 "note_abc123", "tb_xyz"）
    ///
    /// # 返回
    /// 推断成功返回 `Some(ResourceKind)`，无法识别返回 `None`
    pub fn from_id_prefix(id: &str) -> Option<Self> {
        if id.starts_with("note_") {
            Some(ResourceKind::Note)
        } else if id.starts_with("tb_") {
            Some(ResourceKind::Textbook)
        } else if id.starts_with("file_") || id.starts_with("att_") {
            // att_ 是遗留 file 实体
            Some(ResourceKind::File)
        } else if id.starts_with("img_") {
            // 修复：img_ 前缀应归 Image（现状被归为 File）
            Some(ResourceKind::Image)
        } else if id.starts_with("exam_") {
            Some(ResourceKind::Exam)
        } else if id.starts_with("tr_") {
            Some(ResourceKind::Translation)
        } else if id.starts_with("essay_session_") || id.starts_with("essay_") || id.starts_with("es_") {
            Some(ResourceKind::Essay)
        } else if id.starts_with("fld_") {
            Some(ResourceKind::Folder)
        } else if id.starts_with("mm_") {
            Some(ResourceKind::MindMap)
        } else if id.starts_with("card_") {
            Some(ResourceKind::Card)
        } else {
            None
        }
    }

    /// 获取各变体的规范 ID 前缀（生成侧事实源）
    ///
    /// 与 `from_id_prefix` 互逆：生成新资源 ID 时使用本方法得到前缀，
    /// 解析已有 ID 时使用 `from_id_prefix` 还原类型，两者必须保持一致
    /// （互逆性由 `test_id_prefix_from_id_prefix_roundtrip` 保证）。
    ///
    /// # 说明
    /// - `Retrieval` 无专用前缀：检索资源存入 VFS resources 表，
    ///   使用通用 `res_{nanoid}` ID，类型需查库解析，故返回 `None`
    /// - `File` 的规范前缀为 `file_`；遗留 `att_` 前缀仅用于解析侧
    ///   （见 `from_id_prefix`），不参与生成
    ///
    /// # 返回
    /// 各变体的规范 ID 前缀（含下划线），如 `"note_"`；无专用前缀返回 `None`
    pub fn id_prefix(&self) -> Option<&'static str> {
        match self {
            ResourceKind::Note => Some("note_"),
            ResourceKind::Textbook => Some("tb_"),
            ResourceKind::Exam => Some("exam_"),
            ResourceKind::Translation => Some("tr_"),
            ResourceKind::Essay => Some("essay_"),
            ResourceKind::Image => Some("img_"),
            ResourceKind::File => Some("file_"),
            ResourceKind::MindMap => Some("mm_"),
            ResourceKind::Card => Some("card_"),
            ResourceKind::Folder => Some("fld_"),
            ResourceKind::Retrieval => None,
        }
    }

    /// 获取所有资源类型（11 个变体）
    pub fn all() -> Vec<Self> {
        vec![
            ResourceKind::Note,
            ResourceKind::Textbook,
            ResourceKind::Exam,
            ResourceKind::Translation,
            ResourceKind::Essay,
            ResourceKind::Image,
            ResourceKind::File,
            ResourceKind::Retrieval,
            ResourceKind::MindMap,
            ResourceKind::Card,
            ResourceKind::Folder,
        ]
    }

    /// 获取显示名称的 i18n 键
    ///
    /// 迁移自 `DstuNodeType::display_name_key` (dstu/types.rs)，作为协议层通用方法。
    /// Card 为统一后新增变体，使用 "dstu:types.card" 键。
    pub fn display_name_key(&self) -> &'static str {
        match self {
            ResourceKind::Folder => "dstu:types.folder",
            ResourceKind::Note => "dstu:types.note",
            ResourceKind::Textbook => "dstu:types.textbook",
            ResourceKind::Exam => "dstu:types.exam",
            ResourceKind::Translation => "dstu:types.translation",
            ResourceKind::Essay => "dstu:types.essay",
            ResourceKind::Image => "dstu:types.image",
            ResourceKind::File => "dstu:types.file",
            ResourceKind::Retrieval => "dstu:types.retrieval",
            ResourceKind::MindMap => "dstu:types.mindmap",
            // ★ 2026-08-07 迁移补充：Card 变体（统一后新增）
            ResourceKind::Card => "dstu:types.card",
        }
    }

    /// 获取预览类型
    ///
    /// 迁移自 `DstuNodeType::preview_type` (dstu/types.rs)，返回字符串值
    /// （markdown | pdf | card | exam | image | docx | xlsx | pptx | text | mindmap | none），
    /// 与 `DstuNode::preview_type` 字段的文档契约一致。
    pub fn preview_type(&self) -> &'static str {
        match self {
            ResourceKind::Folder => "none",
            ResourceKind::Note => "markdown",
            ResourceKind::Textbook => "pdf",
            ResourceKind::Exam => "exam",
            ResourceKind::Translation => "markdown",
            ResourceKind::Essay => "markdown",
            ResourceKind::Image => "image",
            ResourceKind::File => "none",
            ResourceKind::Retrieval => "markdown",
            ResourceKind::MindMap => "mindmap",
            // ★ 2026-08-07 迁移补充：Card 变体（统一后新增）
            ResourceKind::Card => "card",
        }
    }
}

impl std::fmt::Display for ResourceKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_type_string())
    }
}

// ============================================================================
// 单元测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// 11 个变体的序列化值（持久化契约，禁止修改）
    const EXPECTED_SERIALIZED: [(&str, &ResourceKind); 11] = [
        ("note", &ResourceKind::Note),
        ("textbook", &ResourceKind::Textbook),
        ("exam", &ResourceKind::Exam),
        ("translation", &ResourceKind::Translation),
        ("essay", &ResourceKind::Essay),
        ("image", &ResourceKind::Image),
        ("file", &ResourceKind::File),
        ("retrieval", &ResourceKind::Retrieval),
        ("mindmap", &ResourceKind::MindMap),
        ("card", &ResourceKind::Card),
        ("folder", &ResourceKind::Folder),
    ];

    #[test]
    fn test_serde_roundtrip_all_variants() {
        // 验证 11 个变体序列化输出值精确等于小写字符串，且可往返反序列化
        for (expected, kind) in EXPECTED_SERIALIZED.iter() {
            let json = serde_json::to_string(kind).unwrap();
            assert_eq!(json, format!("\"{}\"", expected), "变体 {:?} 序列化值", kind);
            let parsed: ResourceKind = serde_json::from_str(&json).unwrap();
            assert_eq!(parsed, **kind, "变体 {:?} 反序列化", kind);
        }
        // 反向：每个小写字符串都能反序列化为对应变体
        for (expected, kind) in EXPECTED_SERIALIZED.iter() {
            let parsed: ResourceKind = serde_json::from_str(&format!("\"{}\"", expected)).unwrap();
            assert_eq!(parsed, **kind);
        }
    }

    #[test]
    fn test_from_str_standard_values() {
        // 标准 11 个 lowercase 值
        for (value, kind) in EXPECTED_SERIALIZED.iter() {
            assert_eq!(ResourceKind::from_str(value), Some(**kind), "标准值 {}", value);
        }
    }

    #[test]
    fn test_from_str_aliases() {
        // 合并自 DstuNodeType::from_str 的别名语义
        assert_eq!(ResourceKind::from_str("notes"), Some(ResourceKind::Note));
        assert_eq!(ResourceKind::from_str("笔记"), Some(ResourceKind::Note));
        assert_eq!(ResourceKind::from_str("教材"), Some(ResourceKind::Textbook));
        assert_eq!(ResourceKind::from_str("textbooks"), Some(ResourceKind::Textbook));
        assert_eq!(ResourceKind::from_str("exams"), Some(ResourceKind::Exam));
        assert_eq!(ResourceKind::from_str("题目集"), Some(ResourceKind::Exam));
        assert_eq!(ResourceKind::from_str("试卷"), Some(ResourceKind::Exam));
        assert_eq!(ResourceKind::from_str("translations"), Some(ResourceKind::Translation));
        assert_eq!(ResourceKind::from_str("翻译"), Some(ResourceKind::Translation));
        assert_eq!(ResourceKind::from_str("essays"), Some(ResourceKind::Essay));
        assert_eq!(ResourceKind::from_str("作文"), Some(ResourceKind::Essay));
        assert_eq!(ResourceKind::from_str("作文批改"), Some(ResourceKind::Essay));
        assert_eq!(ResourceKind::from_str("images"), Some(ResourceKind::Image));
        assert_eq!(ResourceKind::from_str("图片"), Some(ResourceKind::Image));
        assert_eq!(ResourceKind::from_str("img"), Some(ResourceKind::Image));
        assert_eq!(ResourceKind::from_str("files"), Some(ResourceKind::File));
        assert_eq!(ResourceKind::from_str("文件"), Some(ResourceKind::File));
        assert_eq!(ResourceKind::from_str("retrievals"), Some(ResourceKind::Retrieval));
        assert_eq!(ResourceKind::from_str("检索"), Some(ResourceKind::Retrieval));
        assert_eq!(ResourceKind::from_str("检索结果"), Some(ResourceKind::Retrieval));
        assert_eq!(ResourceKind::from_str("mindmaps"), Some(ResourceKind::MindMap));
        assert_eq!(ResourceKind::from_str("知识导图"), Some(ResourceKind::MindMap));
        assert_eq!(ResourceKind::from_str("导图"), Some(ResourceKind::MindMap));
        assert_eq!(ResourceKind::from_str("folders"), Some(ResourceKind::Folder));
        assert_eq!(ResourceKind::from_str("文件夹"), Some(ResourceKind::Folder));
        assert_eq!(ResourceKind::from_str("card"), Some(ResourceKind::Card));
        // 附件 → Image（沿用 DSTU 现有语义）
        assert_eq!(ResourceKind::from_str("attachment"), Some(ResourceKind::Image));
        assert_eq!(ResourceKind::from_str("attachments"), Some(ResourceKind::Image));
        assert_eq!(ResourceKind::from_str("附件"), Some(ResourceKind::Image));
        // 大小写不敏感
        assert_eq!(ResourceKind::from_str("NOTE"), Some(ResourceKind::Note));
        assert_eq!(ResourceKind::from_str("Textbook"), Some(ResourceKind::Textbook));
        // 未知值
        assert_eq!(ResourceKind::from_str("unknown"), None);
        assert_eq!(ResourceKind::from_str(""), None);
    }

    #[test]
    fn test_from_id_prefix() {
        // 标准前缀映射
        assert_eq!(ResourceKind::from_id_prefix("note_abc123"), Some(ResourceKind::Note));
        assert_eq!(ResourceKind::from_id_prefix("tb_xyz"), Some(ResourceKind::Textbook));
        assert_eq!(ResourceKind::from_id_prefix("exam_123"), Some(ResourceKind::Exam));
        assert_eq!(ResourceKind::from_id_prefix("tr_123"), Some(ResourceKind::Translation));
        assert_eq!(ResourceKind::from_id_prefix("essay_123"), Some(ResourceKind::Essay));
        assert_eq!(ResourceKind::from_id_prefix("essay_session_123"), Some(ResourceKind::Essay));
        assert_eq!(ResourceKind::from_id_prefix("es_123"), Some(ResourceKind::Essay));
        assert_eq!(ResourceKind::from_id_prefix("fld_123"), Some(ResourceKind::Folder));
        assert_eq!(ResourceKind::from_id_prefix("mm_123"), Some(ResourceKind::MindMap));
        assert_eq!(ResourceKind::from_id_prefix("file_123"), Some(ResourceKind::File));
        assert_eq!(ResourceKind::from_id_prefix("card_abc"), Some(ResourceKind::Card));
        // 修复断言：img_ 前缀应归 Image（现状被归为 File）
        assert_eq!(ResourceKind::from_id_prefix("img_123"), Some(ResourceKind::Image));
        // att_ → File（attachment 是遗留 file 实体）
        assert_eq!(ResourceKind::from_id_prefix("att_123"), Some(ResourceKind::File));
        // res_ 是 VFS resources 表通用 ID（类型需查库解析），不属于前缀推断范围
        assert_eq!(ResourceKind::from_id_prefix("res_123"), None);
        // 未知前缀
        assert_eq!(ResourceKind::from_id_prefix("xyz_123"), None);
        assert_eq!(ResourceKind::from_id_prefix(""), None);
    }

    #[test]
    fn test_display_and_type_string() {
        // Display 输出 11 个 lowercase 值之一（持久化契约）
        for (value, kind) in EXPECTED_SERIALIZED.iter() {
            assert_eq!(kind.to_string(), *value, "Display {:?}", kind);
            assert_eq!(kind.to_type_string(), *value, "to_type_string {:?}", kind);
        }
    }

    #[test]
    fn test_to_path_segment_plural() {
        // 复数路径段（与 DstuNodeType::to_path_segment 一致）
        assert_eq!(ResourceKind::Note.to_path_segment(), "notes");
        assert_eq!(ResourceKind::Textbook.to_path_segment(), "textbooks");
        assert_eq!(ResourceKind::Exam.to_path_segment(), "exams");
        assert_eq!(ResourceKind::Translation.to_path_segment(), "translations");
        assert_eq!(ResourceKind::Essay.to_path_segment(), "essays");
        assert_eq!(ResourceKind::Image.to_path_segment(), "images");
        assert_eq!(ResourceKind::File.to_path_segment(), "files");
        assert_eq!(ResourceKind::Retrieval.to_path_segment(), "retrievals");
        assert_eq!(ResourceKind::MindMap.to_path_segment(), "mindmaps");
        assert_eq!(ResourceKind::Card.to_path_segment(), "cards");
        assert_eq!(ResourceKind::Folder.to_path_segment(), "folders");
    }

    #[test]
    fn test_all_contains_all_variants() {
        // 枚举完整性：与 EXPECTED_SERIALIZED 无遗漏
        let all = ResourceKind::all();
        assert_eq!(all.len(), EXPECTED_SERIALIZED.len());
        for (_, kind) in EXPECTED_SERIALIZED.iter() {
            assert!(all.contains(*kind), "缺少变体 {:?}", kind);
        }
    }

    #[test]
    fn test_id_prefix_values() {
        // 规范 ID 前缀（生成侧事实源，与 from_id_prefix 互逆）
        assert_eq!(ResourceKind::Note.id_prefix(), Some("note_"));
        assert_eq!(ResourceKind::Textbook.id_prefix(), Some("tb_"));
        assert_eq!(ResourceKind::Exam.id_prefix(), Some("exam_"));
        assert_eq!(ResourceKind::Translation.id_prefix(), Some("tr_"));
        assert_eq!(ResourceKind::Essay.id_prefix(), Some("essay_"));
        assert_eq!(ResourceKind::Image.id_prefix(), Some("img_"));
        assert_eq!(ResourceKind::File.id_prefix(), Some("file_"));
        assert_eq!(ResourceKind::MindMap.id_prefix(), Some("mm_"));
        assert_eq!(ResourceKind::Card.id_prefix(), Some("card_"));
        assert_eq!(ResourceKind::Folder.id_prefix(), Some("fld_"));
        // Retrieval 无专用前缀（通用 res_ ID，类型需查库解析）
        assert_eq!(ResourceKind::Retrieval.id_prefix(), None);
    }

    #[test]
    fn test_id_prefix_from_id_prefix_roundtrip() {
        // 互逆性：from_id_prefix(id_prefix(k) + "xxx") == Some(k)
        for (_, kind) in EXPECTED_SERIALIZED.iter() {
            match kind.id_prefix() {
                Some(prefix) => {
                    let id = format!("{}nanoid123", prefix);
                    assert_eq!(
                        ResourceKind::from_id_prefix(&id),
                        Some(**kind),
                        "互逆性失败: {:?} 前缀 {}",
                        kind,
                        prefix
                    );
                }
                // 仅 Retrieval 允许无前缀（res_ 通用 ID 不携带类型信息）
                None => assert_eq!(**kind, ResourceKind::Retrieval),
            }
        }
    }
}
