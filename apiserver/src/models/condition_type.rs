use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Workspace-scoped catalog entry defining a type of working condition.
///
/// Each workspace gets 5 system types seeded by migration (EARLY_LEAVE,
/// BREASTFEEDING, MATERNITY, SICK, UNPAID) and can create custom types.
/// The `affects_rules` JSONB field describes which PolicyRules fields
/// this condition type modifies, enabling a data-driven resolver instead
/// of hardcoded string matching.
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ConditionTypeDefinition {
    id: Uuid,
    workspace_id: Uuid,
    code: String,
    name_i18n: serde_json::Value,
    category: String,
    default_minute_allowance: i32,
    default_is_paid: bool,
    affects_rules: serde_json::Value,
    icon: Option<String>,
    color: Option<String>,
    is_system: bool,
    sort_order: i32,
    is_active: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl ConditionTypeDefinition {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: Uuid,
        workspace_id: Uuid,
        code: String,
        name_i18n: serde_json::Value,
        category: String,
        default_minute_allowance: i32,
        default_is_paid: bool,
        affects_rules: serde_json::Value,
        icon: Option<String>,
        color: Option<String>,
        is_system: bool,
        sort_order: i32,
        is_active: bool,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            workspace_id,
            code,
            name_i18n,
            category,
            default_minute_allowance,
            default_is_paid,
            affects_rules,
            icon,
            color,
            is_system,
            sort_order,
            is_active,
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

    pub fn code(&self) -> &str {
        &self.code
    }

    pub fn name_i18n(&self) -> &serde_json::Value {
        &self.name_i18n
    }

    pub fn category(&self) -> &str {
        &self.category
    }

    pub fn default_minute_allowance(&self) -> i32 {
        self.default_minute_allowance
    }

    pub fn default_is_paid(&self) -> bool {
        self.default_is_paid
    }

    pub fn affects_rules(&self) -> &serde_json::Value {
        &self.affects_rules
    }

    pub fn icon(&self) -> Option<&str> {
        self.icon.as_deref()
    }

    pub fn color(&self) -> Option<&str> {
        self.color.as_deref()
    }

    pub fn is_system(&self) -> bool {
        self.is_system
    }

    pub fn sort_order(&self) -> i32 {
        self.sort_order
    }

    pub fn is_active(&self) -> bool {
        self.is_active
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CreateConditionTypeRequest {
    pub code: String,
    pub name_i18n: serde_json::Value,
    #[serde(default = "default_category")]
    pub category: String,
    #[serde(default)]
    pub default_minute_allowance: i32,
    #[serde(default)]
    pub default_is_paid: bool,
    #[serde(default)]
    pub affects_rules: serde_json::Value,
    pub icon: Option<String>,
    pub color: Option<String>,
    #[serde(default)]
    pub sort_order: i32,
}

fn default_category() -> String {
    "general".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct UpdateConditionTypeRequest {
    pub name_i18n: Option<serde_json::Value>,
    pub category: Option<String>,
    pub default_minute_allowance: Option<i32>,
    pub default_is_paid: Option<bool>,
    pub affects_rules: Option<serde_json::Value>,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub sort_order: Option<i32>,
    pub is_active: Option<bool>,
}
