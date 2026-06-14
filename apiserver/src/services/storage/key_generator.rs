use chrono::Utc;
use uuid::Uuid;

pub fn sanitize_filename(name: &str) -> String {
    let path = std::path::Path::new(name);
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("file");
    let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");

    // Keep alphanumeric, '-', and '_'
    let sanitized_stem: String = stem
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();

    if extension.is_empty() {
        sanitized_stem
    } else {
        // Sanitize extension as well to be safe
        let sanitized_ext: String = extension
            .chars()
            .filter(char::is_ascii_alphanumeric)
            .collect();
        format!("{sanitized_stem}.{sanitized_ext}")
    }
}

pub fn generate_storage_key(workspace_id: Uuid, category: &str, file_name: &str) -> String {
    let date_str = Utc::now().format("%Y/%m").to_string();
    let file_uuid = Uuid::new_v4();
    let sanitized = sanitize_filename(file_name);
    format!("{workspace_id}/{category}/{date_str}/{file_uuid}_{sanitized}")
}

pub fn generate_system_key(category: &str, file_name: &str) -> String {
    let date_str = Utc::now().format("%Y/%m").to_string();
    let file_uuid = Uuid::new_v4();
    let sanitized = sanitize_filename(file_name);
    format!("_system/{category}/{date_str}/{file_uuid}_{sanitized}")
}
