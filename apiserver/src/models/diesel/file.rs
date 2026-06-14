use crate::models::file::{FileCategory, FileOwnerType, ScanStatus, SystemFile, WorkspaceFile};
use crate::schema::{system_files, workspace_files};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use uuid::Uuid;

#[derive(
    Debug,
    Clone,
    Queryable,
    Selectable,
    Insertable,
    AsChangeset,
    Identifiable,
    Serialize,
    Deserialize,
)]
#[diesel(table_name = workspace_files)]
pub struct WorkspaceFileEntity {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub file_name: String,
    pub storage_key: String,
    pub file_size: i64,
    pub mime_type: String,
    pub content_hash: Option<String>,
    pub category: String,
    pub folder_id: Option<Uuid>,
    pub owner_type: String,
    pub owner_id: Uuid,
    pub uploaded_by: Option<Uuid>,
    pub is_active: bool,
    pub expires_at: Option<DateTime<Utc>>,
    pub scan_status: String,
    pub scanned_at: Option<DateTime<Utc>>,
    pub version: i32,
    pub parent_file_id: Option<Uuid>,
    pub is_latest: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<WorkspaceFileEntity> for WorkspaceFile {
    fn from(entity: WorkspaceFileEntity) -> Self {
        Self {
            id: entity.id,
            workspace_id: entity.workspace_id,
            file_name: entity.file_name,
            storage_key: entity.storage_key,
            file_size: entity.file_size,
            mime_type: entity.mime_type,
            content_hash: entity.content_hash,
            category: FileCategory::from_str(&entity.category).unwrap_or(FileCategory::EmployeeDoc),
            folder_id: entity.folder_id,
            owner_type: FileOwnerType::from_str(&entity.owner_type)
                .unwrap_or(FileOwnerType::Employee),
            owner_id: entity.owner_id,
            uploaded_by: entity.uploaded_by,
            is_active: entity.is_active,
            expires_at: entity.expires_at,
            scan_status: ScanStatus::from_str(&entity.scan_status).unwrap_or(ScanStatus::Pending),
            scanned_at: entity.scanned_at,
            version: entity.version,
            parent_file_id: entity.parent_file_id,
            is_latest: entity.is_latest,
            created_at: entity.created_at,
            updated_at: entity.updated_at,
        }
    }
}

#[derive(
    Debug,
    Clone,
    Queryable,
    Selectable,
    Insertable,
    AsChangeset,
    Identifiable,
    Serialize,
    Deserialize,
)]
#[diesel(table_name = system_files)]
pub struct SystemFileEntity {
    pub id: Uuid,
    pub file_name: String,
    pub storage_key: String,
    pub file_size: i64,
    pub mime_type: String,
    pub content_hash: Option<String>,
    pub category: String,
    pub owner_type: String,
    pub owner_id: Uuid,
    pub uploaded_by: Option<Uuid>,
    pub is_active: bool,
    pub scan_status: String,
    pub version: i32,
    pub parent_file_id: Option<Uuid>,
    pub is_latest: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<SystemFileEntity> for SystemFile {
    fn from(entity: SystemFileEntity) -> Self {
        Self {
            id: entity.id,
            file_name: entity.file_name,
            storage_key: entity.storage_key,
            file_size: entity.file_size,
            mime_type: entity.mime_type,
            content_hash: entity.content_hash,
            category: FileCategory::from_str(&entity.category).unwrap_or(FileCategory::SystemAsset),
            owner_type: FileOwnerType::from_str(&entity.owner_type)
                .unwrap_or(FileOwnerType::System),
            owner_id: entity.owner_id,
            uploaded_by: entity.uploaded_by,
            is_active: entity.is_active,
            scan_status: ScanStatus::from_str(&entity.scan_status).unwrap_or(ScanStatus::Pending),
            version: entity.version,
            parent_file_id: entity.parent_file_id,
            is_latest: entity.is_latest,
            created_at: entity.created_at,
            updated_at: entity.updated_at,
        }
    }
}
