use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::diesel_pool::DieselPools;
use crate::errors::{AppError, AppResult};
use crate::models::diesel::ot_claim_behalf_assignment::OtClaimBehalfAssignmentEntity;
use crate::models::OtClaimBehalfAssignment;
use crate::repositories::ot_claim_behalf_assignment_repository::OtClaimBehalfAssignmentRepository;
use crate::schema::ot_claim_behalf_assignments;

pub struct DieselOtClaimBehalfAssignmentRepository {
    pub diesel_pools: DieselPools,
}

impl DieselOtClaimBehalfAssignmentRepository {
    pub fn new(diesel_pools: DieselPools) -> Self {
        Self { diesel_pools }
    }
}

#[async_trait]
impl OtClaimBehalfAssignmentRepository for DieselOtClaimBehalfAssignmentRepository {
    async fn create_assignment(
        &self,
        workspace_id: Uuid,
        assignee_employee_id: Uuid,
        target_employee_id: Option<Uuid>,
        target_department_id: Option<Uuid>,
    ) -> AppResult<OtClaimBehalfAssignment> {
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

        diesel::insert_into(ot_claim_behalf_assignments::table)
            .values((
                ot_claim_behalf_assignments::workspace_id.eq(workspace_id),
                ot_claim_behalf_assignments::assignee_employee_id.eq(assignee_employee_id),
                ot_claim_behalf_assignments::target_employee_id.eq(target_employee_id),
                ot_claim_behalf_assignments::target_department_id.eq(target_department_id),
            ))
            .returning(OtClaimBehalfAssignmentEntity::as_returning())
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

        let affected = diesel::delete(ot_claim_behalf_assignments::table)
            .filter(ot_claim_behalf_assignments::id.eq(id))
            .filter(ot_claim_behalf_assignments::workspace_id.eq(workspace_id))
            .execute(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(affected as u64)
    }

    async fn list_all_assignments(
        &self,
        workspace_id: Uuid,
    ) -> AppResult<Vec<OtClaimBehalfAssignment>> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        ot_claim_behalf_assignments::table
            .filter(ot_claim_behalf_assignments::workspace_id.eq(workspace_id))
            .order(ot_claim_behalf_assignments::created_at.desc())
            .select(OtClaimBehalfAssignmentEntity::as_select())
            .load(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))
            .map(|rows| rows.into_iter().map(Into::into).collect())
    }

    async fn check_behalf_permission(
        &self,
        workspace_id: Uuid,
        assignee_employee_id: Uuid,
        target_employee_id: Uuid,
        target_department_id: Option<Uuid>,
    ) -> AppResult<bool> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let mut query = ot_claim_behalf_assignments::table
            .filter(ot_claim_behalf_assignments::workspace_id.eq(workspace_id))
            .filter(ot_claim_behalf_assignments::assignee_employee_id.eq(assignee_employee_id))
            .into_boxed();

        if let Some(dept_id) = target_department_id {
            query = query.filter(
                ot_claim_behalf_assignments::target_employee_id
                    .eq(target_employee_id)
                    .or(ot_claim_behalf_assignments::target_department_id.eq(dept_id)),
            );
        } else {
            query = query
                .filter(ot_claim_behalf_assignments::target_employee_id.eq(target_employee_id));
        }

        let count: i64 = query
            .count()
            .get_result(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(count > 0)
    }
}
