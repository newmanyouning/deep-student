use anyhow::Result;
use chrono::Utc;
use rusqlite::params;

impl crate::database::Database {
    /// 更新自定义模板
    pub fn update_custom_template(
        &self,
        template_id: &str,
        request: &crate::models::UpdateTemplateRequest,
    ) -> Result<()> {
        let conn = self.get_conn_safe()?;
        let now = Utc::now().to_rfc3339();

        println!("=== Update Template Debug ===");
        println!("Template ID: {}", template_id);
        println!("Request preview_data_json: {:?}", request.preview_data_json);

        let current_version = conn
            .query_row(
                "SELECT version FROM custom_anki_templates WHERE id = ?1",
                params![template_id],
                |row| row.get::<_, String>(0),
            )
            .unwrap_or_else(|_| "1.0.0".to_string());

        let new_version = Self::increment_version(&current_version);

        let mut query_parts = Vec::new();
        let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

        if let Some(preview_data_json) = &request.preview_data_json {
            println!("Adding preview_data_json to query: {}", preview_data_json);
            query_parts.push("preview_data_json = ?".to_string());
            params.push(Box::new(preview_data_json.clone()));
        } else {
            println!("No preview_data_json in request");
        }

        let updated_preview = conn.query_row(
            "SELECT preview_data_json FROM custom_anki_templates WHERE id = ?1",
            params![template_id],
            |row| row.get::<_, Option<String>>(0),
        )?;

        println!(
            "After update, preview_data_json in DB: {:?}",
            updated_preview
        );

        Ok(())
    }
}
