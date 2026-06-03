use std::path::PathBuf;

use axum::extract::Multipart;

use crate::errors::{AppError, AppResult};

/// Maximum file size for uploads (10MB).
pub const MAX_UPLOAD_FILE_SIZE: usize = 10 * 1024 * 1024;

/// Extract a single file from a multipart form upload.
///
/// Handles: field name matching, size validation, storage to disk.
/// Returns the absolute path to the saved file.
///
/// # Arguments
/// * `multipart` — the Axum Multipart extractor (consumed by reference)
/// * `field_name` — the expected form field name (e.g. `"file"`)
/// * `max_size` — maximum allowed file size in bytes
/// * `sub_dir` — subdirectory under STORAGE_DIR (e.g. `"imports"`)
pub async fn extract_uploaded_file(
    multipart: &mut Multipart,
    field_name: &str,
    max_size: usize,
    sub_dir: &str,
) -> AppResult<PathBuf> {
    let mut file_path = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::Validation(format!("Multipart error: {e}")))?
    {
        let name = field.name().unwrap_or("").to_string();
        if name == field_name {
            let file_name = field.file_name().unwrap_or("upload.xlsx").to_string();
            let data = field
                .bytes()
                .await
                .map_err(|e| AppError::Validation(format!("Failed to read file bytes: {e}")))?;

            if data.len() > max_size {
                let max_mb = max_size / (1024 * 1024);
                return Err(AppError::Validation(format!(
                    "File too large (max {max_mb}MB)"
                )));
            }

            let base_dir = crate::utils::get_storage_dir();
            let dir = format!("{base_dir}/{sub_dir}");
            tokio::fs::create_dir_all(&dir).await?;
            let path = format!("{}/{}_{}", dir, crate::utils::id::new_id(), file_name);
            tokio::fs::write(&path, data).await?;
            file_path = Some(PathBuf::from(path));
        }
    }

    file_path.ok_or_else(|| AppError::Validation("No file uploaded".to_string()))
}
