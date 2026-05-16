use async_trait::async_trait;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::contract::{CreateContractRequest, EmployeeContract, UpdateContractRequest};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait EmployeeContractRepository: Send + Sync {
    async fn create_contract(
        &self,
        workspace_id_val: Uuid,
        req: &CreateContractRequest,
    ) -> Result<EmployeeContract, AppError>;

    async fn get_contract_by_id(
        &self,
        workspace_id_val: Uuid,
        contract_id: Uuid,
    ) -> Result<Option<EmployeeContract>, AppError>;

    async fn list_employee_contracts(
        &self,
        workspace_id_val: Uuid,
        employee_id_val: Uuid,
    ) -> Result<Vec<EmployeeContract>, AppError>;

    async fn update_contract(
        &self,
        workspace_id_val: Uuid,
        contract_id: Uuid,
        req: &UpdateContractRequest,
    ) -> Result<EmployeeContract, AppError>;

    async fn delete_contract(
        &self,
        workspace_id_val: Uuid,
        contract_id: Uuid,
    ) -> Result<(), AppError>;

    async fn get_expiring_contracts(
        &self,
        workspace_id_val: Uuid,
        days_ahead: i32,
    ) -> Result<Vec<EmployeeContract>, AppError>;

    async fn get_active_contract_on_date(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        date: chrono::NaiveDate,
    ) -> Result<Option<EmployeeContract>, AppError>;
}
