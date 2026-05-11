use crate::models::{MemberExtraRole, Workspace};
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn get_workspace_by_id(
    pool: &PgPool,
    id: Uuid,
) -> Result<Option<Workspace>, sqlx::Error> {
    sqlx::query_as::<_, Workspace>("SELECT * FROM workspaces WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
}

#[async_trait]
pub trait WorkspaceRepository: super::PoolBacked + Send + Sync {
    async fn create_workspace_with_owner(
        &self,
        name: &str,
        owner_id: Uuid,
    ) -> Result<Workspace, sqlx::Error>;
    async fn get_workspace_by_id(&self, id: Uuid) -> Result<Option<Workspace>, sqlx::Error>;
    async fn add_member(
        &self,
        workspace_id: Uuid,
        user_id: Uuid,
        role_id: Uuid,
    ) -> Result<(), sqlx::Error>;
    async fn update_member_role(
        &self,
        workspace_id: Uuid,
        user_id: Uuid,
        role_id: Uuid,
    ) -> Result<(), sqlx::Error>;
    async fn remove_member(&self, workspace_id: Uuid, user_id: Uuid) -> Result<(), sqlx::Error>;
    async fn find_primary_role(
        &self,
        workspace_id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<Uuid>, sqlx::Error>;
    async fn find_extra_roles(
        &self,
        workspace_id: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<MemberExtraRole>, sqlx::Error>;
    async fn find_extra_permissions(
        &self,
        workspace_id: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<String>, sqlx::Error>;
    /// Get all permission names for a set of role IDs.
    async fn get_permissions_for_roles(
        &self,
        role_ids: &[Uuid],
    ) -> Result<Vec<String>, sqlx::Error>;
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
    ) -> Result<Workspace, sqlx::Error>;
    /// Check if any role in the set has global scope.
    async fn is_any_role_global(&self, role_ids: &[Uuid]) -> Result<bool, sqlx::Error>;
}

pub struct SqlxWorkspaceRepository {
    pub pool: PgPool,
}

impl SqlxWorkspaceRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl WorkspaceRepository for SqlxWorkspaceRepository {
    async fn create_workspace_with_owner(
        &self,
        name: &str,
        owner_id: Uuid,
    ) -> Result<Workspace, sqlx::Error> {
        let mut tx = self.pool.begin().await?;

        let ws = sqlx::query_as::<_, Workspace>(
            "INSERT INTO workspaces (id, name, created_at) 
             VALUES ($1, $2, NOW())
             RETURNING id, name, date_format, time_format, timezone, theme_primary_color, default_language, holiday_overnight_strategy, default_holiday_ot_multiplier, logo_url, tagline, created_at",
        )
        .bind(crate::utils::id::new_id())
        .bind(name)
        .fetch_one(&mut *tx)
        .await?;

        let owner_role_id = Uuid::parse_str("a0000000-0000-0000-0000-000000000001")
            .map_err(|e| sqlx::Error::Configuration(Box::new(e)))?;

        sqlx::query(
            "INSERT INTO workspace_members (workspace_id, user_id, role_id) 
             VALUES ($1, $2, $3)",
        )
        .bind(ws.id)
        .bind(owner_id)
        .bind(owner_role_id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(ws)
    }

    async fn get_workspace_by_id(&self, id: Uuid) -> Result<Option<Workspace>, sqlx::Error> {
        sqlx::query_as::<_, Workspace>("SELECT * FROM workspaces WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
    }

    async fn add_member(
        &self,
        workspace_id: Uuid,
        user_id: Uuid,
        role_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO workspace_members (workspace_id, user_id, role_id) 
             VALUES ($1, $2, $3)",
        )
        .bind(workspace_id)
        .bind(user_id)
        .bind(role_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn update_member_role(
        &self,
        workspace_id: Uuid,
        user_id: Uuid,
        role_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE workspace_members SET role_id = $1 WHERE workspace_id = $2 AND user_id = $3",
        )
        .bind(role_id)
        .bind(workspace_id)
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn remove_member(&self, workspace_id: Uuid, user_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM workspace_members WHERE workspace_id = $1 AND user_id = $2")
            .bind(workspace_id)
            .bind(user_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn find_primary_role(
        &self,
        workspace_id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<Uuid>, sqlx::Error> {
        let row: Option<(Uuid,)> = sqlx::query_as(
            "SELECT role_id FROM workspace_members WHERE workspace_id = $1 AND user_id = $2",
        )
        .bind(workspace_id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.0))
    }

    async fn find_extra_roles(
        &self,
        workspace_id: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<MemberExtraRole>, sqlx::Error> {
        sqlx::query_as::<_, MemberExtraRole>(
            "SELECT * FROM member_extra_roles WHERE workspace_id = $1 AND user_id = $2",
        )
        .bind(workspace_id)
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
    }

    async fn find_extra_permissions(
        &self,
        workspace_id: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<String>, sqlx::Error> {
        sqlx::query_scalar(
            "SELECT permission FROM workspace_member_extra_permissions \
             WHERE workspace_id = $1 AND user_id = $2",
        )
        .bind(workspace_id)
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
    }

    async fn get_permissions_for_roles(
        &self,
        role_ids: &[Uuid],
    ) -> Result<Vec<String>, sqlx::Error> {
        sqlx::query_scalar(
            "SELECT permission_name FROM workspace_role_permissions WHERE role_id = ANY($1)",
        )
        .bind(role_ids)
        .fetch_all(&self.pool)
        .await
    }

    #[allow(clippy::too_many_arguments)]
    async fn update_workspace(
        &self,
        id: Uuid,
        _name: Option<String>,
        _date_format: Option<String>,
        _time_format: Option<String>,
        _timezone: Option<String>,
        _theme_primary_color: Option<String>,
        _default_language: Option<String>,
        _holiday_overnight_strategy: Option<String>,
        _default_holiday_ot_multiplier: Option<bigdecimal::BigDecimal>,
        _logo_url: Option<String>,
        _tagline: Option<String>,
    ) -> Result<Workspace, sqlx::Error> {
        let ws = sqlx::query_as::<_, Workspace>("SELECT * FROM workspaces WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await?;
        Ok(ws)
    }

    async fn is_any_role_global(&self, role_ids: &[Uuid]) -> Result<bool, sqlx::Error> {
        let is_global: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM workspace_roles WHERE id = ANY($1) AND scope_type = 'global')"
        )
        .bind(role_ids)
        .fetch_one(&self.pool)
        .await?;

        Ok(is_global)
    }
}

impl super::PoolBacked for SqlxWorkspaceRepository {
    fn from_pool(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
    fn pool(&self) -> &sqlx::PgPool {
        &self.pool
    }
}
