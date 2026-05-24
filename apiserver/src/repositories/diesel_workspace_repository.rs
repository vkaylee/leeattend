use crate::diesel_pool::DieselPools;
use crate::errors::AppError;
use crate::models::diesel::core::{MemberExtraRoleEntity, WorkspaceEntity, WorkspaceMemberEntity};
use crate::models::{MemberExtraRole, Workspace};
use crate::repositories::workspace_repository::WorkspaceRepository;
use crate::schema::{
    member_extra_roles, workspace_member_extra_permissions, workspace_members,
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

#[async_trait]
impl WorkspaceRepository for DieselWorkspaceRepository {
    async fn create_workspace_with_owner(
        &self,
        id: Uuid,
        name: &str,
        owner_id: Uuid,
        custom_db_url_encrypted: Option<String>,
    ) -> Result<Workspace, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let entity = WorkspaceEntity {
            id,
            name: name.to_string(),
            created_at: Some(Utc::now()),
            date_format: "yyyy-MM-dd".to_string(),
            time_format: "HH:mm".to_string(),
            timezone: "UTC".to_string(),
            theme_primary_color: Some("#1890ff".to_string()),
            default_language: "en".to_string(),
            holiday_overnight_strategy: "split".to_string(),
            default_holiday_ot_multiplier: bigdecimal::BigDecimal::from(2),
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
        conn.transaction(|mut conn| {
            Box::pin(async move {
                diesel::insert_into(workspaces::table)
                    .values(&entity)
                    .execute(&mut conn)
                    .await?;

                diesel::insert_into(workspace_members::table)
                    .values(&member_entity)
                    .execute(&mut conn)
                    .await?;

                Ok::<_, diesel::result::Error>(())
            })
        })
        .await
        .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(entity_clone.into())
    }

    async fn get_workspace_by_id(&self, id: Uuid) -> Result<Option<Workspace>, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let entity = workspaces::table
            .find(id)
            .select(WorkspaceEntity::as_select())
            .first::<WorkspaceEntity>(&mut conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(entity.map(Into::into))
    }

    async fn get_all_workspaces(&self) -> Result<Vec<Workspace>, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let results = workspaces::table
            .select(WorkspaceEntity::as_select())
            .load(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(results.into_iter().map(Into::into).collect())
    }

    async fn get_user_workspaces(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<crate::models::Workspace>, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        use crate::schema::{workspace_members, workspaces};

        let results: Vec<WorkspaceEntity> = workspaces::table
            .inner_join(
                workspace_members::table.on(workspaces::id.eq(workspace_members::workspace_id)),
            )
            .filter(workspace_members::user_id.eq(user_id))
            .select(WorkspaceEntity::as_select())
            .load(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(results.into_iter().map(Into::into).collect())
    }

    async fn clear_workspace_data(&self, workspace_id: Uuid, group: &str) -> Result<(), AppError> {
        let mut conn = self.diesel_pools.tenant_conn(workspace_id).await?;

        conn.transaction(|mut conn| {
            Box::pin(async move {
                let ws_id = workspace_id;

                match group {
                    "attendance" => {
                        diesel::sql_query("DELETE FROM time_entries WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)").bind::<diesel::sql_types::Uuid, _>(ws_id).execute(&mut conn).await?;
                        diesel::sql_query("DELETE FROM raw_attendance_logs WHERE workspace_id = $1").bind::<diesel::sql_types::Uuid, _>(ws_id).execute(&mut conn).await?;
                        diesel::sql_query("DELETE FROM attendance_adjustment_requests WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)").bind::<diesel::sql_types::Uuid, _>(ws_id).execute(&mut conn).await?;
                        diesel::sql_query("DELETE FROM attendance_import_jobs WHERE workspace_id = $1").bind::<diesel::sql_types::Uuid, _>(ws_id).execute(&mut conn).await?;
                    }
                    "schedules" => {
                        diesel::sql_query("DELETE FROM shift_rosters WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)").bind::<diesel::sql_types::Uuid, _>(ws_id).execute(&mut conn).await?;
                        diesel::sql_query("DELETE FROM employee_shifts WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)").bind::<diesel::sql_types::Uuid, _>(ws_id).execute(&mut conn).await?;
                    }
                    "leave" => {
                        diesel::sql_query("DELETE FROM leave_accrual_logs WHERE workspace_id = $1").bind::<diesel::sql_types::Uuid, _>(ws_id).execute(&mut conn).await?;
                        diesel::sql_query("DELETE FROM leave_requests WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)").bind::<diesel::sql_types::Uuid, _>(ws_id).execute(&mut conn).await?;
                        diesel::sql_query("DELETE FROM leave_balances WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)").bind::<diesel::sql_types::Uuid, _>(ws_id).execute(&mut conn).await?;
                    }
                    "payroll" => {
                        diesel::sql_query("DELETE FROM payslips WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)").bind::<diesel::sql_types::Uuid, _>(ws_id).execute(&mut conn).await?;
                        diesel::sql_query("DELETE FROM employee_salary_components WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)").bind::<diesel::sql_types::Uuid, _>(ws_id).execute(&mut conn).await?;
                        diesel::sql_query("DELETE FROM salary_component_definitions WHERE workspace_id = $1").bind::<diesel::sql_types::Uuid, _>(ws_id).execute(&mut conn).await?;
                    }
                    "full_reset" => {
                        diesel::sql_query("DELETE FROM time_entries WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)").bind::<diesel::sql_types::Uuid, _>(ws_id).execute(&mut conn).await?;
                        diesel::sql_query("DELETE FROM raw_attendance_logs WHERE workspace_id = $1").bind::<diesel::sql_types::Uuid, _>(ws_id).execute(&mut conn).await?;
                        diesel::sql_query("DELETE FROM attendance_adjustment_requests WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)").bind::<diesel::sql_types::Uuid, _>(ws_id).execute(&mut conn).await?;
                        diesel::sql_query("DELETE FROM attendance_import_jobs WHERE workspace_id = $1").bind::<diesel::sql_types::Uuid, _>(ws_id).execute(&mut conn).await?;
                        diesel::sql_query("DELETE FROM shift_rosters WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)").bind::<diesel::sql_types::Uuid, _>(ws_id).execute(&mut conn).await?;
                        diesel::sql_query("DELETE FROM employee_shifts WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)").bind::<diesel::sql_types::Uuid, _>(ws_id).execute(&mut conn).await?;
                        diesel::sql_query("DELETE FROM leave_accrual_logs WHERE workspace_id = $1").bind::<diesel::sql_types::Uuid, _>(ws_id).execute(&mut conn).await?;
                        diesel::sql_query("DELETE FROM leave_requests WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)").bind::<diesel::sql_types::Uuid, _>(ws_id).execute(&mut conn).await?;
                        diesel::sql_query("DELETE FROM leave_balances WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)").bind::<diesel::sql_types::Uuid, _>(ws_id).execute(&mut conn).await?;
                        diesel::sql_query("DELETE FROM payslips WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)").bind::<diesel::sql_types::Uuid, _>(ws_id).execute(&mut conn).await?;
                        diesel::sql_query("DELETE FROM employee_salary_components WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)").bind::<diesel::sql_types::Uuid, _>(ws_id).execute(&mut conn).await?;
                        diesel::sql_query("DELETE FROM salary_component_definitions WHERE workspace_id = $1").bind::<diesel::sql_types::Uuid, _>(ws_id).execute(&mut conn).await?;
                        diesel::sql_query("DELETE FROM employee_department_history WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)").bind::<diesel::sql_types::Uuid, _>(ws_id).execute(&mut conn).await?;
                        diesel::sql_query("DELETE FROM leave_policy_assignments WHERE workspace_id = $1").bind::<diesel::sql_types::Uuid, _>(ws_id).execute(&mut conn).await?;
                        diesel::sql_query("DELETE FROM leave_policy_definitions WHERE workspace_id = $1").bind::<diesel::sql_types::Uuid, _>(ws_id).execute(&mut conn).await?;
                        diesel::sql_query("DELETE FROM employees WHERE workspace_id = $1").bind::<diesel::sql_types::Uuid, _>(ws_id).execute(&mut conn).await?;
                        diesel::sql_query("DELETE FROM shifts WHERE workspace_id = $1").bind::<diesel::sql_types::Uuid, _>(ws_id).execute(&mut conn).await?;
                        diesel::sql_query("DELETE FROM departments WHERE workspace_id = $1").bind::<diesel::sql_types::Uuid, _>(ws_id).execute(&mut conn).await?;
                    }
                    _ => return Err(diesel::result::Error::RollbackTransaction),
                }
                Ok::<_, diesel::result::Error>(())
            })
        })
        .await
        .map_err(|e| match e {
            diesel::result::Error::RollbackTransaction => AppError::Validation("Invalid group".to_string()),
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
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let entity = WorkspaceMemberEntity {
            workspace_id,
            user_id,
            role_id,
        };

        diesel::insert_into(workspace_members::table)
            .values(&entity)
            .on_conflict((workspace_members::workspace_id, workspace_members::user_id))
            .do_nothing()
            .execute(&mut conn)
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
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        diesel::update(
            workspace_members::table.filter(
                workspace_members::workspace_id
                    .eq(workspace_id)
                    .and(workspace_members::user_id.eq(user_id)),
            ),
        )
        .set(workspace_members::role_id.eq(role_id))
        .execute(&mut conn)
        .await
        .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(())
    }

    async fn remove_member(&self, workspace_id: Uuid, user_id: Uuid) -> Result<(), AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        diesel::delete(
            workspace_members::table.filter(
                workspace_members::workspace_id
                    .eq(workspace_id)
                    .and(workspace_members::user_id.eq(user_id)),
            ),
        )
        .execute(&mut conn)
        .await
        .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(())
    }

    async fn find_primary_role(
        &self,
        workspace_id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<Uuid>, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let role_id = workspace_members::table
            .filter(
                workspace_members::workspace_id
                    .eq(workspace_id)
                    .and(workspace_members::user_id.eq(user_id)),
            )
            .select(workspace_members::role_id)
            .first::<Uuid>(&mut conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(role_id)
    }

    async fn get_workspace_members(
        &self,
        workspace_id: Uuid,
    ) -> Result<Vec<crate::models::WorkspaceMemberDetail>, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        #[derive(Debug, diesel::QueryableByName)]
        struct Entity {
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            workspace_id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            user_id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            role_id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Text)]
            email: String,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
            full_name: Option<String>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
            avatar_url: Option<String>,
        }

        let results: Vec<Entity> = diesel::sql_query(
            "SELECT wm.workspace_id, wm.user_id, wm.role_id, u.email, u.full_name, u.avatar_url
             FROM workspace_members wm
             JOIN users u ON wm.user_id = u.id
             WHERE wm.workspace_id = $1",
        )
        .bind::<diesel::sql_types::Uuid, _>(workspace_id)
        .load(&mut conn)
        .await
        .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(results
            .into_iter()
            .map(|e| crate::models::WorkspaceMemberDetail {
                workspace_id: e.workspace_id,
                user_id: e.user_id,
                role_id: e.role_id,
                email: e.email,
                full_name: e.full_name,
                avatar_url: e.avatar_url,
            })
            .collect())
    }

    async fn get_admins_with_permissions(
        &self,
        workspace_id_val: Uuid,
        permissions: &[&str],
    ) -> Result<Vec<Uuid>, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        #[derive(Debug, diesel::QueryableByName)]
        struct UserIdEntity {
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            user_id: Uuid,
        }

        // We use ANY to check against the permissions array safely
        let results: Vec<UserIdEntity> = diesel::sql_query(
            r#"
            SELECT DISTINCT wm.user_id 
            FROM workspace_members wm
            JOIN workspace_roles wr ON wm.role_id = wr.id
            LEFT JOIN workspace_role_permissions wrp ON wr.id = wrp.role_id
            WHERE wm.workspace_id = $1 
              AND (wr.scope_type = 'global' OR wrp.permission_name = ANY($2))
            "#,
        )
        .bind::<diesel::sql_types::Uuid, _>(workspace_id_val)
        .bind::<diesel::sql_types::Array<diesel::sql_types::Text>, _>(permissions)
        .load(&mut conn)
        .await
        .map_err(|e| AppError::Internal(format!("db error fetching admins: {e}")))?;

        Ok(results.into_iter().map(|e| e.user_id).collect())
    }

    async fn get_all_extra_roles(
        &self,
        workspace_id: Uuid,
    ) -> Result<Vec<MemberExtraRole>, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let entities = member_extra_roles::table
            .filter(
                member_extra_roles::workspace_id
                    .eq(workspace_id)
                    .and(member_extra_roles::expires_at.gt(Utc::now())),
            )
            .load::<MemberExtraRoleEntity>(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(entities.into_iter().map(Into::into).collect())
    }

    async fn find_extra_roles(
        &self,
        workspace_id: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<MemberExtraRole>, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let entities = member_extra_roles::table
            .filter(
                member_extra_roles::workspace_id
                    .eq(workspace_id)
                    .and(member_extra_roles::user_id.eq(user_id)),
            )
            .load::<MemberExtraRoleEntity>(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(entities.into_iter().map(Into::into).collect())
    }

    async fn find_extra_permissions(
        &self,
        workspace_id: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<String>, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let permissions = workspace_member_extra_permissions::table
            .filter(
                workspace_member_extra_permissions::workspace_id
                    .eq(workspace_id)
                    .and(workspace_member_extra_permissions::user_id.eq(user_id))
                    .and(workspace_member_extra_permissions::expires_at.gt(Utc::now())),
            )
            .select(workspace_member_extra_permissions::permission)
            .load::<String>(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(permissions)
    }

    async fn get_member_extra_permissions(
        &self,
        workspace_id: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<crate::models::MemberExtraPermission>, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        #[derive(Debug, diesel::QueryableByName)]
        struct Entity {
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            workspace_id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            user_id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Text)]
            permission: String,
            #[diesel(sql_type = diesel::sql_types::Timestamptz)]
            expires_at: chrono::DateTime<chrono::Utc>,
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            assigned_by: Uuid,
            #[diesel(sql_type = diesel::sql_types::Timestamptz)]
            assigned_at: chrono::DateTime<chrono::Utc>,
        }

        let results: Vec<Entity> = diesel::sql_query(
            "SELECT id, workspace_id, user_id, permission, expires_at, assigned_by, assigned_at 
             FROM workspace_member_extra_permissions 
             WHERE workspace_id = $1 AND user_id = $2 AND expires_at > NOW() 
             ORDER BY assigned_at DESC",
        )
        .bind::<diesel::sql_types::Uuid, _>(workspace_id)
        .bind::<diesel::sql_types::Uuid, _>(user_id)
        .load(&mut conn)
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
                assigned_by: Some(e.assigned_by),
                assigned_at: Some(e.assigned_at),
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
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        #[derive(Debug, diesel::QueryableByName)]
        struct Entity {
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            workspace_id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            user_id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Text)]
            permission: String,
            #[diesel(sql_type = diesel::sql_types::Timestamptz)]
            expires_at: chrono::DateTime<chrono::Utc>,
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            assigned_by: Uuid,
            #[diesel(sql_type = diesel::sql_types::Timestamptz)]
            assigned_at: chrono::DateTime<chrono::Utc>,
        }

        let entity: Entity = diesel::sql_query(
            "INSERT INTO workspace_member_extra_permissions (workspace_id, user_id, permission, expires_at, assigned_by) 
             VALUES ($1, $2, $3, $4, $5) RETURNING *"
        )
        .bind::<diesel::sql_types::Uuid, _>(workspace_id)
        .bind::<diesel::sql_types::Uuid, _>(user_id)
        .bind::<diesel::sql_types::Text, _>(permission)
        .bind::<diesel::sql_types::Timestamptz, _>(expires_at)
        .bind::<diesel::sql_types::Uuid, _>(assigned_by)
        .get_result(&mut conn)
        .await
        .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(crate::models::MemberExtraPermission {
            id: entity.id,
            workspace_id: entity.workspace_id,
            user_id: entity.user_id,
            permission: entity.permission,
            expires_at: entity.expires_at,
            assigned_by: Some(entity.assigned_by),
            assigned_at: Some(entity.assigned_at),
        })
    }

    async fn revoke_extra_permission(
        &self,
        workspace_id: Uuid,
        _user_id: Uuid,
        perm_id: Uuid,
    ) -> Result<u64, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let affected = diesel::delete(
            workspace_member_extra_permissions::table
                .filter(workspace_member_extra_permissions::id.eq(perm_id))
                .filter(workspace_member_extra_permissions::workspace_id.eq(workspace_id)),
        )
        .execute(&mut conn)
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
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

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
            .get_result(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(entity.into())
    }

    async fn revoke_extra_role(
        &self,
        workspace_id: Uuid,
        extra_role_id: Uuid,
    ) -> Result<u64, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let affected =
            diesel::sql_query("DELETE FROM member_extra_roles WHERE id = $1 AND workspace_id = $2")
                .bind::<diesel::sql_types::Uuid, _>(extra_role_id)
                .bind::<diesel::sql_types::Uuid, _>(workspace_id)
                .execute(&mut conn)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(affected as u64)
    }

    async fn get_workspace_roles(
        &self,
        workspace_id: Uuid,
    ) -> Result<Vec<crate::models::WorkspaceRole>, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        #[derive(Debug, diesel::QueryableByName)]
        struct Entity {
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Uuid>)]
            workspace_id: Option<Uuid>,
            #[diesel(sql_type = diesel::sql_types::Text)]
            name: String,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
            description: Option<String>,
            #[diesel(sql_type = diesel::sql_types::Text)]
            color: String,
            #[diesel(sql_type = diesel::sql_types::Bool)]
            is_system: bool,
            #[diesel(sql_type = diesel::sql_types::Timestamptz)]
            created_at: chrono::DateTime<chrono::Utc>,
            #[diesel(sql_type = diesel::sql_types::Timestamptz)]
            updated_at: chrono::DateTime<chrono::Utc>,
            #[diesel(sql_type = diesel::sql_types::Text)]
            scope_type: String,
        }

        let results: Vec<Entity> = diesel::sql_query(
            "SELECT * FROM workspace_roles WHERE workspace_id = $1 OR is_system = true ORDER BY is_system DESC, created_at ASC"
        )
        .bind::<diesel::sql_types::Uuid, _>(workspace_id)
        .load(&mut conn)
        .await
        .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(results
            .into_iter()
            .map(|e| crate::models::WorkspaceRole {
                id: e.id,
                workspace_id: e.workspace_id,
                name: e.name,
                description: e.description,
                color: e.color,
                is_system: e.is_system,
                created_at: e.created_at,
                updated_at: e.updated_at,
                scope_type: e.scope_type.parse().unwrap_or_default(),
            })
            .collect())
    }

    async fn create_workspace_role(
        &self,
        workspace_id: Uuid,
        name: String,
        description: Option<String>,
        color: String,
        scope_type: String,
    ) -> Result<crate::models::WorkspaceRole, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        #[derive(Debug, diesel::QueryableByName)]
        struct Entity {
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Uuid>)]
            workspace_id: Option<Uuid>,
            #[diesel(sql_type = diesel::sql_types::Text)]
            name: String,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
            description: Option<String>,
            #[diesel(sql_type = diesel::sql_types::Text)]
            color: String,
            #[diesel(sql_type = diesel::sql_types::Bool)]
            is_system: bool,
            #[diesel(sql_type = diesel::sql_types::Timestamptz)]
            created_at: chrono::DateTime<chrono::Utc>,
            #[diesel(sql_type = diesel::sql_types::Timestamptz)]
            updated_at: chrono::DateTime<chrono::Utc>,
            #[diesel(sql_type = diesel::sql_types::Text)]
            scope_type: String,
        }

        let entity: Entity = diesel::sql_query(
            "INSERT INTO workspace_roles (workspace_id, name, description, color, is_system, scope_type) VALUES ($1, $2, $3, $4, false, $5) RETURNING *"
        )
        .bind::<diesel::sql_types::Uuid, _>(workspace_id)
        .bind::<diesel::sql_types::Text, _>(name)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(description)
        .bind::<diesel::sql_types::Text, _>(color)
        .bind::<diesel::sql_types::Text, _>(scope_type)
        .get_result(&mut conn)
        .await
        .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(crate::models::WorkspaceRole {
            id: entity.id,
            workspace_id: entity.workspace_id,
            name: entity.name,
            description: entity.description,
            color: entity.color,
            is_system: entity.is_system,
            created_at: entity.created_at,
            updated_at: entity.updated_at,
            scope_type: entity.scope_type.parse().unwrap_or_default(),
        })
    }

    async fn update_workspace_role(
        &self,
        workspace_id: Uuid,
        role_id: Uuid,
        name: Option<String>,
        description: Option<String>,
        color: Option<String>,
    ) -> Result<Option<crate::models::WorkspaceRole>, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        #[derive(Debug, diesel::QueryableByName)]
        struct Entity {
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Uuid>)]
            workspace_id: Option<Uuid>,
            #[diesel(sql_type = diesel::sql_types::Text)]
            name: String,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
            description: Option<String>,
            #[diesel(sql_type = diesel::sql_types::Text)]
            color: String,
            #[diesel(sql_type = diesel::sql_types::Bool)]
            is_system: bool,
            #[diesel(sql_type = diesel::sql_types::Timestamptz)]
            created_at: chrono::DateTime<chrono::Utc>,
            #[diesel(sql_type = diesel::sql_types::Timestamptz)]
            updated_at: chrono::DateTime<chrono::Utc>,
            #[diesel(sql_type = diesel::sql_types::Text)]
            scope_type: String,
        }

        let result: Option<Entity> = diesel::sql_query(
            "UPDATE workspace_roles SET name = COALESCE($1, name), description = COALESCE($2, description), color = COALESCE($3, color), updated_at = NOW() WHERE id = $4 AND workspace_id = $5 AND is_system = false RETURNING *"
        )
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(name)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(description)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(color)
        .bind::<diesel::sql_types::Uuid, _>(role_id)
        .bind::<diesel::sql_types::Uuid, _>(workspace_id)
        .get_result(&mut conn)
        .await
        .optional()
        .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(result.map(|entity| crate::models::WorkspaceRole {
            id: entity.id,
            workspace_id: entity.workspace_id,
            name: entity.name,
            description: entity.description,
            color: entity.color,
            is_system: entity.is_system,
            created_at: entity.created_at,
            updated_at: entity.updated_at,
            scope_type: entity.scope_type.parse().unwrap_or_default(),
        }))
    }

    async fn delete_workspace_role(
        &self,
        workspace_id: Uuid,
        role_id: Uuid,
    ) -> Result<u64, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let affected = diesel::sql_query(
            "DELETE FROM workspace_roles WHERE id = $1 AND workspace_id = $2 AND is_system = false",
        )
        .bind::<diesel::sql_types::Uuid, _>(role_id)
        .bind::<diesel::sql_types::Uuid, _>(workspace_id)
        .execute(&mut conn)
        .await
        .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(affected as u64)
    }

    async fn get_role_permissions_by_role_id(
        &self,
        workspace_id: Uuid,
        role_id: Uuid,
    ) -> Result<Vec<String>, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        #[derive(Debug, diesel::QueryableByName)]
        struct StringEntity {
            #[diesel(sql_type = diesel::sql_types::Text)]
            permission_name: String,
        }

        let results: Vec<StringEntity> = diesel::sql_query(
            "SELECT permission_name FROM workspace_role_permissions WHERE role_id = $1 AND role_id IN (SELECT id FROM workspace_roles WHERE workspace_id = $2 OR is_system = true)"
        )
        .bind::<diesel::sql_types::Uuid, _>(role_id)
        .bind::<diesel::sql_types::Uuid, _>(workspace_id)
        .load(&mut conn)
        .await
        .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(results.into_iter().map(|e| e.permission_name).collect())
    }

    async fn role_exists_and_is_not_system(
        &self,
        workspace_id: Uuid,
        role_id: Uuid,
    ) -> Result<bool, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        #[derive(Debug, diesel::QueryableByName)]
        struct Exists {
            #[diesel(sql_type = diesel::sql_types::Bool)]
            exists: bool,
        }

        let result: Exists = diesel::sql_query(
            "SELECT EXISTS(SELECT 1 FROM workspace_roles WHERE id = $1 AND workspace_id = $2 AND is_system = false) as exists"
        )
        .bind::<diesel::sql_types::Uuid, _>(role_id)
        .bind::<diesel::sql_types::Uuid, _>(workspace_id)
        .get_result(&mut conn)
        .await
        .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(result.exists)
    }

    async fn set_role_permissions(
        &self,
        role_id: Uuid,
        permissions: Vec<String>,
    ) -> Result<(), AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        conn.transaction(|mut conn| {
            Box::pin(async move {
                diesel::sql_query("DELETE FROM workspace_role_permissions WHERE role_id = $1")
                    .bind::<diesel::sql_types::Uuid, _>(role_id)
                    .execute(&mut conn)
                    .await?;

                for perm in permissions {
                    diesel::sql_query(
                        "INSERT INTO workspace_role_permissions (role_id, permission_name) VALUES ($1, $2)",
                    )
                    .bind::<diesel::sql_types::Uuid, _>(role_id)
                    .bind::<diesel::sql_types::Text, _>(perm)
                    .execute(&mut conn)
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
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let permissions = workspace_role_permissions::table
            .filter(workspace_role_permissions::role_id.eq_any(role_ids))
            .select(workspace_role_permissions::permission_name)
            .load::<String>(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(permissions)
    }

    #[allow(clippy::too_many_arguments)]
    async fn update_workspace(
        &self,
        id: Uuid,
        name: Option<String>,
        date_format: Option<String>,
        time_format: Option<String>,
        timezone: Option<String>,
        theme_primary_color: Option<String>,
        default_language: Option<String>,
        holiday_overnight_strategy: Option<String>,
        default_holiday_ot_multiplier: Option<bigdecimal::BigDecimal>,
        logo_url: Option<String>,
        tagline: Option<String>,
    ) -> Result<Workspace, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

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
            holiday_overnight_strategy: Option<String>,
            default_holiday_ot_multiplier: Option<bigdecimal::BigDecimal>,
            logo_url: Option<Option<String>>,
            tagline: Option<Option<String>>,
        }

        let has_changes = name.is_some()
            || date_format.is_some()
            || time_format.is_some()
            || timezone.is_some()
            || theme_primary_color.is_some()
            || default_language.is_some()
            || holiday_overnight_strategy.is_some()
            || default_holiday_ot_multiplier.is_some()
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
            holiday_overnight_strategy,
            default_holiday_ot_multiplier,
            logo_url: logo_url.map(Some),
            tagline: tagline.map(Some),
        };

        let result: WorkspaceEntity = diesel::update(workspaces::table.find(id))
            .set(&update)
            .get_result(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(result.into())
    }

    async fn is_any_role_global(&self, role_ids: &[Uuid]) -> Result<bool, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let query = workspace_roles::table
            .filter(workspace_roles::id.eq_any(role_ids))
            .filter(workspace_roles::scope_type.eq("global"));

        let is_global: bool = diesel::select(diesel::dsl::exists(query))
            .get_result(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(is_global)
    }

    async fn update_workspace_byod_url(
        &self,
        workspace_id: Uuid,
        encrypted_url: Option<String>,
    ) -> Result<(), AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        diesel::update(workspaces::table.find(workspace_id))
            .set(workspaces::custom_db_url_encrypted.eq(encrypted_url))
            .execute(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(())
    }
}
