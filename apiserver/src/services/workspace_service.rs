use crate::models::{MemberExtraRole, Workspace};
use crate::repositories::WorkspaceRepository;
use uuid::Uuid;

use crate::repositories::user_repository::UserRepository;
use std::str::FromStr;
use std::sync::Arc;

use crate::services::tenant_provisioner::TenantProvisionerTrait;
use crate::supavisor::TenantManager;

pub struct WorkspaceService {
    workspace_repo: Arc<dyn WorkspaceRepository>,
    user_repo: Arc<dyn UserRepository>,
    tenant_provisioner: Arc<dyn TenantProvisionerTrait>,
    tenant_manager: Arc<dyn TenantManager>,
}

impl WorkspaceService {
    pub fn new(
        workspace_repo: Arc<dyn WorkspaceRepository>,
        user_repo: Arc<dyn UserRepository>,
        tenant_provisioner: Arc<dyn TenantProvisionerTrait>,
        tenant_manager: Arc<dyn TenantManager>,
    ) -> Self {
        Self {
            workspace_repo,
            user_repo,
            tenant_provisioner,
            tenant_manager,
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

        // 2. Create workspace with owner using repository
        let ws = self
            .workspace_repo
            .create_workspace_with_owner(name, owner_id)
            .await
            .map_err(|e| e.to_string())?;

        // 3. Provision the physical Tenant Database (or use BYOD)
        let new_db_url = self
            .tenant_provisioner
            .provision_tenant_db(&ws.id, None)
            .await
            .map_err(|e| format!("Failed to provision DB: {e}"))?;

        // 4. Register Tenant with Supavisor connection pooler.
        // The tenant_id must match the format produced by diesel_pool::get_tenant_url(),
        // which constructs tenant_<uuid-without-hyphens>. Using the raw UUID with hyphens
        // causes a mismatch when the API connects via Supavisor.
        if let Some((tenant_id, payload)) = crate::supavisor::build_tenant_payload(&new_db_url) {
            self.tenant_manager
                .register_tenant_dynamic(&tenant_id, &payload)
                .await
                .map_err(|e| format!("Failed to register with Supavisor: {e}"))?;
        }

        Ok(ws)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::AppError;
    use crate::models::{MemberExtraRole, Workspace};
    use async_trait::async_trait;
    use chrono::Utc;
    use rust_decimal::Decimal;
    use uuid::Uuid;

    fn dummy_workspace() -> Workspace {
        use rust_decimal::Decimal;
        Workspace {
            id: Uuid::new_v4(),
            name: "Acme Corp".into(),
            created_at: Utc::now(),
            date_format: "DD/MM/YYYY".into(),
            time_format: "HH:mm".into(),
            timezone: "Asia/Ho_Chi_Minh".into(),
            theme_primary_color: "#000".into(),
            default_language: "vi".into(),
            logo_url: None,
            tagline: None,
            custom_db_url_encrypted: None,
            holiday_overnight_strategy: "split".into(),
            default_holiday_ot_multiplier: Decimal::new(15, 1),
        }
    }

    // ── NopWorkspaceRepo ────────────────────────────────────────────────────

    struct MockWorkspaceRepo {
        workspace: Option<Workspace>,
        primary_role: Option<Uuid>,
    }

    #[async_trait]
    impl WorkspaceRepository for MockWorkspaceRepo {
        async fn create_workspace_with_owner(
            &self,
            name: &str,
            _owner_id: Uuid,
        ) -> Result<Workspace, AppError> {
            let mut ws = dummy_workspace();
            ws.name = name.to_string();
            Ok(ws)
        }
        async fn get_workspace_by_id(&self, _id: Uuid) -> Result<Option<Workspace>, AppError> {
            Ok(self.workspace.clone())
        }
        async fn get_all_workspaces(&self) -> Result<Vec<Workspace>, AppError> {
            Ok(self.workspace.clone().map(|w| vec![w]).unwrap_or_default())
        }
        async fn get_user_workspaces(&self, _u: Uuid) -> Result<Vec<Workspace>, AppError> {
            Ok(vec![])
        }
        async fn clear_workspace_data(&self, _w: Uuid, _g: &str) -> Result<(), AppError> {
            Ok(())
        }
        async fn add_member(&self, _w: Uuid, _u: Uuid, _r: Uuid) -> Result<(), AppError> {
            Ok(())
        }
        async fn update_member_role(&self, _w: Uuid, _u: Uuid, _r: Uuid) -> Result<(), AppError> {
            Ok(())
        }
        async fn remove_member(&self, _w: Uuid, _u: Uuid) -> Result<(), AppError> {
            Ok(())
        }
        async fn find_primary_role(&self, _w: Uuid, _u: Uuid) -> Result<Option<Uuid>, AppError> {
            Ok(self.primary_role)
        }
        async fn get_workspace_members(
            &self,
            _w: Uuid,
        ) -> Result<Vec<crate::models::WorkspaceMemberDetail>, AppError> {
            Ok(vec![])
        }
        async fn get_admins_with_permissions(
            &self,
            _w: Uuid,
            _p: &[&str],
        ) -> Result<Vec<Uuid>, AppError> {
            Ok(vec![])
        }
        async fn get_all_extra_roles(&self, _w: Uuid) -> Result<Vec<MemberExtraRole>, AppError> {
            Ok(vec![])
        }
        async fn find_extra_roles(
            &self,
            _w: Uuid,
            _u: Uuid,
        ) -> Result<Vec<MemberExtraRole>, AppError> {
            Ok(vec![])
        }
        async fn find_extra_permissions(
            &self,
            _w: Uuid,
            _u: Uuid,
        ) -> Result<Vec<String>, AppError> {
            Ok(vec![])
        }
        async fn get_member_extra_permissions(
            &self,
            _w: Uuid,
            _u: Uuid,
        ) -> Result<Vec<crate::models::MemberExtraPermission>, AppError> {
            Ok(vec![])
        }
        async fn assign_extra_permission(
            &self,
            _w: Uuid,
            _u: Uuid,
            _p: String,
            _e: chrono::DateTime<chrono::Utc>,
            _a: Uuid,
        ) -> Result<crate::models::MemberExtraPermission, AppError> {
            unimplemented!()
        }
        async fn revoke_extra_permission(
            &self,
            _w: Uuid,
            _u: Uuid,
            _p: Uuid,
        ) -> Result<u64, AppError> {
            Ok(1)
        }
        async fn assign_extra_role(
            &self,
            _w: Uuid,
            _u: Uuid,
            _r: Uuid,
            _e: chrono::DateTime<chrono::Utc>,
        ) -> Result<MemberExtraRole, AppError> {
            unimplemented!()
        }
        async fn revoke_extra_role(&self, _w: Uuid, _r: Uuid) -> Result<u64, AppError> {
            Ok(1)
        }
        async fn get_workspace_roles(
            &self,
            _w: Uuid,
        ) -> Result<Vec<crate::models::WorkspaceRole>, AppError> {
            Ok(vec![])
        }
        async fn create_workspace_role(
            &self,
            _w: Uuid,
            _n: String,
            _d: Option<String>,
            _c: String,
            _s: String,
        ) -> Result<crate::models::WorkspaceRole, AppError> {
            unimplemented!()
        }
        async fn update_workspace_role(
            &self,
            _w: Uuid,
            _r: Uuid,
            _n: Option<String>,
            _d: Option<String>,
            _c: Option<String>,
        ) -> Result<Option<crate::models::WorkspaceRole>, AppError> {
            unimplemented!()
        }
        async fn delete_workspace_role(&self, _w: Uuid, _r: Uuid) -> Result<u64, AppError> {
            Ok(1)
        }
        async fn get_role_permissions_by_role_id(
            &self,
            _w: Uuid,
            _r: Uuid,
        ) -> Result<Vec<String>, AppError> {
            Ok(vec![])
        }
        async fn role_exists_and_is_not_system(
            &self,
            _w: Uuid,
            _r: Uuid,
        ) -> Result<bool, AppError> {
            Ok(true)
        }
        async fn set_role_permissions(&self, _r: Uuid, _p: Vec<String>) -> Result<(), AppError> {
            Ok(())
        }
        async fn get_permissions_for_roles(&self, _r: &[Uuid]) -> Result<Vec<String>, AppError> {
            Ok(vec![])
        }
        #[allow(clippy::too_many_arguments)]
        async fn update_workspace(
            &self,
            _id: Uuid,
            name: Option<String>,
            _df: Option<String>,
            _tf: Option<String>,
            _tz: Option<String>,
            _tc: Option<String>,
            _dl: Option<String>,
            _hs: Option<String>,
            _hm: Option<bigdecimal::BigDecimal>,
            _lu: Option<String>,
            _tl: Option<String>,
        ) -> Result<Workspace, AppError> {
            let mut ws = dummy_workspace();
            if let Some(n) = name {
                ws.name = n;
            }
            Ok(ws)
        }
        async fn update_workspace_byod_url(
            &self,
            _workspace_id: Uuid,
            _encrypted_url: Option<String>,
        ) -> Result<(), AppError> {
            Ok(())
        }
        async fn is_any_role_global(&self, _r: &[Uuid]) -> Result<bool, AppError> {
            Ok(false)
        }
    }

    struct MockUserRepo {
        user_exists: bool,
    }
    #[async_trait]
    impl crate::repositories::user_repository::UserRepository for MockUserRepo {
        async fn get_user_by_id(&self, id: Uuid) -> Result<Option<crate::models::User>, AppError> {
            if self.user_exists {
                Ok(Some(crate::models::User {
                    id,
                    email: "test@example.com".into(),
                    password_hash: "hash".into(),
                    created_at: Utc::now(),
                    full_name: Some("Test User".into()),
                    avatar_url: None,
                    bio: None,
                    preferred_language: None,
                }))
            } else {
                Ok(None)
            }
        }
        async fn create_user(&self, _u: &crate::models::User) -> Result<(), AppError> {
            unimplemented!()
        }
        async fn get_user_count(&self) -> Result<i64, AppError> {
            Ok(1)
        }
        async fn assign_platform_admin(&self, _id: Uuid) -> Result<(), AppError> {
            Ok(())
        }
        async fn get_user_by_email(
            &self,
            _email: &str,
        ) -> Result<Option<crate::models::User>, AppError> {
            Ok(None)
        }
        async fn update_password_hash(&self, _id: Uuid, _pw: &str) -> Result<(), AppError> {
            Ok(())
        }
        async fn get_system_roles_for_user(&self, _id: Uuid) -> Result<Vec<String>, AppError> {
            Ok(vec![])
        }
        async fn update_user_profile(
            &self,
            id: Uuid,
            full_name: Option<String>,
            _a: Option<String>,
            _b: Option<String>,
        ) -> Result<crate::models::User, AppError> {
            Ok(crate::models::User {
                id,
                email: "test@example.com".into(),
                password_hash: "hash".into(),
                created_at: Utc::now(),
                full_name,
                avatar_url: None,
                bio: None,
                preferred_language: None,
            })
        }
        async fn get_workspace_users(
            &self,
            _w: Uuid,
        ) -> Result<Vec<crate::models::User>, AppError> {
            Ok(vec![])
        }
        async fn get_dashboard_preferences(
            &self,
            _u: Uuid,
            _w: Uuid,
        ) -> Result<Option<crate::handlers::dashboard_preferences::DashboardPreferences>, AppError>
        {
            Ok(None)
        }
        async fn update_dashboard_preferences(
            &self,
            _u: Uuid,
            _w: Uuid,
            _r: crate::handlers::dashboard_preferences::UpdateDashboardPreferences,
        ) -> Result<crate::handlers::dashboard_preferences::DashboardPreferences, AppError>
        {
            unimplemented!()
        }
        async fn get_notifications(
            &self,
            _u: Uuid,
            _l: i64,
            _c: Option<chrono::DateTime<chrono::Utc>>,
        ) -> Result<Vec<crate::models::Notification>, AppError> {
            Ok(vec![])
        }
        async fn get_unread_notification_count(&self, _u: Uuid) -> Result<i64, AppError> {
            Ok(0)
        }
        async fn mark_notification_as_read(&self, _u: Uuid, _n: Uuid) -> Result<(), AppError> {
            Ok(())
        }
        async fn mark_all_notifications_as_read(&self, _u: Uuid) -> Result<(), AppError> {
            Ok(())
        }
        async fn check_recent_notification_exists(
            &self,
            _u: Uuid,
            _t: &str,
            _l: &str,
            _d: i32,
        ) -> Result<bool, AppError> {
            Ok(false)
        }
        async fn create_notification(
            &self,
            _n: &crate::models::Notification,
        ) -> Result<(), AppError> {
            Ok(())
        }
    }

    struct MockTenantProvisioner;
    #[async_trait]
    impl crate::services::tenant_provisioner::TenantProvisionerTrait for MockTenantProvisioner {
        async fn provision_tenant_db(
            &self,
            workspace_id: &Uuid,
            custom_db_url: Option<String>,
        ) -> crate::errors::AppResult<String> {
            if let Some(url) = custom_db_url {
                Ok(url)
            } else {
                let db_name = format!("tenant_{}", workspace_id.to_string().replace('-', ""));
                Ok(format!(
                    "postgres://test_user:test_password@localhost:5432/{db_name}"
                ))
            }
        }
    }

    struct MockTenantManager {
        registered: std::sync::Mutex<Vec<(String, serde_json::Value)>>,
    }

    impl MockTenantManager {
        fn new() -> Self {
            Self {
                registered: std::sync::Mutex::new(Vec::new()),
            }
        }
    }

    #[async_trait]
    impl crate::supavisor::TenantManager for MockTenantManager {
        async fn init_tenant(&self) {}
        async fn delete_tenant(&self, _tenant_id: &str) -> crate::errors::AppResult<()> {
            Ok(())
        }
        async fn register_tenant_dynamic(
            &self,
            tenant_id: &str,
            payload: &serde_json::Value,
        ) -> crate::errors::AppResult<()> {
            self.registered
                .lock()
                .unwrap()
                .push((tenant_id.to_string(), payload.clone()));
            Ok(())
        }
        async fn get_tenant(
            &self,
            _tenant_id: &str,
        ) -> crate::errors::AppResult<serde_json::Value> {
            Ok(serde_json::json!({}))
        }
        async fn update_tenant_mode(
            &self,
            _tenant_id: &str,
            _mode_type: &str,
        ) -> crate::errors::AppResult<()> {
            Ok(())
        }
    }

    fn build_service(user_exists: bool) -> WorkspaceService {
        build_service_with_tenant_manager(
            user_exists,
            std::sync::Arc::new(MockTenantManager::new()),
        )
    }

    fn build_service_with_tenant_manager(
        user_exists: bool,
        tenant_manager: std::sync::Arc<dyn crate::supavisor::TenantManager>,
    ) -> WorkspaceService {
        WorkspaceService::new(
            std::sync::Arc::new(MockWorkspaceRepo {
                workspace: Some(dummy_workspace()),
                primary_role: Some(Uuid::new_v4()),
            }),
            std::sync::Arc::new(MockUserRepo { user_exists }),
            std::sync::Arc::new(MockTenantProvisioner),
            tenant_manager,
        )
    }

    // ── Tests ────────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_create_workspace_user_not_found() {
        let svc = build_service(false);
        let err = svc
            .create_workspace("Acme", Uuid::new_v4())
            .await
            .unwrap_err();
        assert!(err.contains("does not exist"));
    }

    #[tokio::test]
    async fn test_create_workspace_success() {
        let svc = build_service(true);
        let ws = svc
            .create_workspace("Acme Corp", Uuid::new_v4())
            .await
            .unwrap();
        assert_eq!(ws.name, "Acme Corp");
    }

    #[tokio::test]
    async fn test_create_workspace_registers_tenant_with_supavisor() {
        let tenant_manager = std::sync::Arc::new(MockTenantManager::new());
        let svc = build_service_with_tenant_manager(true, tenant_manager.clone());

        let ws = svc
            .create_workspace("Acme Corp", Uuid::new_v4())
            .await
            .unwrap();

        let registered = tenant_manager.registered.lock().unwrap();
        assert_eq!(
            registered.len(),
            1,
            "should have registered exactly one tenant with Supavisor"
        );
        let (tenant_id, payload) = &registered[0];
        // Tenant ID must match the format produced by diesel_pool::get_tenant_url() —
        // tenant_<uuid-without-hyphens> — not the raw UUID.
        let expected_tenant_id = format!("tenant_{}", ws.id.to_string().replace('-', ""));
        assert_eq!(
            tenant_id, &expected_tenant_id,
            "tenant_id must use the same format as diesel_pool::get_tenant_url()"
        );
        // Verify payload contains expected Supavisor tenant config
        let tenant = payload
            .get("tenant")
            .expect("payload should have 'tenant' key");
        assert!(tenant.get("db_host").is_some(), "should have db_host");
        assert!(tenant.get("db_port").is_some(), "should have db_port");
        assert!(
            tenant.get("db_database").is_some(),
            "should have db_database"
        );
        let db_name = tenant["db_database"].as_str().unwrap();
        assert!(
            db_name.starts_with("tenant_"),
            "db_database should start with 'tenant_': got {db_name}"
        );
    }

    #[tokio::test]
    async fn test_get_member_roles_found() {
        let repo = MockWorkspaceRepo {
            workspace: Some(dummy_workspace()),
            primary_role: Some(Uuid::new_v4()),
        };
        let (role_id, extras) =
            WorkspaceService::get_member_roles(&repo, Uuid::new_v4(), Uuid::new_v4())
                .await
                .unwrap();
        assert!(!role_id.is_nil());
        assert!(extras.is_empty());
    }

    #[tokio::test]
    async fn test_get_member_roles_not_found() {
        let repo = MockWorkspaceRepo {
            workspace: None,
            primary_role: None,
        };
        let err = WorkspaceService::get_member_roles(&repo, Uuid::new_v4(), Uuid::new_v4())
            .await
            .unwrap_err();
        assert_eq!(err, "NOT_FOUND");
    }

    #[tokio::test]
    async fn test_update_workspace_no_fields_error() {
        let svc = build_service(true);
        let args = UpdateWorkspaceArgs {
            ws_id: Uuid::new_v4(),
            name: None,
            date_format: None,
            time_format: None,
            timezone: None,
            theme_primary_color: None,
            default_language: None,
            logo_url: None,
            tagline: None,
            holiday_overnight_strategy: None,
            default_holiday_ot_multiplier: None,
        };
        let err = svc.update_workspace(args).await.unwrap_err();
        assert!(err.contains("No fields to update"));
    }

    #[tokio::test]
    async fn test_update_workspace_success() {
        let svc = build_service(true);
        let args = UpdateWorkspaceArgs {
            ws_id: Uuid::new_v4(),
            name: Some("Updated Corp".into()),
            date_format: None,
            time_format: None,
            timezone: None,
            theme_primary_color: None,
            default_language: None,
            logo_url: None,
            tagline: None,
            holiday_overnight_strategy: None,
            default_holiday_ot_multiplier: None,
        };
        let ws = svc.update_workspace(args).await.unwrap();
        assert_eq!(ws.name, "Updated Corp");
    }

    #[tokio::test]
    async fn test_update_workspace_with_multiplier() {
        let svc = build_service(true);
        let args = UpdateWorkspaceArgs {
            ws_id: Uuid::new_v4(),
            name: None,
            date_format: None,
            time_format: None,
            timezone: None,
            theme_primary_color: None,
            default_language: None,
            logo_url: None,
            tagline: None,
            holiday_overnight_strategy: None,
            default_holiday_ot_multiplier: Some(Decimal::new(15, 1)), // 1.5
        };
        let ws = svc.update_workspace(args).await.unwrap();
        assert!(!ws.name.is_empty());
    }
}
