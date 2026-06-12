use chrono::{DateTime, NaiveDate, Utc};
use diesel::prelude::*;
use rust_decimal::Decimal;
use serde_json::Value;
use uuid::Uuid;

use crate::schema::{custom_field_definitions, custom_field_values};

#[derive(Debug, Clone, Queryable, Selectable, Insertable, Identifiable, AsChangeset)]
#[diesel(table_name = custom_field_definitions)]
#[allow(clippy::struct_excessive_bools)]
pub struct CustomFieldDefinitionEntity {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub entity_type: String,
    pub field_key: String,
    pub field_label: String,
    pub field_type: String,
    pub options: Option<Value>,
    pub default_value: Option<String>,
    pub is_required: bool,
    pub is_visible_to_employee: bool,
    pub is_filterable: bool,
    pub is_exportable: bool,
    pub display_order: i32,
    pub section_name: Option<String>,
    pub validation_rules: Option<Value>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Queryable, Selectable, Insertable, Identifiable, AsChangeset)]
#[diesel(table_name = custom_field_values)]
pub struct CustomFieldValueEntity {
    pub id: Uuid,
    pub definition_id: Uuid,
    pub entity_id: Uuid,
    pub value_text: Option<String>,
    pub value_number: Option<Decimal>,
    pub value_date: Option<NaiveDate>,
    pub value_bool: Option<bool>,
    pub value_json: Option<Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
