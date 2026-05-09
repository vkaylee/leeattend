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
        .fetch_one(&self.pool)
        .await?;

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
