use crate::diesel_pool::{DieselPools, TenantAuthToken};
use crate::errors::AppError;
use crate::utils::i18n::I18nKey;
use crate::models::diesel::custom_field::{CustomFieldDefinitionEntity, CustomFieldValueEntity};
use crate::models::{
    CreateCustomFieldDefinitionRequest, CustomFieldDefinition, CustomFieldValue,
    SetCustomFieldValueRequest, UpdateCustomFieldDefinitionRequest,
};
use crate::repositories::custom_field_repository::CustomFieldRepository;
use crate::schema::{custom_field_definitions, custom_field_values};
use async_trait::async_trait;
use chrono::Utc;
use diesel::prelude::*;
use diesel_async::{AsyncConnection, RunQueryDsl};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Clone)]
pub struct DieselCustomFieldRepository {
    diesel_pools: DieselPools,
}

impl DieselCustomFieldRepository {
    pub fn new(diesel_pools: DieselPools) -> Self {
        Self { diesel_pools }
    }
}

#[async_trait]
impl CustomFieldRepository for DieselCustomFieldRepository {
    async fn create_definition(
        &self,
        auth_token: &TenantAuthToken,
        data: CreateCustomFieldDefinitionRequest,
    ) -> Result<CustomFieldDefinition, AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;
        let workspace_id = auth_token.workspace_id();

        let entity = CustomFieldDefinitionEntity {
            id: Uuid::new_v4(),
            workspace_id,
            entity_type: data.entity_type,
            field_key: data.field_key,
            field_label: data.field_label,
            field_type: data.field_type,
            options: data.options,
            default_value: data.default_value,
            is_required: data.is_required,
            is_visible_to_employee: data.is_visible_to_employee,
            is_filterable: data.is_filterable,
            is_exportable: data.is_exportable,
            display_order: data.display_order,
            section_name: data.section_name,
            validation_rules: data.validation_rules,
            deleted_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let result = diesel::insert_into(custom_field_definitions::table)
            .values(&entity)
            .get_result::<CustomFieldDefinitionEntity>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db create cfd: {e}")))?;

        Ok(result.into())
    }

    async fn get_definition_by_id(
        &self,
        auth_token: &TenantAuthToken,
        id: Uuid,
    ) -> Result<Option<CustomFieldDefinition>, AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;
        let workspace_id = auth_token.workspace_id();

        let result = custom_field_definitions::table
            .filter(custom_field_definitions::id.eq(id))
            .filter(custom_field_definitions::workspace_id.eq(workspace_id))
            .filter(custom_field_definitions::deleted_at.is_null())
            .first::<CustomFieldDefinitionEntity>(&mut *conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(format!("db get cfd: {e}")))?;

        Ok(result.map(Into::into))
    }

    async fn list_definitions(
        &self,
        auth_token: &TenantAuthToken,
        entity_type: Option<String>,
    ) -> Result<Vec<CustomFieldDefinition>, AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;
        let workspace_id = auth_token.workspace_id();

        let mut query = custom_field_definitions::table
            .filter(custom_field_definitions::workspace_id.eq(workspace_id))
            .filter(custom_field_definitions::deleted_at.is_null())
            .into_boxed();

        if let Some(et) = entity_type {
            query = query.filter(custom_field_definitions::entity_type.eq(et));
        }

        let results = query
            .order((
                custom_field_definitions::display_order.asc(),
                custom_field_definitions::created_at.asc(),
            ))
            .load::<CustomFieldDefinitionEntity>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db list cfd: {e}")))?;

        Ok(results.into_iter().map(Into::into).collect())
    }

    async fn update_definition(
        &self,
        auth_token: &TenantAuthToken,
        id: Uuid,
        data: UpdateCustomFieldDefinitionRequest,
    ) -> Result<CustomFieldDefinition, AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;
        let workspace_id = auth_token.workspace_id();

        let mut existing = custom_field_definitions::table
            .filter(custom_field_definitions::id.eq(id))
            .filter(custom_field_definitions::workspace_id.eq(workspace_id))
            .filter(custom_field_definitions::deleted_at.is_null())
            .first::<CustomFieldDefinitionEntity>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db get cfd to update: {e}")))?;

        if let Some(label) = data.field_label {
            existing.field_label = label;
        }
        if let Some(opts) = data.options {
            existing.options = Some(opts);
        }
        if let Some(def_val_opt) = data.default_value {
            existing.default_value = def_val_opt;
        }
        if let Some(req) = data.is_required {
            existing.is_required = req;
        }
        if let Some(vis) = data.is_visible_to_employee {
            existing.is_visible_to_employee = vis;
        }
        if let Some(filt) = data.is_filterable {
            existing.is_filterable = filt;
        }
        if let Some(exp) = data.is_exportable {
            existing.is_exportable = exp;
        }
        if let Some(order) = data.display_order {
            existing.display_order = order;
        }
        if let Some(sec_opt) = data.section_name {
            existing.section_name = sec_opt;
        }
        if let Some(val_opt) = data.validation_rules {
            existing.validation_rules = val_opt;
        }

        existing.updated_at = Utc::now();

        let result = diesel::update(
            custom_field_definitions::table
                .filter(custom_field_definitions::id.eq(id))
                .filter(custom_field_definitions::workspace_id.eq(workspace_id)),
        )
        .set(&existing)
        .get_result::<CustomFieldDefinitionEntity>(&mut *conn)
        .await
        .map_err(|e| AppError::Internal(format!("db update cfd: {e}")))?;

        Ok(result.into())
    }

    async fn soft_delete_definition(
        &self,
        auth_token: &TenantAuthToken,
        id: Uuid,
    ) -> Result<(), AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;
        let workspace_id = auth_token.workspace_id();

        // Perform soft delete inside transaction
        conn.transaction::<_, AppError, _>(|conn| {
            Box::pin(async move {
                // Check if definition belongs to workspace and is not already deleted
                let definition_exists = custom_field_definitions::table
                    .filter(custom_field_definitions::id.eq(id))
                    .filter(custom_field_definitions::workspace_id.eq(workspace_id))
                    .filter(custom_field_definitions::deleted_at.is_null())
                    .execute(conn)
                    .await
                    .map_err(|e| AppError::Internal(format!("db check cfd existence: {e}")))?;

                if definition_exists == 0 {
                    return Err(AppError::not_found(I18nKey::CustomFieldNotFound));
                }

                // Set deleted_at timestamp
                diesel::update(
                    custom_field_definitions::table.filter(custom_field_definitions::id.eq(id)),
                )
                .set(custom_field_definitions::deleted_at.eq(Some(Utc::now())))
                .execute(conn)
                .await
                .map_err(|e| AppError::Internal(format!("db soft delete cfd: {e}")))?;

                // Clean up related values physically
                diesel::delete(
                    custom_field_values::table.filter(custom_field_values::definition_id.eq(id)),
                )
                .execute(conn)
                .await
                .map_err(|e| AppError::Internal(format!("db delete related cfv: {e}")))?;

                Ok(())
            })
        })
        .await?;

        Ok(())
    }

    async fn get_values_for_entity(
        &self,
        auth_token: &TenantAuthToken,
        entity_id: Uuid,
    ) -> Result<Vec<(CustomFieldValue, CustomFieldDefinition)>, AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;
        let workspace_id = auth_token.workspace_id();

        let results = custom_field_values::table
            .inner_join(custom_field_definitions::table)
            .filter(custom_field_definitions::workspace_id.eq(workspace_id))
            .filter(custom_field_definitions::deleted_at.is_null())
            .filter(custom_field_values::entity_id.eq(entity_id))
            .load::<(CustomFieldValueEntity, CustomFieldDefinitionEntity)>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db get cfv: {e}")))?;

        Ok(results
            .into_iter()
            .map(|(val_entity, def_entity)| (val_entity.into(), def_entity.into()))
            .collect())
    }

    async fn get_values_for_entities(
        &self,
        auth_token: &TenantAuthToken,
        entity_ids: &[Uuid],
    ) -> Result<HashMap<Uuid, Vec<(CustomFieldValue, CustomFieldDefinition)>>, AppError> {
        if entity_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;
        let workspace_id = auth_token.workspace_id();

        let results = custom_field_values::table
            .inner_join(custom_field_definitions::table)
            .filter(custom_field_definitions::workspace_id.eq(workspace_id))
            .filter(custom_field_definitions::deleted_at.is_null())
            .filter(custom_field_values::entity_id.eq_any(entity_ids))
            .load::<(CustomFieldValueEntity, CustomFieldDefinitionEntity)>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db batch get cfv: {e}")))?;

        let mut map: HashMap<Uuid, Vec<(CustomFieldValue, CustomFieldDefinition)>> = HashMap::new();
        for (val_entity, def_entity) in results {
            let val: CustomFieldValue = val_entity.into();
            let def: CustomFieldDefinition = def_entity.into();
            map.entry(val.entity_id()).or_default().push((val, def));
        }

        Ok(map)
    }

    async fn set_values(
        &self,
        auth_token: &TenantAuthToken,
        entity_id: Uuid,
        values: Vec<SetCustomFieldValueRequest>,
    ) -> Result<Vec<CustomFieldValue>, AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;
        let workspace_id = auth_token.workspace_id();

        let mut updated_values = Vec::new();

        // Wrap value mutations in a transaction for safety
        conn.transaction::<_, AppError, _>(|conn| {
            Box::pin(async move {
                for val_req in values {
                    // Verify definition exists in workspace
                    let definition = custom_field_definitions::table
                        .filter(custom_field_definitions::id.eq(val_req.definition_id))
                        .filter(custom_field_definitions::workspace_id.eq(workspace_id))
                        .filter(custom_field_definitions::deleted_at.is_null())
                        .first::<CustomFieldDefinitionEntity>(conn)
                        .await
                        .optional()
                        .map_err(|e| AppError::Internal(format!("db check cfd: {e}")))?;

                    let Some(def) = definition else {
                        return Err(AppError::not_found(I18nKey::CustomFieldNotFound));
                    };

                    let val_json = val_req.value.filter(|v| !v.is_null());
                    if let Some(val_json) = val_json {
                        // Map value to fields based on field type
                        let domain_val = CustomFieldValue::from_json_value(
                            Uuid::new_v4(),
                            val_req.definition_id,
                            entity_id,
                            &val_json,
                            &def.field_type,
                        )
                        .map_err(|e| AppError::validation_args(I18nKey::Validation, [("details", e)]))?;

                        let entity: CustomFieldValueEntity = domain_val.into();

                        let result = diesel::insert_into(custom_field_values::table)
                            .values(&entity)
                            .on_conflict((
                                custom_field_values::definition_id,
                                custom_field_values::entity_id,
                            ))
                            .do_update()
                            .set((
                                custom_field_values::value_text.eq(&entity.value_text),
                                custom_field_values::value_number.eq(&entity.value_number),
                                custom_field_values::value_date.eq(&entity.value_date),
                                custom_field_values::value_bool.eq(&entity.value_bool),
                                custom_field_values::value_json.eq(&entity.value_json),
                                custom_field_values::updated_at.eq(Utc::now()),
                            ))
                            .get_result::<CustomFieldValueEntity>(conn)
                            .await
                            .map_err(|e| AppError::Internal(format!("db set cfv: {e}")))?;

                        updated_values.push(result.into());
                    } else {
                        // Delete the value record if clearing the value
                        diesel::delete(
                            custom_field_values::table
                                .filter(
                                    custom_field_values::definition_id.eq(val_req.definition_id),
                                )
                                .filter(custom_field_values::entity_id.eq(entity_id)),
                        )
                        .execute(conn)
                        .await
                        .map_err(|e| AppError::Internal(format!("db clear cfv: {e}")))?;
                    }
                }
                Ok(updated_values)
            })
        })
        .await
    }

    async fn find_entity_ids_by_value(
        &self,
        auth_token: &TenantAuthToken,
        definition_id: Uuid,
        value_query: String,
    ) -> Result<Vec<Uuid>, AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

        // Get the definition first to know the type
        let Some(def) = self.get_definition_by_id(auth_token, definition_id).await? else {
            return Ok(vec![]);
        };

        let workspace_id = auth_token.workspace_id();
        let mut query = custom_field_values::table
            .inner_join(custom_field_definitions::table)
            .filter(custom_field_definitions::workspace_id.eq(workspace_id))
            .filter(custom_field_values::definition_id.eq(definition_id))
            .select(custom_field_values::entity_id)
            .into_boxed();

        match def.field_type() {
            "boolean" => {
                if let Ok(b) = value_query.parse::<bool>() {
                    query = query.filter(custom_field_values::value_bool.eq(b));
                }
            }
            "number" => {
                if let Ok(n) = value_query.parse::<rust_decimal::Decimal>() {
                    query = query.filter(custom_field_values::value_number.eq(n));
                }
            }
            "date" => {
                if let Ok(d) = chrono::NaiveDate::parse_from_str(&value_query, "%Y-%m-%d") {
                    query = query.filter(custom_field_values::value_date.eq(d));
                }
            }
            "multiselect" => {
                query = query.filter(
                    diesel::dsl::sql::<diesel::sql_types::Bool>(
                        "value_json @> jsonb_build_array($1::text)",
                    )
                    .bind::<diesel::sql_types::Text, _>(&value_query),
                );
            }
            _ => {
                query = query.filter(custom_field_values::value_text.eq(value_query));
            }
        }

        let ids: Vec<Uuid> = query
            .load::<Uuid>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db find entity ids by cf: {e}")))?;

        Ok(ids)
    }
}
