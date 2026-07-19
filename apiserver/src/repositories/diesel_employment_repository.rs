use chrono::Utc;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::{
    diesel_pool::DieselPools,
    errors::AppError,
    models::diesel::employment::{EmploymentEntity, UpdateDieselEmployment},
    models::employment::{CreateEmploymentRequest, Employment, UpdateEmploymentRequest},
};

pub struct DieselEmploymentRepository {
    diesel_pools: DieselPools,
}

impl DieselEmploymentRepository {
    pub fn new(diesel_pools: DieselPools) -> Self {
        Self { diesel_pools }
    }
}

#[async_trait::async_trait]
impl super::EmploymentRepository for DieselEmploymentRepository {
    async fn create_employment(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        req: &CreateEmploymentRequest,
    ) -> Result<Employment, AppError> {
        use crate::schema::employments::dsl::*;

        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&auth_token.workspace_id())
                .await?,
        );

        let new_emp = EmploymentEntity {
            id: Uuid::new_v4(),
            workspace_id: auth_token.workspace_id(),
            employee_id: req.employee_id,
            employer_id: req.employer_id,
            primary_location_id: req.primary_location_id,
            department_id: req.department_id,
            status: req.status.clone(),
            start_date: req.start_date,
            end_date: req.end_date,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
            job_title: req.job_title.clone(),
            employment_type: req.employment_type.clone(),
            manager_employment_id: req.manager_employment_id,
            notes: None,
            termination_reason: None,
        };

        let inserted = diesel::insert_into(employments)
            .values(&new_emp)
            .get_result::<EmploymentEntity>(&mut *conn)
            .await?;

        Ok(inserted.into())
    }

    async fn get_employment_by_id(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_id_val: Uuid,
    ) -> Result<Option<Employment>, AppError> {
        use crate::schema::employments::dsl::*;

        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&auth_token.workspace_id())
                .await?,
        );

        let entity = employments
            .filter(id.eq(employment_id_val))
            .filter(workspace_id.eq(auth_token.workspace_id()))
            .filter(deleted_at.is_null())
            .first::<EmploymentEntity>(&mut *conn)
            .await
            .optional()?;

        Ok(entity.map(Into::into))
    }

    async fn list_employee_employments(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id_val: Uuid,
    ) -> Result<Vec<Employment>, AppError> {
        use crate::schema::employments::dsl::*;

        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&auth_token.workspace_id())
                .await?,
        );

        let list = employments
            .filter(employee_id.eq(employee_id_val))
            .filter(workspace_id.eq(auth_token.workspace_id()))
            .filter(deleted_at.is_null())
            .order(start_date.desc())
            .load::<EmploymentEntity>(&mut *conn)
            .await?;

        Ok(list.into_iter().map(Into::into).collect())
    }

    async fn update_employment(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_id_val: Uuid,
        req: &UpdateEmploymentRequest,
    ) -> Result<Employment, AppError> {
        use crate::schema::employments::dsl::*;

        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&auth_token.workspace_id())
                .await?,
        );

        let update_model = UpdateDieselEmployment {
            employer_id: req.employer_id,
            primary_location_id: req.primary_location_id,
            department_id: req.department_id,
            status: req.status.clone(),
            start_date: req.start_date,
            end_date: req.end_date,
            job_title: req.job_title.clone(),
            employment_type: req.employment_type.clone(),
            manager_employment_id: req.manager_employment_id,
            notes: req.notes.as_ref().map(|n| Some(n.clone())),
            termination_reason: req.termination_reason.as_ref().map(|r| Some(r.clone())),
            updated_at: Some(Utc::now()),
        };

        let updated = diesel::update(
            employments
                .filter(id.eq(employment_id_val))
                .filter(workspace_id.eq(auth_token.workspace_id()))
                .filter(deleted_at.is_null()),
        )
        .set(&update_model)
        .get_result::<EmploymentEntity>(&mut *conn)
        .await?;

        Ok(updated.into())
    }

    async fn terminate_employment_transactional(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_id_val: Uuid,
        termination_date: chrono::NaiveDate,
        termination_reason: String,
        notes: Option<String>,
        balances: Vec<crate::models::LeaveBalance>,
    ) -> Result<Employment, AppError> {
        use crate::schema::employments::dsl::{deleted_at, employments, id, workspace_id};
        use crate::schema::leave_balances;
        use bigdecimal::BigDecimal;
        use diesel_async::AsyncConnection;
        use std::str::FromStr;

        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

        conn.transaction(|conn| {
            Box::pin(async move {
                let update_model = UpdateDieselEmployment {
                    employer_id: None,
                    primary_location_id: None,
                    department_id: None,
                    status: Some("terminated".to_string()),
                    start_date: None,
                    end_date: Some(Some(termination_date)),
                    job_title: None,
                    employment_type: None,
                    manager_employment_id: None,
                    notes: Some(notes),
                    termination_reason: Some(Some(termination_reason)),
                    updated_at: Some(Utc::now()),
                };

                let updated = diesel::update(
                    employments
                        .filter(id.eq(employment_id_val))
                        .filter(workspace_id.eq(auth_token.workspace_id()))
                        .filter(deleted_at.is_null()),
                )
                .set(&update_model)
                .get_result::<EmploymentEntity>(&mut *conn)
                .await
                .map_err(|e| AppError::Internal(format!("Failed to update employment: {e}")))?;

                for bal in &balances {
                    let accrued_bd = BigDecimal::from_str(&bal.accrued_amount().to_string())
                        .map_err(|e| AppError::Internal(format!("BigDecimal parse failed: {e}")))?;
                    let bal_entity = crate::models::diesel::leave::LeaveBalanceEntity {
                        id: bal.id(),
                        employment_id: bal.employment_id(),
                        leave_type_id: bal.leave_type_id(),
                        year: bal.year(),
                        entitlement: bal.entitlement(),
                        accrued_amount: accrued_bd,
                        carried_over: bal.carried_over(),
                        created_at: Some(bal.created_at()),
                        updated_at: Some(bal.updated_at()),
                    };

                    diesel::insert_into(leave_balances::table)
                        .values(&bal_entity)
                        .on_conflict((
                            leave_balances::employment_id,
                            leave_balances::leave_type_id,
                            leave_balances::year,
                        ))
                        .do_update()
                        .set((
                            leave_balances::entitlement.eq(bal_entity.entitlement),
                            leave_balances::accrued_amount.eq(bal_entity.accrued_amount.clone()),
                            leave_balances::carried_over.eq(bal_entity.carried_over),
                            leave_balances::updated_at.eq(diesel::dsl::now),
                        ))
                        .execute(&mut *conn)
                        .await
                        .map_err(|e| {
                            AppError::Internal(format!("Failed to upsert leave balance: {e}"))
                        })?;
                }

                Ok::<EmploymentEntity, AppError>(updated)
            })
        })
        .await
        .map(Into::into)
    }

    async fn delete_employment(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_id_val: Uuid,
    ) -> Result<(), AppError> {
        use crate::schema::employments::dsl::*;

        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&auth_token.workspace_id())
                .await?,
        );

        diesel::update(
            employments
                .filter(id.eq(employment_id_val))
                .filter(workspace_id.eq(auth_token.workspace_id())),
        )
        .set(deleted_at.eq(Some(Utc::now())))
        .execute(&mut *conn)
        .await?;

        Ok(())
    }

    async fn get_active_employment_on_date(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id_val: Uuid,
        date_val: chrono::NaiveDate,
    ) -> Result<Option<Employment>, AppError> {
        use crate::schema::employments::dsl::*;

        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&auth_token.workspace_id())
                .await?,
        );

        let entity = employments
            .filter(workspace_id.eq(auth_token.workspace_id()))
            .filter(employee_id.eq(employee_id_val))
            .filter(status.eq("active"))
            .filter(start_date.le(date_val))
            .filter(end_date.is_null().or(end_date.ge(date_val)))
            .filter(deleted_at.is_null())
            .order(start_date.desc())
            .first::<EmploymentEntity>(&mut *conn)
            .await
            .optional()?;

        Ok(entity.map(Into::into))
    }

    async fn get_active_employments_for_workspace(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Vec<Employment>, AppError> {
        use crate::schema::employments::dsl::*;

        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&auth_token.workspace_id())
                .await?,
        );

        let list = employments
            .filter(workspace_id.eq(auth_token.workspace_id()))
            .filter(status.eq("active"))
            .filter(deleted_at.is_null())
            .load::<EmploymentEntity>(&mut *conn)
            .await?;

        Ok(list.into_iter().map(Into::into).collect())
    }

    async fn get_active_employments_for_employees(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_ids_val: &[Uuid],
        date_val: chrono::NaiveDate,
    ) -> Result<Vec<Employment>, AppError> {
        use crate::schema::employments::dsl::*;

        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&auth_token.workspace_id())
                .await?,
        );

        let list = employments
            .filter(workspace_id.eq(auth_token.workspace_id()))
            .filter(employee_id.eq_any(employee_ids_val))
            .filter(status.eq("active"))
            .filter(start_date.le(date_val))
            .filter(end_date.is_null().or(end_date.ge(date_val)))
            .filter(deleted_at.is_null())
            .load::<EmploymentEntity>(&mut *conn)
            .await?;

        Ok(list.into_iter().map(Into::into).collect())
    }

    async fn get_employments_for_employees(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_ids_val: &[Uuid],
    ) -> Result<Vec<Employment>, AppError> {
        use crate::schema::employments::dsl::*;

        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&auth_token.workspace_id())
                .await?,
        );

        let list = employments
            .filter(workspace_id.eq(auth_token.workspace_id()))
            .filter(employee_id.eq_any(employee_ids_val))
            .filter(deleted_at.is_null())
            .load::<EmploymentEntity>(&mut *conn)
            .await?;

        Ok(list.into_iter().map(Into::into).collect())
    }

    async fn get_employments_by_ids(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_ids_val: &[Uuid],
    ) -> Result<Vec<Employment>, AppError> {
        use crate::schema::employments::dsl::*;

        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&auth_token.workspace_id())
                .await?,
        );

        let list = employments
            .filter(workspace_id.eq(auth_token.workspace_id()))
            .filter(id.eq_any(employment_ids_val))
            .filter(deleted_at.is_null())
            .load::<EmploymentEntity>(&mut *conn)
            .await?;

        Ok(list.into_iter().map(Into::into).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bigdecimal::BigDecimal;
    use std::str::FromStr;

    // --- BigDecimal safety regression tests (Debt #2) ---
    // Verify that from_str for invalid values never silently defaults to zero

    #[test]
    fn test_bigdecimal_from_str_invalid_returns_error() {
        let result = BigDecimal::from_str("not_a_number");
        assert!(
            result.is_err(),
            "BigDecimal::from_str('not_a_number') must return Err, not silently default to zero"
        );
    }

    #[test]
    fn test_bigdecimal_from_str_empty_returns_error() {
        let result = BigDecimal::from_str("");
        assert!(result.is_err(), "BigDecimal::from_str('') must return Err");
    }

    #[test]
    fn test_bigdecimal_error_propagation_pattern() {
        // Verify the exact pattern used in terminate_employment_transactional:
        // BigDecimal::from_str(...).map_err(|e| AppError::Internal(format!(...)))?;
        let result: Result<BigDecimal, AppError> = BigDecimal::from_str("garbage")
            .map_err(|e| AppError::Internal(format!("BigDecimal parse failed: {e}")));
        assert!(result.is_err());
        match result {
            Err(AppError::Internal(msg)) => {
                assert!(
                    msg.contains("BigDecimal parse failed"),
                    "Error message should describe the problem, got: {msg}"
                );
            }
            _ => panic!("Expected AppError::Internal"),
        }
    }

    #[test]
    fn test_bigdecimal_from_str_valid_returns_ok() {
        let result = BigDecimal::from_str("42.5");
        assert!(result.is_ok(), "BigDecimal::from_str('42.5') should return Ok");
    }
}
