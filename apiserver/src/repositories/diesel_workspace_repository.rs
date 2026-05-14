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
        name: &str,
        owner_id: Uuid,
    ) -> Result<Workspace, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let workspace_id = crate::utils::id::new_id();
        let entity = WorkspaceEntity {
            id: workspace_id,
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
        };

        let owner_role_id = Uuid::parse_str("a0000000-0000-0000-0000-000000000001")
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let member_entity = WorkspaceMemberEntity {
            workspace_id,
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
}
