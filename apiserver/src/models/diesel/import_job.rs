use crate::models::{
    enums::{ImportJobStatus, ImportTargetType},
    AttendanceImportJob,
};
use chrono::{DateTime, Utc};
use std::str::FromStr;
use uuid::Uuid;

#[derive(Debug, Clone, diesel::QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ImportJobEntity {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    pub id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    pub workspace_id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    pub user_id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub status: String,
    #[diesel(sql_type = diesel::sql_types::Integer)]
    pub total_rows: i32,
    #[diesel(sql_type = diesel::sql_types::Integer)]
    pub processed_rows: i32,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub file_path: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub error_log: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub target_type: String,
    #[diesel(sql_type = diesel::sql_types::Timestamptz)]
    pub created_at: DateTime<Utc>,
    #[diesel(sql_type = diesel::sql_types::Timestamptz)]
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
