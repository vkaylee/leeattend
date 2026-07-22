use crate::errors::AppError;
use crate::models::{OvertimeRequest, OvertimeRequestEmployee};
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait OvertimeRequestRepository: Send + Sync {
    async fn create_request(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        request: OvertimeRequest,
        employees: Vec<OvertimeRequestEmployee>,
    ) -> Result<OvertimeRequest, AppError>;

    async fn update_request(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        request: OvertimeRequest,
    ) -> Result<OvertimeRequest, AppError>;

    async fn get_request(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        workspace_id: Uuid,
        request_id: Uuid,
    ) -> Result<Option<OvertimeRequest>, AppError>;

    async fn get_request_employees(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        request_id: Uuid,
    ) -> Result<Vec<OvertimeRequestEmployee>, AppError>;

    async fn update_employee_consent(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        request_id: Uuid,
        employee_id: Uuid,
        consent_status: &str,
        reason: Option<String>,
    ) -> Result<OvertimeRequestEmployee, AppError>;

    async fn list_requests(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        page: i64,
        limit: i64,
    ) -> Result<(Vec<OvertimeRequest>, i64), AppError>;

    async fn get_requests_by_ids(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        ids: &[Uuid],
    ) -> Result<Vec<OvertimeRequest>, AppError>;
}
