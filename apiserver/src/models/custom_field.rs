use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::models::diesel::custom_field::{CustomFieldDefinitionEntity, CustomFieldValueEntity};

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
#[allow(clippy::struct_excessive_bools)]
pub struct CustomFieldDefinition {
    id: Uuid,
    workspace_id: Uuid,
    entity_type: String,
    field_key: String,
    field_label: String,
    field_type: String,
    options: Option<Value>,
    default_value: Option<String>,
    is_required: bool,
    is_visible_to_employee: bool,
    is_filterable: bool,
    is_exportable: bool,
    display_order: i32,
    section_name: Option<String>,
    validation_rules: Option<Value>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl CustomFieldDefinition {
    #[allow(clippy::fn_params_excessive_bools)]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: Uuid,
        workspace_id: Uuid,
        entity_type: String,
        field_key: String,
        field_label: String,
        field_type: String,
        options: Option<Value>,
        default_value: Option<String>,
        is_required: bool,
        is_visible_to_employee: bool,
        is_filterable: bool,
        is_exportable: bool,
        display_order: i32,
        section_name: Option<String>,
        validation_rules: Option<Value>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            workspace_id,
            entity_type,
            field_key,
            field_label,
            field_type,
            options,
            default_value,
            is_required,
            is_visible_to_employee,
            is_filterable,
            is_exportable,
            display_order,
            section_name,
            validation_rules,
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

    pub fn entity_type(&self) -> &str {
        &self.entity_type
    }

    pub fn field_key(&self) -> &str {
        &self.field_key
    }

    pub fn field_label(&self) -> &str {
        &self.field_label
    }

    pub fn field_type(&self) -> &str {
        &self.field_type
    }

    pub fn options(&self) -> Option<&Value> {
        self.options.as_ref()
    }

    pub fn default_value(&self) -> Option<&str> {
        self.default_value.as_deref()
    }

    pub fn is_required(&self) -> bool {
        self.is_required
    }

    pub fn is_visible_to_employee(&self) -> bool {
        self.is_visible_to_employee
    }

    pub fn is_filterable(&self) -> bool {
        self.is_filterable
    }

    pub fn is_exportable(&self) -> bool {
        self.is_exportable
    }

    pub fn display_order(&self) -> i32 {
        self.display_order
    }

    pub fn section_name(&self) -> Option<&str> {
        self.section_name.as_deref()
    }

    pub fn validation_rules(&self) -> Option<&Value> {
        self.validation_rules.as_ref()
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}

impl From<CustomFieldDefinitionEntity> for CustomFieldDefinition {
    fn from(entity: CustomFieldDefinitionEntity) -> Self {
        Self::new(
            entity.id,
            entity.workspace_id,
            entity.entity_type,
            entity.field_key,
            entity.field_label,
            entity.field_type,
            entity.options,
            entity.default_value,
            entity.is_required,
            entity.is_visible_to_employee,
            entity.is_filterable,
            entity.is_exportable,
            entity.display_order,
            entity.section_name,
            entity.validation_rules,
            entity.created_at,
            entity.updated_at,
        )
    }
}

impl From<CustomFieldDefinition> for CustomFieldDefinitionEntity {
    fn from(d: CustomFieldDefinition) -> Self {
        Self {
            id: d.id,
            workspace_id: d.workspace_id,
            entity_type: d.entity_type,
            field_key: d.field_key,
            field_label: d.field_label,
            field_type: d.field_type,
            options: d.options,
            default_value: d.default_value,
            is_required: d.is_required,
            is_visible_to_employee: d.is_visible_to_employee,
            is_filterable: d.is_filterable,
            is_exportable: d.is_exportable,
            display_order: d.display_order,
            section_name: d.section_name,
            validation_rules: d.validation_rules,
            deleted_at: None,
            created_at: d.created_at,
            updated_at: d.updated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomFieldValue {
    id: Uuid,
    definition_id: Uuid,
    entity_id: Uuid,
    value_text: Option<String>,
    value_number: Option<Decimal>,
    value_date: Option<NaiveDate>,
    value_bool: Option<bool>,
    value_json: Option<Value>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl CustomFieldValue {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: Uuid,
        definition_id: Uuid,
        entity_id: Uuid,
        value_text: Option<String>,
        value_number: Option<Decimal>,
        value_date: Option<NaiveDate>,
        value_bool: Option<bool>,
        value_json: Option<Value>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            definition_id,
            entity_id,
            value_text,
            value_number,
            value_date,
            value_bool,
            value_json,
            created_at,
            updated_at,
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn definition_id(&self) -> Uuid {
        self.definition_id
    }

    pub fn entity_id(&self) -> Uuid {
        self.entity_id
    }

    pub fn value_text(&self) -> Option<&str> {
        self.value_text.as_deref()
    }

    pub fn value_number(&self) -> Option<Decimal> {
        self.value_number
    }

    pub fn value_date(&self) -> Option<NaiveDate> {
        self.value_date
    }

    pub fn value_bool(&self) -> Option<bool> {
        self.value_bool
    }

    pub fn value_json(&self) -> Option<&Value> {
        self.value_json.as_ref()
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    pub fn to_json_value(&self, field_type: &str) -> Value {
        match field_type {
            "text" | "select" | "url" | "email" | "phone" => self
                .value_text
                .as_ref()
                .map(|s| Value::String(s.clone()))
                .unwrap_or(Value::Null),
            "number" => self
                .value_number
                .as_ref()
                .and_then(|d| serde_json::Number::from_f64(d.to_f64().unwrap_or_default()))
                .map(Value::Number)
                .unwrap_or(Value::Null),
            "date" => self
                .value_date
                .as_ref()
                .map(|d| Value::String(d.to_string()))
                .unwrap_or(Value::Null),
            "boolean" => self.value_bool.map(Value::Bool).unwrap_or(Value::Null),
            "multiselect" => self.value_json.clone().unwrap_or(Value::Null),
            _ => Value::Null,
        }
    }

    pub fn from_json_value(
        id: Uuid,
        definition_id: Uuid,
        entity_id: Uuid,
        value: &Value,
        field_type: &str,
    ) -> Result<Self, String> {
        let mut value_text = None;
        let mut value_number = None;
        let mut value_date = None;
        let mut value_bool = None;
        let mut value_json = None;

        if !value.is_null() {
            match field_type {
                "text" | "select" | "url" | "email" | "phone" => {
                    value_text = Some(
                        value
                            .as_str()
                            .ok_or_else(|| {
                                format!("Expected string value for field type {field_type}")
                            })?
                            .to_string(),
                    );
                }
                "number" => {
                    let num_f64 = value.as_f64().ok_or_else(|| {
                        format!("Expected numeric value for field type {field_type}")
                    })?;
                    value_number = Some(
                        Decimal::try_from(num_f64)
                            .map_err(|e| format!("Invalid decimal number: {e}"))?,
                    );
                }
                "date" => {
                    let date_str = value.as_str().ok_or_else(|| {
                        format!("Expected string value for field type {field_type}")
                    })?;
                    let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
                        .map_err(|e| format!("Failed to parse date '{date_str}': {e}"))?;
                    value_date = Some(date);
                }
                "boolean" => {
                    value_bool = Some(value.as_bool().ok_or_else(|| {
                        format!("Expected boolean value for field type {field_type}")
                    })?);
                }
                "multiselect" => {
                    if !value.is_array() {
                        return Err("Expected array value for multiselect field".to_string());
                    }
                    value_json = Some(value.clone());
                }
                _ => {
                    return Err(format!("Unsupported field type {field_type}"));
                }
            }
        }

        Ok(Self {
            id,
            definition_id,
            entity_id,
            value_text,
            value_number,
            value_date,
            value_bool,
            value_json,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }
}

impl From<CustomFieldValueEntity> for CustomFieldValue {
    fn from(entity: CustomFieldValueEntity) -> Self {
        Self::new(
            entity.id,
            entity.definition_id,
            entity.entity_id,
            entity.value_text,
            entity.value_number,
            entity.value_date,
            entity.value_bool,
            entity.value_json,
            entity.created_at,
            entity.updated_at,
        )
    }
}

impl From<CustomFieldValue> for CustomFieldValueEntity {
    fn from(v: CustomFieldValue) -> Self {
        Self {
            id: v.id,
            definition_id: v.definition_id,
            entity_id: v.entity_id,
            value_text: v.value_text,
            value_number: v.value_number,
            value_date: v.value_date,
            value_bool: v.value_bool,
            value_json: v.value_json,
            created_at: v.created_at,
            updated_at: v.updated_at,
        }
    }
}

// ── API Request/Response DTOs ──────────────────────────────────────────

#[derive(Debug, Clone, Deserialize, Serialize, utoipa::ToSchema)]
#[allow(clippy::struct_excessive_bools)]
pub struct CreateCustomFieldDefinitionRequest {
    pub entity_type: String, // 'employee', 'employment', 'department'
    pub field_key: String,
    pub field_label: String,
    pub field_type: String, // 'text', 'number', 'date', 'boolean', 'select', 'multiselect', 'url', 'email', 'phone'
    pub options: Option<Value>,
    pub default_value: Option<String>,
    pub is_required: bool,
    pub is_visible_to_employee: bool,
    pub is_filterable: bool,
    pub is_exportable: bool,
    pub display_order: i32,
    pub section_name: Option<String>,
    pub validation_rules: Option<Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize, utoipa::ToSchema)]
pub struct UpdateCustomFieldDefinitionRequest {
    pub field_label: Option<String>,
    pub options: Option<Value>,
    #[serde(
        default,
        deserialize_with = "crate::utils::serde_helpers::deserialize_option_option"
    )]
    pub default_value: Option<Option<String>>,
    pub is_required: Option<bool>,
    pub is_visible_to_employee: Option<bool>,
    pub is_filterable: Option<bool>,
    pub is_exportable: Option<bool>,
    pub display_order: Option<i32>,
    #[serde(
        default,
        deserialize_with = "crate::utils::serde_helpers::deserialize_option_option"
    )]
    pub section_name: Option<Option<String>>,
    #[serde(
        default,
        deserialize_with = "crate::utils::serde_helpers::deserialize_option_option"
    )]
    pub validation_rules: Option<Option<Value>>,
}

#[derive(Debug, Clone, Deserialize, Serialize, utoipa::ToSchema)]
pub struct SetCustomFieldValueRequest {
    pub definition_id: Uuid,
    pub value: Option<Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize, utoipa::ToSchema)]
pub struct BulkSetCustomFieldValuesRequest {
    pub values: Vec<SetCustomFieldValueRequest>,
}

#[derive(Debug, Clone, Deserialize, Serialize, utoipa::ToSchema)]
pub struct CustomFieldValueResponse {
    pub definition_id: Uuid,
    pub field_key: String,
    pub field_label: String,
    pub field_type: String,
    pub value: Option<Value>,
}
