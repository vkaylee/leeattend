use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Department {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub manager_id: Option<Uuid>,
    pub default_working_pattern_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub active_employee_count: i64,
}

#[derive(Debug, Deserialize)]
pub struct CreateDepartmentRequest {
    pub name: String,
    pub description: Option<String>,
    pub manager_id: Option<Uuid>,
    pub default_working_pattern_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateDepartmentRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub manager_id: Option<Uuid>,
    #[serde(
        default,
        deserialize_with = "crate::utils::serde_helpers::deserialize_option_option"
    )]
    pub default_working_pattern_id: Option<Option<Uuid>>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct EmployeeDepartmentHistory {
    pub id: Uuid,
    pub employee_id: Uuid,
    pub department_id: Uuid,
    pub effective_date: NaiveDate,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct TransferEmployeeRequest {
    pub department_id: Uuid,
    pub effective_date: NaiveDate,
    pub notes: Option<String>,
}
