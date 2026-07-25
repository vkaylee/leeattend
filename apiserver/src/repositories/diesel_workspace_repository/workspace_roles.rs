use super::DieselWorkspaceRepository;
use crate::errors::AppError;
use crate::models::diesel::core::{MemberExtraPermissionEntity, MemberExtraRoleEntity, WorkspaceRoleEntity};
use crate::schema::{
    member_extra_roles, workspace_member_extra_permissions, workspace_role_permissions,
    workspace_roles,
};
use chrono::Utc;
use diesel::prelude::*;
use diesel_async::{AsyncConnection, RunQueryDsl};
use uuid::Uuid;

impl DieselWorkspaceRepository {
    pub(super) async fn get_all_extra_roles_impl(
        &self,
        workspace_id: Uuid,
    ) -> Result<Vec<crate::models::MemberExtraRole>, AppError> {
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
            .map_err(AppError::from)?;

        Ok(entities.into_iter().map(Into::into).collect())
    }

    pub(super) async fn find_extra_roles_impl(
        &self,
        workspace_id: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<crate::models::MemberExtraRole>, AppError> {
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
            .map_err(AppError::from)?;

        Ok(entities.into_iter().map(Into::into).collect())
    }

    pub(super) async fn find_extra_permissions_impl(
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
            .map_err(AppError::from)?;

        Ok(permissions)
    }

    pub(super) async fn get_member_extra_permissions_impl(
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
            .map(|e| {
                crate::models::MemberExtraPermission::new(
                    e.id,
                    e.workspace_id,
                    e.user_id,
                    e.permission,
                    e.assigned_at,
                    e.expires_at,
                    e.assigned_by,
                )
            })
            .collect())
    }

    pub(super) async fn assign_extra_permission_impl(
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
                .map_err(AppError::from)?;

        Ok(crate::models::MemberExtraPermission::new(
            entity.id,
            entity.workspace_id,
            entity.user_id,
            entity.permission,
            entity.assigned_at,
            entity.expires_at,
            entity.assigned_by,
        ))
    }

    pub(super) async fn revoke_extra_permission_impl(
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
        .map_err(AppError::from)?;

        Ok(affected as u64)
    }

    pub(super) async fn assign_extra_role_impl(
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

        let new_role = MemberExtraRoleEntity {
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
            .map_err(AppError::from)?;

        Ok(entity.into())
    }

    pub(super) async fn revoke_extra_role_impl(
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

        let affected = diesel::delete(
            member_extra_roles::dsl::member_extra_roles
                .filter(member_extra_roles::dsl::id.eq(extra_role_id))
                .filter(member_extra_roles::dsl::workspace_id.eq(workspace_id)),
        )
        .execute(&mut *conn)
        .await
        .map_err(AppError::from)?;

        Ok(affected as u64)
    }

    pub(super) async fn get_workspace_roles_impl(
        &self,
        workspace_id: Uuid,
        page: i64,
        limit: i64,
    ) -> Result<(Vec<crate::models::WorkspaceRole>, i64), AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .api
                .get()
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let total: i64 = workspace_roles::table
            .filter(
                workspace_roles::dsl::workspace_id
                    .eq(workspace_id)
                    .or(workspace_roles::dsl::is_system.eq(true)),
            )
            .count()
            .get_result(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db count: {e}")))?;

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
            .limit(limit)
            .offset((page - 1) * limit)
            .load(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok((results.into_iter().map(Into::into).collect(), total))
    }

    pub(super) async fn create_workspace_role_impl(
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
            .map_err(AppError::from)?;

        Ok(entity.into())
    }

    pub(super) async fn update_workspace_role_impl(
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
        .map_err(AppError::from)?;

        Ok(result.map(Into::into))
    }

    pub(super) async fn delete_workspace_role_impl(
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

        let affected = diesel::delete(
            workspace_roles::dsl::workspace_roles
                .filter(workspace_roles::dsl::id.eq(role_id))
                .filter(workspace_roles::dsl::workspace_id.eq(workspace_id))
                .filter(workspace_roles::dsl::is_system.eq(false)),
        )
        .execute(&mut *conn)
        .await
        .map_err(AppError::from)?;

        Ok(affected as u64)
    }

    pub(super) async fn get_role_permissions_by_role_id_impl(
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

    pub(super) async fn role_exists_and_is_not_system_impl(
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

        use diesel::dsl::{exists, select};

        let result: bool = select(exists(
            workspace_roles::dsl::workspace_roles
                .filter(workspace_roles::dsl::id.eq(role_id))
                .filter(workspace_roles::dsl::workspace_id.eq(workspace_id))
                .filter(workspace_roles::dsl::is_system.eq(false)),
        ))
        .get_result(&mut *conn)
        .await
        .map_err(AppError::from)?;

        Ok(result)
    }

    pub(super) async fn set_role_permissions_impl(
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

    pub(super) async fn get_permissions_for_roles_impl(
        &self,
        role_ids: &[Uuid],
    ) -> Result<Vec<String>, AppError> {
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
            .map_err(AppError::from)?;

        Ok(permissions)
    }

    pub(super) async fn get_users_with_permission_impl(
        &self,
        workspace_id: Uuid,
        permission: &str,
    ) -> Result<Vec<Uuid>, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .api
                .get()
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        use crate::schema::{member_extra_roles, workspace_members};

        // 1. Users whose primary role has the permission, or whose primary role is global
        let primary_role_users = workspace_members::table
            .inner_join(
                workspace_roles::table.on(workspace_members::role_id.eq(workspace_roles::id)),
            )
            .left_join(
                workspace_role_permissions::table
                    .on(workspace_roles::id.eq(workspace_role_permissions::role_id)),
            )
            .filter(workspace_members::workspace_id.eq(workspace_id))
            .filter(
                workspace_roles::scope_type
                    .eq("global")
                    .or(workspace_role_permissions::permission_name.eq(permission)),
            )
            .select(workspace_members::user_id)
            .load::<Uuid>(&mut *conn)
            .await
            .map_err(|e| {
                AppError::Internal(format!("db error fetching primary role permissions: {e}"))
            })?;

        // 2. Users whose active extra role has the permission, or whose extra role is global
        let extra_role_users = member_extra_roles::table
            .inner_join(
                workspace_roles::table.on(member_extra_roles::role_id.eq(workspace_roles::id)),
            )
            .left_join(
                workspace_role_permissions::table
                    .on(workspace_roles::id.eq(workspace_role_permissions::role_id)),
            )
            .filter(member_extra_roles::workspace_id.eq(workspace_id))
            .filter(member_extra_roles::expires_at.gt(Utc::now()))
            .filter(
                workspace_roles::scope_type
                    .eq("global")
                    .or(workspace_role_permissions::permission_name.eq(permission)),
            )
            .select(member_extra_roles::user_id)
            .load::<Uuid>(&mut *conn)
            .await
            .map_err(|e| {
                AppError::Internal(format!("db error fetching extra role permissions: {e}"))
            })?;

        // 3. Users who have the permission assigned directly as an extra permission
        let extra_perm_users = workspace_member_extra_permissions::table
            .filter(workspace_member_extra_permissions::workspace_id.eq(workspace_id))
            .filter(workspace_member_extra_permissions::expires_at.gt(Utc::now()))
            .filter(workspace_member_extra_permissions::permission.eq(permission))
            .select(workspace_member_extra_permissions::user_id)
            .load::<Uuid>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db error fetching extra permissions: {e}")))?;

        let mut user_ids = std::collections::HashSet::new();
        user_ids.extend(primary_role_users);
        user_ids.extend(extra_role_users);
        user_ids.extend(extra_perm_users);
        let result: Vec<Uuid> = user_ids.into_iter().collect();
        Ok(result)
    }
}
