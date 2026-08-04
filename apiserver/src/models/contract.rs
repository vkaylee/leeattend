use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::models::enums::{ContractStatus, ContractType};

pub struct NewEmployeeContractParams {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub employment_id: Uuid,
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
pub struct EmployeeContract {
    id: Uuid,
    workspace_id: Uuid,
    employment_id: Uuid,
    contract_number: Option<String>,
    contract_type: ContractType,
    status: ContractStatus,
    start_date: NaiveDate,
    end_date: Option<NaiveDate>,
    document_id: Option<Uuid>,
    notes: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl EmployeeContract {
    pub fn new(params: NewEmployeeContractParams) -> Self {
        Self {
            id: params.id,
            workspace_id: params.workspace_id,
            employment_id: params.employment_id,
            contract_number: params.contract_number,
            contract_type: params.contract_type,
            status: params.status,
            start_date: params.start_date,
            end_date: params.end_date,
            document_id: params.document_id,
            notes: params.notes,
            created_at: params.created_at,
            updated_at: params.updated_at,
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn workspace_id(&self) -> Uuid {
        self.workspace_id
    }

    pub fn employment_id(&self) -> Uuid {
        self.employment_id
    }

    pub fn contract_number(&self) -> Option<&str> {
        self.contract_number.as_deref()
    }

    pub fn contract_type(&self) -> ContractType {
        self.contract_type
    }

    pub fn status(&self) -> ContractStatus {
        self.status
    }

    pub fn start_date(&self) -> NaiveDate {
        self.start_date
    }

    pub fn end_date(&self) -> Option<NaiveDate> {
        self.end_date
    }

    pub fn document_id(&self) -> Option<Uuid> {
        self.document_id
    }

    pub fn notes(&self) -> Option<&str> {
        self.notes.as_deref()
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct CreateContractRequest {
    pub employment_id: Uuid,
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

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct EmployeeContractWithEmployee {
    #[serde(flatten)]
    pub contract: EmployeeContract,
    pub employee_name: String,
    pub employee_code: String,
    pub employee_id: Uuid,
}
