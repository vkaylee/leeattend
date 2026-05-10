use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    errors::AppError,
    models::contract::{CreateContractRequest, EmployeeContract, UpdateContractRequest},
    repositories::PoolBacked,
};

pub struct SqlxEmployeeContractRepository {
    pub pool: PgPool,
}

impl PoolBacked for SqlxEmployeeContractRepository {
    fn from_pool(pool: PgPool) -> Self {
        Self { pool }
    }

    fn pool(&self) -> &PgPool {
        &self.pool
    }
}

impl SqlxEmployeeContractRepository {
    pub async fn create_contract(
        &self,
        workspace_id: Uuid,
        req: &CreateContractRequest,
    ) -> Result<EmployeeContract, AppError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to start transaction: {e}")))?;

        // 1. Check for overlapping active contracts
        let overlaps = sqlx::query!(
            r#"
            SELECT id FROM employee_contracts
            WHERE workspace_id = $1 AND employee_id = $2 AND status = 'active'
              AND start_date <= $3
              AND (end_date IS NULL OR end_date >= $3)
            "#,
            workspace_id,
            req.employee_id,
            req.start_date
        )
        .fetch_all(&mut *tx)
        .await?;

        if !overlaps.is_empty() {
            if req.auto_close_overlapping.unwrap_or(false) {
                // Auto close overlapping contracts by truncating their end_date to start_date - 1 day
                let new_end_date = req.start_date.pred_opt().unwrap_or(req.start_date);
                let auto_close_note = format!(
                    "Hệ thống tự động chấm dứt sớm vào ngày {} để ký hợp đồng mới{}.",
                    new_end_date.format("%d/%m/%Y"),
                    if let Some(ref num) = req.contract_number {
                        format!(" {num}")
                    } else {
                        String::new()
                    }
                );

                sqlx::query!(
                    r#"
                    UPDATE employee_contracts
                    SET 
                        end_date = $4, 
                        status = 'expired', 
                        notes = CASE 
                            WHEN notes IS NULL OR notes = '' THEN $5
                            ELSE notes || E'\n---\n' || $5
                        END,
                        updated_at = NOW()
                    WHERE workspace_id = $1 AND employee_id = $2 AND status = 'active'
                      AND start_date <= $3
                      AND (end_date IS NULL OR end_date >= $3)
                    "#,
                    workspace_id,
                    req.employee_id,
                    req.start_date,
                    new_end_date,
                    auto_close_note
                )
                .execute(&mut *tx)
                .await?;
            } else {
                return Err(AppError::Conflict(
                    "overlapping_contract_exists".to_string(),
                ));
            }
        }

        // 2. Insert the new contract
        let contract = sqlx::query_as::<_, EmployeeContract>(
            r#"
            INSERT INTO employee_contracts (
                workspace_id, employee_id, contract_number, contract_type,
                start_date, end_date, document_id, notes
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
            "#,
        )
        .bind(workspace_id)
        .bind(req.employee_id)
        .bind(&req.contract_number)
        .bind(req.contract_type)
        .bind(req.start_date)
        .bind(req.end_date)
        .bind(req.document_id)
        .bind(&req.notes)
        .fetch_one(&mut *tx)
        .await?;

        tx.commit()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to commit transaction: {e}")))?;

        Ok(contract)
    }

    pub async fn get_contract_by_id(
        &self,
        workspace_id: Uuid,
        contract_id: Uuid,
    ) -> Result<Option<EmployeeContract>, AppError> {
        let contract = sqlx::query_as::<_, EmployeeContract>(
            r#"
            SELECT * FROM employee_contracts
            WHERE id = $1 AND workspace_id = $2
            "#,
        )
        .bind(contract_id)
        .bind(workspace_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(contract)
    }

    pub async fn list_employee_contracts(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
    ) -> Result<Vec<EmployeeContract>, AppError> {
        let contracts = sqlx::query_as::<_, EmployeeContract>(
            r#"
            SELECT * FROM employee_contracts
            WHERE employee_id = $1 AND workspace_id = $2
            ORDER BY start_date DESC
            "#,
        )
        .bind(employee_id)
        .bind(workspace_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(contracts)
    }

    pub async fn update_contract(
        &self,
        workspace_id: Uuid,
        contract_id: Uuid,
        req: &UpdateContractRequest,
    ) -> Result<EmployeeContract, AppError> {
        let contract = sqlx::query_as::<_, EmployeeContract>(
            r#"
            UPDATE employee_contracts
            SET 
                contract_number = COALESCE($3, contract_number),
                contract_type = COALESCE($4, contract_type),
                status = COALESCE($5, status),
                start_date = COALESCE($6, start_date),
                end_date = COALESCE($7, end_date),
                document_id = COALESCE($8, document_id),
                notes = COALESCE($9, notes),
                updated_at = NOW()
            WHERE id = $1 AND workspace_id = $2
            RETURNING *
            "#,
        )
        .bind(contract_id)
        .bind(workspace_id)
        .bind(&req.contract_number)
        .bind(req.contract_type)
        .bind(req.status)
        .bind(req.start_date)
        .bind(req.end_date)
        .bind(req.document_id)
        .bind(&req.notes)
        .fetch_one(&self.pool)
        .await?;

        Ok(contract)
    }

    pub async fn delete_contract(
        &self,
        workspace_id: Uuid,
        contract_id: Uuid,
    ) -> Result<(), AppError> {
        sqlx::query(
            r#"
            DELETE FROM employee_contracts
            WHERE id = $1 AND workspace_id = $2
            "#,
        )
        .bind(contract_id)
        .bind(workspace_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_expiring_contracts(
        &self,
        workspace_id: Uuid,
        days_ahead: i32,
    ) -> Result<Vec<EmployeeContract>, AppError> {
        let contracts = sqlx::query_as::<_, EmployeeContract>(
            r#"
            SELECT * FROM employee_contracts
            WHERE workspace_id = $1
              AND status = 'active'
              AND end_date IS NOT NULL
              AND end_date <= CURRENT_DATE + $2::int
            "#,
        )
        .bind(workspace_id)
        .bind(days_ahead)
        .fetch_all(&self.pool)
        .await?;

        Ok(contracts)
    }
}
