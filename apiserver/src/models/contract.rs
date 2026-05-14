use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::models::enums::{ContractStatus, ContractType};

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema, sqlx::FromRow)]
pub struct EmployeeContract {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub employee_id: Uuid,
    pub contract_number: Option<String>,
    pub contract_type: ContractType,
    pub status: ContractStatus,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub document_id: Option<Uuid>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct CreateContractRequest {
    pub employee_id: Uuid,
    pub contract_number: Option<String>,
    pub contract_type: ContractType,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub document_id: Option<Uuid>,
    pub notes: Option<String>,
    pub assign_phase_id: Option<Uuid>,
    pub auto_close_overlapping: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct UpdateContractRequest {
    pub contract_number: Option<String>,
    pub contract_type: Option<ContractType>,
    pub status: Option<ContractStatus>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub document_id: Option<Uuid>,
    pub notes: Option<String>,
}
