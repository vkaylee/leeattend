use crate::models::{MemberExtraRole, Workspace};
use crate::repositories::WorkspaceRepository;
use sqlx::PgPool;
use uuid::Uuid;

pub struct WorkspaceService;

impl WorkspaceService {
    // Note: create_workspace still needs PgPool for transaction across multiple repo-like steps
    // or we need to add transaction support to repo trait.
    // For now, let's keep it simple or implement the TX inside repo if it's one atomic operation.
    pub async fn create_workspace(
        db: &PgPool,
        name: &str,
        owner_id: Uuid,
    ) -> Result<Workspace, String> {
        // Kiểm tra user có tồn tại không trước khi bắt đầu transaction
        let user_exists: bool =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE id = $1)")
                .bind(owner_id)
                .fetch_one(db)
                .await
                .map_err(|e| e.to_string())?;

        if !user_exists {
            return Err(format!("User with ID {} does not exist", owner_id));
        }

        let mut tx = db.begin().await.map_err(|e| e.to_string())?;

        let workspace: Workspace =
            sqlx::query_as::<_, Workspace>("INSERT INTO workspaces (name) VALUES ($1) RETURNING *")
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

    pub async fn get_member_roles(
        repo: &dyn WorkspaceRepository,
        workspace_id: Uuid,
        user_id: Uuid,
    ) -> Result<(String, Vec<MemberExtraRole>), String> {
        let primary_role = repo
            .find_primary_role(workspace_id, user_id)
            .await?
            .ok_or("NOT_FOUND".to_string())?;

        let extra_roles = repo.find_extra_roles(workspace_id, user_id).await?;

        Ok((primary_role, extra_roles))
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn update_workspace(
        db: &PgPool,
        ws_id: Uuid,
        name: Option<String>,
        work_unit_minutes: Option<i32>,
        rounding_policy: Option<String>,
        date_format: Option<String>,
        time_format: Option<String>,
        timezone: Option<String>,
        theme_primary_color: Option<String>,
        default_language: Option<String>,
    ) -> Result<Workspace, String> {
        let mut query = String::from("UPDATE workspaces SET ");
        let mut has_fields = false;
        let mut arg_idx = 1;

        if name.is_some() {
            query.push_str(&format!("name = ${}", arg_idx));
            arg_idx += 1;
            has_fields = true;
        }
        if work_unit_minutes.is_some() {
            if has_fields { query.push_str(", "); }
            query.push_str(&format!("work_unit_minutes = ${}", arg_idx));
            arg_idx += 1;
            has_fields = true;
        }
        if rounding_policy.is_some() {
            if has_fields { query.push_str(", "); }
            query.push_str(&format!("rounding_policy = ${}", arg_idx));
            arg_idx += 1;
            has_fields = true;
        }
        if date_format.is_some() {
            if has_fields { query.push_str(", "); }
            query.push_str(&format!("date_format = ${}", arg_idx));
            arg_idx += 1;
            has_fields = true;
        }
        if time_format.is_some() {
            if has_fields { query.push_str(", "); }
            query.push_str(&format!("time_format = ${}", arg_idx));
            arg_idx += 1;
            has_fields = true;
        }
        if timezone.is_some() {
            if has_fields { query.push_str(", "); }
            query.push_str(&format!("timezone = ${}", arg_idx));
            arg_idx += 1;
            has_fields = true;
        }
        if theme_primary_color.is_some() {
            if has_fields { query.push_str(", "); }
            query.push_str(&format!("theme_primary_color = ${}", arg_idx));
            arg_idx += 1;
            has_fields = true;
        }
        if default_language.is_some() {
            if has_fields { query.push_str(", "); }
            query.push_str(&format!("default_language = ${}", arg_idx));
            arg_idx += 1;
            has_fields = true;
        }

        if !has_fields {
            return Err("No fields to update".to_string());
        }

        query.push_str(&format!(" WHERE id = ${} RETURNING *", arg_idx));

        let mut q = sqlx::query_as::<_, Workspace>(&query);

        if let Some(n) = name {
            q = q.bind(n);
        }
        if let Some(w) = work_unit_minutes {
            q = q.bind(w);
        }
        if let Some(r) = rounding_policy {
            q = q.bind(r);
        }
        if let Some(d) = date_format {
            q = q.bind(d);
        }
        if let Some(t) = time_format {
            q = q.bind(t);
        }
        if let Some(tz) = timezone {
            q = q.bind(tz);
        }
        if let Some(tpc) = theme_primary_color {
            q = q.bind(tpc);
        }
        if let Some(dl) = default_language {
            q = q.bind(dl);
        }
        q = q.bind(ws_id);

        let ws = q.fetch_one(db).await.map_err(|e| e.to_string())?;
        Ok(ws)
    }
}
