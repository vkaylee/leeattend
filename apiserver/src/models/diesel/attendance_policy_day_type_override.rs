use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde_json::Value;
use uuid::Uuid;

use crate::domain::policy::cascade::partial::PartialPolicyRules;
use crate::schema::attendance_policy_day_type_overrides;

#[derive(Debug, Clone, Queryable, Selectable, Insertable, Identifiable)]
#[diesel(table_name = attendance_policy_day_type_overrides)]
pub struct AttendancePolicyDayTypeOverrideEntity {
    pub id: Uuid,
    pub policy_version_id: Uuid,
    pub workspace_id: Uuid,
    pub day_type: String,
    pub contributions: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl AttendancePolicyDayTypeOverrideEntity {
    pub fn new(
        id: Uuid,
        policy_version_id: Uuid,
        workspace_id: Uuid,
        day_type: String,
        contributions: Value,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            policy_version_id,
            workspace_id,
            day_type,
            contributions,
            created_at,
            updated_at,
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }
    pub fn policy_version_id(&self) -> Uuid {
        self.policy_version_id
    }
    pub fn workspace_id(&self) -> Uuid {
        self.workspace_id
    }
    pub fn day_type(&self) -> &str {
        &self.day_type
    }
    pub fn contributions_raw(&self) -> &Value {
        &self.contributions
    }
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    /// Deserialize contributions JSONB to PartialPolicyRules.
    /// Returns `PartialPolicyRules::default()` (empty) on parse failure — an
    /// invalid override is treated as no contribution rather than blocking eval.
    pub fn contributions(&self) -> PartialPolicyRules {
        serde_json::from_value(self.contributions.clone()).unwrap_or_default()
    }
}
