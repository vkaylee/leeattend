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
    pub key: String,
    pub value: ConfigValue,
    pub scope_type: String,
    pub scope_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Insertable, ToSchema)]
#[diesel(table_name = workspace_config_entries)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct WorkspaceConfigEntry {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub scope_type: String,
    pub scope_id: Option<Uuid>,
    pub config_key: String,
    pub value_int: Option<i32>,
    pub value_decimal: Option<rust_decimal::Decimal>,
    pub value_text: Option<String>,
    pub value_bool: Option<bool>,
    pub value_json: Option<serde_json::Value>,
    pub effective_from: NaiveDate,
    pub effective_to: Option<NaiveDate>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
