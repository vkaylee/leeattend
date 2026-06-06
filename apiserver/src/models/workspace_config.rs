use crate::schema::workspace_config_entries;
use chrono::{DateTime, NaiveDate, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum ConfigValue {
    Int(i32),
    Decimal(rust_decimal::Decimal),
    Text(String),
    Bool(bool),
    Json(serde_json::Value),
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ResolvedConfig {
    key: String,
    value: ConfigValue,
    scope_type: String,
    scope_id: Option<Uuid>,
}

impl ResolvedConfig {
    pub fn new(
        key: String,
        value: ConfigValue,
        scope_type: String,
        scope_id: Option<Uuid>,
    ) -> Self {
        Self {
            key,
            value,
            scope_type,
            scope_id,
        }
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn value(&self) -> &ConfigValue {
        &self.value
    }

    pub fn scope_type(&self) -> &str {
        &self.scope_type
    }

    pub fn scope_id(&self) -> Option<Uuid> {
        self.scope_id
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Insertable, ToSchema)]
#[diesel(table_name = workspace_config_entries)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct WorkspaceConfigEntry {
    id: Uuid,
    workspace_id: Uuid,
    scope_type: String,
    scope_id: Option<Uuid>,
    config_key: String,
    value_int: Option<i32>,
    value_decimal: Option<rust_decimal::Decimal>,
    value_text: Option<String>,
    value_bool: Option<bool>,
    value_json: Option<serde_json::Value>,
    effective_from: NaiveDate,
    effective_to: Option<NaiveDate>,
    created_by: Option<Uuid>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl WorkspaceConfigEntry {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: Uuid,
        workspace_id: Uuid,
        scope_type: String,
        scope_id: Option<Uuid>,
        config_key: String,
        value_int: Option<i32>,
        value_decimal: Option<rust_decimal::Decimal>,
        value_text: Option<String>,
        value_bool: Option<bool>,
        value_json: Option<serde_json::Value>,
        effective_from: NaiveDate,
        effective_to: Option<NaiveDate>,
        created_by: Option<Uuid>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            workspace_id,
            scope_type,
            scope_id,
            config_key,
            value_int,
            value_decimal,
            value_text,
            value_bool,
            value_json,
            effective_from,
            effective_to,
            created_by,
            created_at,
            updated_at,
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn workspace_id(&self) -> Uuid {
        self.workspace_id
    }

    pub fn set_workspace_id(&mut self, workspace_id: Uuid) {
        self.workspace_id = workspace_id;
    }

    pub fn scope_type(&self) -> &str {
        &self.scope_type
    }

    pub fn scope_id(&self) -> Option<Uuid> {
        self.scope_id
    }

    pub fn config_key(&self) -> &str {
        &self.config_key
    }

    pub fn value_int(&self) -> Option<i32> {
        self.value_int
    }

    pub fn value_decimal(&self) -> Option<rust_decimal::Decimal> {
        self.value_decimal
    }

    pub fn value_text(&self) -> Option<&str> {
        self.value_text.as_deref()
    }

    pub fn value_bool(&self) -> Option<bool> {
        self.value_bool
    }

    pub fn value_json(&self) -> Option<&serde_json::Value> {
        self.value_json.as_ref()
    }

    pub fn effective_from(&self) -> NaiveDate {
        self.effective_from
    }

    pub fn effective_to(&self) -> Option<NaiveDate> {
        self.effective_to
    }

    pub fn created_by(&self) -> Option<Uuid> {
        self.created_by
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}
