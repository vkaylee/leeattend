use uuid::Uuid;
use crate::models::{Workspace, MemberExtraRole};
use crate::repositories::WorkspaceRepository;
use sqlx::PgPool;

pub struct WorkspaceService;

impl WorkspaceService {
    // Note: create_workspace still needs PgPool for transaction across multiple repo-like steps 
    // or we need to add transaction support to repo trait. 
    // For now, let's keep it simple or implement the TX inside repo if it's one atomic operation.
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

    pub async fn get_member_roles(repo: &dyn WorkspaceRepository, workspace_id: Uuid, user_id: Uuid) -> Result<(String, Vec<MemberExtraRole>), String> {
        let primary_role = repo.find_primary_role(workspace_id, user_id).await?
            .ok_or("NOT_FOUND".to_string())?;

        let extra_roles = repo.find_extra_roles(workspace_id, user_id).await?;

        Ok((primary_role, extra_roles))
    }
}