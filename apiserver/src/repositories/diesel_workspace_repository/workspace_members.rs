use super::DieselWorkspaceRepository;
use crate::errors::AppError;
use crate::models::diesel::core::WorkspaceMemberEntity;
use crate::schema::{users, workspace_members, workspace_role_permissions, workspace_roles};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

impl DieselWorkspaceRepository {
    pub(super) async fn add_member_impl(
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
            .map_err(AppError::from)?;

        Ok(())
    }

    pub(super) async fn update_member_role_impl(
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
        .map_err(AppError::from)?;

        Ok(())
    }

    pub(super) async fn remove_member_impl(
        &self,
        workspace_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), AppError> {
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
        .map_err(AppError::from)?;

        Ok(())
    }

    pub(super) async fn find_primary_role_impl(
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
            .map_err(AppError::from)?;

        Ok(role_id)
    }

    pub(super) async fn get_workspace_members_impl(
        &self,
        workspace_id: Uuid,
        page: i64,
        limit: i64,
    ) -> Result<(Vec<crate::models::WorkspaceMemberDetail>, i64), AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .api
                .get()
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let total: i64 = workspace_members::table
            .inner_join(users::table.on(workspace_members::user_id.eq(users::id)))
            .filter(workspace_members::workspace_id.eq(workspace_id))
            .count()
            .get_result(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db count: {e}")))?;

        let results: Vec<(
            WorkspaceMemberEntity,
            crate::models::diesel::core::UserEntity,
        )> = workspace_members::table
            .inner_join(users::table.on(workspace_members::user_id.eq(users::id)))
            .filter(workspace_members::workspace_id.eq(workspace_id))
            .limit(limit)
            .offset((page - 1) * limit)
            .load::<(
                WorkspaceMemberEntity,
                crate::models::diesel::core::UserEntity,
            )>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let details = results
            .into_iter()
            .map(|(wm, u)| {
                crate::models::WorkspaceMemberDetail::new(
                    wm.workspace_id,
                    wm.user_id,
                    wm.role_id,
                    u.email,
                    u.full_name,
                    u.avatar_url,
                )
            })
            .collect();

        Ok((details, total))
    }

    pub(super) async fn get_admins_with_permissions_impl(
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
}
