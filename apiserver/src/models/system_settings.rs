use crate::schema::system_settings;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

#[derive(
    Debug,
    Clone,
    Serialize,
    Deserialize,
    Queryable,
    Selectable,
    Insertable,
    AsChangeset,
    utoipa::ToSchema,
)]
#[diesel(table_name = system_settings)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct SystemSetting {
    pub key: String,
    pub value_json: JsonValue,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Insertable)]
#[diesel(table_name = system_settings)]
pub struct NewSystemSetting {
    pub key: String,
    pub value_json: JsonValue,
}
