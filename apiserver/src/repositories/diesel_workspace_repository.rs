use crate::diesel_pool::DieselPools;
use crate::errors::AppError;
use crate::models::diesel::core::{MemberExtraRoleEntity, WorkspaceEntity, WorkspaceMemberEntity};
use crate::models::{MemberExtraRole, Workspace};
use crate::repositories::workspace_repository::WorkspaceRepository;
use crate::schema::{
    member_extra_roles, users, workspace_member_extra_permissions, workspace_members,
    workspace_role_permissions, workspace_roles, workspaces,
};
use async_trait::async_trait;
use chrono::Utc;
use diesel::prelude::*;
use diesel_async::{AsyncConnection, RunQueryDsl};
use uuid::Uuid;

pub struct DieselWorkspaceRepository {
    pub diesel_pools: DieselPools,
}

impl DieselWorkspaceRepository {
    pub fn new(diesel_pools: DieselPools) -> Self {
        Self { diesel_pools }
    }
}

#[allow(clippy::too_many_lines)]
#[async_trait]
impl WorkspaceRepository for DieselWorkspaceRepository {
    async fn create_workspace_with_owner(
        &self,
        id: Uuid,
        name: &str,
        owner_id: Uuid,
        custom_db_url_encrypted: Option<String>,
    ) -> Result<Workspace, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .api
                .get()
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let entity = WorkspaceEntity {
            id,
            name: name.to_string(),
            created_at: Some(Utc::now()),
            date_format: "yyyy-MM-dd".to_string(),
            time_format: "HH:mm".to_string(),
            timezone: "UTC".to_string(),
            theme_primary_color: Some("#1890ff".to_string()),
            default_language: "en".to_string(),
            logo_url: None,
            tagline: None,
            custom_db_url_encrypted,
        };

        let owner_role_id = Uuid::parse_str("a0000000-0000-0000-0000-000000000001")
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let member_entity = WorkspaceMemberEntity {
            workspace_id: id,
            user_id: owner_id,
            role_id: owner_role_id,
        };

        let entity_clone = entity.clone();
        conn.transaction(|conn| {
            Box::pin(async move {
                diesel::insert_into(workspaces::table)
                    .values(&entity)
                    .execute(&mut *conn)
                    .await?;

                diesel::insert_into(workspace_members::table)
                    .values(&member_entity)
                    .execute(&mut *conn)
                    .await?;

                Ok::<_, diesel::result::Error>(())
            })
        })
        .await
        .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(entity_clone.into())
    }

    async fn get_workspace_by_id(&self, id: Uuid) -> Result<Option<Workspace>, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .api
                .get()
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let entity = workspaces::table
            .find(id)
            .select(WorkspaceEntity::as_select())
            .first::<WorkspaceEntity>(&mut *conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(entity.map(Into::into))
    }

    async fn get_all_workspaces(&self) -> Result<Vec<Workspace>, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .api
                .get()
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let results = workspaces::table
            .select(WorkspaceEntity::as_select())
            .load(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(results.into_iter().map(Into::into).collect())
    }

    async fn get_user_workspaces(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<crate::models::Workspace>, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .api
                .get()
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        use crate::schema::{workspace_members, workspaces};

        let results: Vec<WorkspaceEntity> = workspaces::table
            .inner_join(
                workspace_members::table.on(workspaces::id.eq(workspace_members::workspace_id)),
            )
            .filter(workspace_members::user_id.eq(user_id))
            .select(WorkspaceEntity::as_select())
            .load(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(results.into_iter().map(Into::into).collect())
    }

    async fn clear_workspace_data(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        group: &str,
    ) -> Result<(), AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;
        let workspace_id = auth_token.workspace_id();

        conn.transaction(|conn| {
            Box::pin(async move {
                let ws_id = workspace_id;

                use crate::schema::{
                    attendance_adjustment_requests, attendance_import_jobs, departments,
                    employee_department_history, employee_salary_components, employee_shifts,
                    employees, leave_accrual_logs, leave_balances, leave_policy_assignments,
                    leave_policy_definitions, leave_requests, payslips, raw_attendance_logs,
                    salary_component_definitions, shift_rosters, shifts, time_entries,
                };

                let emp_subquery = employees::dsl::employees
                    .filter(employees::dsl::workspace_id.eq(ws_id))
                    .select(employees::dsl::id);

                match group {
                    "attendance" => {
                        diesel::delete(
                            time_entries::dsl::time_entries
                                .filter(time_entries::dsl::employee_id.eq_any(emp_subquery)),
                        )
                        .execute(&mut *conn)
                        .await?;
                        diesel::delete(
                            raw_attendance_logs::dsl::raw_attendance_logs
                                .filter(raw_attendance_logs::dsl::workspace_id.eq(ws_id)),
                        )
                        .execute(&mut *conn)
                        .await?;
                        diesel::delete(
                            attendance_adjustment_requests::dsl::attendance_adjustment_requests
                                .filter(
                                    attendance_adjustment_requests::dsl::employee_id
                                        .eq_any(emp_subquery),
                                ),
                        )
                        .execute(&mut *conn)
                        .await?;
                        diesel::delete(
                            attendance_import_jobs::dsl::attendance_import_jobs
                                .filter(attendance_import_jobs::dsl::workspace_id.eq(ws_id)),
                        )
                        .execute(&mut *conn)
                        .await?;
                    }
                    "schedules" => {
                        diesel::delete(
                            shift_rosters::dsl::shift_rosters
                                .filter(shift_rosters::dsl::employee_id.eq_any(emp_subquery)),
                        )
                        .execute(&mut *conn)
                        .await?;
                        diesel::delete(
                            employee_shifts::dsl::employee_shifts
                                .filter(employee_shifts::dsl::employee_id.eq_any(emp_subquery)),
                        )
                        .execute(&mut *conn)
                        .await?;
                    }
                    "leave" => {
                        diesel::delete(
                            leave_accrual_logs::dsl::leave_accrual_logs
                                .filter(leave_accrual_logs::dsl::workspace_id.eq(ws_id)),
                        )
                        .execute(&mut *conn)
                        .await?;
                        diesel::delete(
                            leave_requests::dsl::leave_requests
                                .filter(leave_requests::dsl::employee_id.eq_any(emp_subquery)),
                        )
                        .execute(&mut *conn)
                        .await?;
                        diesel::delete(
                            leave_balances::dsl::leave_balances
                                .filter(leave_balances::dsl::employee_id.eq_any(emp_subquery)),
                        )
                        .execute(&mut *conn)
                        .await?;
                    }
                    "payroll" => {
                        diesel::delete(
                            payslips::dsl::payslips
                                .filter(payslips::dsl::employee_id.eq_any(emp_subquery)),
                        )
                        .execute(&mut *conn)
                        .await?;
                        diesel::delete(
                            employee_salary_components::dsl::employee_salary_components.filter(
                                employee_salary_components::dsl::employee_id.eq_any(emp_subquery),
                            ),
                        )
                        .execute(&mut *conn)
                        .await?;
                        diesel::delete(
                            salary_component_definitions::dsl::salary_component_definitions
                                .filter(salary_component_definitions::dsl::workspace_id.eq(ws_id)),
                        )
                        .execute(&mut *conn)
                        .await?;
                    }
                    "full_reset" => {
                        diesel::delete(
                            time_entries::dsl::time_entries
                                .filter(time_entries::dsl::employee_id.eq_any(emp_subquery)),
                        )
                        .execute(&mut *conn)
                        .await?;
                        diesel::delete(
                            raw_attendance_logs::dsl::raw_attendance_logs
                                .filter(raw_attendance_logs::dsl::workspace_id.eq(ws_id)),
                        )
                        .execute(&mut *conn)
                        .await?;
                        diesel::delete(
                            attendance_adjustment_requests::dsl::attendance_adjustment_requests
                                .filter(
                                    attendance_adjustment_requests::dsl::employee_id
                                        .eq_any(emp_subquery),
                                ),
                        )
                        .execute(&mut *conn)
                        .await?;
                        diesel::delete(
                            attendance_import_jobs::dsl::attendance_import_jobs
                                .filter(attendance_import_jobs::dsl::workspace_id.eq(ws_id)),
                        )
                        .execute(&mut *conn)
                        .await?;
                        diesel::delete(
                            shift_rosters::dsl::shift_rosters
                                .filter(shift_rosters::dsl::employee_id.eq_any(emp_subquery)),
                        )
                        .execute(&mut *conn)
                        .await?;
                        diesel::delete(
                            employee_shifts::dsl::employee_shifts
                                .filter(employee_shifts::dsl::employee_id.eq_any(emp_subquery)),
                        )
                        .execute(&mut *conn)
                        .await?;
                        diesel::delete(
                            leave_accrual_logs::dsl::leave_accrual_logs
                                .filter(leave_accrual_logs::dsl::workspace_id.eq(ws_id)),
                        )
                        .execute(&mut *conn)
                        .await?;
                        diesel::delete(
                            leave_requests::dsl::leave_requests
                                .filter(leave_requests::dsl::employee_id.eq_any(emp_subquery)),
                        )
                        .execute(&mut *conn)
                        .await?;
                        diesel::delete(
                            leave_balances::dsl::leave_balances
                                .filter(leave_balances::dsl::employee_id.eq_any(emp_subquery)),
                        )
                        .execute(&mut *conn)
                        .await?;
                        diesel::delete(
                            payslips::dsl::payslips
                                .filter(payslips::dsl::employee_id.eq_any(emp_subquery)),
                        )
                        .execute(&mut *conn)
                        .await?;
                        diesel::delete(
                            employee_salary_components::dsl::employee_salary_components.filter(
                                employee_salary_components::dsl::employee_id.eq_any(emp_subquery),
                            ),
                        )
                        .execute(&mut *conn)
                        .await?;
                        diesel::delete(
                            salary_component_definitions::dsl::salary_component_definitions
                                .filter(salary_component_definitions::dsl::workspace_id.eq(ws_id)),
                        )
                        .execute(&mut *conn)
                        .await?;

                        diesel::delete(
                            employee_department_history::dsl::employee_department_history.filter(
                                employee_department_history::dsl::employee_id.eq_any(emp_subquery),
                            ),
                        )
                        .execute(&mut *conn)
                        .await?;
                        diesel::delete(
                            leave_policy_assignments::dsl::leave_policy_assignments
                                .filter(leave_policy_assignments::dsl::workspace_id.eq(ws_id)),
                        )
                        .execute(&mut *conn)
                        .await?;
                        diesel::delete(
                            leave_policy_definitions::dsl::leave_policy_definitions
                                .filter(leave_policy_definitions::dsl::workspace_id.eq(ws_id)),
                        )
                        .execute(&mut *conn)
                        .await?;
                        diesel::delete(
                            employees::dsl::employees
                                .filter(employees::dsl::workspace_id.eq(ws_id)),
                        )
                        .execute(&mut *conn)
                        .await?;
                        diesel::delete(
                            shifts::dsl::shifts.filter(shifts::dsl::workspace_id.eq(ws_id)),
                        )
                        .execute(&mut *conn)
                        .await?;
                        diesel::delete(
                            departments::dsl::departments
                                .filter(departments::dsl::workspace_id.eq(ws_id)),
                        )
                        .execute(&mut *conn)
                        .await?;
                    }
                    _ => return Err(diesel::result::Error::RollbackTransaction),
                }
                Ok::<_, diesel::result::Error>(())
            })
        })
        .await
        .map_err(|e| match e {
            diesel::result::Error::RollbackTransaction => {
                AppError::Validation("Invalid group".to_string())
            }
            e => AppError::Internal(format!("db error: {e}")),
        })?;

        Ok(())
    }

    async fn add_member(
        &self,
        workspace_id: Uuid,
        user_id: Uuid,
        role_id: Uuid,
    ) -> Result<(), AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .api
                .get()
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let entity = WorkspaceMemberEntity {
            workspace_id,
            user_id,
            role_id,
        };

        diesel::insert_into(workspace_members::table)
            .values(&entity)
            .on_conflict((workspace_members::workspace_id, workspace_members::user_id))
            .do_nothing()
            .execute(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(())
    }

    async fn update_member_role(
        &self,
        workspace_id: Uuid,
        user_id: Uuid,
        role_id: Uuid,
    ) -> Result<(), AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .api
                .get()
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        diesel::update(
            workspace_members::table.filter(
                workspace_members::workspace_id
                    .eq(workspace_id)
                    .and(workspace_members::user_id.eq(user_id)),
            ),
        )
        .set(workspace_members::role_id.eq(role_id))
        .execute(&mut *conn)
        .await
        .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(())
    }

    async fn remove_member(&self, workspace_id: Uuid, user_id: Uuid) -> Result<(), AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .api
                .get()
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        diesel::delete(
            workspace_members::table.filter(
                workspace_members::workspace_id
                    .eq(workspace_id)
                    .and(workspace_members::user_id.eq(user_id)),
            ),
        )
        .execute(&mut *conn)
        .await
        .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(())
    }

    async fn find_primary_role(
        &self,
        workspace_id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<Uuid>, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .api
                .get()
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let role_id = workspace_members::table
            .filter(
                workspace_members::workspace_id
                    .eq(workspace_id)
                    .and(workspace_members::user_id.eq(user_id)),
            )
            .select(workspace_members::role_id)
            .first::<Uuid>(&mut *conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(role_id)
    }

    async fn get_workspace_members(
        &self,
        workspace_id: Uuid,
    ) -> Result<Vec<crate::models::WorkspaceMemberDetail>, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .api
                .get()
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let results: Vec<(
            WorkspaceMemberEntity,
            crate::models::diesel::core::UserEntity,
        )> = workspace_members::table
            .inner_join(users::table.on(workspace_members::user_id.eq(users::id)))
            .filter(workspace_members::workspace_id.eq(workspace_id))
            .load::<(
                WorkspaceMemberEntity,
                crate::models::diesel::core::UserEntity,
            )>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(results
            .into_iter()
            .map(|(wm, u)| crate::models::WorkspaceMemberDetail {
                workspace_id: wm.workspace_id,
                user_id: wm.user_id,
                role_id: wm.role_id,
                email: u.email,
                full_name: u.full_name,
                avatar_url: u.avatar_url,
            })
            .collect())
    }

    async fn get_admins_with_permissions(
        &self,
        workspace_id_val: Uuid,
        permissions: &[&str],
    ) -> Result<Vec<Uuid>, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .api
                .get()
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let results: Vec<Uuid> = workspace_members::table
            .inner_join(
                workspace_roles::table.on(workspace_members::role_id.eq(workspace_roles::id)),
            )
            .left_join(
                workspace_role_permissions::table
                    .on(workspace_roles::id.eq(workspace_role_permissions::role_id)),
            )
            .filter(workspace_members::workspace_id.eq(workspace_id_val))
            .filter(
                workspace_roles::scope_type
                    .eq("global")
                    .or(workspace_role_permissions::permission_name.eq_any(permissions)),
            )
            .select(workspace_members::user_id)
            .distinct()
            .load::<Uuid>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db error fetching admins: {e}")))?;

        Ok(results)
    }

    async fn get_all_extra_roles(
        &self,
        workspace_id: Uuid,
    ) -> Result<Vec<MemberExtraRole>, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .api
                .get()
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let entities = member_extra_roles::table
            .filter(
                member_extra_roles::workspace_id
                    .eq(workspace_id)
                    .and(member_extra_roles::expires_at.gt(Utc::now())),
            )
            .load::<MemberExtraRoleEntity>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(entities.into_iter().map(Into::into).collect())
    }

    async fn find_extra_roles(
        &self,
        workspace_id: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<MemberExtraRole>, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .api
                .get()
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let entities = member_extra_roles::table
            .filter(
                member_extra_roles::workspace_id
                    .eq(workspace_id)
                    .and(member_extra_roles::user_id.eq(user_id)),
            )
            .load::<MemberExtraRoleEntity>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(entities.into_iter().map(Into::into).collect())
    }

    async fn find_extra_permissions(
        &self,
        workspace_id: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<String>, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .api
                .get()
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let permissions = workspace_member_extra_permissions::table
            .filter(
                workspace_member_extra_permissions::workspace_id
                    .eq(workspace_id)
                    .and(workspace_member_extra_permissions::user_id.eq(user_id))
                    .and(workspace_member_extra_permissions::expires_at.gt(Utc::now())),
            )
            .select(workspace_member_extra_permissions::permission)
            .load::<String>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(permissions)
    }

    async fn get_member_extra_permissions(
        &self,
        workspace_id: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<crate::models::MemberExtraPermission>, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .api
                .get()
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        use crate::models::diesel::core::MemberExtraPermissionEntity;
        use crate::schema::workspace_member_extra_permissions;

        let results: Vec<MemberExtraPermissionEntity> =
            workspace_member_extra_permissions::dsl::workspace_member_extra_permissions
                .filter(workspace_member_extra_permissions::dsl::workspace_id.eq(workspace_id))
                .filter(workspace_member_extra_permissions::dsl::user_id.eq(user_id))
                .filter(workspace_member_extra_permissions::dsl::expires_at.gt(chrono::Utc::now()))
                .order_by(workspace_member_extra_permissions::dsl::assigned_at.desc())
                .load(&mut *conn)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(results
            .into_iter()
            .map(|e| crate::models::MemberExtraPermission {
                id: e.id,
                workspace_id: e.workspace_id,
                user_id: e.user_id,
                permission: e.permission,
                expires_at: e.expires_at,
                assigned_by: e.assigned_by,
                assigned_at: e.assigned_at,
            })
            .collect())
    }

    async fn assign_extra_permission(
        &self,
        workspace_id: Uuid,
        user_id: Uuid,
        permission: String,
        expires_at: chrono::DateTime<chrono::Utc>,
        assigned_by: Uuid,
    ) -> Result<crate::models::MemberExtraPermission, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .api
                .get()
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        use crate::models::diesel::core::MemberExtraPermissionEntity;
        use crate::schema::workspace_member_extra_permissions;

        let entity: MemberExtraPermissionEntity =
            diesel::insert_into(workspace_member_extra_permissions::table)
                .values((
                    workspace_member_extra_permissions::dsl::workspace_id.eq(workspace_id),
                    workspace_member_extra_permissions::dsl::user_id.eq(user_id),
                    workspace_member_extra_permissions::dsl::permission.eq(permission),
                    workspace_member_extra_permissions::dsl::expires_at.eq(expires_at),
                    workspace_member_extra_permissions::dsl::assigned_by.eq(assigned_by),
                ))
                .get_result(&mut *conn)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(crate::models::MemberExtraPermission {
            id: entity.id,
            workspace_id: entity.workspace_id,
            user_id: entity.user_id,
            permission: entity.permission,
            expires_at: entity.expires_at,
            assigned_by: entity.assigned_by,
            assigned_at: entity.assigned_at,
        })
    }

    async fn revoke_extra_permission(
        &self,
        workspace_id: Uuid,
        _user_id: Uuid,
        perm_id: Uuid,
    ) -> Result<u64, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .api
                .get()
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let affected = diesel::delete(
            workspace_member_extra_permissions::table
                .filter(workspace_member_extra_permissions::id.eq(perm_id))
                .filter(workspace_member_extra_permissions::workspace_id.eq(workspace_id)),
        )
        .execute(&mut *conn)
        .await
        .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(affected as u64)
    }

    async fn assign_extra_role(
        &self,
        workspace_id: Uuid,
        user_id: Uuid,
        role_id: Uuid,
        expires_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<crate::models::MemberExtraRole, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .api
                .get()
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        use crate::schema::member_extra_roles;

        let new_role = crate::models::diesel::core::MemberExtraRoleEntity {
            id: uuid::Uuid::new_v4(),
            workspace_id,
            user_id,
            role_id,
            expires_at,
            assigned_at: Some(chrono::Utc::now()),
        };

        let entity: MemberExtraRoleEntity = diesel::insert_into(member_extra_roles::table)
            .values(&new_role)
            .get_result(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(entity.into())
    }

    async fn revoke_extra_role(
        &self,
        workspace_id: Uuid,
        extra_role_id: Uuid,
    ) -> Result<u64, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .api
                .get()
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        use crate::schema::member_extra_roles;
        let affected = diesel::delete(
            member_extra_roles::dsl::member_extra_roles
                .filter(member_extra_roles::dsl::id.eq(extra_role_id))
                .filter(member_extra_roles::dsl::workspace_id.eq(workspace_id)),
        )
        .execute(&mut *conn)
        .await
        .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(affected as u64)
    }

    async fn get_workspace_roles(
        &self,
        workspace_id: Uuid,
    ) -> Result<Vec<crate::models::WorkspaceRole>, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .api
                .get()
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        use crate::models::diesel::core::WorkspaceRoleEntity;
        use crate::schema::workspace_roles;

        let results: Vec<WorkspaceRoleEntity> = workspace_roles::dsl::workspace_roles
            .filter(
                workspace_roles::dsl::workspace_id
                    .eq(workspace_id)
                    .or(workspace_roles::dsl::is_system.eq(true)),
            )
            .order_by((
                workspace_roles::dsl::is_system.desc(),
                workspace_roles::dsl::created_at.asc(),
            ))
            .load(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(results.into_iter().map(Into::into).collect())
    }

    async fn create_workspace_role(
        &self,
        workspace_id: Uuid,
        name: String,
        description: Option<String>,
        color: String,
        scope_type: String,
    ) -> Result<crate::models::WorkspaceRole, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .api
                .get()
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        use crate::models::diesel::core::WorkspaceRoleEntity;
        use crate::schema::workspace_roles;

        let entity: WorkspaceRoleEntity = diesel::insert_into(workspace_roles::table)
            .values((
                workspace_roles::dsl::workspace_id.eq(workspace_id),
                workspace_roles::dsl::name.eq(name),
                workspace_roles::dsl::description.eq(description),
                workspace_roles::dsl::color.eq(color),
                workspace_roles::dsl::is_system.eq(false),
                workspace_roles::dsl::scope_type.eq(scope_type),
            ))
            .get_result(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(entity.into())
    }

    async fn update_workspace_role(
        &self,
        workspace_id: Uuid,
        role_id: Uuid,
        name: Option<String>,
        description: Option<String>,
        color: Option<String>,
    ) -> Result<Option<crate::models::WorkspaceRole>, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .api
                .get()
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        use crate::models::diesel::core::WorkspaceRoleEntity;
        use crate::schema::workspace_roles;

        #[derive(AsChangeset)]
        #[diesel(table_name = crate::schema::workspace_roles)]
        struct UpdateRole {
            name: Option<String>,
            description: Option<String>,
            color: Option<String>,
            updated_at: chrono::DateTime<chrono::Utc>,
        }

        let update_data = UpdateRole {
            name,
            description,
            color,
            updated_at: chrono::Utc::now(),
        };

        let result: Option<WorkspaceRoleEntity> = diesel::update(
            workspace_roles::dsl::workspace_roles
                .filter(workspace_roles::dsl::id.eq(role_id))
                .filter(workspace_roles::dsl::workspace_id.eq(workspace_id))
                .filter(workspace_roles::dsl::is_system.eq(false)),
        )
        .set(&update_data)
        .get_result(&mut *conn)
        .await
        .optional()
        .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(result.map(Into::into))
    }

    async fn delete_workspace_role(
        &self,
        workspace_id: Uuid,
        role_id: Uuid,
    ) -> Result<u64, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .api
                .get()
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        use crate::schema::workspace_roles;
        let affected = diesel::delete(
            workspace_roles::dsl::workspace_roles
                .filter(workspace_roles::dsl::id.eq(role_id))
                .filter(workspace_roles::dsl::workspace_id.eq(workspace_id))
                .filter(workspace_roles::dsl::is_system.eq(false)),
        )
        .execute(&mut *conn)
        .await
        .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(affected as u64)
    }

    async fn get_role_permissions_by_role_id(
        &self,
        workspace_id: Uuid,
        role_id: Uuid,
    ) -> Result<Vec<String>, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .api
                .get()
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        use crate::schema::{workspace_role_permissions, workspace_roles};
        let valid_roles = workspace_roles::dsl::workspace_roles
            .filter(
                workspace_roles::dsl::workspace_id
                    .eq(workspace_id)
                    .or(workspace_roles::dsl::is_system.eq(true)),
            )
            .select(workspace_roles::dsl::id);

        let results: Vec<String> = workspace_role_permissions::dsl::workspace_role_permissions
            .filter(workspace_role_permissions::dsl::role_id.eq(role_id))
            .filter(workspace_role_permissions::dsl::role_id.eq_any(valid_roles))
            .select(workspace_role_permissions::dsl::permission_name)
            .load(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(results)
    }

    async fn role_exists_and_is_not_system(
        &self,
        workspace_id: Uuid,
        role_id: Uuid,
    ) -> Result<bool, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .api
                .get()
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        use crate::schema::workspace_roles;
        use diesel::dsl::{exists, select};

        let result: bool = select(exists(
            workspace_roles::dsl::workspace_roles
                .filter(workspace_roles::dsl::id.eq(role_id))
                .filter(workspace_roles::dsl::workspace_id.eq(workspace_id))
                .filter(workspace_roles::dsl::is_system.eq(false)),
        ))
        .get_result(&mut *conn)
        .await
        .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(result)
    }

    async fn set_role_permissions(
        &self,
        role_id: Uuid,
        permissions: Vec<String>,
    ) -> Result<(), AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .api
                .get()
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        conn.transaction(|conn| {
            Box::pin(async move {
                use crate::schema::workspace_role_permissions;

                diesel::delete(
                    workspace_role_permissions::dsl::workspace_role_permissions
                        .filter(workspace_role_permissions::dsl::role_id.eq(role_id)),
                )
                .execute(&mut *conn)
                .await?;

                if !permissions.is_empty() {
                    let inserts: Vec<_> = permissions
                        .into_iter()
                        .map(|perm| {
                            (
                                workspace_role_permissions::dsl::role_id.eq(role_id),
                                workspace_role_permissions::dsl::permission_name.eq(perm),
                            )
                        })
                        .collect();

                    diesel::insert_into(workspace_role_permissions::table)
                        .values(&inserts)
                        .execute(&mut *conn)
                        .await?;
                }

                Ok::<_, diesel::result::Error>(())
            })
        })
        .await
        .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(())
    }

    async fn get_permissions_for_roles(&self, role_ids: &[Uuid]) -> Result<Vec<String>, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .api
                .get()
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let permissions = workspace_role_permissions::table
            .filter(workspace_role_permissions::role_id.eq_any(role_ids))
            .select(workspace_role_permissions::permission_name)
            .load::<String>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(permissions)
    }

    async fn update_workspace(
        &self,
        id: Uuid,
        name: Option<String>,
        date_format: Option<String>,
        time_format: Option<String>,
        timezone: Option<String>,
        theme_primary_color: Option<String>,
        default_language: Option<String>,
        logo_url: Option<String>,
        tagline: Option<String>,
    ) -> Result<Workspace, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .api
                .get()
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        #[derive(AsChangeset)]
        #[diesel(table_name = workspaces)]
        #[allow(clippy::option_option)]
        struct WorkspaceUpdate {
            name: Option<String>,
            date_format: Option<String>,
            time_format: Option<String>,
            timezone: Option<String>,
            theme_primary_color: Option<Option<String>>,
            default_language: Option<String>,
            logo_url: Option<Option<String>>,
            tagline: Option<Option<String>>,
        }

        let has_changes = name.is_some()
            || date_format.is_some()
            || time_format.is_some()
            || timezone.is_some()
            || theme_primary_color.is_some()
            || default_language.is_some()
            || logo_url.is_some()
            || tagline.is_some();

        if !has_changes {
            return self
                .get_workspace_by_id(id)
                .await?
                .ok_or(AppError::NotFound("Workspace not found".to_string()));
        }

        let update = WorkspaceUpdate {
            name,
            date_format,
            time_format,
            timezone,
            theme_primary_color: theme_primary_color.map(Some),
            default_language,
            logo_url: logo_url.map(Some),
            tagline: tagline.map(Some),
        };

        let result: WorkspaceEntity = diesel::update(workspaces::table.find(id))
            .set(&update)
            .get_result(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        if let Some(ref new_tz) = update.timezone {
            let mut tenant_conn = crate::diesel_pool::TransactionConn(
                self.diesel_pools
                    .tenants_tx
                    .get_connection(&id)
                    .await
                    .map_err(|e| AppError::Internal(format!("Failed to get tenant conn: {e}")))?,
            );
            use crate::schema::work_locations;
            diesel::update(
                work_locations::table
                    .filter(work_locations::workspace_id.eq(id))
                    .filter(work_locations::is_headquarters.eq(true)),
            )
            .set(work_locations::timezone.eq(new_tz))
            .execute(&mut *tenant_conn)
            .await
            .map_err(|e| AppError::Internal(format!("db tenant: {e}")))?;
        }

        Ok(result.into())
    }

    async fn is_any_role_global(&self, role_ids: &[Uuid]) -> Result<bool, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .api
                .get()
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let query = workspace_roles::table
            .filter(workspace_roles::id.eq_any(role_ids))
            .filter(workspace_roles::scope_type.eq("global"));

        let is_global: bool = diesel::select(diesel::dsl::exists(query))
            .get_result(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(is_global)
    }

    async fn update_workspace_byod_url(
        &self,
        workspace_id: Uuid,
        encrypted_url: Option<String>,
    ) -> Result<(), AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .api
                .get()
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        diesel::update(workspaces::table.find(workspace_id))
            .set(workspaces::custom_db_url_encrypted.eq(encrypted_url))
            .execute(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(())
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
        let _repo = DieselWorkspaceRepository::new(pools);
        // DI connection test passes if it compiles
    }
}
