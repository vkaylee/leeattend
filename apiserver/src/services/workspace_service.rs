use crate::models::{MemberExtraRole, Workspace};
use crate::repositories::WorkspaceRepository;
use sqlx::PgPool;
use uuid::Uuid;

pub struct WorkspaceService {
    pool: PgPool,
}

impl WorkspaceService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

pub struct UpdateWorkspaceArgs {
    pub ws_id: Uuid,
    pub name: Option<String>,
    pub date_format: Option<String>,
    pub time_format: Option<String>,
    pub timezone: Option<String>,
    pub theme_primary_color: Option<String>,
    pub default_language: Option<String>,
    pub logo_url: Option<String>,
    pub tagline: Option<String>,
    pub holiday_overnight_strategy: Option<String>,
    pub default_holiday_ot_multiplier: Option<rust_decimal::Decimal>,
}

impl WorkspaceService {
    #[tracing::instrument(skip_all, err)]
    pub async fn create_workspace(&self, name: &str, owner_id: Uuid) -> Result<Workspace, String> {
        let db = &self.pool;
        // Kiểm tra user có tồn tại không trước khi bắt đầu transaction
        let user_exists: bool =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE id = $1)")
                .bind(owner_id)
                .fetch_one(db)
                .await
                .map_err(|e| e.to_string())?;

        if !user_exists {
            return Err(format!("User with ID {owner_id} does not exist"));
        }

        let mut tx = db.begin().await.map_err(|e| e.to_string())?;

        let workspace: Workspace =
            sqlx::query_as::<_, Workspace>("INSERT INTO workspaces (name) VALUES ($1) RETURNING *")
                .bind(name)
                .fetch_one(&mut *tx)
                .await
                .map_err(|e| e.to_string())?;

        let owner_role_id =
            Uuid::parse_str("a0000000-0000-0000-0000-000000000001").map_err(|e| e.to_string())?;

        sqlx::query(
            "INSERT INTO workspace_members (workspace_id, user_id, role_id) VALUES ($1, $2, $3)",
        )
        .bind(workspace.id)
        .bind(owner_id)
        .bind(owner_role_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

        tx.commit().await.map_err(|e| e.to_string())?;

        Ok(workspace)
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn get_member_roles(
        repo: &dyn WorkspaceRepository,
        workspace_id: Uuid,
        user_id: Uuid,
    ) -> Result<(Uuid, Vec<MemberExtraRole>), String> {
        let primary_role_id = repo
            .find_primary_role(workspace_id, user_id)
            .await?
            .ok_or("NOT_FOUND".to_string())?;

        let extra_roles = repo.find_extra_roles(workspace_id, user_id).await?;

        Ok((primary_role_id, extra_roles))
    }

    #[tracing::instrument(skip_all, err)]
    #[allow(clippy::too_many_lines)]
    pub async fn update_workspace(&self, args: UpdateWorkspaceArgs) -> Result<Workspace, String> {
        let mut query = String::from("UPDATE workspaces SET ");
        let mut has_fields = false;
        let mut arg_idx = 1;

        if args.name.is_some() {
            query.push_str(&format!("name = ${arg_idx}"));
            arg_idx += 1;
            has_fields = true;
        }
        if args.date_format.is_some() {
            if has_fields {
                query.push_str(", ");
            }
            query.push_str(&format!("date_format = ${arg_idx}"));
            arg_idx += 1;
            has_fields = true;
        }
        if args.time_format.is_some() {
            if has_fields {
                query.push_str(", ");
            }
            query.push_str(&format!("time_format = ${arg_idx}"));
            arg_idx += 1;
            has_fields = true;
        }
        if args.timezone.is_some() {
            if has_fields {
                query.push_str(", ");
            }
            query.push_str(&format!("timezone = ${arg_idx}"));
            arg_idx += 1;
            has_fields = true;
        }
        if args.theme_primary_color.is_some() {
            if has_fields {
                query.push_str(", ");
            }
            query.push_str(&format!("theme_primary_color = ${arg_idx}"));
            arg_idx += 1;
            has_fields = true;
        }
        if args.default_language.is_some() {
            if has_fields {
                query.push_str(", ");
            }
            query.push_str(&format!("default_language = ${arg_idx}"));
            arg_idx += 1;
            has_fields = true;
        }
        if args.logo_url.is_some() {
            if has_fields {
                query.push_str(", ");
            }
            query.push_str(&format!("logo_url = ${arg_idx}"));
            arg_idx += 1;
            has_fields = true;
        }
        if args.tagline.is_some() {
            if has_fields {
                query.push_str(", ");
            }
            query.push_str(&format!("tagline = ${arg_idx}"));
            arg_idx += 1;
            has_fields = true;
        }

        if args.holiday_overnight_strategy.is_some() {
            if has_fields {
                query.push_str(", ");
            }
            query.push_str(&format!("holiday_overnight_strategy = ${arg_idx}"));
            arg_idx += 1;
            has_fields = true;
        }
        if args.default_holiday_ot_multiplier.is_some() {
            if has_fields {
                query.push_str(", ");
            }
            query.push_str(&format!("default_holiday_ot_multiplier = ${arg_idx}"));
            arg_idx += 1;
            has_fields = true;
        }

        if !has_fields {
            return Err("No fields to update".to_string());
        }

        query.push_str(&format!(" WHERE id = ${arg_idx} RETURNING *"));

        let mut q = sqlx::query_as::<_, Workspace>(&query);

        if let Some(n) = args.name {
            q = q.bind(n);
        }
        if let Some(d) = args.date_format {
            q = q.bind(d);
        }
        if let Some(t) = args.time_format {
            q = q.bind(t);
        }
        if let Some(tz) = args.timezone {
            q = q.bind(tz);
        }
        if let Some(tpc) = args.theme_primary_color {
            q = q.bind(tpc);
        }
        if let Some(dl) = args.default_language {
            q = q.bind(dl);
        }
        if let Some(lu) = args.logo_url {
            q = q.bind(lu);
        }
        if let Some(tl) = args.tagline {
            q = q.bind(tl);
        }

        if let Some(hos) = args.holiday_overnight_strategy {
            q = q.bind(hos);
        }
        if let Some(dhom) = args.default_holiday_ot_multiplier {
            q = q.bind(dhom);
        }
        q = q.bind(args.ws_id);

        let ws = q.fetch_one(&self.pool).await.map_err(|e| e.to_string())?;
        Ok(ws)
    }
}
