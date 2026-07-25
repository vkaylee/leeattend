use crate::diesel_pool::TenantAuthToken;
use crate::errors::AppError;
use crate::models::{
    CreateCustomFieldDefinitionRequest, CustomFieldDefinition, CustomFieldValue,
    CustomFieldValueResponse, SetCustomFieldValueRequest, UpdateCustomFieldDefinitionRequest,
};
use crate::repositories::audit_log_repository::AuditLogRepository;
use crate::repositories::custom_field_repository::CustomFieldRepository;
use crate::services::audit_service::{AuditHelper, AuditLogArgs};
use crate::utils::i18n::I18nKey;
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
        auth_token: &TenantAuthToken,
        actor_id: Uuid,
        data: CreateCustomFieldDefinitionRequest,
    ) -> Result<CustomFieldDefinition, AppError> {
        let workspace_id = auth_token.workspace_id();
        // Validate entity_type
        if data.entity_type != "employee"
            && data.entity_type != "employment"
            && data.entity_type != "department"
        {
            return Err(AppError::validation_args(
                I18nKey::Validation,
                vec![(
                    "details",
                    format!(
                    "Invalid entity type '{}'. Must be 'employee', 'employment', or 'department'.",
                    data.entity_type
                ),
                )],
            ));
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
            return Err(AppError::validation_args(
                I18nKey::Validation,
                vec![(
                    "details",
                    format!(
                        "Invalid field type '{}'. Must be one of: {:?}.",
                        data.field_type, valid_types
                    ),
                )],
            ));
        }

        // Validate field_key format (alphanumeric and underscores only)
        if data.field_key.is_empty()
            || !data
                .field_key
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_')
        {
            return Err(AppError::validation_args(
                I18nKey::Validation,
                vec![(
                    "details",
                    format!(
                        "Invalid field key '{}'. Must be alphanumeric and underscores only.",
                        data.field_key
                    ),
                )],
            ));
        }

        // Check if options are present for select/multiselect
        if data.field_type == "select" || data.field_type == "multiselect" {
            let options = data.options.as_ref().and_then(|v| v.as_array());
            let is_empty = match options {
                None => true,
                Some(arr) => arr.is_empty(),
            };
            if is_empty {
                return Err(AppError::validation_args(
                    I18nKey::Validation,
                    vec![(
                        "details",
                        format!("Options are required for field type '{}'.", data.field_type),
                    )],
                ));
            }
        }

        let definition = self.repo.create_definition(auth_token, data).await?;

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
        auth_token: &TenantAuthToken,
        id: Uuid,
    ) -> Result<CustomFieldDefinition, AppError> {
        self.repo
            .get_definition_by_id(auth_token, id)
            .await?
            .ok_or_else(|| AppError::not_found(crate::utils::i18n::I18nKey::CustomFieldNotFound))
    }

    pub async fn list_definitions(
        &self,
        auth_token: &TenantAuthToken,
        entity_type: Option<String>,
    ) -> Result<Vec<CustomFieldDefinition>, AppError> {
        self.repo.list_definitions(auth_token, entity_type).await
    }

    pub async fn update_definition(
        &self,
        auth_token: &TenantAuthToken,
        id: Uuid,
        actor_id: Uuid,
        data: UpdateCustomFieldDefinitionRequest,
    ) -> Result<CustomFieldDefinition, AppError> {
        let workspace_id = auth_token.workspace_id();
        let definition = self.repo.update_definition(auth_token, id, data).await?;

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
        auth_token: &TenantAuthToken,
        id: Uuid,
        actor_id: Uuid,
    ) -> Result<(), AppError> {
        let workspace_id = auth_token.workspace_id();
        let definition = self.get_definition(auth_token, id).await?;

        self.repo.soft_delete_definition(auth_token, id).await?;

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
        auth_token: &TenantAuthToken,
        entity_id: Uuid,
    ) -> Result<Vec<CustomFieldValueResponse>, AppError> {
        let pairs = self
            .repo
            .get_values_for_entity(auth_token, entity_id)
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
        auth_token: &TenantAuthToken,
        entity_ids: &[Uuid],
    ) -> Result<HashMap<Uuid, Vec<CustomFieldValueResponse>>, AppError> {
        let map = self
            .repo
            .get_values_for_entities(auth_token, entity_ids)
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
        auth_token: &TenantAuthToken,
        actor_id: Uuid,
        entity_type: &str,
        entity_id: Uuid,
        values: Vec<SetCustomFieldValueRequest>,
    ) -> Result<Vec<CustomFieldValueResponse>, AppError> {
        let workspace_id = auth_token.workspace_id();

        // Track request definitions
        let requested_def_ids: std::collections::HashSet<Uuid> =
            values.iter().map(|val_req| val_req.definition_id).collect();

        // 1. Fetch current values for comparison (for audit logging)
        let current_pairs = self
            .repo
            .get_values_for_entity(auth_token, entity_id)
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
                .get_definition_by_id(auth_token, val_req.definition_id)
                .await?;
            let Some(def) = definition else {
                return Err(AppError::not_found(I18nKey::CustomFieldNotFound));
            };

            let has_value = val_req
                .value
                .as_ref()
                .map(|v| !v.is_null())
                .unwrap_or(false);

            if def.is_required() && !has_value {
                return Err(AppError::validation_args(
                    I18nKey::Validation,
                    vec![(
                        "details",
                        format!("Field '{}' is required.", def.field_label()),
                    )],
                ));
            }

            if let Some(val_json) = val_req.value.as_ref() {
                if !val_json.is_null() {
                    // Validate Select Options
                    if def.field_type() == "select" {
                        let select_val = val_json.as_str().ok_or_else(|| {
                            AppError::validation_args(
                                I18nKey::Validation,
                                vec![(
                                    "details",
                                    format!(
                                        "Expected string value for select field '{}'.",
                                        def.field_label()
                                    ),
                                )],
                            )
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
                            return Err(AppError::validation_args(
                                I18nKey::Validation,
                                vec![(
                                    "details",
                                    format!(
                                        "Value '{}' is not a valid option for field '{}'.",
                                        select_val,
                                        def.field_label()
                                    ),
                                )],
                            ));
                        }
                    }

                    // Validate MultiSelect Options
                    if def.field_type() == "multiselect" {
                        let multiselect_arr = val_json.as_array().ok_or_else(|| {
                            AppError::validation_args(
                                I18nKey::Validation,
                                vec![(
                                    "details",
                                    format!(
                                        "Expected array value for multiselect field '{}'.",
                                        def.field_label()
                                    ),
                                )],
                            )
                        })?;

                        let options = def.options().and_then(|v| v.as_array());
                        for item in multiselect_arr {
                            let item_str = item.as_str().ok_or_else(|| {
                                AppError::validation_args(
                                    I18nKey::Validation,
                                    vec![(
                                        "details",
                                        format!(
                                            "Expected array of strings for multiselect field '{}'.",
                                            def.field_label()
                                        ),
                                    )],
                                )
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
                                return Err(AppError::validation_args(
                                    I18nKey::Validation,
                                    vec![(
                                        "details",
                                        format!(
                                            "Value '{}' is not a valid option for field '{}'.",
                                            item_str,
                                            def.field_label()
                                        ),
                                    )],
                                ));
                            }
                        }
                    }
                }
            }
        }

        // 3. Save the values
        let saved_values = self.repo.set_values(auth_token, entity_id, values).await?;

        // 4. Load definitions for the saved values to construct the response and log audit trail
        let saved_definitions = self
            .repo
            .list_definitions(auth_token, Some(entity_type.to_string()))
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
        auth_token: &TenantAuthToken,
        definition_id: Uuid,
        value_query: String,
    ) -> Result<Vec<Uuid>, AppError> {
        self.repo
            .find_entity_ids_by_value(auth_token, definition_id, value_query)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repositories::audit_log_repository::MockAuditLogRepository;
    use crate::repositories::custom_field_repository::MockCustomFieldRepository;
    use chrono::Utc;
    use std::sync::Arc;

    fn make_token(ws: Uuid) -> TenantAuthToken {
        TenantAuthToken::system_override(ws)
    }

    fn make_definition(ws: Uuid) -> CustomFieldDefinition {
        CustomFieldDefinition::new(
            Uuid::new_v4(),
            ws,
            "employee".into(),
            "test_field".into(),
            "Test Field".into(),
            "text".into(),
            None,
            None,
            false,
            true,
            false,
            true,
            1,
            None,
            None,
            Utc::now(),
            Utc::now(),
        )
    }

    fn build_service(repo: MockCustomFieldRepository) -> CustomFieldService {
        CustomFieldService::new(Arc::new(repo), Arc::new(MockAuditLogRepository::new()))
    }

    fn build_service_with_audit(
        repo: MockCustomFieldRepository,
        audit: MockAuditLogRepository,
    ) -> CustomFieldService {
        CustomFieldService::new(Arc::new(repo), Arc::new(audit))
    }

    // ── create_definition validation tests ────────────────────────────────

    #[tokio::test]
    async fn create_definition_invalid_entity_type() {
        let ws = Uuid::new_v4();
        let repo = MockCustomFieldRepository::new();
        let svc = build_service(repo);

        let result = svc
            .create_definition(
                &make_token(ws),
                Uuid::new_v4(),
                CreateCustomFieldDefinitionRequest {
                    entity_type: "invalid_type".into(),
                    field_key: "test_field".into(),
                    field_label: "Test Field".into(),
                    field_type: "text".into(),
                    options: None,
                    default_value: None,
                    is_required: false,
                    is_visible_to_employee: true,
                    is_filterable: false,
                    is_exportable: true,
                    display_order: 1,
                    section_name: None,
                    validation_rules: None,
                },
            )
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
        assert!(err.to_string().contains("Invalid entity type"));
    }

    #[tokio::test]
    async fn create_definition_invalid_field_type() {
        let ws = Uuid::new_v4();
        let repo = MockCustomFieldRepository::new();
        let svc = build_service(repo);

        let result = svc
            .create_definition(
                &make_token(ws),
                Uuid::new_v4(),
                CreateCustomFieldDefinitionRequest {
                    entity_type: "employee".into(),
                    field_key: "test_field".into(),
                    field_label: "Test Field".into(),
                    field_type: "invalid_field_type".into(),
                    options: None,
                    default_value: None,
                    is_required: false,
                    is_visible_to_employee: true,
                    is_filterable: false,
                    is_exportable: true,
                    display_order: 1,
                    section_name: None,
                    validation_rules: None,
                },
            )
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
        assert!(err.to_string().contains("Invalid field type"));
    }

    #[tokio::test]
    async fn create_definition_invalid_field_key_empty() {
        let ws = Uuid::new_v4();
        let repo = MockCustomFieldRepository::new();
        let svc = build_service(repo);

        let result = svc
            .create_definition(
                &make_token(ws),
                Uuid::new_v4(),
                CreateCustomFieldDefinitionRequest {
                    entity_type: "employee".into(),
                    field_key: "".into(),
                    field_label: "Test Field".into(),
                    field_type: "text".into(),
                    options: None,
                    default_value: None,
                    is_required: false,
                    is_visible_to_employee: true,
                    is_filterable: false,
                    is_exportable: true,
                    display_order: 1,
                    section_name: None,
                    validation_rules: None,
                },
            )
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
        assert!(err.to_string().contains("Invalid field key"));
    }

    #[tokio::test]
    async fn create_definition_invalid_field_key_special_chars() {
        let ws = Uuid::new_v4();
        let repo = MockCustomFieldRepository::new();
        let svc = build_service(repo);

        let result = svc
            .create_definition(
                &make_token(ws),
                Uuid::new_v4(),
                CreateCustomFieldDefinitionRequest {
                    entity_type: "employee".into(),
                    field_key: "test-field".into(), // hyphen is invalid
                    field_label: "Test Field".into(),
                    field_type: "text".into(),
                    options: None,
                    default_value: None,
                    is_required: false,
                    is_visible_to_employee: true,
                    is_filterable: false,
                    is_exportable: true,
                    display_order: 1,
                    section_name: None,
                    validation_rules: None,
                },
            )
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
        assert!(err.to_string().contains("Invalid field key"));
    }

    #[tokio::test]
    async fn create_definition_select_missing_options() {
        let ws = Uuid::new_v4();
        let repo = MockCustomFieldRepository::new();
        let svc = build_service(repo);

        let result = svc
            .create_definition(
                &make_token(ws),
                Uuid::new_v4(),
                CreateCustomFieldDefinitionRequest {
                    entity_type: "employee".into(),
                    field_key: "test_field".into(),
                    field_label: "Test Field".into(),
                    field_type: "select".into(),
                    options: None, // select requires options
                    default_value: None,
                    is_required: false,
                    is_visible_to_employee: true,
                    is_filterable: false,
                    is_exportable: true,
                    display_order: 1,
                    section_name: None,
                    validation_rules: None,
                },
            )
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
        assert!(err.to_string().contains("Options are required"));
    }

    #[tokio::test]
    async fn create_definition_multiselect_missing_options() {
        let ws = Uuid::new_v4();
        let repo = MockCustomFieldRepository::new();
        let svc = build_service(repo);

        let result = svc
            .create_definition(
                &make_token(ws),
                Uuid::new_v4(),
                CreateCustomFieldDefinitionRequest {
                    entity_type: "employee".into(),
                    field_key: "test_field".into(),
                    field_label: "Test Field".into(),
                    field_type: "multiselect".into(),
                    options: None, // multiselect requires options
                    default_value: None,
                    is_required: false,
                    is_visible_to_employee: true,
                    is_filterable: false,
                    is_exportable: true,
                    display_order: 1,
                    section_name: None,
                    validation_rules: None,
                },
            )
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
        assert!(err.to_string().contains("Options are required"));
    }

    #[tokio::test]
    async fn create_definition_select_empty_options_array() {
        let ws = Uuid::new_v4();
        let repo = MockCustomFieldRepository::new();
        let svc = build_service(repo);

        let result = svc
            .create_definition(
                &make_token(ws),
                Uuid::new_v4(),
                CreateCustomFieldDefinitionRequest {
                    entity_type: "employee".into(),
                    field_key: "test_field".into(),
                    field_label: "Test Field".into(),
                    field_type: "select".into(),
                    options: Some(serde_json::json!([])), // empty array
                    default_value: None,
                    is_required: false,
                    is_visible_to_employee: true,
                    is_filterable: false,
                    is_exportable: true,
                    display_order: 1,
                    section_name: None,
                    validation_rules: None,
                },
            )
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
        assert!(err.to_string().contains("Options are required"));
    }

    #[tokio::test]
    async fn create_definition_success() {
        let ws = Uuid::new_v4();
        let definition = make_definition(ws);
        let expected = definition.clone();

        let mut repo = MockCustomFieldRepository::new();
        repo.expect_create_definition()
            .returning(move |_, _| Ok(expected.clone()));

        let mut audit = MockAuditLogRepository::new();
        audit.expect_create_log().returning(|_| {
            Ok(crate::domain::audit::AuditLog::new(
                Uuid::new_v4(),
                Uuid::new_v4(),
                Some(Uuid::new_v4()),
                Some("user".into()),
                None,
                None,
                None,
                Some(Uuid::new_v4()),
                "custom_field.definition_created".into(),
                None,
                None,
                None,
                Utc::now(),
            ))
        });

        let svc = build_service_with_audit(repo, audit);
        let result = svc
            .create_definition(
                &make_token(ws),
                Uuid::new_v4(),
                CreateCustomFieldDefinitionRequest {
                    entity_type: "employee".into(),
                    field_key: "test_field".into(),
                    field_label: "Test Field".into(),
                    field_type: "text".into(),
                    options: None,
                    default_value: None,
                    is_required: false,
                    is_visible_to_employee: true,
                    is_filterable: false,
                    is_exportable: true,
                    display_order: 1,
                    section_name: None,
                    validation_rules: None,
                },
            )
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().id(), definition.id());
    }

    // ── get_definition tests ──────────────────────────────────────────────

    #[tokio::test]
    async fn get_definition_not_found() {
        let ws = Uuid::new_v4();
        let mut repo = MockCustomFieldRepository::new();
        repo.expect_get_definition_by_id()
            .returning(|_, _| Ok(None));

        let svc = build_service(repo);
        let result = svc.get_definition(&make_token(ws), Uuid::new_v4()).await;

        assert!(matches!(result.unwrap_err(), AppError::NotFound(_)));
    }

    #[tokio::test]
    async fn get_definition_found() {
        let ws = Uuid::new_v4();
        let definition = make_definition(ws);
        let def_id = definition.id();
        let expected = definition.clone();

        let mut repo = MockCustomFieldRepository::new();
        repo.expect_get_definition_by_id()
            .returning(move |_, _| Ok(Some(expected.clone())));

        let svc = build_service(repo);
        let result = svc.get_definition(&make_token(ws), def_id).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().id(), def_id);
    }

    // ── list_definitions tests ────────────────────────────────────────────

    #[tokio::test]
    async fn list_definitions_with_filter() {
        let ws = Uuid::new_v4();
        let definition = make_definition(ws);
        let expected = vec![definition.clone()];

        let mut repo = MockCustomFieldRepository::new();
        repo.expect_list_definitions()
            .returning(move |_, _| Ok(expected.clone()));

        let svc = build_service(repo);
        let result = svc
            .list_definitions(&make_token(ws), Some("employee".into()))
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 1);
    }

    // ── delete_definition tests ───────────────────────────────────────────

    #[tokio::test]
    async fn delete_definition_not_found() {
        let ws = Uuid::new_v4();
        let mut repo = MockCustomFieldRepository::new();
        repo.expect_get_definition_by_id()
            .returning(|_, _| Ok(None));

        let svc = build_service(repo);
        let result = svc
            .delete_definition(&make_token(ws), Uuid::new_v4(), Uuid::new_v4())
            .await;

        assert!(matches!(result.unwrap_err(), AppError::NotFound(_)));
    }

    // ── entity_type acceptance tests ──────────────────────────────────────

    #[tokio::test]
    async fn create_definition_valid_entity_types() {
        // Test that all three valid entity types pass validation
        for entity_type in &["employee", "employment", "department"] {
            let ws = Uuid::new_v4();
            let mut repo = MockCustomFieldRepository::new();
            let captured_entity_type = entity_type.to_string();
            repo.expect_create_definition().returning(move |_, req| {
                assert_eq!(req.entity_type, captured_entity_type);
                let def = make_definition(ws);
                Ok(def)
            });

            let mut audit = MockAuditLogRepository::new();
            audit.expect_create_log().returning(move |_| {
                Ok(crate::domain::audit::AuditLog::new(
                    Uuid::new_v4(),
                    ws,
                    Some(Uuid::new_v4()),
                    Some("user".into()),
                    None,
                    None,
                    None,
                    Some(Uuid::new_v4()),
                    "custom_field.definition_created".into(),
                    None,
                    None,
                    None,
                    Utc::now(),
                ))
            });

            let svc = build_service_with_audit(repo, audit);
            let result = svc
                .create_definition(
                    &make_token(ws),
                    Uuid::new_v4(),
                    CreateCustomFieldDefinitionRequest {
                        entity_type: entity_type.to_string(),
                        field_key: format!("field_{entity_type}"),
                        field_label: "Test Field".into(),
                        field_type: "text".into(),
                        options: None,
                        default_value: None,
                        is_required: false,
                        is_visible_to_employee: true,
                        is_filterable: false,
                        is_exportable: true,
                        display_order: 1,
                        section_name: None,
                        validation_rules: None,
                    },
                )
                .await;

            assert!(result.is_ok(), "Failed for entity_type: {entity_type}");
        }
    }
}
