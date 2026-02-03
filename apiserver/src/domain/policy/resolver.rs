use chrono::NaiveDate;
use rust_decimal::Decimal;
use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;
use crate::models::{EarlyArrivalStrategy, PolicyRules, WorkingCondition};

#[derive(Debug, Clone)]
pub struct StandardMonthConfig {
    pub std_working_days: Decimal,
    pub hours_per_day: Decimal,
    pub source: String,
}

pub struct PolicyResolver;

impl PolicyResolver {
    pub async fn resolve_standard_month(
        pool: &PgPool,
        employee_id: Uuid,
        department_id: Option<Uuid>,
        workspace_id: Uuid,
        date: NaiveDate,
    ) -> Result<StandardMonthConfig, sqlx::Error> {
        // 1. Check Individual Override (Highest Priority)
        let emp_config = crate::repositories::employee_repository::get_employee_payroll_config_at(
            pool,
            employee_id,
            date
        ).await?;
        
        if let Some(config) = emp_config {
            return Ok(StandardMonthConfig {
                std_working_days: config.std_working_days,
                hours_per_day: config.hours_per_day,
                source: "individual".to_string(),
            });
        }

        // 2. Check Department Override
        if let Some(dept_id) = department_id {
            let dept_config = crate::repositories::department_repository::get_department_payroll_config_at(
                pool,
                dept_id,
                date
            ).await?;

            if let Some(config) = dept_config {
                return Ok(StandardMonthConfig {
                    std_working_days: config.std_working_days,
                    hours_per_day: config.hours_per_day,
                    source: "department".to_string(),
                });
            }
        }

        // 3. Fallback to Workspace Settings (Temporal)
        let ws_settings = crate::repositories::payroll_repository::get_payroll_settings_at(
            pool,
            workspace_id,
            date
        ).await?;

        if let Some(settings) = ws_settings {
             return Ok(StandardMonthConfig {
                 std_working_days: settings.fixed_std_days,
                 hours_per_day: settings.hours_per_day,
                 source: "workspace".to_string(),
             });
        }

        // 4. System Default fallback
        Ok(StandardMonthConfig {
            std_working_days: Decimal::from(26),
            hours_per_day: Decimal::from(8),
            source: "system".to_string(),
        })
    }

    // Deprecated in favor of resolve_standard_month, keeping for small logic pieces if needed
    pub async fn resolve_standard_days(
        pool: &PgPool,
        employee_id: Uuid,
        department_id: Option<Uuid>,
        workspace_id: Uuid,
        date: NaiveDate,
    ) -> Result<Decimal, sqlx::Error> {
        let config = Self::resolve_standard_month(pool, employee_id, department_id, workspace_id, date).await?;
        Ok(config.std_working_days)
    }

    pub async fn resolve_attendance_rules(
        pool: &PgPool,
        employee_id: Uuid,
        workspace_id: Uuid,
        date: NaiveDate,
    ) -> Result<PolicyRules, sqlx::Error> {
        // 1. Get Workspace Defaults
        let ws = crate::repositories::workspace_repository::get_workspace_by_id(pool, workspace_id)
            .await?
            .ok_or_else(|| sqlx::Error::RowNotFound)?;

        // 2. Get Employee Special Conditions
        let conditions = Self::resolve_working_conditions(pool, employee_id, date).await?;

        // 3. Merge logic
        let mut rules = PolicyRules {
            grace_period_minutes: 15, // Default company grace
            authorized_early_leave_minutes: 0,
            is_early_leave_paid: false,
            early_arrival_strategy: ws.early_arrival_strategy,
            is_break_auto_deduct: true, // Default
            is_break_overtime: false,
            allow_early_ot: false,
        };

        for cond in conditions {
            match cond.type_.as_str() {
                "EARLY_LEAVE" => {
                    // Take the most generous allowance
                    if cond.minute_allowance > rules.authorized_early_leave_minutes {
                        rules.authorized_early_leave_minutes = cond.minute_allowance;
                        rules.is_early_leave_paid = cond.is_paid;
                    }
                }
                "BREASTFEEDING" => {
                    // Example: breastfeeding might also be 60m paid early leave
                    if cond.minute_allowance > rules.authorized_early_leave_minutes {
                        rules.authorized_early_leave_minutes = cond.minute_allowance;
                        rules.is_early_leave_paid = true;
                    }
                }
                _ => {}
            }
        }

        Ok(rules)
    }

    pub async fn resolve_working_conditions(
        pool: &PgPool,
        employee_id: Uuid,
        date: NaiveDate,
    ) -> Result<Vec<WorkingCondition>, sqlx::Error> {
        let conditions = sqlx::query_as!(
            WorkingCondition,
            r#"
            SELECT id, employee_id, type as "type_", minute_allowance, is_paid, start_date, end_date
            FROM working_conditions 
            WHERE employee_id = $1 
              AND start_date <= $2 
              AND (end_date IS NULL OR end_date >= $2)
            "#,
            employee_id,
            date
        )
        .fetch_all(pool)
        .await?;

        Ok(conditions)
    }

    /// Batch resolve attendance rules for multiple employees.
    /// Returns a HashMap keyed by employee_id.
    /// This eliminates N+1 queries when processing batches.
    pub async fn resolve_attendance_rules_batch(
        pool: &PgPool,
        employee_ids: &[Uuid],
        _workspace_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<HashMap<Uuid, Vec<WorkingCondition>>, sqlx::Error> {
        if employee_ids.is_empty() {
            return Ok(HashMap::new());
        }

        // Batch fetch all working conditions for all employees in date range
        let conditions = sqlx::query_as!(
            WorkingCondition,
            r#"
            SELECT id, employee_id, type as "type_", minute_allowance, is_paid, start_date, end_date
            FROM working_conditions
            WHERE employee_id = ANY($1)
              AND start_date <= $3
              AND (end_date IS NULL OR end_date >= $2)
            ORDER BY employee_id, start_date
            "#,
            employee_ids,
            start_date,
            end_date
        )
        .fetch_all(pool)
        .await?;

        // Group by employee_id
        let mut result: HashMap<Uuid, Vec<WorkingCondition>> = HashMap::with_capacity(employee_ids.len());
        for cond in conditions {
            result.entry(cond.employee_id).or_default().push(cond);
        }

        Ok(result)
    }

    /// Build PolicyRules from workspace defaults and working conditions.
    /// Used after batch fetching conditions to construct rules per employee/date.
    pub fn build_rules_from_conditions(
        conditions: &[WorkingCondition],
        date: NaiveDate,
        early_arrival_strategy: EarlyArrivalStrategy,
    ) -> PolicyRules {
        let mut rules = PolicyRules {
            grace_period_minutes: 15,
            authorized_early_leave_minutes: 0,
            is_early_leave_paid: false,
            early_arrival_strategy,
            is_break_auto_deduct: true,
            is_break_overtime: false,
            allow_early_ot: false,
        };

        // Filter conditions active on this specific date
        for cond in conditions.iter().filter(|c| {
            c.start_date <= date && c.end_date.is_none_or(|end| end >= date)
        }) {
            match cond.type_.as_str() {
                "EARLY_LEAVE" => {
                    if cond.minute_allowance > rules.authorized_early_leave_minutes {
                        rules.authorized_early_leave_minutes = cond.minute_allowance;
                        rules.is_early_leave_paid = cond.is_paid;
                    }
                }
                "BREASTFEEDING" => {
                    if cond.minute_allowance > rules.authorized_early_leave_minutes {
                        rules.authorized_early_leave_minutes = cond.minute_allowance;
                        rules.is_early_leave_paid = true;
                    }
                }
                _ => {}
            }
        }

        rules
    }
}