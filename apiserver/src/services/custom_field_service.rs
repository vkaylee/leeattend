use crate::diesel_pool::TenantAuthToken;
use crate::errors::AppError;
use crate::models::{
    CreateCustomFieldDefinitionRequest, CustomFieldDefinition, CustomFieldValue,
    CustomFieldValueResponse, SetCustomFieldValueRequest, UpdateCustomFieldDefinitionRequest,
};
use crate::repositories::audit_log_repository::AuditLogRepository;
use crate::repositories::custom_field_repository::CustomFieldRepository;
use crate::services::audit_service::{AuditHelper, AuditLogArgs};
use axum::http::HeaderMap;
use serde_json::json;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct CustomFieldService {
    repo: Arc<dyn CustomFieldRepository>,
    audit_repo: Arc<dyn AuditLogRepository>,
}

impl CustomFieldService {
    pub fn new(
        repo: Arc<dyn CustomFieldRepository>,
        audit_repo: Arc<dyn AuditLogRepository>,
    ) -> Self {
        Self { repo, audit_repo }
    }

    pub async fn create_definition(
        &self,
        workspace_id: Uuid,
        actor_id: Uuid,
        data: CreateCustomFieldDefinitionRequest,
    ) -> Result<CustomFieldDefinition, AppError> {
        // Validate entity_type
        if data.entity_type != "employee"
            && data.entity_type != "employment"
            && data.entity_type != "department"
        {
            return Err(AppError::Validation(format!(
                "Invalid entity type '{}'. Must be 'employee', 'employment', or 'department'.",
                data.entity_type
            ).into()));
        }

        // Validate field_type
        let valid_types = [
            "text",
            "number",
            "date",
            "boolean",
            "select",
            "multiselect",
            "url",
            "email",
            "phone",
        ];
        if !valid_types.contains(&data.field_type.as_str()) {
            return Err(AppError::Validation(format!(
                "Invalid field type '{}'. Must be one of: {:?}.",
                data.field_type, valid_types
            ).into()));
        }

        // Validate field_key format (alphanumeric and underscores only)
        if data.field_key.is_empty()
            || !data
                .field_key
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_')
        {
            return Err(AppError::Validation(format!(
                "Invalid field key '{}'. Must be alphanumeric and underscores only.",
                data.field_key
            ).into()));
        }

        // Check if options are present for select/multiselect
        if data.field_type == "select" || data.field_type == "multiselect" {
            let options = data.options.as_ref().and_then(|v| v.as_array());
            let is_empty = match options {
                None => true,
                Some(arr) => arr.is_empty(),
            };
            if is_empty {
                return Err(AppError::Validation(format!(
                    "Options are required for field type '{}'.",
                    data.field_type
                ).into()));
            }
        }

        let auth_token = TenantAuthToken::system_override(workspace_id);
        let definition = self.repo.create_definition(&auth_token, data).await?;

        // Log audit
        let empty_headers = HeaderMap::new();
        AuditHelper::log(AuditLogArgs {
            repo: &self.audit_repo,
            workspace_id,
            actor_id: Some(actor_id),
            actor_type: Some("user".to_string()),
            actor_name: None,
            actor_email: None,
            target_user_id: None,
            target_resource_id: Some(definition.id()),
            action: "custom_field.definition_created",
            details: Some(json!({
                "field_key": definition.field_key(),
                "entity_type": definition.entity_type(),
                "field_type": definition.field_type()
            })),
            headers: &empty_headers,
        })
        .await;

        Ok(definition)
    }

    pub async fn get_definition(
        &self,
        workspace_id: Uuid,
        id: Uuid,
    ) -> Result<CustomFieldDefinition, AppError> {
        let auth_token = TenantAuthToken::system_override(workspace_id);
        self.repo
            .get_definition_by_id(&auth_token, id)
            .await?
            .ok_or_else(|| AppError::not_found(crate::utils::i18n::I18nKey::CustomFieldNotFound))
    }

    pub async fn list_definitions(
        &self,
        workspace_id: Uuid,
        entity_type: Option<String>,
    ) -> Result<Vec<CustomFieldDefinition>, AppError> {
        let auth_token = TenantAuthToken::system_override(workspace_id);
        self.repo.list_definitions(&auth_token, entity_type).await
    }

    pub async fn update_definition(
        &self,
        workspace_id: Uuid,
        id: Uuid,
        actor_id: Uuid,
        data: UpdateCustomFieldDefinitionRequest,
    ) -> Result<CustomFieldDefinition, AppError> {
        let auth_token = TenantAuthToken::system_override(workspace_id);
        let definition = self.repo.update_definition(&auth_token, id, data).await?;

        // Log audit
        let empty_headers = HeaderMap::new();
        AuditHelper::log(AuditLogArgs {
            repo: &self.audit_repo,
            workspace_id,
            actor_id: Some(actor_id),
            actor_type: Some("user".to_string()),
            actor_name: None,
            actor_email: None,
            target_user_id: None,
            target_resource_id: Some(definition.id()),
            action: "custom_field.definition_updated",
            details: Some(json!({
                "field_key": definition.field_key(),
                "entity_type": definition.entity_type()
            })),
            headers: &empty_headers,
        })
        .await;

        Ok(definition)
    }

    pub async fn delete_definition(
        &self,
        workspace_id: Uuid,
        id: Uuid,
        actor_id: Uuid,
    ) -> Result<(), AppError> {
        let auth_token = TenantAuthToken::system_override(workspace_id);
        let definition = self.get_definition(workspace_id, id).await?;

        self.repo.soft_delete_definition(&auth_token, id).await?;

        // Log audit
        let empty_headers = HeaderMap::new();
        AuditHelper::log(AuditLogArgs {
            repo: &self.audit_repo,
            workspace_id,
            actor_id: Some(actor_id),
            actor_type: Some("user".to_string()),
            actor_name: None,
            actor_email: None,
            target_user_id: None,
            target_resource_id: Some(definition.id()),
            action: "custom_field.definition_deleted",
            details: Some(json!({
                "field_key": definition.field_key(),
                "entity_type": definition.entity_type()
            })),
            headers: &empty_headers,
        })
        .await;

        Ok(())
    }

    pub async fn get_values_for_entity(
        &self,
        workspace_id: Uuid,
        entity_id: Uuid,
    ) -> Result<Vec<CustomFieldValueResponse>, AppError> {
        let auth_token = TenantAuthToken::system_override(workspace_id);
        let pairs = self
            .repo
            .get_values_for_entity(&auth_token, entity_id)
            .await?;

        Ok(pairs
            .into_iter()
            .map(|(val, def)| CustomFieldValueResponse {
                definition_id: val.definition_id(),
                field_key: def.field_key().to_string(),
                field_label: def.field_label().to_string(),
                field_type: def.field_type().to_string(),
                value: Some(val.to_json_value(def.field_type())),
            })
            .collect())
    }

    pub async fn get_values_for_entities(
        &self,
        workspace_id: Uuid,
        entity_ids: &[Uuid],
    ) -> Result<HashMap<Uuid, Vec<CustomFieldValueResponse>>, AppError> {
        let auth_token = TenantAuthToken::system_override(workspace_id);
        let map = self
            .repo
            .get_values_for_entities(&auth_token, entity_ids)
            .await?;

        let mut response_map = HashMap::new();
        for (entity_id, pairs) in map {
            let responses = pairs
                .into_iter()
                .map(|(val, def)| CustomFieldValueResponse {
                    definition_id: val.definition_id(),
                    field_key: def.field_key().to_string(),
                    field_label: def.field_label().to_string(),
                    field_type: def.field_type().to_string(),
                    value: Some(val.to_json_value(def.field_type())),
                })
                .collect();
            response_map.insert(entity_id, responses);
        }

        Ok(response_map)
    }

    #[allow(clippy::too_many_lines)]
    pub async fn set_values(
        &self,
        workspace_id: Uuid,
        actor_id: Uuid,
        entity_type: &str,
        entity_id: Uuid,
        values: Vec<SetCustomFieldValueRequest>,
    ) -> Result<Vec<CustomFieldValueResponse>, AppError> {
        let auth_token = TenantAuthToken::system_override(workspace_id);

        // Track request definitions
        let requested_def_ids: std::collections::HashSet<Uuid> =
            values.iter().map(|val_req| val_req.definition_id).collect();

        // 1. Fetch current values for comparison (for audit logging)
        let current_pairs = self
            .repo
            .get_values_for_entity(&auth_token, entity_id)
            .await?;
        let current_values_map: HashMap<Uuid, (CustomFieldValue, CustomFieldDefinition)> =
            current_pairs
                .into_iter()
                .map(|(val, def)| (val.definition_id(), (val, def)))
                .collect();

        // 2. Perform validation before saving
        for val_req in &values {
            let definition = self
                .repo
                .get_definition_by_id(&auth_token, val_req.definition_id)
                .await?;
            let Some(def) = definition else {
                return Err(AppError::NotFound("error.custom_field_not_found".into()));
            };

            let has_value = val_req
                .value
                .as_ref()
                .map(|v| !v.is_null())
                .unwrap_or(false);

            if def.is_required() && !has_value {
                return Err(AppError::Validation(format!(
                    "Field '{}' is required.",
                    def.field_label()
                ).into()));
            }

            if let Some(val_json) = val_req.value.as_ref() {
                if !val_json.is_null() {
                    // Validate Select Options
                    if def.field_type() == "select" {
                        let select_val = val_json.as_str().ok_or_else(|| {
                            AppError::Validation(format!(
                                "Expected string value for select field '{}'.",
                                def.field_label()
                            ).into())
                        })?;

                        let options = def.options().and_then(|v| v.as_array());
                        let has_match = options
                            .map(|arr| {
                                arr.iter().any(|opt| {
                                    opt.get("value")
                                        .and_then(|v| v.as_str())
                                        .map(|s| s == select_val)
                                        .unwrap_or(false)
                                })
                            })
                            .unwrap_or(false);

                        if !has_match {
                            return Err(AppError::Validation(format!(
                                "Value '{}' is not a valid option for field '{}'.",
                                select_val,
                                def.field_label()
                            ).into()));
                        }
                    }

                    // Validate MultiSelect Options
                    if def.field_type() == "multiselect" {
                        let multiselect_arr = val_json.as_array().ok_or_else(|| {
                            AppError::Validation(format!(
                                "Expected array value for multiselect field '{}'.",
                                def.field_label()
                            ).into())
                        })?;

                        let options = def.options().and_then(|v| v.as_array());
                        for item in multiselect_arr {
                            let item_str = item.as_str().ok_or_else(|| {
                                AppError::Validation(format!(
                                    "Expected array of strings for multiselect field '{}'.",
                                    def.field_label()
                                ).into())
                            })?;

                            let has_match = options
                                .map(|arr| {
                                    arr.iter().any(|opt| {
                                        opt.get("value")
                                            .and_then(|v| v.as_str())
                                            .map(|s| s == item_str)
                                            .unwrap_or(false)
                                    })
                                })
                                .unwrap_or(false);

                            if !has_match {
                                return Err(AppError::Validation(format!(
                                    "Value '{}' is not a valid option for field '{}'.",
                                    item_str,
                                    def.field_label()
                                ).into()));
                            }
                        }
                    }
                }
            }
        }

        // 3. Save the values
        let saved_values = self.repo.set_values(&auth_token, entity_id, values).await?;

        // 4. Load definitions for the saved values to construct the response and log audit trail
        let saved_definitions = self
            .repo
            .list_definitions(&auth_token, Some(entity_type.to_string()))
            .await?;
        let definitions_map: HashMap<Uuid, CustomFieldDefinition> = saved_definitions
            .into_iter()
            .map(|def| (def.id(), def))
            .collect();

        let mut responses = Vec::new();
        let empty_headers = HeaderMap::new();

        for val in saved_values {
            if let Some(def) = definitions_map.get(&val.definition_id()) {
                let new_val_json = val.to_json_value(def.field_type());

                // Find old value for audit logging
                let old_val_json = current_values_map
                    .get(&val.definition_id())
                    .map(|(old_val, old_def)| old_val.to_json_value(old_def.field_type()))
                    .unwrap_or(Value::Null);

                // Only audit if value has changed
                if new_val_json != old_val_json {
                    AuditHelper::log(AuditLogArgs {
                        repo: &self.audit_repo,
                        workspace_id,
                        actor_id: Some(actor_id),
                        actor_type: Some("user".to_string()),
                        actor_name: None,
                        actor_email: None,
                        target_user_id: if entity_type == "employee" {
                            Some(entity_id)
                        } else {
                            None
                        },
                        target_resource_id: Some(entity_id),
                        action: "custom_field.value_changed",
                        details: Some(json!({
                            "field_key": def.field_key(),
                            "entity_type": entity_type,
                            "entity_id": entity_id,
                            "old_value": old_val_json,
                            "new_value": new_val_json
                        })),
                        headers: &empty_headers,
                    })
                    .await;
                }

                responses.push(CustomFieldValueResponse {
                    definition_id: val.definition_id(),
                    field_key: def.field_key().to_string(),
                    field_label: def.field_label().to_string(),
                    field_type: def.field_type().to_string(),
                    value: Some(new_val_json),
                });
            }
        }

        // Check if any fields were explicitly cleared (deleted)
        for (def_id, (old_val, def)) in current_values_map {
            if requested_def_ids.contains(&def_id)
                && !responses.iter().any(|r| r.definition_id == def_id)
            {
                let old_val_json = old_val.to_json_value(def.field_type());
                AuditHelper::log(AuditLogArgs {
                    repo: &self.audit_repo,
                    workspace_id,
                    actor_id: Some(actor_id),
                    actor_type: Some("user".to_string()),
                    actor_name: None,
                    actor_email: None,
                    target_user_id: if entity_type == "employee" {
                        Some(entity_id)
                    } else {
                        None
                    },
                    target_resource_id: Some(entity_id),
                    action: "custom_field.value_changed",
                    details: Some(json!({
                        "field_key": def.field_key(),
                        "entity_type": entity_type,
                        "entity_id": entity_id,
                        "old_value": old_val_json,
                        "new_value": Value::Null
                    })),
                    headers: &empty_headers,
                })
                .await;
            }
        }

        Ok(responses)
    }

    pub async fn find_entity_ids_by_value(
        &self,
        workspace_id: Uuid,
        definition_id: Uuid,
        value_query: String,
    ) -> Result<Vec<Uuid>, AppError> {
        let auth_token = TenantAuthToken::system_override(workspace_id);
        self.repo
            .find_entity_ids_by_value(&auth_token, definition_id, value_query)
            .await
    }
}
