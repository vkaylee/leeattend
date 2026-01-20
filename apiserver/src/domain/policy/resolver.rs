use chrono::NaiveDate;
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct WorkingCondition {
    pub type_: String,
    pub minute_allowance: i32,
    pub is_paid: bool,
}

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
            
            // Legacy Fallback
            let dept_legacy = sqlx::query!(
                "SELECT std_working_days FROM departments WHERE id = $1",
                dept_id
            )
            .fetch_optional(pool)
            .await?;

            if let Some(row) = dept_legacy {
                if let Some(days) = row.std_working_days {
                    return Ok(StandardMonthConfig {
                        std_working_days: days,
                        hours_per_day: Decimal::from(8), // Default legacy hours
                        source: "department".to_string(),
                    });
                }
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

    pub async fn resolve_working_conditions(
        pool: &PgPool,
        employee_id: Uuid,
        date: NaiveDate,
    ) -> Result<Vec<WorkingCondition>, sqlx::Error> {
        let conditions = sqlx::query_as!(
            WorkingCondition,
            r#"
            SELECT type as type_, minute_allowance, is_paid 
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
}
