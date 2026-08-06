use std::path::PathBuf;

use axum::body::Bytes;
use axum::extract::multipart::{Field, MultipartError};
use axum::extract::Multipart;
use axum::http::StatusCode;
use futures_util::StreamExt;

use crate::errors::{AppError, AppResult};
use crate::utils::i18n::I18nKey;

/// Maximum file size for uploads (10MB).
pub const MAX_UPLOAD_FILE_SIZE: usize = 10 * 1024 * 1024;

/// Convert a multipart read failure into an [`AppError`], preserving a
/// size-limit rejection as HTTP 413 instead of collapsing it to 400.
///
/// When the transport-level `DefaultBodyLimit` trips, multer reports a
/// length-limit failure and [`MultipartError::status`] resolves to
/// `PAYLOAD_TOO_LARGE`. Mapping everything to `validation` would hide that
/// distinction from clients, who need 413 to know the upload was too big
/// rather than malformed.
pub fn multipart_error_to_app_error(err: &MultipartError, fallback: I18nKey) -> AppError {
    if err.status() == StatusCode::PAYLOAD_TOO_LARGE {
        AppError::payload_too_large(I18nKey::UploadFileTooLarge)
    } else {
        AppError::validation(fallback)
    }
}

/// Read a multipart field body while enforcing `max_size` incrementally.
///
/// The field is consumed chunk by chunk and the accumulator is checked after
/// every chunk, so a request that exceeds the limit is rejected as soon as the
/// threshold is crossed — the remaining bytes are never buffered. This bounds
/// peak memory to roughly `max_size` plus one chunk regardless of how much the
/// client claims to be sending.
///
/// The transport-level cap (`DefaultBodyLimit`, configured per route from
/// `MAX_REQUEST_BODY_BYTES` / `MAX_*_UPLOAD_BODY_BYTES`) is the outer defence;
/// this function is the business-level limit that produces a translated
/// [`I18nKey::UploadFileTooLarge`] instead of an opaque multipart error.
pub async fn read_field_limited(mut field: Field<'_>, max_size: usize) -> AppResult<Bytes> {
    let mut buf = Vec::new();

    while let Some(chunk) = field.next().await {
        let chunk =
            chunk.map_err(|e| multipart_error_to_app_error(&e, I18nKey::UploadReadFailed))?;

        // Check before extending so an oversized chunk cannot push the
        // accumulator past the limit even transiently.
        if buf.len() + chunk.len() > max_size {
            return Err(AppError::payload_too_large(I18nKey::UploadFileTooLarge));
        }
        buf.extend_from_slice(&chunk);
    }

    Ok(Bytes::from(buf))
}

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
        .map_err(|e| multipart_error_to_app_error(&e, I18nKey::UploadMultipart))?
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
                return Err(AppError::validation(I18nKey::InvalidFileFormat));
            }

            // Size is enforced while streaming — an oversized upload is
            // rejected before the whole body is buffered.
            let data = read_field_limited(field, max_size).await?;

            // Write to local disk for backward compatibility (Calamine parsing)
            let base_dir = crate::utils::get_storage_dir();
            let dir = format!("{base_dir}/{sub_dir}");
            tokio::fs::create_dir_all(&dir).await?;
            let path = format!("{}/{}_{}", dir, crate::utils::id::new_id(), file_name);
            tokio::fs::write(&path, &data).await?;

            // Delegate to FileService (S3) instead of legacy direct upload.
            // `Bytes` is reference-counted, so passing it on is a refcount
            // bump, not a copy of the payload.
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
                        data,
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
                        data,
                        None,
                    )
                    .await;
            }

            file_path = Some(PathBuf::from(path));
        }
    }

    file_path.ok_or_else(|| AppError::validation(I18nKey::UploadNoFile))
}
