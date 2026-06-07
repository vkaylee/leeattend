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

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct EffectivePayrollSettings {
    pub std_working_day_mode: String,
    pub fixed_std_days: rust_decimal::Decimal,
    pub working_days_per_week: i32,
    pub hours_per_day: rust_decimal::Decimal,
    pub max_ot_hours_per_day: Option<rust_decimal::Decimal>,
    pub max_ot_hours_per_month: Option<rust_decimal::Decimal>,
}

pub struct WorkspaceConfigResolver {
    entries: Vec<WorkspaceConfigEntry>,
}

impl WorkspaceConfigResolver {
    pub fn new(entries: Vec<WorkspaceConfigEntry>) -> Self {
        Self { entries }
    }

    pub fn resolve_key(
        &self,
        config_key: &str,
        employee_id: Uuid,
        department_id: Option<Uuid>,
        employer_id: Option<Uuid>,
        target_date: NaiveDate,
    ) -> Option<&WorkspaceConfigEntry> {
        let mut best_entry: Option<&WorkspaceConfigEntry> = None;
        let mut best_scope_priority = 0; // 0: none, 1: workspace, 2: employer, 3: department, 4: employee

        for entry in &self.entries {
            if entry.config_key() != config_key {
                continue;
            }
            if entry.effective_from() > target_date {
                continue;
            }
            if let Some(to) = entry.effective_to() {
                if to < target_date {
                    continue;
                }
            }

            let (scope_match, priority) = match entry.scope_type() {
                "employee" => (entry.scope_id() == Some(employee_id), 4),
                "department" => (
                    department_id.is_some() && entry.scope_id() == department_id,
                    3,
                ),
                "employer" => (employer_id.is_some() && entry.scope_id() == employer_id, 2),
                "workspace" => (true, 1),
                _ => (false, 0),
            };

            if scope_match {
                if priority > best_scope_priority {
                    best_scope_priority = priority;
                    best_entry = Some(entry);
                } else if priority == best_scope_priority {
                    if let Some(current_best) = best_entry {
                        if entry.effective_from() > current_best.effective_from() {
                            best_entry = Some(entry);
                        }
                    } else {
                        best_entry = Some(entry);
                    }
                }
            }
        }

        best_entry
    }

    pub fn resolve_payroll_settings(
        &self,
        employee_id: Uuid,
        department_id: Option<Uuid>,
        employer_id: Option<Uuid>,
        target_date: NaiveDate,
    ) -> EffectivePayrollSettings {
        let std_working_day_mode = self
            .resolve_key(
                "payroll.std_working_day_mode",
                employee_id,
                department_id,
                employer_id,
                target_date,
            )
            .and_then(|e| e.value_text())
            .unwrap_or("fixed")
            .to_string();

        let fixed_std_days = self
            .resolve_key(
                "payroll.fixed_std_days",
                employee_id,
                department_id,
                employer_id,
                target_date,
            )
            .and_then(WorkspaceConfigEntry::value_decimal)
            .unwrap_or_else(|| {
                rust_decimal::Decimal::from(
                    crate::domain::policy::defaults::DEFAULT_STD_WORKING_DAYS,
                )
            });

        let working_days_per_week = self
            .resolve_key(
                "payroll.working_days_per_week",
                employee_id,
                department_id,
                employer_id,
                target_date,
            )
            .and_then(WorkspaceConfigEntry::value_int)
            .unwrap_or(5);

        let hours_per_day = self
            .resolve_key(
                "payroll.hours_per_day",
                employee_id,
                department_id,
                employer_id,
                target_date,
            )
            .and_then(WorkspaceConfigEntry::value_decimal)
            .unwrap_or_else(|| {
                rust_decimal::Decimal::from(crate::domain::policy::defaults::DEFAULT_HOURS_PER_DAY)
            });

        let max_ot_hours_per_day = self
            .resolve_key(
                "payroll.max_ot_hours_per_day",
                employee_id,
                department_id,
                employer_id,
                target_date,
            )
            .and_then(WorkspaceConfigEntry::value_decimal);

        let max_ot_hours_per_month = self
            .resolve_key(
                "payroll.max_ot_hours_per_month",
                employee_id,
                department_id,
                employer_id,
                target_date,
            )
            .and_then(WorkspaceConfigEntry::value_decimal);

        EffectivePayrollSettings {
            std_working_day_mode,
            fixed_std_days,
            working_days_per_week,
            hours_per_day,
            max_ot_hours_per_day,
            max_ot_hours_per_month,
        }
    }
}
