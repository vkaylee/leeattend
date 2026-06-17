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
    state: &crate::AppState,
    auth_token: Option<&crate::diesel_pool::TenantAuthToken>,
    multipart: &mut Multipart,
    field_name: &str,
    max_size: usize,
    sub_dir: &str,
) -> AppResult<PathBuf> {
    let mut file_path = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_e| AppError::validation(crate::utils::i18n::I18nKey::UploadMultipart))?
    {
        let name = field.name().unwrap_or("").to_string();
        if name == field_name {
            let file_name = field.file_name().unwrap_or("upload.xlsx").to_string();
            let mime_type = field
                .content_type()
                .unwrap_or("application/octet-stream")
                .to_string();

            let has_csv_ext = std::path::Path::new(&file_name)
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("csv"));
            if has_csv_ext || mime_type.eq_ignore_ascii_case("text/csv") {
                return Err(AppError::validation(
                    crate::utils::i18n::I18nKey::InvalidFileFormat,
                ));
            }

            let data = field
                .bytes()
                .await
                .map_err(|_e| AppError::validation(crate::utils::i18n::I18nKey::UploadReadFailed))?;

            if data.len() > max_size {
                return Err(AppError::validation(
                    crate::utils::i18n::I18nKey::UploadFileTooLarge,
                ));
            }

            // Write to local disk for backward compatibility (Calamine parsing)
            let base_dir = crate::utils::get_storage_dir();
            let dir = format!("{base_dir}/{sub_dir}");
            tokio::fs::create_dir_all(&dir).await?;
            let path = format!("{}/{}_{}", dir, crate::utils::id::new_id(), file_name);
            tokio::fs::write(&path, data.clone()).await?;

            // Delegate to FileService (S3) instead of legacy direct upload
            if let Some(token) = auth_token {
                let _ = state
                    .file_service()
                    .upload_file(
                        token,
                        crate::models::file::FileOwnerType::Workspace,
                        token.workspace_id(),
                        crate::models::file::FileCategory::ImportFile,
                        &file_name,
                        &mime_type,
                        data.clone(),
                        None,
                        None,
                    )
                    .await;
            } else {
                let _ = state
                    .file_service()
                    .upload_system_file(
                        crate::models::file::FileOwnerType::System,
                        uuid::Uuid::nil(),
                        crate::models::file::FileCategory::SystemAsset,
                        &file_name,
                        &mime_type,
                        data.clone(),
                        None,
                    )
                    .await;
            }

            file_path = Some(PathBuf::from(path));
        }
    }

    file_path.ok_or_else(|| AppError::validation(crate::utils::i18n::I18nKey::UploadNoFile))
}
