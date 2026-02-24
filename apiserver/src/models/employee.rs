use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct EmploymentPeriod {
    pub id: Uuid,
    pub employee_id: Uuid,
    pub hire_date: NaiveDate,
    pub termination_date: Option<NaiveDate>,
    pub is_rehire: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct Employee {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub user_id: Option<Uuid>,
    pub full_name: String,
    pub employee_id_number: String,
    pub hire_date: NaiveDate, // Fetched from latest employment_period
    pub termination_date: Option<NaiveDate>, // Fetched from latest employment_period
    pub tags: Vec<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AssignmentRuleConditions {
    pub department_id: Option<Uuid>,
    pub employee_id_numbers: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct AssignmentRule {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub pattern_id: Uuid,
    pub name: String,
    pub priority: i32,
    pub conditions: sqlx::types::Json<AssignmentRuleConditions>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateAssignmentRuleRequest {
    pub pattern_id: Uuid,
    pub name: String,
    pub priority: i32,
    pub conditions: AssignmentRuleConditions,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateAssignmentRuleRequest {
    pub pattern_id: Option<Uuid>,
    pub name: Option<String>,
    pub priority: Option<i32>,
    pub conditions: Option<AssignmentRuleConditions>,
    pub is_active: Option<bool>,
    pub max_concurrent_workers: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RehireEmployeeRequest {
    pub hire_date: NaiveDate,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateEmployeeRequest {
    pub user_id: Option<Uuid>,
    pub full_name: String,
    pub employee_id_number: String,
    pub hire_date: NaiveDate,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateEmployeeRequest {
    pub user_id: Option<Uuid>,
    pub full_name: Option<String>,
    pub employee_id_number: Option<String>,
    pub hire_date: Option<NaiveDate>,
    pub termination_date: Option<NaiveDate>,
    pub tags: Option<Vec<String>>,
    pub is_active: Option<bool>,
    pub max_concurrent_workers: Option<i32>,
}
