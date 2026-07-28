use chrono::{DateTime, NaiveDate, Utc};
use diesel::prelude::*;
use serde_json::Value;
use uuid::Uuid;

use crate::domain::policy::cascade::partial::PartialPolicyRules;
use crate::schema::attendance_exceptions;

#[derive(Debug, Clone, Queryable, Selectable, Insertable, Identifiable)]
#[diesel(table_name = attendance_exceptions)]
pub struct AttendanceExceptionEntity {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub employee_id: Uuid,
    pub policy_version_id: Uuid,
    pub exception_from: NaiveDate,
    pub exception_to: NaiveDate,
    pub contributions: Value,
    pub reason: String,
    pub approved_by: Option<Uuid>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl AttendanceExceptionEntity {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: Uuid,
        workspace_id: Uuid,
        employee_id: Uuid,
        policy_version_id: Uuid,
        exception_from: NaiveDate,
        exception_to: NaiveDate,
        contributions: Value,
        reason: String,
        approved_by: Option<Uuid>,
        created_by: Option<Uuid>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            workspace_id,
            employee_id,
            policy_version_id,
            exception_from,
            exception_to,
            contributions,
            reason,
            approved_by,
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
    pub fn exception_from(&self) -> NaiveDate {
        self.exception_from
    }
    pub fn exception_to(&self) -> NaiveDate {
        self.exception_to
    }
    pub fn contributions_raw(&self) -> &Value {
        &self.contributions
    }
    pub fn reason(&self) -> &str {
        &self.reason
    }
    pub fn approved_by(&self) -> Option<Uuid> {
        self.approved_by
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
