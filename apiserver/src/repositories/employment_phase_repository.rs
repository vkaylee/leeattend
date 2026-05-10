use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::{EmployeePhase, EmploymentPhaseDefinition};

pub struct EmploymentPhaseRepository;

impl EmploymentPhaseRepository {
    pub async fn create_definition(
        pool: &PgPool,
        workspace_id: Uuid,
        name: &str,
        description: Option<&str>,
        phase_type: &str,
        rule_config: &serde_json::Value,
        display_order: i32,
    ) -> Result<EmploymentPhaseDefinition, AppError> {
        let row = sqlx::query_as!(
            EmploymentPhaseDefinition,
            r#"
            INSERT INTO employment_phase_definitions (
                workspace_id, name, description, phase_type, rule_config, display_order
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
            "#,
            workspace_id,
            name,
            description,
            phase_type,
            rule_config,
            display_order
        )
        .fetch_one(pool)
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(db_err) = &e {
                if db_err.constraint() == Some("employment_phase_definitions_workspace_id_name_key")
                {
                    return AppError::Validation(
                        "Phase definition with this name already exists in the workspace."
                            .to_string(),
                    );
                }
            }
            AppError::Internal(format!("Failed to create employment phase definition: {e}"))
        })?;

        Ok(row)
    }

    pub async fn get_definitions(
        pool: &PgPool,
        workspace_id: Uuid,
    ) -> Result<Vec<EmploymentPhaseDefinition>, AppError> {
        let rows = sqlx::query_as!(
            EmploymentPhaseDefinition,
            r#"
            SELECT * FROM employment_phase_definitions
            WHERE workspace_id = $1
            ORDER BY display_order ASC, created_at DESC
            "#,
            workspace_id
        )
        .fetch_all(pool)
        .await
        .map_err(|e| {
            AppError::Internal(format!("Failed to fetch employment phase definitions: {e}"))
        })?;

        Ok(rows)
    }

    pub async fn assign_phase(
        pool: &PgPool,
        workspace_id: Uuid,
        employee_id: Uuid,
        phase_definition_id: Uuid,
        start_date: chrono::NaiveDate,
        end_date: chrono::NaiveDate,
        override_config: Option<&serde_json::Value>,
    ) -> Result<EmployeePhase, AppError> {
        let mut tx = pool.begin().await.map_err(|e| {
            AppError::Internal(format!("Failed to start transaction for assign_phase: {e}"))
        })?;

        // Auto-close any active phases that started before or on the new start_date
        let auto_close_date = start_date
            .checked_sub_days(chrono::Days::new(1))
            .unwrap_or(start_date);

        sqlx::query!(
            r#"
            UPDATE employee_phases
            SET status = 'completed', 
                end_date = LEAST(end_date, GREATEST(start_date, $1)),
                completed_at = CURRENT_TIMESTAMP,
                updated_at = CURRENT_TIMESTAMP
            WHERE workspace_id = $2 AND employee_id = $3
              AND status = 'active'
              AND start_date <= $4
            "#,
            auto_close_date,
            workspace_id,
            employee_id,
            start_date
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to auto-close overlapping phases: {e}")))?;

        let row = sqlx::query_as!(
            EmployeePhase,
            r#"
            INSERT INTO employee_phases (
                workspace_id, employee_id, phase_definition_id, start_date, end_date, override_config
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
            "#,
            workspace_id,
            employee_id,
            phase_definition_id,
            start_date,
            end_date,
            override_config
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to assign employment phase: {e}")))?;

        tx.commit().await.map_err(|e| {
            AppError::Internal(format!(
                "Failed to commit transaction for assign_phase: {e}"
            ))
        })?;

        Ok(row)
    }

    pub async fn update_phase_status(
        pool: &PgPool,
        workspace_id: Uuid,
        employee_id: Uuid,
        phase_id: Uuid,
        status: &str,
        evaluation_notes: Option<&str>,
        evaluated_by: Uuid,
    ) -> Result<EmployeePhase, AppError> {
        let row = sqlx::query_as!(
            EmployeePhase,
            r#"
            UPDATE employee_phases
            SET status = $1, evaluation_notes = $2, evaluated_by = $3, completed_at = CURRENT_TIMESTAMP, updated_at = CURRENT_TIMESTAMP
            WHERE id = $4 AND workspace_id = $5 AND employee_id = $6
            RETURNING *
            "#,
            status,
            evaluation_notes,
            evaluated_by,
            phase_id,
            workspace_id,
            employee_id
        )
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to update employment phase status: {e}")))?;

        row.ok_or_else(|| AppError::NotFound("Employee phase not found".to_string()))
    }

    pub async fn get_employee_phases(
        pool: &PgPool,
        workspace_id: Uuid,
        employee_id: Uuid,
    ) -> Result<Vec<EmployeePhase>, AppError> {
        let rows = sqlx::query_as!(
            EmployeePhase,
            r#"
            SELECT * FROM employee_phases
            WHERE workspace_id = $1 AND employee_id = $2
            ORDER BY start_date ASC, created_at DESC
            "#,
            workspace_id,
            employee_id
        )
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to fetch employee phases: {e}")))?;

        Ok(rows)
    }
}
