use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// Sync state machine for a device enrollment.
///
/// Stored as VARCHAR (with a DB CHECK constraint) rather than a native PG enum,
/// so the entity layer keeps it as `String` and this enum is used at the API
/// boundary. Transitions:
///   pending_push -> synced
///   synced -> pending_removal -> removed
///   any -> failed
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    strum_macros::Display,
    strum_macros::EnumString,
    Serialize,
    Deserialize,
    ToSchema,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum SyncStatus {
    PendingPush,
    Synced,
    PendingRemoval,
    Failed,
    Removed,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct DeviceEnrollment {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub employee_id: Uuid,
    pub device_id: Uuid,
    /// The u16 uid the device assigned to this user; null until first
    /// successful push. Per-device — same employee has different uid on
    /// different devices.
    pub device_uid: Option<i32>,
    pub sync_status: SyncStatus,
    pub last_synced_at: Option<DateTime<Utc>>,
    pub last_error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Enroll an employee onto one or more specific devices. HR picks exact
/// targets — even a single device out of many at the same location.
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct CreateEnrollmentsRequest {
    pub device_ids: Vec<Uuid>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    // The string form of each variant MUST match the values allowed by the
    // device_enrollments.sync_status CHECK constraint in migration
    // 20260719000000_biometric_enrollment. If a variant is renamed without
    // updating the migration, inserts fail at runtime — this test pins the
    // contract so the break surfaces at compile/test time instead.
    #[test]
    fn sync_status_wire_values_match_db_check() {
        let cases = [
            (SyncStatus::PendingPush, "pending_push"),
            (SyncStatus::Synced, "synced"),
            (SyncStatus::PendingRemoval, "pending_removal"),
            (SyncStatus::Failed, "failed"),
            (SyncStatus::Removed, "removed"),
        ];
        for (variant, wire) in cases {
            // strum Display (used when writing to DB as VARCHAR)
            assert_eq!(
                variant.to_string(),
                wire,
                "Display mismatch for {variant:?}"
            );
            // strum FromStr (used when reading String back into the enum)
            assert_eq!(
                SyncStatus::from_str(wire).unwrap(),
                variant,
                "FromStr mismatch for {wire}"
            );
            // serde (used at the API boundary)
            assert_eq!(
                serde_json::to_string(&variant).unwrap(),
                format!("\"{wire}\""),
                "serde mismatch for {variant:?}"
            );
        }
    }

    #[test]
    fn sync_status_from_str_rejects_unknown() {
        assert!(SyncStatus::from_str("bogus").is_err());
    }
}
