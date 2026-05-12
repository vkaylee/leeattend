use crate::models::{
    enums::{ImportJobStatus, ImportTargetType},
    AttendanceImportJob,
};
use chrono::{DateTime, Utc};
use std::str::FromStr;
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ImportJobEntity {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub user_id: Uuid,
    pub status: String,
    pub total_rows: i32,
    pub processed_rows: i32,
    pub file_path: Option<String>,
    pub error_log: Option<String>,
    pub target_type: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[allow(clippy::match_same_arms)]
impl From<ImportJobEntity> for AttendanceImportJob {
    fn from(entity: ImportJobEntity) -> Self {
        let status = match entity.status.as_str() {
            "pending" => ImportJobStatus::Pending,
            "processing" => ImportJobStatus::Processing,
            "completed" => ImportJobStatus::Completed,
            "failed" => ImportJobStatus::Failed,
            _ => ImportJobStatus::Pending,
        };
        let target_type =
            ImportTargetType::from_str(&entity.target_type).unwrap_or(ImportTargetType::Log);
        Self {
            id: entity.id,
            workspace_id: entity.workspace_id,
            user_id: entity.user_id,
            status,
            total_rows: entity.total_rows,
            processed_rows: entity.processed_rows,
            file_path: entity.file_path,
            error_log: entity.error_log,
            target_type,
            created_at: entity.created_at,
            updated_at: entity.updated_at,
        }
    }
}
