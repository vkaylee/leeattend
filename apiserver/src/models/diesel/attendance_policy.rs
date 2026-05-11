use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde_json::Value;
use uuid::Uuid;

use crate::models::{
    AttendancePolicy, AttendancePolicyAssignment, AttendancePolicyVersion, AttendanceRules,
};
use crate::schema::{
    attendance_policies, attendance_policy_assignments, attendance_policy_versions,
};

#[derive(Debug, Clone, Queryable, Selectable, Insertable, Identifiable, AsChangeset)]
#[diesel(table_name = attendance_policies)]
pub struct AttendancePolicyEntity {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub name: String,
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<AttendancePolicyEntity> for AttendancePolicy {
    fn from(val: AttendancePolicyEntity) -> Self {
        AttendancePolicy {
            id: val.id,
            workspace_id: val.workspace_id,
            name: val.name,
            is_default: val.is_default,
            created_at: val.created_at,
            updated_at: val.updated_at,
        }
    }
}

#[derive(Debug, Clone, Queryable, Selectable, Insertable, Identifiable, AsChangeset)]
#[diesel(table_name = attendance_policy_versions)]
pub struct AttendancePolicyVersionEntity {
    pub id: Uuid,
    pub policy_id: Uuid,
    pub rules: Value,
    pub valid_from: DateTime<Utc>,
    pub valid_to: Option<DateTime<Utc>>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

impl From<AttendancePolicyVersionEntity> for AttendancePolicyVersion {
    fn from(val: AttendancePolicyVersionEntity) -> Self {
        AttendancePolicyVersion {
            id: val.id,
            policy_id: val.policy_id,
            rules: sqlx::types::Json(serde_json::from_value(val.rules).unwrap_or(
                AttendanceRules {
                    work_unit_minutes: 0,
                    rounding_policy: "none".to_string(),
                    early_arrival_strategy: crate::models::enums::EarlyArrivalStrategy::Block,
                },
            )),
            valid_from: val.valid_from,
            valid_to: val.valid_to,
            created_by: val.created_by,
            created_at: val.created_at,
        }
    }
}

#[derive(Debug, Clone, Queryable, Selectable, Insertable, Identifiable, AsChangeset)]
#[diesel(table_name = attendance_policy_assignments)]
pub struct AttendancePolicyAssignmentEntity {
    pub id: Uuid,
    pub policy_id: Uuid,
    pub target_type: String,
    pub target_id: Uuid,
    pub valid_from: DateTime<Utc>,
    pub valid_to: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<AttendancePolicyAssignmentEntity> for AttendancePolicyAssignment {
    fn from(val: AttendancePolicyAssignmentEntity) -> Self {
        AttendancePolicyAssignment {
            id: val.id,
            policy_id: val.policy_id,
            target_type: val.target_type,
            target_id: val.target_id,
            valid_from: val.valid_from,
            valid_to: val.valid_to,
            created_at: val.created_at,
            updated_at: val.updated_at,
        }
    }
}
