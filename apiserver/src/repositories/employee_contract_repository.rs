use async_trait::async_trait;
use std::collections::HashMap;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::contract::{CreateContractRequest, EmployeeContract, UpdateContractRequest};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait EmployeeContractRepository: Send + Sync {
    async fn create_contract(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        req: &CreateContractRequest,
    ) -> Result<EmployeeContract, AppError>;

    async fn get_contract_by_id(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        contract_id: Uuid,
    ) -> Result<Option<EmployeeContract>, AppError>;

    async fn list_employee_contracts(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_id_val: Uuid,
    ) -> Result<Vec<EmployeeContract>, AppError>;

    async fn list_contracts_batch(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_ids: &[Uuid],
    ) -> Result<HashMap<Uuid, Vec<EmployeeContract>>, AppError>;

    async fn update_contract(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        contract_id: Uuid,
        req: &UpdateContractRequest,
    ) -> Result<EmployeeContract, AppError>;

    async fn delete_contract(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        contract_id: Uuid,
    ) -> Result<(), AppError>;

    async fn get_expiring_contracts(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        days_ahead: i32,
    ) -> Result<Vec<EmployeeContract>, AppError>;

    async fn get_active_contract_on_date(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_id: Uuid,
        date: chrono::NaiveDate,
    ) -> Result<Option<EmployeeContract>, AppError>;
}
