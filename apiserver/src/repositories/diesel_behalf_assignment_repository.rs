use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::diesel_pool::DieselPools;
use crate::errors::{AppError, AppResult};
use crate::models::diesel::behalf_assignment::BehalfAssignmentEntity;
use crate::models::BehalfAssignment;
use crate::repositories::behalf_assignment_repository::BehalfAssignmentRepository;
use crate::schema::behalf_assignments;

pub struct DieselBehalfAssignmentRepository {
    pub diesel_pools: DieselPools,
}

impl DieselBehalfAssignmentRepository {
    pub fn new(diesel_pools: DieselPools) -> Self {
        Self { diesel_pools }
    }
}

#[async_trait]
impl BehalfAssignmentRepository for DieselBehalfAssignmentRepository {
    async fn create_assignment(
        &self,
        workspace_id: Uuid,
        assignee_user_id: Uuid,
        target_employee_id: Option<Uuid>,
        target_department_id: Option<Uuid>,
        assignment_type: &str,
    ) -> AppResult<BehalfAssignment> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        if target_employee_id.is_none() && target_department_id.is_none() {
            return Err(AppError::Validation(
                "error.behalf_assignment_missing_target".to_string(),
            ));
        }
        if target_employee_id.is_some() && target_department_id.is_some() {
            return Err(AppError::Validation(
                "error.behalf_assignment_dual_target".to_string(),
            ));
        }

        diesel::insert_into(behalf_assignments::table)
            .values((
                behalf_assignments::workspace_id.eq(workspace_id),
                behalf_assignments::assignee_user_id.eq(assignee_user_id),
                behalf_assignments::target_employee_id.eq(target_employee_id),
                behalf_assignments::target_department_id.eq(target_department_id),
                behalf_assignments::assignment_type.eq(assignment_type),
            ))
            .returning(BehalfAssignmentEntity::as_returning())
            .get_result(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))
            .map(Into::into)
    }

    async fn delete_assignment(&self, workspace_id: Uuid, id: Uuid) -> AppResult<u64> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let affected = diesel::delete(behalf_assignments::table)
            .filter(behalf_assignments::id.eq(id))
            .filter(behalf_assignments::workspace_id.eq(workspace_id))
            .execute(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(affected as u64)
    }

    async fn list_all_assignments(
        &self,
        workspace_id: Uuid,
        assignment_type: &str,
    ) -> AppResult<Vec<BehalfAssignment>> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        behalf_assignments::table
            .filter(behalf_assignments::workspace_id.eq(workspace_id))
            .filter(behalf_assignments::assignment_type.eq(assignment_type))
            .order(behalf_assignments::created_at.desc())
            .select(BehalfAssignmentEntity::as_select())
            .load(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))
            .map(|rows| rows.into_iter().map(Into::into).collect())
    }

    async fn check_behalf_permission(
        &self,
        workspace_id: Uuid,
        assignee_user_id: Uuid,
        target_employee_id: Uuid,
        target_department_id: Option<Uuid>,
        assignment_type: &str,
    ) -> AppResult<bool> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let mut query = behalf_assignments::table
            .filter(behalf_assignments::workspace_id.eq(workspace_id))
            .filter(behalf_assignments::assignee_user_id.eq(assignee_user_id))
            .filter(behalf_assignments::assignment_type.eq(assignment_type))
            .into_boxed();

        if let Some(dept_id) = target_department_id {
            query = query.filter(
                behalf_assignments::target_employee_id
                    .eq(target_employee_id)
                    .or(behalf_assignments::target_department_id.eq(dept_id)),
            );
        } else {
            query = query.filter(behalf_assignments::target_employee_id.eq(target_employee_id));
        }

        let count: i64 = query
            .count()
            .get_result(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(count > 0)
    }

    async fn list_assignments_for_assignee(
        &self,
        workspace_id: Uuid,
        assignee_user_id: Uuid,
        assignment_type: &str,
    ) -> AppResult<Vec<BehalfAssignment>> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        behalf_assignments::table
            .filter(behalf_assignments::workspace_id.eq(workspace_id))
            .filter(behalf_assignments::assignee_user_id.eq(assignee_user_id))
            .filter(behalf_assignments::assignment_type.eq(assignment_type))
            .order(behalf_assignments::created_at.desc())
            .select(BehalfAssignmentEntity::as_select())
            .load(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))
            .map(|rows| rows.into_iter().map(Into::into).collect())
    }
}
