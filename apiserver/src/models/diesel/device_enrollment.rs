use crate::models::device_enrollment::{DeviceEnrollment, SyncStatus};
use crate::schema::device_enrollments;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use std::str::FromStr;
use uuid::Uuid;

#[derive(Debug, Queryable, Selectable, Identifiable)]
#[diesel(table_name = device_enrollments)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DeviceEnrollmentEntity {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub employee_id: Uuid,
    pub device_id: Uuid,
    pub device_uid: Option<i32>,
    pub sync_status: String,
    pub last_synced_at: Option<DateTime<Utc>>,
    pub last_error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<DeviceEnrollmentEntity> for DeviceEnrollment {
    fn from(entity: DeviceEnrollmentEntity) -> Self {
        // Any value that fails to parse is treated as Failed so a corrupt row
        // never crashes a read path; the state machine only ever writes valid
        // values, so this is defensive-only.
        let sync_status = SyncStatus::from_str(&entity.sync_status).unwrap_or(SyncStatus::Failed);
        Self {
            id: entity.id,
            workspace_id: entity.workspace_id,
            employee_id: entity.employee_id,
            device_id: entity.device_id,
            device_uid: entity.device_uid,
            sync_status,
            last_synced_at: entity.last_synced_at,
            last_error: entity.last_error,
            created_at: entity.created_at,
            updated_at: entity.updated_at,
        }
    }
}

#[derive(Debug, Insertable)]
#[diesel(table_name = device_enrollments)]
pub struct NewDeviceEnrollment {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub employee_id: Uuid,
    pub device_id: Uuid,
    pub sync_status: String,
}

#[derive(Debug, AsChangeset)]
#[diesel(table_name = device_enrollments)]
pub struct UpdateDeviceEnrollment {
    pub device_uid: Option<i32>,
    pub sync_status: Option<String>,
    pub last_synced_at: Option<DateTime<Utc>>,
    pub last_error: Option<Option<String>>,
    pub updated_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entity_with_status(status: &str) -> DeviceEnrollmentEntity {
        DeviceEnrollmentEntity {
            id: Uuid::nil(),
            workspace_id: Uuid::nil(),
            employee_id: Uuid::nil(),
            device_id: Uuid::nil(),
            device_uid: None,
            sync_status: status.to_string(),
            last_synced_at: None,
            last_error: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn valid_status_parses_to_variant() {
        let dto: DeviceEnrollment = entity_with_status("pending_removal").into();
        assert_eq!(dto.sync_status, SyncStatus::PendingRemoval);
    }

    #[test]
    fn unrecognized_status_falls_back_to_failed() {
        let dto: DeviceEnrollment = entity_with_status("garbage_value").into();
        assert_eq!(dto.sync_status, SyncStatus::Failed);
    }
}
