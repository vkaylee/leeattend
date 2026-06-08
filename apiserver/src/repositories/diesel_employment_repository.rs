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
}
