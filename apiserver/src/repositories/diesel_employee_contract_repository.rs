use chrono::Utc;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::{
    diesel_pool::DieselPools,
    errors::AppError,
    models::contract::{CreateContractRequest, EmployeeContract, UpdateContractRequest, EmployeeContractWithEmployee},
    models::diesel::employee_contract::{EmployeeContractEntity, UpdateDieselEmployeeContract},
};

pub struct DieselEmployeeContractRepository {
    diesel_pools: DieselPools,
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

        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&auth_token.workspace_id())
                .await?,
        );

        let created_entity = conn
            .transaction::<_, diesel::result::Error, _>(|conn| {
                Box::pin(async move {
                    // 1. Check for overlapping active contracts
                    let overlaps: Vec<(Uuid, Option<String>)> = employee_contracts
                        .filter(workspace_id.eq(auth_token.workspace_id()))
                        .filter(employment_id.eq(req.employment_id))
                        .filter(status.eq(ContractStatus::Active))
                        .filter(start_date.le(req.start_date))
                        .filter(end_date.is_null().or(end_date.ge(req.start_date)))
                        .filter(deleted_at.is_null())
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

                            let overlap_ids: Vec<Uuid> =
                                overlaps.into_iter().map(|(o_id, _)| o_id).collect();

                            diesel::sql_query(
                                r"
                                UPDATE employee_contracts
                                SET end_date = $1,
                                    status = 'expired',
                                    updated_at = NOW(),
                                    notes = CASE
                                        WHEN notes IS NULL OR notes = '' THEN $2
                                        ELSE notes || E'\n---\n' || $2
                                    END
                                WHERE id = ANY($3)
                                ",
                            )
                            .bind::<diesel::sql_types::Date, _>(new_end_date)
                            .bind::<diesel::sql_types::Text, _>(auto_close_note)
                            .bind::<diesel::sql_types::Array<diesel::sql_types::Uuid>, _>(
                                overlap_ids,
                            )
                            .execute(conn)
                            .await?;
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
                        employment_id: req.employment_id,
                        contract_number: req.contract_number.clone(),
                        contract_type: req.contract_type,
                        status: ContractStatus::Active, // Default status or maybe from req? Wait, req.status isn't in CreateContractRequest
                        start_date: req.start_date,
                        end_date: req.end_date,
                        document_id: req.document_id,
                        notes: req.notes.clone(),
                        created_at: Utc::now(),
                        updated_at: Utc::now(),
                        deleted_at: None,
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
            Err(diesel::result::Error::RollbackTransaction) => Err(AppError::conflict_args(
                crate::utils::i18n::I18nKey::Conflict,
                vec![("details", "overlapping_contract_exists".to_string())],
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

        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&auth_token.workspace_id())
                .await?,
        );
        let contract = employee_contracts
            .filter(id.eq(contract_id))
            .filter(workspace_id.eq(auth_token.workspace_id()))
            .filter(deleted_at.is_null())
            .first::<EmployeeContractEntity>(&mut *conn)
            .await
            .optional()?;

        Ok(contract.map(Into::into))
    }

    async fn list_employee_contracts(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_id_val: Uuid,
    ) -> Result<Vec<EmployeeContract>, AppError> {
        use crate::schema::employee_contracts::dsl::*;

        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&auth_token.workspace_id())
                .await?,
        );
        let contracts = employee_contracts
            .filter(employment_id.eq(employment_id_val))
            .filter(workspace_id.eq(auth_token.workspace_id()))
            .filter(deleted_at.is_null())
            .order(start_date.desc())
            .load::<EmployeeContractEntity>(&mut *conn)
            .await?;

        Ok(contracts.into_iter().map(Into::into).collect())
    }

    async fn list_workspace_contracts(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Vec<EmployeeContractWithEmployee>, AppError> {
        use crate::schema::employee_contracts::dsl::*;
        use crate::schema::employments::dsl::{employments, id as emp_emp_id, employee_id as emp_employee_id, deleted_at as emp_deleted_at};
        use crate::schema::employees::dsl::{employees, id as emp_id, full_name, employee_id_number, deleted_at as employee_deleted_at};

        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&auth_token.workspace_id())
                .await?,
        );

        let results = employee_contracts
            .inner_join(employments.on(employment_id.eq(emp_emp_id)))
            .inner_join(employees.on(emp_employee_id.eq(emp_id)))
            .filter(workspace_id.eq(auth_token.workspace_id()))
            .filter(deleted_at.is_null())
            .filter(emp_deleted_at.is_null())
            .filter(employee_deleted_at.is_null())
            .order(start_date.desc())
            .select((
                EmployeeContractEntity::as_select(),
                full_name,
                employee_id_number,
                emp_id,
            ))
            .load::<(EmployeeContractEntity, String, String, Uuid)>(&mut *conn)
            .await?;

        Ok(results
            .into_iter()
            .map(|(entity, name, code, emp_uuid)| {
                EmployeeContractWithEmployee {
                    contract: entity.into(),
                    employee_name: name,
                    employee_code: code,
                    employee_id: emp_uuid,
                }
            })
            .collect())
    }

    async fn list_contracts_batch(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_ids: &[Uuid],
    ) -> Result<std::collections::HashMap<Uuid, Vec<EmployeeContract>>, AppError> {
        use crate::schema::employee_contracts::dsl::*;

        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&auth_token.workspace_id())
                .await?,
        );
        let contracts = employee_contracts
            .filter(employment_id.eq_any(employment_ids))
            .filter(workspace_id.eq(auth_token.workspace_id()))
            .filter(deleted_at.is_null())
            .order((employment_id.asc(), start_date.desc()))
            .load::<EmployeeContractEntity>(&mut *conn)
            .await?;

        let mut map: std::collections::HashMap<Uuid, Vec<EmployeeContract>> =
            std::collections::HashMap::new();
        for contract in contracts {
            map.entry(contract.employment_id)
                .or_default()
                .push(contract.into());
        }

        Ok(map)
    }

    async fn update_contract(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        contract_id: Uuid,
        req: &UpdateContractRequest,
    ) -> Result<EmployeeContract, AppError> {
        use crate::schema::employee_contracts::dsl::*;

        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&auth_token.workspace_id())
                .await?,
        );

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
                .filter(workspace_id.eq(auth_token.workspace_id()))
                .filter(deleted_at.is_null()),
        )
        .set(&update_model)
        .get_result::<EmployeeContractEntity>(&mut *conn)
        .await?;

        Ok(updated_contract.into())
    }

    async fn delete_contract(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        contract_id: Uuid,
    ) -> Result<(), AppError> {
        use crate::schema::employee_contracts::dsl::*;

        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&auth_token.workspace_id())
                .await?,
        );
        diesel::update(
            employee_contracts
                .filter(id.eq(contract_id))
                .filter(workspace_id.eq(auth_token.workspace_id())),
        )
        .set(deleted_at.eq(Some(Utc::now())))
        .execute(&mut *conn)
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

        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&auth_token.workspace_id())
                .await?,
        );

        let target_date =
            chrono::Utc::now().naive_utc().date() + chrono::Duration::days(i64::from(days_ahead));

        let contracts = employee_contracts
            .filter(workspace_id.eq(auth_token.workspace_id()))
            .filter(status.eq(ContractStatus::Active))
            .filter(end_date.is_not_null())
            .filter(end_date.le(target_date))
            .filter(deleted_at.is_null())
            .load::<EmployeeContractEntity>(&mut *conn)
            .await?;

        Ok(contracts.into_iter().map(Into::into).collect())
    }

    async fn get_active_contract_on_date(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_id_val: Uuid,
        date_val: chrono::NaiveDate,
    ) -> Result<Option<EmployeeContract>, AppError> {
        use crate::models::enums::ContractStatus;
        use crate::schema::employee_contracts::dsl::*;

        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&auth_token.workspace_id())
                .await?,
        );

        let entity = employee_contracts
            .filter(workspace_id.eq(auth_token.workspace_id()))
            .filter(employment_id.eq(employment_id_val))
            .filter(status.eq(ContractStatus::Active))
            .filter(start_date.le(date_val))
            .filter(end_date.is_null().or(end_date.ge(date_val)))
            .filter(deleted_at.is_null())
            .order(start_date.desc())
            .first::<EmployeeContractEntity>(&mut *conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(format!("db error: {e}")))?;

        Ok(entity.map(Into::into))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diesel_pool::DieselPools;

    #[tokio::test]
    async fn test_repo_initialization() {
        let test_db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://user:pass@localhost:5432/leeattend_test_db".to_string()
        });

        let pools = DieselPools::for_test(&test_db_url);
        let _repo = DieselEmployeeContractRepository::new(pools);
        // DI connection test passes if it compiles
    }
}
