use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct OvertimePolicy {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OvertimeRule {
    pub id: Uuid,
    pub policy_id: Uuid,
    pub priority: i32,
    pub multiplier: rust_decimal::Decimal,
    pub time_start: Option<NaiveTime>,
    pub time_end: Option<NaiveTime>,
    pub days_of_week: i32,
    pub is_holiday: bool,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OvertimePolicyAssignment {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub assignee_type: String,
    pub assignee_id: Option<Uuid>,
    pub policy_id: Uuid,
    pub effective_from: NaiveDate,
    pub effective_to: Option<NaiveDate>,
    pub created_at: Option<DateTime<Utc>>,
}
