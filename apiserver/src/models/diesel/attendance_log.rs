use chrono::{DateTime, Utc};
use diesel::deserialize::{self, FromSql};
use diesel::pg::{Pg, PgValue};
use diesel::prelude::*;
use diesel::serialize::{self, IsNull, Output, ToSql};
use std::io::Write;
use uuid::Uuid;

use crate::models::attendance::RawAttendanceLog;
use crate::models::enums::LogSource;
use crate::schema::sql_types::LogSourceType;
use crate::schema::{attendance_log_errors, attendance_staging_logs, raw_attendance_logs};

impl ToSql<LogSourceType, Pg> for LogSource {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        match *self {
            LogSource::Device => out.write_all(b"device")?,
            LogSource::Import => out.write_all(b"import")?,
            LogSource::Manual => out.write_all(b"manual")?,
        }
        Ok(IsNull::No)
    }
}

impl FromSql<LogSourceType, Pg> for LogSource {
    fn from_sql(bytes: PgValue<'_>) -> deserialize::Result<Self> {
        match bytes.as_bytes() {
            b"device" => Ok(LogSource::Device),
            b"import" => Ok(LogSource::Import),
            b"manual" => Ok(LogSource::Manual),
            _ => Err("Unrecognized enum variant".into()),
        }
    }
}

// ---------------------------------------------------------------------------
// Raw Attendance Log
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Queryable, Selectable, Insertable, Identifiable, AsChangeset)]
#[diesel(table_name = raw_attendance_logs)]
pub struct RawAttendanceLogEntity {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub employee_id: Uuid,
    pub punch_time: DateTime<Utc>,
    pub source: LogSource,
    pub device_id: Option<String>,
    pub raw_data: Option<serde_json::Value>,
    pub created_at: Option<DateTime<Utc>>,
    pub punch: i32,
    pub status: i32,
}

impl From<RawAttendanceLogEntity> for RawAttendanceLog {
    fn from(val: RawAttendanceLogEntity) -> Self {
        RawAttendanceLog {
            id: val.id,
            workspace_id: val.workspace_id,
            employee_id: val.employee_id,
            punch_time: val.punch_time,
            source: val.source,
            device_id: val.device_id,
            raw_data: val.raw_data,
            punch: val.punch,
            status: val.status,
            created_at: val.created_at.unwrap_or_else(Utc::now),
        }
    }
}

// ---------------------------------------------------------------------------
// Attendance Staging Log
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Queryable, Selectable, Insertable, Identifiable, AsChangeset)]
#[diesel(table_name = attendance_staging_logs)]
#[diesel(primary_key(id, created_at))]
pub struct AttendanceStagingLogEntity {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub device_id: Option<String>,
    pub raw_user_id: String,
    pub punch_time: DateTime<Utc>,
    pub raw_data: Option<serde_json::Value>,
    pub fingerprint: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// Attendance Log Error
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Queryable, Selectable, Insertable, Identifiable, AsChangeset)]
#[diesel(table_name = attendance_log_errors)]
pub struct AttendanceLogErrorEntity {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub agent_id: Option<String>,
    pub raw_content: Option<serde_json::Value>,
    pub error_message: String,
    pub source: String,
    pub created_at: Option<DateTime<Utc>>,
    pub fingerprint: Option<String>,
}
