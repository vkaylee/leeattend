use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::models::enums::{ContractStatus, ContractType};

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct EmployeeContract {
    id: Uuid,
    workspace_id: Uuid,
    employee_id: Uuid,
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
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: Uuid,
        workspace_id: Uuid,
        employee_id: Uuid,
        contract_number: Option<String>,
        contract_type: ContractType,
        status: ContractStatus,
        start_date: NaiveDate,
        end_date: Option<NaiveDate>,
        document_id: Option<Uuid>,
        notes: Option<String>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            workspace_id,
            employee_id,
            contract_number,
            contract_type,
            status,
            start_date,
            end_date,
            document_id,
            notes,
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
