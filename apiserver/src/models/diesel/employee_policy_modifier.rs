use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde_json::Value;
use uuid::Uuid;

use crate::domain::policy::cascade::partial::PartialPolicyRules;
use crate::schema::employee_policy_modifiers;

#[derive(Debug, Clone, Queryable, Selectable, Insertable, Identifiable)]
#[diesel(table_name = employee_policy_modifiers)]
pub struct EmployeePolicyModifierEntity {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub employee_id: Uuid,
    pub policy_version_id: Uuid,
    pub priority_rank: i32,
    pub effective_from: DateTime<Utc>,
    pub effective_to: Option<DateTime<Utc>>,
    pub contributions: Value,
    pub reason: Option<String>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl EmployeePolicyModifierEntity {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: Uuid,
        workspace_id: Uuid,
        employee_id: Uuid,
        policy_version_id: Uuid,
        priority_rank: i32,
        effective_from: DateTime<Utc>,
        effective_to: Option<DateTime<Utc>>,
        contributions: Value,
        reason: Option<String>,
        created_by: Option<Uuid>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            workspace_id,
            employee_id,
            policy_version_id,
            priority_rank,
            effective_from,
            effective_to,
            contributions,
            reason,
            created_by,
            created_at,
            updated_at,
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }
    pub fn workspace_id(&self) -> Uuid {
        self.workspace_id
    }
    pub fn employee_id(&self) -> Uuid {
        self.employee_id
    }
    pub fn policy_version_id(&self) -> Uuid {
        self.policy_version_id
    }
    pub fn priority_rank(&self) -> i32 {
        self.priority_rank
    }
    pub fn effective_from(&self) -> DateTime<Utc> {
        self.effective_from
    }
    pub fn effective_to(&self) -> Option<DateTime<Utc>> {
        self.effective_to
    }
    pub fn contributions_raw(&self) -> &Value {
        &self.contributions
    }
    pub fn reason(&self) -> Option<&str> {
        self.reason.as_deref()
    }
    pub fn created_by(&self) -> Option<Uuid> {
        self.created_by
    }
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    pub fn contributions(&self) -> PartialPolicyRules {
        serde_json::from_value(self.contributions.clone()).unwrap_or_default()
    }
}
