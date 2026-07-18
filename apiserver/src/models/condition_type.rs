use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::EarlyArrivalStrategy;

/// JSONB contract for the policy fields a condition type changes.
///
/// `authorized_early_leave_minutes` and `grace_period_minutes` derive from
/// the assigned condition's `minute_allowance`; `is_early_leave_paid` derives
/// from its `is_paid` flag. The break and early-OT fields are literal values.
/// `early_arrival_strategy` accepts the API strings `Allow`, `Block`, or
/// `Clamp`.
#[derive(Debug, Clone, Default, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
pub struct ConditionRuleEffects {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authorized_early_leave_minutes: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_early_leave_paid: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub grace_period_minutes: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_break_auto_deduct: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_break_overtime: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allow_early_ot: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub early_arrival_strategy: Option<ConditionEarlyArrivalStrategy>,
}

impl ConditionRuleEffects {
    pub fn has_effect(&self) -> bool {
        self.authorized_early_leave_minutes == Some(true)
            || self.is_early_leave_paid == Some(true)
            || self.grace_period_minutes == Some(true)
            || self.is_break_auto_deduct.is_some()
            || self.is_break_overtime.is_some()
            || self.allow_early_ot.is_some()
            || self.early_arrival_strategy.is_some()
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, utoipa::ToSchema)]
pub enum ConditionEarlyArrivalStrategy {
    Allow,
    Block,
    Clamp,
}

impl From<ConditionEarlyArrivalStrategy> for EarlyArrivalStrategy {
    fn from(value: ConditionEarlyArrivalStrategy) -> Self {
        match value {
            ConditionEarlyArrivalStrategy::Allow => Self::Allow,
            ConditionEarlyArrivalStrategy::Block => Self::Block,
            ConditionEarlyArrivalStrategy::Clamp => Self::Clamp,
        }
    }
}

/// Workspace-scoped catalog entry defining a type of working condition.
///
/// The `affects_rules` JSONB field follows [`ConditionRuleEffects`], enabling
/// a data-driven resolver instead of hardcoded string matching.
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
