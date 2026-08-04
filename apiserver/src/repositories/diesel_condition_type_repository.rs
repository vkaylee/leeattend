use crate::diesel_pool::{DieselPools, TenantAuthToken};
use crate::errors::AppError;
use crate::models::{ConditionTypeDefinition, NewConditionTypeParams, UpdateConditionTypeRequest};
use crate::repositories::condition_type_repository::ConditionTypeRepository;
use crate::schema::condition_type_definitions;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

#[derive(Clone)]
pub struct DieselConditionTypeRepository {
    diesel_pools: DieselPools,
}

impl DieselConditionTypeRepository {
    pub fn new(diesel_pools: DieselPools) -> Self {
        Self { diesel_pools }
    }
}

// Column type alias for readability
type CondTypeRow = (
    Uuid,              // id
    Uuid,              // workspace_id
    String,            // code
    serde_json::Value, // name_i18n
    String,            // category
    i32,               // default_minute_allowance
    bool,              // default_is_paid
    serde_json::Value, // affects_rules
    Option<String>,    // icon
    Option<String>,    // color
    bool,              // is_system
    i32,               // sort_order
    bool,              // is_active
    DateTime<Utc>,     // created_at
    DateTime<Utc>,     // updated_at
);

fn row_to_definition(row: CondTypeRow) -> ConditionTypeDefinition {
    ConditionTypeDefinition::new(NewConditionTypeParams {
        id: row.0,
        workspace_id: row.1,
        code: row.2,
        name_i18n: row.3,
        category: row.4,
        default_minute_allowance: row.5,
        default_is_paid: row.6,
        affects_rules: row.7,
        icon: row.8,
        color: row.9,
        is_system: row.10,
        sort_order: row.11,
        is_active: row.12,
        created_at: row.13,
        updated_at: row.14,
    })
}

type AllColumns = (
    condition_type_definitions::id,
    condition_type_definitions::workspace_id,
    condition_type_definitions::code,
    condition_type_definitions::name_i18n,
    condition_type_definitions::category,
    condition_type_definitions::default_minute_allowance,
    condition_type_definitions::default_is_paid,
    condition_type_definitions::affects_rules,
    condition_type_definitions::icon,
    condition_type_definitions::color,
    condition_type_definitions::is_system,
    condition_type_definitions::sort_order,
    condition_type_definitions::is_active,
    condition_type_definitions::created_at,
    condition_type_definitions::updated_at,
);

const ALL_COLUMNS: AllColumns = (
    condition_type_definitions::id,
    condition_type_definitions::workspace_id,
    condition_type_definitions::code,
    condition_type_definitions::name_i18n,
    condition_type_definitions::category,
    condition_type_definitions::default_minute_allowance,
    condition_type_definitions::default_is_paid,
    condition_type_definitions::affects_rules,
    condition_type_definitions::icon,
    condition_type_definitions::color,
    condition_type_definitions::is_system,
    condition_type_definitions::sort_order,
    condition_type_definitions::is_active,
    condition_type_definitions::created_at,
    condition_type_definitions::updated_at,
);

#[async_trait]
impl ConditionTypeRepository for DieselConditionTypeRepository {
    async fn list_condition_types(
        &self,
        auth_token: &TenantAuthToken,
        active_only: bool,
    ) -> Result<Vec<ConditionTypeDefinition>, AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;
        let workspace_id = auth_token.workspace_id();

        let mut query = condition_type_definitions::table
            .filter(condition_type_definitions::workspace_id.eq(workspace_id))
            .order(condition_type_definitions::sort_order.asc())
            .select(ALL_COLUMNS)
            .into_boxed();

        if active_only {
            query = query.filter(condition_type_definitions::is_active.eq(true));
        }

        let rows = query
            .load::<CondTypeRow>(&mut *conn)
            .await
            .map_err(AppError::from)?;

        Ok(rows.into_iter().map(row_to_definition).collect())
    }

    async fn get_condition_type_by_id(
        &self,
        auth_token: &TenantAuthToken,
        id: Uuid,
    ) -> Result<Option<ConditionTypeDefinition>, AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;
        let workspace_id = auth_token.workspace_id();

        let row = condition_type_definitions::table
            .filter(condition_type_definitions::id.eq(id))
            .filter(condition_type_definitions::workspace_id.eq(workspace_id))
            .select(ALL_COLUMNS)
            .first::<CondTypeRow>(&mut *conn)
            .await
            .optional()
            .map_err(AppError::from)?;

        Ok(row.map(row_to_definition))
    }

    async fn get_condition_type_by_code(
        &self,
        auth_token: &TenantAuthToken,
        code: &str,
    ) -> Result<Option<ConditionTypeDefinition>, AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;
        let workspace_id = auth_token.workspace_id();

        let row = condition_type_definitions::table
            .filter(condition_type_definitions::workspace_id.eq(workspace_id))
            .filter(condition_type_definitions::code.eq(code))
            .select(ALL_COLUMNS)
            .first::<CondTypeRow>(&mut *conn)
            .await
            .optional()
            .map_err(AppError::from)?;

        Ok(row.map(row_to_definition))
    }

    async fn create_condition_type(
        &self,
        auth_token: &TenantAuthToken,
        definition: &ConditionTypeDefinition,
    ) -> Result<ConditionTypeDefinition, AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

        let row = diesel::insert_into(condition_type_definitions::table)
            .values((
                condition_type_definitions::id.eq(definition.id()),
                condition_type_definitions::workspace_id.eq(definition.workspace_id()),
                condition_type_definitions::code.eq(definition.code()),
                condition_type_definitions::name_i18n.eq(definition.name_i18n()),
                condition_type_definitions::category.eq(definition.category()),
                condition_type_definitions::default_minute_allowance
                    .eq(definition.default_minute_allowance()),
                condition_type_definitions::default_is_paid.eq(definition.default_is_paid()),
                condition_type_definitions::affects_rules.eq(definition.affects_rules()),
                condition_type_definitions::icon.eq(definition.icon()),
                condition_type_definitions::color.eq(definition.color()),
                condition_type_definitions::is_system.eq(definition.is_system()),
                condition_type_definitions::sort_order.eq(definition.sort_order()),
                condition_type_definitions::is_active.eq(definition.is_active()),
                condition_type_definitions::created_at.eq(definition.created_at()),
                condition_type_definitions::updated_at.eq(definition.updated_at()),
            ))
            .returning(ALL_COLUMNS)
            .get_result::<CondTypeRow>(&mut *conn)
            .await
            .map_err(AppError::from)?;

        Ok(row_to_definition(row))
    }

    async fn update_condition_type(
        &self,
        auth_token: &TenantAuthToken,
        id: Uuid,
        req: &UpdateConditionTypeRequest,
    ) -> Result<ConditionTypeDefinition, AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;
        let workspace_id = auth_token.workspace_id();

        // Fetch existing record first, merge, then update all fields.
        // This avoids Diesel's inability to conditionally set individual columns.
        let existing = condition_type_definitions::table
            .filter(condition_type_definitions::id.eq(id))
            .filter(condition_type_definitions::workspace_id.eq(workspace_id))
            .select(ALL_COLUMNS)
            .first::<CondTypeRow>(&mut *conn)
            .await
            .optional()
            .map_err(AppError::from)?;

        let (
            _id,
            ws_id,
            _code,
            name_i18n,
            category,
            min_allow,
            is_pd,
            affects,
            icon,
            color,
            _is_sys,
            sort,
            _active,
            _created,
            _updated,
        ) = existing.ok_or_else(|| AppError::not_found(crate::utils::i18n::I18nKey::NotFound))?;

        let new_name = req.name_i18n.clone().unwrap_or(name_i18n);
        let new_category = req.category.clone().unwrap_or(category);
        let new_min = req.default_minute_allowance.unwrap_or(min_allow);
        let new_paid = req.default_is_paid.unwrap_or(is_pd);
        let new_affects = req.affects_rules.clone().unwrap_or(affects);
        let new_icon = req.icon.clone().or(icon);
        let new_color = req.color.clone().or(color);
        let new_sort = req.sort_order.unwrap_or(sort);
        let new_active = req.is_active.unwrap_or(true);
        let now = Utc::now();

        let row = diesel::update(
            condition_type_definitions::table
                .filter(condition_type_definitions::id.eq(id))
                .filter(condition_type_definitions::workspace_id.eq(ws_id)),
        )
        .set((
            condition_type_definitions::name_i18n.eq(new_name),
            condition_type_definitions::category.eq(new_category),
            condition_type_definitions::default_minute_allowance.eq(new_min),
            condition_type_definitions::default_is_paid.eq(new_paid),
            condition_type_definitions::affects_rules.eq(new_affects),
            condition_type_definitions::icon.eq(new_icon),
            condition_type_definitions::color.eq(new_color),
            condition_type_definitions::sort_order.eq(new_sort),
            condition_type_definitions::is_active.eq(new_active),
            condition_type_definitions::updated_at.eq(now),
        ))
        .returning(ALL_COLUMNS)
        .get_result::<CondTypeRow>(&mut *conn)
        .await
        .map_err(AppError::from)?;

        Ok(row_to_definition(row))
    }

    async fn delete_condition_type(
        &self,
        auth_token: &TenantAuthToken,
        id: Uuid,
    ) -> Result<(), AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;
        let workspace_id = auth_token.workspace_id();

        let definition = condition_type_definitions::table
            .filter(condition_type_definitions::id.eq(id))
            .filter(condition_type_definitions::workspace_id.eq(workspace_id))
            .select((
                condition_type_definitions::id,
                condition_type_definitions::is_system,
            ))
            .first::<(Uuid, bool)>(&mut *conn)
            .await
            .optional()
            .map_err(AppError::from)?;

        let Some((_found_id, is_system)) = definition else {
            return Ok(());
        };

        if is_system {
            return Err(AppError::forbidden_args(
                crate::utils::i18n::I18nKey::Forbidden,
                [("reason", "System condition types cannot be deleted")],
            ));
        }

        diesel::delete(
            condition_type_definitions::table
                .filter(condition_type_definitions::id.eq(id))
                .filter(condition_type_definitions::workspace_id.eq(workspace_id)),
        )
        .execute(&mut *conn)
        .await
        .map_err(AppError::from)?;

        Ok(())
    }
}
