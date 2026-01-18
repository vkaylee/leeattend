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

pub struct PolicyResolver;

impl PolicyResolver {
    pub async fn resolve_standard_days(
        pool: &PgPool,
        _employee_id: Uuid,
        department_id: Option<Uuid>,
        workspace_id: Uuid,
        _date: NaiveDate,
    ) -> Result<Decimal, sqlx::Error> {
        // 1. Check Department Override
        if let Some(dept_id) = department_id {
            let dept_settings = sqlx::query!(
                "SELECT std_working_days FROM departments WHERE id = $1",
                dept_id
            )
            .fetch_optional(pool)
            .await?;

            if let Some(row) = dept_settings {
                if let Some(days) = row.std_working_days {
                    return Ok(days);
                }
            }
        }

        // 2. Fallback to Workspace Settings
        let ws_settings = sqlx::query!(
            "SELECT fixed_std_days FROM payroll_settings WHERE workspace_id = $1",
            workspace_id
        )
        .fetch_optional(pool)
        .await?;

        if let Some(settings) = ws_settings {
             return Ok(settings.fixed_std_days);
        }

        // 3. Default fallback
        Ok(Decimal::from(26))
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
