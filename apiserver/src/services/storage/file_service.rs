use bytes::Bytes;
use sha2::{Digest, Sha256};
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

use crate::diesel_pool::TenantAuthToken;
use crate::errors::{AppError, AppResult};
use crate::models::file::{
    FileCategory, FileDownloadResponse, FileOwnerType, ScanStatus, SystemFile, WorkspaceFile,
};
use crate::repositories::FileRepository;
use crate::services::storage::{
    key_generator::{generate_storage_key, generate_system_key},
    StorageResolver,
};

pub struct FileService {
    storage_resolver: Arc<StorageResolver>,
    file_repository: Arc<dyn FileRepository>,
    file_proxy_threshold_bytes: i64,
    presigned_url_ttl: Duration,
}

impl FileService {
    pub fn new(
        storage_resolver: Arc<StorageResolver>,
        file_repository: Arc<dyn FileRepository>,
    ) -> Self {
        let file_proxy_threshold_bytes = std::env::var("FILE_PROXY_THRESHOLD_BYTES")
            .map(|v| v.parse::<i64>().unwrap_or(1024 * 1024))
            .unwrap_or(1024 * 1024); // default 1MB

        let presigned_url_ttl_secs = std::env::var("PRESIGNED_URL_TTL_SECS")
            .map(|v| v.parse::<u64>().unwrap_or(900))
            .unwrap_or(900); // default 15 minutes

        Self {
            storage_resolver,
            file_repository,
            file_proxy_threshold_bytes,
            presigned_url_ttl: Duration::from_secs(presigned_url_ttl_secs),
        }
    }

    /// Upload a workspace file
    pub async fn upload_file(
        &self,
        auth_token: &TenantAuthToken,
        owner_type: FileOwnerType,
        owner_id: Uuid,
        category: FileCategory,
        file_name: &str,
        mime_type: &str,
        data: Bytes,
        uploaded_by: Option<Uuid>,
        expires_in_secs: Option<i64>,
    ) -> AppResult<WorkspaceFile> {
        let size = data.len() as i64;
        validate_file(category, size, mime_type)?;

        let content_hash = compute_sha256(&data);
        let storage_key =
            generate_storage_key(auth_token.workspace_id(), category.as_ref(), file_name);

        let storage = self
            .storage_resolver
            .resolve(auth_token.workspace_id())
            .await?;
        storage.put_object(&storage_key, data, mime_type).await?;

        let expires_at = expires_in_secs.map(|s| chrono::Utc::now() + chrono::Duration::seconds(s));

        let file = WorkspaceFile {
            id: Uuid::new_v4(),
            workspace_id: auth_token.workspace_id(),
            file_name: file_name.to_string(),
            storage_key,
            file_size: size,
            mime_type: mime_type.to_string(),
            content_hash: Some(content_hash),
            category,
            folder_id: None,
            owner_type,
            owner_id,
            uploaded_by,
            is_active: true,
            expires_at,
            scan_status: ScanStatus::Pending,
            scanned_at: None,
            version: 1,
            parent_file_id: None,
            is_latest: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        self.file_repository
            .insert_workspace_file(auth_token, &file)
            .await?;

        Ok(file)
    }

    /// Upload a system file (global scope)
    pub async fn upload_system_file(
        &self,
        owner_type: FileOwnerType,
        owner_id: Uuid,
        category: FileCategory,
        file_name: &str,
        mime_type: &str,
        data: Bytes,
        uploaded_by: Option<Uuid>,
    ) -> AppResult<SystemFile> {
        let size = data.len() as i64;
        validate_file(category, size, mime_type)?;

        let content_hash = compute_sha256(&data);
        let storage_key = generate_system_key(category.as_ref(), file_name);

        let storage = self.storage_resolver.central();
        storage.put_object(&storage_key, data, mime_type).await?;

        let file = SystemFile {
            id: Uuid::new_v4(),
            file_name: file_name.to_string(),
            storage_key,
            file_size: size,
            mime_type: mime_type.to_string(),
            content_hash: Some(content_hash),
            category,
            owner_type,
            owner_id,
            uploaded_by,
            is_active: true,
            scan_status: ScanStatus::Pending,
            version: 1,
            parent_file_id: None,
            is_latest: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        self.file_repository.insert_system_file(&file).await?;

        Ok(file)
    }

    /// Download a workspace file
    pub async fn download_file(
        &self,
        auth_token: &TenantAuthToken,
        file_id: Uuid,
    ) -> AppResult<FileDownloadResponse> {
        let metadata = self.get_file_metadata(auth_token, file_id).await?;
        let storage = self
            .storage_resolver
            .resolve(auth_token.workspace_id())
            .await?;

        if metadata.file_size <= self.file_proxy_threshold_bytes {
            let bytes = storage.get_object(&metadata.storage_key).await?;
            Ok(FileDownloadResponse::ProxyBytes(bytes.to_vec()))
        } else {
            let url = storage
                .presigned_get_url(&metadata.storage_key, self.presigned_url_ttl)
                .await?;
            Ok(FileDownloadResponse::PresignedUrl(url))
        }
    }

    /// Download a system file
    pub async fn download_system_file(&self, file_id: Uuid) -> AppResult<FileDownloadResponse> {
        let metadata = self.get_system_file_metadata(file_id).await?;
        let storage = self.storage_resolver.central();

        if metadata.file_size <= self.file_proxy_threshold_bytes {
            let bytes = storage.get_object(&metadata.storage_key).await?;
            Ok(FileDownloadResponse::ProxyBytes(bytes.to_vec()))
        } else {
            let url = storage
                .presigned_get_url(&metadata.storage_key, self.presigned_url_ttl)
                .await?;
            Ok(FileDownloadResponse::PresignedUrl(url))
        }
    }

    /// Get direct presigned download URL for workspace file
    pub async fn get_download_url(
        &self,
        auth_token: &TenantAuthToken,
        file_id: Uuid,
    ) -> AppResult<String> {
        let metadata = self.get_file_metadata(auth_token, file_id).await?;
        let storage = self
            .storage_resolver
            .resolve(auth_token.workspace_id())
            .await?;
        storage
            .presigned_get_url(&metadata.storage_key, self.presigned_url_ttl)
            .await
    }

    /// Pre-register a workspace file and generate a presigned PUT URL for direct upload
    pub async fn create_upload_url(
        &self,
        auth_token: &TenantAuthToken,
        owner_type: FileOwnerType,
        owner_id: Uuid,
        category: FileCategory,
        file_name: &str,
        mime_type: &str,
        file_size: i64,
        uploaded_by: Option<Uuid>,
        expires_in_secs: Option<i64>,
    ) -> AppResult<(WorkspaceFile, String)> {
        validate_file(category, file_size, mime_type)?;

        let storage_key = crate::services::storage::key_generator::generate_storage_key(
            auth_token.workspace_id(),
            category.as_ref(),
            file_name,
        );

        let file = WorkspaceFile {
            id: Uuid::new_v4(),
            workspace_id: auth_token.workspace_id(),
            file_name: file_name.to_string(),
            storage_key: storage_key.clone(),
            file_size,
            mime_type: mime_type.to_string(),
            content_hash: None,
            category,
            folder_id: None,
            owner_type,
            owner_id,
            uploaded_by,
            is_active: true,
            expires_at: expires_in_secs.map(|s| chrono::Utc::now() + chrono::Duration::seconds(s)),
            scan_status: crate::models::file::ScanStatus::Pending,
            scanned_at: None,
            version: 1,
            parent_file_id: None,
            is_latest: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        self.file_repository
            .insert_workspace_file(auth_token, &file)
            .await?;

        let storage = self
            .storage_resolver
            .resolve(auth_token.workspace_id())
            .await?;
        let url = storage
            .presigned_put_url(&storage_key, mime_type, self.presigned_url_ttl)
            .await?;

        Ok((file, url))
    }

    /// Get direct presigned download URL for system file
    pub async fn get_system_download_url(&self, file_id: Uuid) -> AppResult<String> {
        let metadata = self.get_system_file_metadata(file_id).await?;
        let storage = self.storage_resolver.central();
        storage
            .presigned_get_url(&metadata.storage_key, self.presigned_url_ttl)
            .await
    }

    /// Directly download workspace file bytes (bypass proxy threshold, internal use)
    pub async fn download_file_direct(
        &self,
        auth_token: &TenantAuthToken,
        file_id: Uuid,
    ) -> AppResult<Bytes> {
        let metadata = self.get_file_metadata(auth_token, file_id).await?;
        let storage = self
            .storage_resolver
            .resolve(auth_token.workspace_id())
            .await?;
        storage.get_object(&metadata.storage_key).await
    }

    /// Directly download system file bytes (bypass proxy threshold, internal use)
    pub async fn download_system_file_direct(&self, file_id: Uuid) -> AppResult<Bytes> {
        let metadata = self.get_system_file_metadata(file_id).await?;
        let storage = self.storage_resolver.central();
        storage.get_object(&metadata.storage_key).await
    }

    /// Soft delete workspace file
    pub async fn delete_file(&self, auth_token: &TenantAuthToken, file_id: Uuid) -> AppResult<()> {
        self.file_repository
            .soft_delete_workspace_file(auth_token, file_id)
            .await
    }

    /// Soft delete system file
    pub async fn delete_system_file(&self, file_id: Uuid) -> AppResult<()> {
        self.file_repository.soft_delete_system_file(file_id).await
    }

    /// CoW version update for workspace file
    pub async fn update_file(
        &self,
        auth_token: &TenantAuthToken,
        file_id: Uuid,
        file_name: &str,
        mime_type: &str,
        data: Bytes,
        uploaded_by: Option<Uuid>,
    ) -> AppResult<WorkspaceFile> {
        let current_file = self.get_file_metadata(auth_token, file_id).await?;

        let size = data.len() as i64;
        validate_file(current_file.category, size, mime_type)?;

        let content_hash = compute_sha256(&data);

        // Phase 3: CoW Short-Circuit Optimization
        if let Some(ref old_hash) = current_file.content_hash {
            if old_hash == &content_hash {
                tracing::info!(
                    "File content is identical (Hash: {}), bypassing upload.",
                    content_hash
                );
                return Ok(current_file);
            }
        }

        let storage_key = generate_storage_key(
            auth_token.workspace_id(),
            current_file.category.as_ref(),
            file_name,
        );

        let storage = self
            .storage_resolver
            .resolve(auth_token.workspace_id())
            .await?;
        storage.put_object(&storage_key, data, mime_type).await?;

        let new_file = WorkspaceFile {
            id: Uuid::new_v4(),
            workspace_id: auth_token.workspace_id(),
            file_name: file_name.to_string(),
            storage_key,
            file_size: size,
            mime_type: mime_type.to_string(),
            content_hash: Some(content_hash),
            category: current_file.category,
            folder_id: current_file.folder_id,
            owner_type: current_file.owner_type,
            owner_id: current_file.owner_id,
            uploaded_by,
            is_active: true,
            expires_at: current_file.expires_at,
            scan_status: ScanStatus::Pending,
            scanned_at: None,
            version: current_file.version + 1,
            parent_file_id: Some(current_file.id),
            is_latest: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        self.file_repository
            .create_new_version(auth_token, current_file.id, &new_file)
            .await?;

        Ok(new_file)
    }

    /// CoW version update for system file
    pub async fn update_system_file(
        &self,
        file_id: Uuid,
        file_name: &str,
        mime_type: &str,
        data: Bytes,
        uploaded_by: Option<Uuid>,
    ) -> AppResult<SystemFile> {
        let current_file = self.get_system_file_metadata(file_id).await?;

        let size = data.len() as i64;
        validate_file(current_file.category, size, mime_type)?;

        let content_hash = compute_sha256(&data);

        // Phase 3: CoW Short-Circuit Optimization
        if let Some(ref old_hash) = current_file.content_hash {
            if old_hash == &content_hash {
                tracing::info!(
                    "System file content is identical (Hash: {}), bypassing upload.",
                    content_hash
                );
                return Ok(current_file);
            }
        }

        let storage_key = generate_system_key(current_file.category.as_ref(), file_name);

        let storage = self.storage_resolver.central();
        storage.put_object(&storage_key, data, mime_type).await?;

        let new_file = SystemFile {
            id: Uuid::new_v4(),
            file_name: file_name.to_string(),
            storage_key,
            file_size: size,
            mime_type: mime_type.to_string(),
            content_hash: Some(content_hash),
            category: current_file.category,
            owner_type: current_file.owner_type,
            owner_id: current_file.owner_id,
            uploaded_by,
            is_active: true,
            scan_status: ScanStatus::Pending,
            version: current_file.version + 1,
            parent_file_id: Some(current_file.id),
            is_latest: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        self.file_repository
            .create_new_system_version(current_file.id, &new_file)
            .await?;

        Ok(new_file)
    }

    /// Get all versions of a workspace file
    pub async fn get_file_versions(
        &self,
        auth_token: &TenantAuthToken,
        file_id: Uuid,
    ) -> AppResult<Vec<WorkspaceFile>> {
        let mut versions = Vec::new();
        let mut current_id = Some(file_id);

        while let Some(id) = current_id {
            if let Some(file) = self
                .file_repository
                .get_workspace_file_by_id(auth_token, id)
                .await?
            {
                current_id = file.parent_file_id;
                versions.push(file);
            } else {
                break;
            }
        }

        Ok(versions)
    }

    /// Get workspace file metadata
    pub async fn get_file_metadata(
        &self,
        auth_token: &TenantAuthToken,
        file_id: Uuid,
    ) -> AppResult<WorkspaceFile> {
        self.file_repository
            .get_workspace_file_by_id(auth_token, file_id)
            .await?
            .ok_or_else(|| AppError::not_found_raw(format!("Workspace file not found: {file_id}")))
    }

    /// Get system file metadata
    pub async fn get_system_file_metadata(&self, file_id: Uuid) -> AppResult<SystemFile> {
        self.file_repository
            .get_system_file_by_id(file_id)
            .await?
            .ok_or_else(|| AppError::not_found_raw(format!("System file not found: {file_id}")))
    }

    /// List active workspace files by owner
    pub async fn list_files_by_owner(
        &self,
        auth_token: &TenantAuthToken,
        owner_type: FileOwnerType,
        owner_id: Uuid,
    ) -> AppResult<Vec<WorkspaceFile>> {
        self.file_repository
            .list_workspace_files_by_owner(auth_token, owner_type.as_ref(), owner_id)
            .await
    }
}

fn validate_file(category: FileCategory, size: i64, mime_type: &str) -> AppResult<()> {
    let config = category.config();
    if size > config.max_size {
        return Err(AppError::validation_raw(format!(
            "File size {} exceeds maximum allowed limit of {} bytes for category {:?}",
            size, config.max_size, category
        )));
    }

    let allowed = config.allowed_mimes.iter().any(|&pattern| {
        if pattern.ends_with("/*") {
            let prefix = &pattern[..pattern.len() - 1]; // e.g. "image/"
            mime_type.starts_with(prefix)
        } else {
            mime_type == pattern
        }
    });

    if !allowed {
        return Err(AppError::validation_raw(format!(
            "Mime type {mime_type} is not allowed for category {category:?}"
        )));
    }

    Ok(())
}

fn compute_sha256(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    format!("{result:x}")
}
