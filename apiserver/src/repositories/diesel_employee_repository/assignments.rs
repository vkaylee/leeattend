use crate::diesel_pool::DieselPoolsTenantTxExt;
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
        employment_id: Uuid,
    ) -> Result<Vec<EmployeeAssignment>, AppError>;

    async fn get_employee_assignment(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_id: Uuid,
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
        employment_id: Uuid,
        assignment_id: Uuid,
        req: &UpdateEmployeeAssignmentRequest,
    ) -> Result<EmployeeAssignment, AppError>;

    async fn delete_employee_assignment(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_id: Uuid,
        assignment_id: Uuid,
    ) -> Result<u64, AppError>;

    async fn list_assignments_batch(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_ids: &[Uuid],
    ) -> Result<std::collections::HashMap<Uuid, Vec<EmployeeAssignment>>, AppError> {
        let mut map = std::collections::HashMap::new();
        for &emp_id in employment_ids {
            let list = self.list_employee_assignments(auth_token, emp_id).await?;
            map.insert(emp_id, list);
        }
        Ok(map)
    }
}

#[async_trait]
impl EmployeeAssignmentRepository for DieselEmployeeRepository {
    async fn list_employee_assignments(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_id: Uuid,
    ) -> Result<Vec<EmployeeAssignment>, AppError> {
        let workspace_id = auth_token.workspace_id();
        self.diesel_pools
            .with_tenant_tx(auth_token, |conn| {
                Box::pin(async move {
                    let assignments = employee_assignments::table
                        .filter(employee_assignments::workspace_id.eq(workspace_id))
                        .filter(employee_assignments::employment_id.eq(employment_id))
                        .order(employee_assignments::start_date.desc())
                        .load::<EmployeeAssignment>(conn)
                        .await?;
                    Ok(assignments)
                })
            })
            .await
    }

    async fn get_employee_assignment(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_id: Uuid,
        assignment_id: Uuid,
    ) -> Result<Option<EmployeeAssignment>, AppError> {
        let workspace_id = auth_token.workspace_id();
        self.diesel_pools
            .with_tenant_tx(auth_token, |conn| {
                Box::pin(async move {
                    let assignment = employee_assignments::table
                        .filter(employee_assignments::id.eq(assignment_id))
                        .filter(employee_assignments::workspace_id.eq(workspace_id))
                        .filter(employee_assignments::employment_id.eq(employment_id))
                        .first::<EmployeeAssignment>(conn)
                        .await
                        .optional()?;
                    Ok(assignment)
                })
            })
            .await
    }

    async fn create_employee_assignment(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        assignment: &NewEmployeeAssignment,
    ) -> Result<EmployeeAssignment, AppError> {
        let workspace_id = auth_token.workspace_id();
        if assignment.workspace_id != workspace_id {
            return Err(AppError::forbidden(
                crate::utils::i18n::I18nKey::TenantMismatch,
            ));
        }

        self.diesel_pools
            .with_tenant_tx(auth_token, |conn| {
                Box::pin(async move {
                    let created = diesel::insert_into(employee_assignments::table)
                        .values(assignment)
                        .get_result(conn)
                        .await?;
                    Ok(created)
                })
            })
            .await
    }

    async fn update_employee_assignment(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_id: Uuid,
        assignment_id: Uuid,
        req: &UpdateEmployeeAssignmentRequest,
    ) -> Result<EmployeeAssignment, AppError> {
        let workspace_id = auth_token.workspace_id();
        self.diesel_pools
            .with_tenant_tx(auth_token, |conn| {
                Box::pin(async move {
                    let updated = diesel::update(
                        employee_assignments::table
                            .filter(employee_assignments::id.eq(assignment_id))
                            .filter(employee_assignments::workspace_id.eq(workspace_id))
                            .filter(employee_assignments::employment_id.eq(employment_id)),
                    )
                    .set(req)
                    .get_result(conn)
                    .await?;
                    Ok(updated)
                })
            })
            .await
    }

    async fn delete_employee_assignment(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_id: Uuid,
        assignment_id: Uuid,
    ) -> Result<u64, AppError> {
        let workspace_id = auth_token.workspace_id();
        self.diesel_pools
            .with_tenant_tx(auth_token, |conn| {
                Box::pin(async move {
                    let deleted = diesel::delete(
                        employee_assignments::table
                            .filter(employee_assignments::id.eq(assignment_id))
                            .filter(employee_assignments::workspace_id.eq(workspace_id))
                            .filter(employee_assignments::employment_id.eq(employment_id)),
                    )
                    .execute(conn)
                    .await?;
                    Ok(deleted as u64)
                })
            })
            .await
    }

    async fn list_assignments_batch(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_ids: &[Uuid],
    ) -> Result<std::collections::HashMap<Uuid, Vec<EmployeeAssignment>>, AppError> {
        let workspace_id = auth_token.workspace_id();
        self.diesel_pools
            .with_tenant_tx(auth_token, |conn| {
                Box::pin(async move {
                    let assignments = employee_assignments::table
                        .filter(employee_assignments::workspace_id.eq(workspace_id))
                        .filter(employee_assignments::employment_id.eq_any(employment_ids))
                        .order((
                            employee_assignments::employment_id.asc(),
                            employee_assignments::start_date.desc(),
                        ))
                        .load::<EmployeeAssignment>(conn)
                        .await?;

                    let mut map: std::collections::HashMap<Uuid, Vec<EmployeeAssignment>> =
                        std::collections::HashMap::new();
                    for assignment in assignments {
                        map.entry(assignment.employment_id)
                            .or_default()
                            .push(assignment);
                    }
                    Ok(map)
                })
            })
            .await
    }
}
