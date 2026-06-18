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
use crate::utils::i18n::I18nKey;

pub struct DieselBehalfAssignmentRepository {
    diesel_pools: DieselPools,
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
        employer_id: Option<Uuid>,
    ) -> AppResult<BehalfAssignment> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        if target_employee_id.is_none() && target_department_id.is_none() {
            return Err(AppError::validation(I18nKey::BehalfAssignmentMissingTarget));
        }
        if target_employee_id.is_some() && target_department_id.is_some() {
            return Err(AppError::validation(I18nKey::BehalfAssignmentDualTarget));
        }

        diesel::insert_into(behalf_assignments::table)
            .values((
                behalf_assignments::workspace_id.eq(workspace_id),
                behalf_assignments::assignee_user_id.eq(assignee_user_id),
                behalf_assignments::target_employee_id.eq(target_employee_id),
                behalf_assignments::target_department_id.eq(target_department_id),
                behalf_assignments::assignment_type.eq(assignment_type),
                behalf_assignments::employer_id.eq(employer_id),
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

        let rows: Vec<BehalfAssignmentEntity> = behalf_assignments::table
            .filter(behalf_assignments::workspace_id.eq(workspace_id))
            .filter(behalf_assignments::assignment_type.eq(assignment_type))
            .order(behalf_assignments::created_at.desc())
            .select(BehalfAssignmentEntity::as_select())
            .load(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        if rows.is_empty() {
            return Ok(vec![]);
        }

        // 1. Fetch Assignee Employees
        let assignee_user_ids: Vec<Uuid> = rows.iter().map(|a| a.assignee_user_id).collect();
        let assignees: Vec<(Option<Uuid>, String, String)> = crate::schema::employees::table
            .filter(crate::schema::employees::workspace_id.eq(workspace_id))
            .filter(crate::schema::employees::deleted_at.is_null())
            .filter(crate::schema::employees::user_id.is_not_null())
            .filter(crate::schema::employees::user_id.eq_any(assignee_user_ids))
            .select((
                crate::schema::employees::user_id,
                crate::schema::employees::full_name,
                crate::schema::employees::employee_id_number,
            ))
            .load(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        // 2. Fetch Target Employees
        let target_emp_ids: Vec<Uuid> = rows.iter().filter_map(|a| a.target_employee_id).collect();
        let targets: Vec<(Uuid, String, String)> = crate::schema::employees::table
            .filter(crate::schema::employees::workspace_id.eq(workspace_id))
            .filter(crate::schema::employees::deleted_at.is_null())
            .filter(crate::schema::employees::id.eq_any(target_emp_ids))
            .select((
                crate::schema::employees::id,
                crate::schema::employees::full_name,
                crate::schema::employees::employee_id_number,
            ))
            .load(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        // 3. Fetch Target Departments
        let target_dept_ids: Vec<Uuid> =
            rows.iter().filter_map(|a| a.target_department_id).collect();
        let depts: Vec<(Uuid, String)> = crate::schema::departments::table
            .filter(crate::schema::departments::workspace_id.eq(workspace_id))
            .filter(crate::schema::departments::deleted_at.is_null())
            .filter(crate::schema::departments::id.eq_any(target_dept_ids))
            .select((
                crate::schema::departments::id,
                crate::schema::departments::name,
            ))
            .load(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let assignee_map: std::collections::HashMap<Uuid, (String, String)> = assignees
            .into_iter()
            .filter_map(|(uid, name, id_num)| uid.map(|id| (id, (name, id_num))))
            .collect();

        let target_map: std::collections::HashMap<Uuid, (String, String)> = targets
            .into_iter()
            .map(|(eid, name, id_num)| (eid, (name, id_num)))
            .collect();

        let dept_map: std::collections::HashMap<Uuid, String> = depts.into_iter().collect();

        let mut results = Vec::new();
        for row in rows {
            let mut item: BehalfAssignment = row.into();
            if let Some((name, id_num)) = assignee_map.get(&item.assignee_user_id()) {
                item.set_assignee_name(Some(name.clone()));
                item.set_assignee_employee_id_number(Some(id_num.clone()));
            }
            if let Some(target_id) = item.target_employee_id() {
                if let Some((name, id_num)) = target_map.get(&target_id) {
                    item.set_target_employee_name(Some(name.clone()));
                    item.set_target_employee_id_number(Some(id_num.clone()));
                }
            }
            if let Some(dept_id) = item.target_department_id() {
                if let Some(name) = dept_map.get(&dept_id) {
                    item.set_target_department_name(Some(name.clone()));
                }
            }
            results.push(item);
        }

        Ok(results)
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

        // Fetch target employee's active employer_id
        let target_employer_id: Option<Uuid> = crate::schema::employments::table
            .filter(crate::schema::employments::workspace_id.eq(workspace_id))
            .filter(crate::schema::employments::employee_id.eq(target_employee_id))
            .filter(crate::schema::employments::status.eq("active"))
            .filter(crate::schema::employments::deleted_at.is_null())
            .select(crate::schema::employments::employer_id)
            .first(&mut *conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let mut query = behalf_assignments::table
            .filter(behalf_assignments::workspace_id.eq(workspace_id))
            .filter(behalf_assignments::assignee_user_id.eq(assignee_user_id))
            .filter(behalf_assignments::assignment_type.eq(assignment_type))
            .into_boxed();

        if let Some(emp_employer_id) = target_employer_id {
            query = query.filter(
                behalf_assignments::employer_id
                    .is_null()
                    .or(behalf_assignments::employer_id.eq(emp_employer_id)),
            );
        } else {
            query = query.filter(behalf_assignments::employer_id.is_null());
        }

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
