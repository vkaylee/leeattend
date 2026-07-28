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

#[derive(Debug, Clone, Queryable, Identifiable)]
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
            rules: serde_json::from_value(val.rules).unwrap_or_else(|_| AttendanceRules {
                work_unit_minutes: 0,
                rounding_policy: "none".to_string(),
                early_arrival_strategy: crate::models::enums::EarlyArrivalStrategy::Block,
                grace_period_minutes: None,
            }),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attendance_policy_entity_from() {
        let id = Uuid::new_v4();
        let workspace_id = Uuid::new_v4();
        let now = Utc::now();

        let entity = AttendancePolicyEntity {
            id,
            workspace_id,
            name: "Default Policy".to_string(),
            is_default: true,
            created_at: now,
            updated_at: now,
        };

        let policy: AttendancePolicy = entity.into();
        assert_eq!(policy.id, id);
        assert_eq!(policy.workspace_id, workspace_id);
        assert_eq!(policy.name, "Default Policy");
        assert!(policy.is_default);
        assert_eq!(policy.created_at, now);
        assert_eq!(policy.updated_at, now);
    }

    #[test]
    fn test_attendance_policy_version_entity_from_valid_json() {
        let id = Uuid::new_v4();
        let policy_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let now = Utc::now();

        let rules_json = serde_json::json!({
            "work_unit_minutes": 15,
            "rounding_policy": "round_down",
            "early_arrival_strategy": "block"
        });

        let entity = AttendancePolicyVersionEntity {
            id,
            policy_id,
            rules: rules_json,
            valid_from: now,
            valid_to: None,
            created_by: Some(user_id),
            created_at: now,
        };

        let version: AttendancePolicyVersion = entity.into();
        assert_eq!(version.id, id);
        assert_eq!(version.policy_id, policy_id);
        assert_eq!(version.rules.work_unit_minutes, 15);
        assert_eq!(version.rules.rounding_policy, "round_down");
        assert_eq!(version.created_by, Some(user_id));
        assert_eq!(version.created_at, now);
    }

    #[test]
    fn test_attendance_policy_version_entity_from_invalid_json() {
        let id = Uuid::new_v4();
        let policy_id = Uuid::new_v4();
        let now = Utc::now();

        // Missing required fields
        let rules_json = serde_json::json!({});

        let entity = AttendancePolicyVersionEntity {
            id,
            policy_id,
            rules: rules_json,
            valid_from: now,
            valid_to: None,
            created_by: None,
            created_at: now,
        };

        let version: AttendancePolicyVersion = entity.into();
        assert_eq!(version.id, id);
        assert_eq!(version.rules.work_unit_minutes, 0);
        assert_eq!(version.rules.rounding_policy, "none");
    }

    #[test]
    fn test_attendance_policy_assignment_entity_from() {
        let id = Uuid::new_v4();
        let policy_id = Uuid::new_v4();
        let target_id = Uuid::new_v4();
        let now = Utc::now();

        let entity = AttendancePolicyAssignmentEntity {
            id,
            policy_id,
            target_type: "employee".to_string(),
            target_id,
            valid_from: now,
            valid_to: Some(now),
            created_at: now,
            updated_at: now,
        };

        let assignment: AttendancePolicyAssignment = entity.into();
        assert_eq!(assignment.id, id);
        assert_eq!(assignment.policy_id, policy_id);
        assert_eq!(assignment.target_type, "employee");
        assert_eq!(assignment.target_id, target_id);
        assert_eq!(assignment.valid_to, Some(now));
    }
}
