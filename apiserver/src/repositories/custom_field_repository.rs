use crate::diesel_pool::TenantAuthToken;
use crate::errors::AppError;
use crate::models::{
    CreateCustomFieldDefinitionRequest, CustomFieldDefinition, CustomFieldValue,
    SetCustomFieldValueRequest, UpdateCustomFieldDefinitionRequest,
};
use async_trait::async_trait;
use std::collections::HashMap;
use uuid::Uuid;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait CustomFieldRepository: Send + Sync {
    async fn create_definition(
        &self,
        auth_token: &TenantAuthToken,
        data: CreateCustomFieldDefinitionRequest,
    ) -> Result<CustomFieldDefinition, AppError>;

    async fn get_definition_by_id(
        &self,
        auth_token: &TenantAuthToken,
        id: Uuid,
    ) -> Result<Option<CustomFieldDefinition>, AppError>;

    async fn list_definitions(
        &self,
        auth_token: &TenantAuthToken,
        entity_type: Option<String>,
    ) -> Result<Vec<CustomFieldDefinition>, AppError>;

    async fn update_definition(
        &self,
        auth_token: &TenantAuthToken,
        id: Uuid,
        data: UpdateCustomFieldDefinitionRequest,
    ) -> Result<CustomFieldDefinition, AppError>;

    async fn soft_delete_definition(
        &self,
        auth_token: &TenantAuthToken,
        id: Uuid,
    ) -> Result<(), AppError>;

    async fn get_values_for_entity(
        &self,
        auth_token: &TenantAuthToken,
        entity_id: Uuid,
    ) -> Result<Vec<(CustomFieldValue, CustomFieldDefinition)>, AppError>;

    async fn get_values_for_entities(
        &self,
        auth_token: &TenantAuthToken,
        entity_ids: &[Uuid],
    ) -> Result<HashMap<Uuid, Vec<(CustomFieldValue, CustomFieldDefinition)>>, AppError>;

    async fn set_values(
        &self,
        auth_token: &TenantAuthToken,
        entity_id: Uuid,
        values: Vec<SetCustomFieldValueRequest>,
    ) -> Result<Vec<CustomFieldValue>, AppError>;

    async fn find_entity_ids_by_value(
        &self,
        auth_token: &TenantAuthToken,
        definition_id: Uuid,
        value_query: String,
    ) -> Result<Vec<Uuid>, AppError>;
}
