use crate::errors::AppError;
use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::models::employee_assignment::{
    EmployeeAssignment, NewEmployeeAssignment, UpdateEmployeeAssignmentRequest,
};
use crate::repositories::diesel_employee_repository::DieselEmployeeRepository;
use crate::schema::employee_assignments;

#[async_trait]
pub trait EmployeeAssignmentRepository: Send + Sync {
    async fn list_employee_assignments(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
    ) -> Result<Vec<EmployeeAssignment>, AppError>;

    async fn get_employee_assignment(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
        assignment_id: Uuid,
    ) -> Result<Option<EmployeeAssignment>, AppError>;

    async fn create_employee_assignment(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        assignment: &NewEmployeeAssignment,
    ) -> Result<EmployeeAssignment, AppError>;

    async fn update_employee_assignment(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
        assignment_id: Uuid,
        req: &UpdateEmployeeAssignmentRequest,
    ) -> Result<EmployeeAssignment, AppError>;

    async fn delete_employee_assignment(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
        assignment_id: Uuid,
    ) -> Result<u64, AppError>;
}

#[async_trait]
impl EmployeeAssignmentRepository for DieselEmployeeRepository {
    async fn list_employee_assignments(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
    ) -> Result<Vec<EmployeeAssignment>, AppError> {
        let mut conn = self
            .diesel_pools
            .tenants_tx
            .get_connection(&auth_token.workspace_id())
            .await?;
        let assignments = employee_assignments::table
            .filter(employee_assignments::workspace_id.eq(auth_token.workspace_id()))
            .filter(employee_assignments::employee_id.eq(employee_id))
            .order(employee_assignments::start_date.desc())
            .load::<EmployeeAssignment>(&mut conn)
            .await?;
        Ok(assignments)
    }

    async fn get_employee_assignment(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
        assignment_id: Uuid,
    ) -> Result<Option<EmployeeAssignment>, AppError> {
        let mut conn = self
            .diesel_pools
            .tenants_tx
            .get_connection(&auth_token.workspace_id())
            .await?;
        let assignment = employee_assignments::table
            .filter(employee_assignments::id.eq(assignment_id))
            .filter(employee_assignments::workspace_id.eq(auth_token.workspace_id()))
            .filter(employee_assignments::employee_id.eq(employee_id))
            .first::<EmployeeAssignment>(&mut conn)
            .await
            .optional()?;
        Ok(assignment)
    }

    async fn create_employee_assignment(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        assignment: &NewEmployeeAssignment,
    ) -> Result<EmployeeAssignment, AppError> {
        let mut conn = self
            .diesel_pools
            .tenants_tx
            .get_connection(&auth_token.workspace_id())
            .await?;
        let created = diesel::insert_into(employee_assignments::table)
            .values(assignment)
            .get_result(&mut conn)
            .await?;
        Ok(created)
    }

    async fn update_employee_assignment(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
        assignment_id: Uuid,
        req: &UpdateEmployeeAssignmentRequest,
    ) -> Result<EmployeeAssignment, AppError> {
        let mut conn = self
            .diesel_pools
            .tenants_tx
            .get_connection(&auth_token.workspace_id())
            .await?;
        let updated = diesel::update(
            employee_assignments::table
                .filter(employee_assignments::id.eq(assignment_id))
                .filter(employee_assignments::workspace_id.eq(auth_token.workspace_id()))
                .filter(employee_assignments::employee_id.eq(employee_id)),
        )
        .set(req)
        .get_result(&mut conn)
        .await?;
        Ok(updated)
    }

    async fn delete_employee_assignment(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
        assignment_id: Uuid,
    ) -> Result<u64, AppError> {
        let mut conn = self
            .diesel_pools
            .tenants_tx
            .get_connection(&auth_token.workspace_id())
            .await?;
        let deleted = diesel::delete(
            employee_assignments::table
                .filter(employee_assignments::id.eq(assignment_id))
                .filter(employee_assignments::workspace_id.eq(auth_token.workspace_id()))
                .filter(employee_assignments::employee_id.eq(employee_id)),
        )
        .execute(&mut conn)
        .await?;
        Ok(deleted as u64)
    }
}
