use chrono::Utc;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::{
    diesel_pool::DieselPools,
    errors::AppError,
    models::contract::{CreateContractRequest, EmployeeContract, UpdateContractRequest},
    models::diesel::employee_contract::{EmployeeContractEntity, UpdateDieselEmployeeContract},
};

pub struct DieselEmployeeContractRepository {
    pub diesel_pools: DieselPools,
}

impl DieselEmployeeContractRepository {
    pub fn new(diesel_pools: DieselPools) -> Self {
        Self { diesel_pools }
    }
}

#[async_trait::async_trait]
impl super::EmployeeContractRepository for DieselEmployeeContractRepository {
    async fn create_contract(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        req: &CreateContractRequest,
    ) -> Result<EmployeeContract, AppError> {
        use crate::models::enums::ContractStatus;
        use crate::schema::employee_contracts::dsl::*;
        use diesel_async::AsyncConnection;

        let mut conn = self
            .diesel_pools
            .tenants
            .get_pool(&auth_token.workspace_id())
            .await
            .map_err(|e| AppError::Internal(format!("db pool error: {e}")))?
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to get connection: {e}")))?;

        let created_entity = conn
            .transaction::<_, diesel::result::Error, _>(|conn| {
                Box::pin(async move {
                    // 1. Check for overlapping active contracts
                    let overlaps: Vec<(Uuid, Option<String>)> = employee_contracts
                        .filter(workspace_id.eq(auth_token.workspace_id()))
                        .filter(employee_id.eq(req.employee_id))
                        .filter(status.eq(ContractStatus::Active))
                        .filter(start_date.le(req.start_date))
                        .filter(end_date.is_null().or(end_date.ge(req.start_date)))
                        .select((id, notes))
                        .load(conn)
                        .await?;

                    if !overlaps.is_empty() {
                        if req.auto_close_overlapping.unwrap_or(false) {
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

                            for (overlap_id, current_notes) in overlaps {
                                let new_notes = match current_notes {
                                    Some(ref n) if !n.is_empty() => {
                                        format!("{n}\n---\n{auto_close_note}")
                                    }
                                    _ => auto_close_note.clone(),
                                };

                                diesel::update(employee_contracts.filter(id.eq(overlap_id)))
                                    .set((
                                        end_date.eq(new_end_date),
                                        status.eq(ContractStatus::Expired),
                                        notes.eq(new_notes),
                                        updated_at.eq(chrono::Utc::now()),
                                    ))
                                    .execute(conn)
                                    .await?;
                            }
                        } else {
                            // If we shouldn't auto close, rollback transaction and return a specific custom error later
                            // by returning a diesel error that we will map to AppError
                            return Err(diesel::result::Error::RollbackTransaction);
                        }
                    }

                    // 2. Insert the new contract
                    let new_contract = EmployeeContractEntity {
                        id: Uuid::new_v4(),
                        workspace_id: auth_token.workspace_id(),
                        employee_id: req.employee_id,
                        contract_number: req.contract_number.clone(),
                        contract_type: req.contract_type,
                        status: ContractStatus::Active, // Default status or maybe from req? Wait, req.status isn't in CreateContractRequest
                        start_date: req.start_date,
                        end_date: req.end_date,
                        document_id: req.document_id,
                        notes: req.notes.clone(),
                        created_at: Utc::now(),
                        updated_at: Utc::now(),
                    };

                    let inserted = diesel::insert_into(employee_contracts)
                        .values(&new_contract)
                        .get_result::<EmployeeContractEntity>(conn)
                        .await?;

                    Ok(inserted)
                })
            })
            .await;

        match created_entity {
            Ok(entity) => Ok(entity.into()),
            Err(diesel::result::Error::RollbackTransaction) => Err(AppError::Conflict(
                "overlapping_contract_exists".to_string(),
            )),
            Err(e) => Err(AppError::Internal(format!("Transaction failed: {e}"))),
        }
    }

    async fn get_contract_by_id(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        contract_id: Uuid,
    ) -> Result<Option<EmployeeContract>, AppError> {
        use crate::schema::employee_contracts::dsl::*;

        let mut conn = self
            .diesel_pools
            .tenants
            .get_pool(&auth_token.workspace_id())
            .await
            .map_err(|e| AppError::Internal(format!("db pool error: {e}")))?
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to get connection: {e}")))?;
        let contract = employee_contracts
            .filter(id.eq(contract_id))
            .filter(workspace_id.eq(auth_token.workspace_id()))
            .first::<EmployeeContractEntity>(&mut conn)
            .await
            .optional()?;

        Ok(contract.map(Into::into))
    }

    async fn list_employee_contracts(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id_val: Uuid,
    ) -> Result<Vec<EmployeeContract>, AppError> {
        use crate::schema::employee_contracts::dsl::*;

        let mut conn = self
            .diesel_pools
            .tenants
            .get_pool(&auth_token.workspace_id())
            .await
            .map_err(|e| AppError::Internal(format!("db pool error: {e}")))?
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to get connection: {e}")))?;
        let contracts = employee_contracts
            .filter(employee_id.eq(employee_id_val))
            .filter(workspace_id.eq(auth_token.workspace_id()))
            .order(start_date.desc())
            .load::<EmployeeContractEntity>(&mut conn)
            .await?;

        Ok(contracts.into_iter().map(Into::into).collect())
    }

    async fn update_contract(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        contract_id: Uuid,
        req: &UpdateContractRequest,
    ) -> Result<EmployeeContract, AppError> {
        use crate::schema::employee_contracts::dsl::*;

        let mut conn = self
            .diesel_pools
            .tenants
            .get_pool(&auth_token.workspace_id())
            .await
            .map_err(|e| AppError::Internal(format!("db pool error: {e}")))?
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
                .filter(workspace_id.eq(auth_token.workspace_id())),
        )
        .set(&update_model)
        .get_result::<EmployeeContractEntity>(&mut conn)
        .await?;

        Ok(updated_contract.into())
    }

    async fn delete_contract(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        contract_id: Uuid,
    ) -> Result<(), AppError> {
        use crate::schema::employee_contracts::dsl::*;

        let mut conn = self
            .diesel_pools
            .tenants
            .get_pool(&auth_token.workspace_id())
            .await
            .map_err(|e| AppError::Internal(format!("db pool error: {e}")))?
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to get connection: {e}")))?;
        diesel::delete(
            employee_contracts
                .filter(id.eq(contract_id))
                .filter(workspace_id.eq(auth_token.workspace_id())),
        )
        .execute(&mut conn)
        .await?;

        Ok(())
    }

    async fn get_expiring_contracts(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        days_ahead: i32,
    ) -> Result<Vec<EmployeeContract>, AppError> {
        use crate::models::enums::ContractStatus;
        use crate::schema::employee_contracts::dsl::*;

        let mut conn = self
            .diesel_pools
            .tenants
            .get_pool(&auth_token.workspace_id())
            .await
            .map_err(|e| AppError::Internal(format!("db pool error: {e}")))?
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to get connection: {e}")))?;

        let target_date =
            chrono::Utc::now().naive_utc().date() + chrono::Duration::days(days_ahead as i64);

        let contracts = employee_contracts
            .filter(workspace_id.eq(auth_token.workspace_id()))
            .filter(status.eq(ContractStatus::Active))
            .filter(end_date.is_not_null())
            .filter(end_date.le(target_date))
            .load::<EmployeeContractEntity>(&mut conn)
            .await?;

        Ok(contracts.into_iter().map(Into::into).collect())
    }

    async fn get_active_contract_on_date(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id_val: Uuid,
        date_val: chrono::NaiveDate,
    ) -> Result<Option<EmployeeContract>, AppError> {
        use crate::models::enums::ContractStatus;
        use crate::schema::employee_contracts::dsl::*;

        let mut conn = self
            .diesel_pools
            .tenants
            .get_pool(&auth_token.workspace_id())
            .await
            .map_err(|e| AppError::Internal(format!("db pool error: {e}")))?
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to get connection: {e}")))?;

        let entity = employee_contracts
            .filter(workspace_id.eq(auth_token.workspace_id()))
            .filter(employee_id.eq(employee_id_val))
            .filter(status.eq(ContractStatus::Active))
            .filter(start_date.le(date_val))
            .filter(end_date.is_null().or(end_date.ge(date_val)))
            .order(start_date.desc())
            .first::<EmployeeContractEntity>(&mut conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(format!("db error: {e}")))?;

        Ok(entity.map(Into::into))
    }
}
