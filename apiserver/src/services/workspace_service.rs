use sqlx::PgPool;
use uuid::Uuid;
use crate::models::Workspace;
use crate::models::MemberExtraRole;

pub struct WorkspaceService;

impl WorkspaceService {
    pub async fn create_workspace(db: &PgPool, name: &str, owner_id: Uuid) -> Result<Workspace, String> {
        let mut tx = db.begin().await.map_err(|e| e.to_string())?;

        let workspace: Workspace = sqlx::query_as::<_, Workspace>(
            "INSERT INTO workspaces (name) VALUES ($1) RETURNING *",
        )
        .bind(name)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

        sqlx::query(
            "INSERT INTO workspace_members (workspace_id, user_id, role) VALUES ($1, $2, $3)",
        )
        .bind(workspace.id)
        .bind(owner_id)
        .bind("owner")
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

        tx.commit().await.map_err(|e| e.to_string())?;

        Ok(workspace)
    }

    pub async fn get_member_roles(db: &PgPool, workspace_id: Uuid, user_id: Uuid) -> Result<(String, Vec<MemberExtraRole>), String> {
        let primary_role: String = sqlx::query_scalar(
            "SELECT role FROM workspace_members WHERE workspace_id = $1 AND user_id = $2",
        )
        .bind(workspace_id)
        .bind(user_id)
        .fetch_optional(db)
        .await
        .map_err(|e| e.to_string())?
        .ok_or("NOT_FOUND".to_string())?;

        let extra_roles = sqlx::query_as::<_, MemberExtraRole>(
            "SELECT * FROM member_extra_roles WHERE workspace_id = $1 AND user_id = $2",
        )
        .bind(workspace_id)
        .bind(user_id)
        .fetch_all(db)
        .await
        .map_err(|e| e.to_string())?;

        Ok((primary_role, extra_roles))
    }
}
