use crate::errors::AppError;
use crate::models::{ConditionTypeDefinition, UpdateConditionTypeRequest};
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
#[cfg_attr(test, mockall::automock)]
pub trait ConditionTypeRepository: Send + Sync {
    async fn list_condition_types(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        active_only: bool,
    ) -> Result<Vec<ConditionTypeDefinition>, AppError>;

    async fn get_condition_type_by_id(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        id: Uuid,
    ) -> Result<Option<ConditionTypeDefinition>, AppError>;

    async fn get_condition_type_by_code(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        code: &str,
    ) -> Result<Option<ConditionTypeDefinition>, AppError>;

    async fn create_condition_type(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        definition: &ConditionTypeDefinition,
    ) -> Result<ConditionTypeDefinition, AppError>;

    async fn update_condition_type(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        id: Uuid,
        req: &UpdateConditionTypeRequest,
    ) -> Result<ConditionTypeDefinition, AppError>;

    async fn delete_condition_type(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        id: Uuid,
    ) -> Result<(), AppError>;
}
