use crate::models::{
    enums::{ImportJobStatus, ImportTargetType},
    AttendanceImportJob,
};
use chrono::{DateTime, Utc};
use diesel::deserialize::{self, FromSql};
use diesel::pg::{Pg, PgValue};
use diesel::serialize::{self, IsNull, Output, ToSql};
use std::io::Write;
use std::str::FromStr;
use uuid::Uuid;
use crate::schema::sql_types::ImportJobStatusType;

// ---------------------------------------------------------------------------
// Enum Mapping: ImportJobStatusType
// ---------------------------------------------------------------------------

impl ToSql<ImportJobStatusType, Pg> for ImportJobStatus {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        match *self {
            ImportJobStatus::Pending => out.write_all(b"pending")?,
            ImportJobStatus::Processing => out.write_all(b"processing")?,
            ImportJobStatus::Completed => out.write_all(b"completed")?,
            ImportJobStatus::Failed => out.write_all(b"failed")?,
        }
        Ok(IsNull::No)
    }
}

impl FromSql<ImportJobStatusType, Pg> for ImportJobStatus {
    fn from_sql(bytes: PgValue<'_>) -> deserialize::Result<Self> {
        match bytes.as_bytes() {
            b"pending" => Ok(ImportJobStatus::Pending),
            b"processing" => Ok(ImportJobStatus::Processing),
            b"completed" => Ok(ImportJobStatus::Completed),
            b"failed" => Ok(ImportJobStatus::Failed),
            _ => Err("Unrecognized enum variant".into()),
        }
    }
}

#[derive(Debug, Clone, diesel::QueryableByName, diesel::Queryable, diesel::Selectable, diesel::Insertable)]
#[diesel(table_name = crate::schema::attendance_import_jobs)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ImportJobEntity {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    pub id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    pub workspace_id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    pub user_id: Uuid,
    #[diesel(sql_type = crate::schema::sql_types::ImportJobStatusType)]
    pub status: ImportJobStatus,
    #[diesel(sql_type = diesel::sql_types::Integer)]
    pub total_rows: i32,
    #[diesel(sql_type = diesel::sql_types::Integer)]
    pub processed_rows: i32,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub error_log: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Timestamptz)]
    pub created_at: DateTime<Utc>,
    #[diesel(sql_type = diesel::sql_types::Timestamptz)]
    pub updated_at: DateTime<Utc>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub file_path: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Varchar)]
    pub target_type: String,
}

#[allow(clippy::match_same_arms)]
impl From<ImportJobEntity> for AttendanceImportJob {
    fn from(entity: ImportJobEntity) -> Self {
        let status = entity.status;
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
