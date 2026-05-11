use crate::models::{MemberExtraRole, Workspace};
use crate::repositories::WorkspaceRepository;
use uuid::Uuid;

use crate::repositories::user_repository::UserRepository;
use std::str::FromStr;
use std::sync::Arc;

pub struct WorkspaceService {
    workspace_repo: Arc<dyn WorkspaceRepository>,
    user_repo: Arc<dyn UserRepository>,
}

impl WorkspaceService {
    pub fn new(
        workspace_repo: Arc<dyn WorkspaceRepository>,
        user_repo: Arc<dyn UserRepository>,
    ) -> Self {
        Self {
            workspace_repo,
            user_repo,
        }
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
        // 1. Check if user exists
        let user_exists = self
            .user_repo
            .get_user_by_id(owner_id)
            .await
            .map_err(|e| e.to_string())?
            .is_some();

        if !user_exists {
            return Err(format!("User with ID {owner_id} does not exist"));
        }

        // 2. Create workspace with owner using repository (which handles transaction)
        self.workspace_repo
            .create_workspace_with_owner(name, owner_id)
            .await
            .map_err(|e| e.to_string())
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn get_member_roles(
        repo: &dyn WorkspaceRepository,
        workspace_id: Uuid,
        user_id: Uuid,
    ) -> Result<(Uuid, Vec<MemberExtraRole>), String> {
        let primary_role_id = repo
            .find_primary_role(workspace_id, user_id)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "NOT_FOUND".to_string())?;

        let extra_roles = repo
            .find_extra_roles(workspace_id, user_id)
            .await
            .map_err(|e| e.to_string())?;

        Ok((primary_role_id, extra_roles))
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn update_workspace(&self, args: UpdateWorkspaceArgs) -> Result<Workspace, String> {
        let multiplier = args.default_holiday_ot_multiplier.map(|m| {
            bigdecimal::BigDecimal::from_str(&m.to_string())
                .unwrap_or_else(|_| bigdecimal::BigDecimal::from(2))
        });

        if args.name.is_none()
            && args.date_format.is_none()
            && args.time_format.is_none()
            && args.timezone.is_none()
            && args.theme_primary_color.is_none()
            && args.default_language.is_none()
            && args.logo_url.is_none()
            && args.tagline.is_none()
            && args.holiday_overnight_strategy.is_none()
            && multiplier.is_none()
        {
            return Err("No fields to update".to_string());
        }

        self.workspace_repo
            .update_workspace(
                args.ws_id,
                args.name,
                args.date_format,
                args.time_format,
                args.timezone,
                args.theme_primary_color,
                args.default_language,
                args.holiday_overnight_strategy,
                multiplier,
                args.logo_url,
                args.tagline,
            )
            .await
            .map_err(|e| e.to_string())
    }
}
