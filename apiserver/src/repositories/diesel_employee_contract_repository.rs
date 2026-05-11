use chrono::Utc;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    diesel_pool::DieselPools,
    errors::AppError,
    models::contract::{CreateContractRequest, EmployeeContract, UpdateContractRequest},
    models::diesel::employee_contract::{EmployeeContractEntity, UpdateDieselEmployeeContract},
};

pub struct DieselEmployeeContractRepository {
    pub sqlx_pool: PgPool,
    pub diesel_pools: DieselPools,
}

impl DieselEmployeeContractRepository {
    pub fn new(sqlx_pool: PgPool, diesel_pools: DieselPools) -> Self {
        Self {
            sqlx_pool,
            diesel_pools,
        }
    }

    pub async fn create_contract(
        &self,
        workspace_id: Uuid,
        req: &CreateContractRequest,
    ) -> Result<EmployeeContract, AppError> {
        // Complex business logic involving string concatenation for notes
        // Delegated to legacy Sqlx implementation for safety and accuracy
        crate::repositories::employee_contract_repository::SqlxEmployeeContractRepository {
            pool: self.sqlx_pool.clone(),
        }
        .create_contract(workspace_id, req)
        .await
    }

    pub async fn get_contract_by_id(
        &self,
        workspace_id_val: Uuid,
        contract_id: Uuid,
    ) -> Result<Option<EmployeeContract>, AppError> {
        use crate::schema::employee_contracts::dsl::*;

        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to get connection: {e}")))?;
        let contract = employee_contracts
            .filter(id.eq(contract_id))
            .filter(workspace_id.eq(workspace_id_val))
            .first::<EmployeeContractEntity>(&mut conn)
            .await
            .optional()?;

        Ok(contract.map(Into::into))
    }

    pub async fn list_employee_contracts(
        &self,
        workspace_id_val: Uuid,
        employee_id_val: Uuid,
    ) -> Result<Vec<EmployeeContract>, AppError> {
        use crate::schema::employee_contracts::dsl::*;

        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to get connection: {e}")))?;
        let contracts = employee_contracts
            .filter(employee_id.eq(employee_id_val))
            .filter(workspace_id.eq(workspace_id_val))
            .order(start_date.desc())
            .load::<EmployeeContractEntity>(&mut conn)
            .await?;

        Ok(contracts.into_iter().map(Into::into).collect())
    }

    pub async fn update_contract(
        &self,
        workspace_id_val: Uuid,
        contract_id: Uuid,
        req: &UpdateContractRequest,
    ) -> Result<EmployeeContract, AppError> {
        use crate::schema::employee_contracts::dsl::*;

        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to get connection: {e}")))?;

        let update_model = UpdateDieselEmployeeContract {
            contract_number: req.contract_number.clone(),
            contract_type: req.contract_type,
            status: req.status,
            start_date: req.start_date,
            end_date: req.end_date.map(Some), // wrap in Some because Option<Option<T>> where None means don't update
            document_id: req.document_id.map(Some),
            notes: req.notes.clone().map(Some),
            updated_at: Some(Utc::now()),
        };

        let updated_contract = diesel::update(
            employee_contracts
                .filter(id.eq(contract_id))
                .filter(workspace_id.eq(workspace_id_val)),
        )
        .set(&update_model)
        .get_result::<EmployeeContractEntity>(&mut conn)
        .await?;

        Ok(updated_contract.into())
    }

    pub async fn delete_contract(
        &self,
        workspace_id_val: Uuid,
        contract_id: Uuid,
    ) -> Result<(), AppError> {
        use crate::schema::employee_contracts::dsl::*;

        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to get connection: {e}")))?;
        diesel::delete(
            employee_contracts
                .filter(id.eq(contract_id))
                .filter(workspace_id.eq(workspace_id_val)),
        )
        .execute(&mut conn)
        .await?;

        Ok(())
    }

    pub async fn get_expiring_contracts(
        &self,
        workspace_id_val: Uuid,
        days_ahead: i32,
    ) -> Result<Vec<EmployeeContract>, AppError> {
        use crate::models::enums::ContractStatus;
        use crate::schema::employee_contracts::dsl::*;

        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to get connection: {e}")))?;

        let target_date =
            chrono::Utc::now().naive_utc().date() + chrono::Duration::days(days_ahead as i64);

        let contracts = employee_contracts
            .filter(workspace_id.eq(workspace_id_val))
            .filter(status.eq(ContractStatus::Active))
            .filter(end_date.is_not_null())
            .filter(end_date.le(target_date))
            .load::<EmployeeContractEntity>(&mut conn)
            .await?;

        Ok(contracts.into_iter().map(Into::into).collect())
    }
}
