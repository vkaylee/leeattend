use crate::diesel_pool::TenantAuthToken;
use crate::errors::AppError;
use crate::models::{CreateEmployerRequest, Employer, UpdateEmployerRequest};
use async_trait::async_trait;
use uuid::Uuid;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait EmployerRepository: Send + Sync {
    async fn create_employer(
        &self,
        auth_token: &TenantAuthToken,
        data: CreateEmployerRequest,
    ) -> Result<Employer, AppError>;

    async fn get_employer_by_id(
        &self,
        auth_token: &TenantAuthToken,
        id: Uuid,
    ) -> Result<Option<Employer>, AppError>;

    async fn get_employers(
        &self,
        auth_token: &TenantAuthToken,
        page: i64,
        limit: i64,
    ) -> Result<(Vec<Employer>, i64), AppError>;

    async fn update_employer(
        &self,
        auth_token: &TenantAuthToken,
        id: Uuid,
        data: UpdateEmployerRequest,
    ) -> Result<Employer, AppError>;

    async fn delete_employer(&self, auth_token: &TenantAuthToken, id: Uuid)
        -> Result<(), AppError>;

    async fn count_employees_by_employer(
        &self,
        auth_token: &TenantAuthToken,
        id: Uuid,
    ) -> Result<i64, AppError>;
}
