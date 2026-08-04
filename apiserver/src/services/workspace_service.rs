use crate::models::{MemberExtraRole, NewWorkspaceConfigEntryParams, Workspace};
use crate::repositories::WorkspaceRepository;
use crate::utils::i18n::I18nKey;
use uuid::Uuid;

use crate::repositories::user_repository::UserRepository;
use std::sync::Arc;

use crate::repositories::system_db_cluster_repository::SystemDbClusterRepository;
use crate::repositories::workspace_config_repository::WorkspaceConfigRepository;
use crate::services::tenant_provisioner::{DatabaseMigratorTrait, TenantProvisionerTrait};
pub struct WorkspaceService {
    workspace_repo: Arc<dyn WorkspaceRepository>,
    user_repo: Arc<dyn UserRepository>,
    tenant_provisioner: Arc<dyn TenantProvisionerTrait>,
    system_db_cluster_repo: Arc<dyn SystemDbClusterRepository>,
    workspace_config_repo: Arc<dyn WorkspaceConfigRepository>,
    database_migrator: Arc<dyn DatabaseMigratorTrait>,
}

impl WorkspaceService {
    pub fn new(
        workspace_repo: Arc<dyn WorkspaceRepository>,
        user_repo: Arc<dyn UserRepository>,
        tenant_provisioner: Arc<dyn TenantProvisionerTrait>,
        system_db_cluster_repo: Arc<dyn SystemDbClusterRepository>,
        workspace_config_repo: Arc<dyn WorkspaceConfigRepository>,
        database_migrator: Arc<dyn DatabaseMigratorTrait>,
    ) -> Self {
        Self {
            workspace_repo,
            user_repo,
            tenant_provisioner,
            system_db_cluster_repo,
            workspace_config_repo,
            database_migrator,
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
    #[allow(clippy::disallowed_methods)]
    pub async fn create_workspace(
        &self,
        id: Uuid,
        name: &str,
        owner_id: Uuid,
        database_url: Option<String>,
        custom_db_url_encrypted: Option<String>,
    ) -> crate::errors::AppResult<Workspace> {
        // 1. Check if user exists
        let user_exists = self.user_repo.get_user_by_id(owner_id).await?.is_some();

        if !user_exists {
            return Err(crate::errors::AppError::not_found(I18nKey::UserNotFound));
        }

        // 2. Resolve database_url
        let mut auto_provisioned_info = None;

        let final_db_url = if let Some(url) = database_url {
            url
        } else {
            // Auto provisioning logic
            let cluster = self
                .system_db_cluster_repo
                .get_least_loaded_cluster()
                .await?
                .ok_or_else(|| {
                    crate::errors::AppError::Internal(
                        "No available database clusters for auto-provisioning".to_string(),
                    )
                })?;

            let admin_url = cluster.admin_database_url;
            let tenant_db_name = format!("tenant_{}", id.to_string().replace('-', ""));

            self.database_migrator
                .create_tenant_database(&admin_url, &tenant_db_name)
                .await?;

            // Increment active tenants
            self.system_db_cluster_repo
                .increment_active_tenants(cluster.id)
                .await?;

            auto_provisioned_info = Some((admin_url.clone(), tenant_db_name.clone(), cluster.id));

            // Replace the database name in the url
            let mut parsed_url = reqwest::Url::parse(&admin_url)
                .map_err(|e| crate::errors::AppError::Internal(e.to_string()))?;
            parsed_url.set_path(&format!("/{tenant_db_name}"));
            parsed_url.to_string()
        };

        // 3. Run migrations on the provided or newly created database
        let result = async {
            let new_db_url = self
                .tenant_provisioner
                .provision_tenant_db(&id, &final_db_url)
                .await?;

            // 4. Encrypt the final database URL so ALL workspaces are treated as BYOD in the database
            let final_encrypted_url = if let Some(encrypted) = custom_db_url_encrypted {
                encrypted
            } else {
                let crypto_service = std::env::var("ENCRYPTION_KEYS_JSON")
                    .ok()
                    .and_then(|keys| crate::crypto::CryptoService::from_json(&keys).ok());

                if let Some(crypto) = crypto_service {
                    crypto.encrypt(&new_db_url).unwrap_or(new_db_url)
                } else {
                    new_db_url
                }
            };

            // 5. Create workspace with owner using repository
            let ws = self
                .workspace_repo
                .create_workspace_with_owner(id, name, owner_id, Some(final_encrypted_url))
                .await?;

            Ok(ws)
        }
        .await;

        match result {
            Ok(ws) => Ok(ws),
            Err(e) => {
                if let Some((admin_url, tenant_db_name, cluster_id)) = auto_provisioned_info {
                    tracing::warn!(
                        workspace_id = %id,
                        tenant_db_name = %tenant_db_name,
                        error = %e,
                        "Workspace onboarding failed. Reverting auto-provisioned resources (Saga compensation)..."
                    );

                    if let Err(drop_err) = self
                        .database_migrator
                        .delete_tenant_database(&admin_url, &tenant_db_name)
                        .await
                    {
                        tracing::error!(
                            workspace_id = %id,
                            tenant_db_name = %tenant_db_name,
                            error = %drop_err,
                            "Failed to drop tenant database during compensation"
                        );
                    }

                    if let Err(dec_err) = self
                        .system_db_cluster_repo
                        .decrement_active_tenants(cluster_id)
                        .await
                    {
                        tracing::error!(
                            workspace_id = %id,
                            cluster_id = %cluster_id,
                            error = %dec_err,
                            "Failed to decrement active tenants count during compensation"
                        );
                    }
                }
                Err(e)
            }
        }
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn get_member_roles(
        repo: &dyn WorkspaceRepository,
        workspace_id: Uuid,
        user_id: Uuid,
    ) -> crate::errors::AppResult<(Uuid, Vec<MemberExtraRole>)> {
        let primary_role_id = repo
            .find_primary_role(workspace_id, user_id)
            .await?
            .ok_or_else(|| crate::errors::AppError::not_found(I18nKey::RoleNotFound))?;

        let extra_roles = repo.find_extra_roles(workspace_id, user_id).await?;

        Ok((primary_role_id, extra_roles))
    }

    #[allow(clippy::too_many_lines)]
    pub async fn update_workspace(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        args: UpdateWorkspaceArgs,
    ) -> crate::errors::AppResult<Workspace> {
        let has_system_changes = args.name.is_some()
            || args.date_format.is_some()
            || args.time_format.is_some()
            || args.timezone.is_some()
            || args.theme_primary_color.is_some()
            || args.default_language.is_some()
            || args.logo_url.is_some()
            || args.tagline.is_some();

        let has_eav_changes = args.holiday_overnight_strategy.is_some()
            || args.default_holiday_ot_multiplier.is_some();

        if !has_system_changes && !has_eav_changes {
            return Err(crate::errors::AppError::validation(
                I18nKey::NoFieldsToUpdate,
            ));
        }

        if has_eav_changes {
            if let Some(strategy) = &args.holiday_overnight_strategy {
                self.workspace_config_repo
                    .delete_by_key(
                        auth_token,
                        args.ws_id,
                        "workspace",
                        "overtime.holiday_overnight_strategy",
                    )
                    .await?;

                let entry = crate::models::WorkspaceConfigEntry::new(NewWorkspaceConfigEntryParams {
        id: Uuid::new_v4(),
        workspace_id: args.ws_id,
        scope_type: "workspace".to_string(),
        scope_id: None,
        config_key: "overtime.holiday_overnight_strategy".to_string(),
        value_int: None,
        value_decimal: None,
        value_text: Some(strategy.clone()),
        value_bool: None,
        value_json: None,
        effective_from: chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap_or_default(),
        effective_to: None,
        created_by: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    });

                self.workspace_config_repo
                    .create_entry(auth_token, entry)
                    .await?;
            }

            if let Some(multiplier) = args.default_holiday_ot_multiplier {
                self.workspace_config_repo
                    .delete_by_key(
                        auth_token,
                        args.ws_id,
                        "workspace",
                        "overtime.default_holiday_ot_multiplier",
                    )
                    .await?;

                let entry = crate::models::WorkspaceConfigEntry::new(NewWorkspaceConfigEntryParams {
        id: Uuid::new_v4(),
        workspace_id: args.ws_id,
        scope_type: "workspace".to_string(),
        scope_id: None,
        config_key: "overtime.default_holiday_ot_multiplier".to_string(),
        value_int: None,
        value_decimal: Some(multiplier),
        value_text: None,
        value_bool: None,
        value_json: None,
        effective_from: chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap_or_default(),
        effective_to: None,
        created_by: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    });

                self.workspace_config_repo
                    .create_entry(auth_token, entry)
                    .await?;
            }
        }

        if has_system_changes {
            self.workspace_repo
                .update_workspace(crate::repositories::workspace_repository::UpdateWorkspaceParams {
                    id: args.ws_id,
                    name: args.name,
                    date_format: args.date_format,
                    time_format: args.time_format,
                    timezone: args.timezone,
                    theme_primary_color: args.theme_primary_color,
                    default_language: args.default_language,
                    logo_url: args.logo_url,
                    tagline: args.tagline,
                })
                .await
        } else {
            self.workspace_repo
                .get_workspace_by_id(args.ws_id)
                .await?
                .ok_or_else(|| crate::errors::AppError::not_found(I18nKey::WorkspaceNotFound))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::AppError;
    use crate::models::{MemberExtraRole, NewWorkspaceConfigEntryParams, Workspace};
    use async_trait::async_trait;
    use chrono::Utc;
    use rust_decimal::Decimal;
    use uuid::Uuid;

    fn dummy_workspace() -> Workspace {
        use rust_decimal::Decimal;
        Workspace::new(NewWorkspaceParams {
            id: Uuid::new_v4(),
            name: "Acme Corp".into(),
            date_format: "DD/MM/YYYY".into(),
            time_format: "HH:mm".into(),
            timezone: "Asia/Ho_Chi_Minh".into(),
            theme_primary_color: "#000".into(),
            default_language: "vi".into(),
            logo_url: None,
            tagline: None,
            holiday_overnight_strategy: "split".into(),
            default_holiday_ot_multiplier: Decimal::new(15, 1),
            created_at: Utc::now(),
            custom_db_url_encrypted: None,
        })
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
            id: Uuid,
            name: &str,
            _owner_id: Uuid,
            _custom_db_url_encrypted: Option<String>,
        ) -> Result<Workspace, AppError> {
            let mut ws = dummy_workspace();
            ws.set_id(id);
            ws.set_name(name.to_string());
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
        async fn clear_workspace_data(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _g: &str,
        ) -> Result<(), AppError> {
            Ok(())
        }
        async fn add_member(&self, _ws: Uuid, _u: Uuid, _r: Uuid) -> Result<(), AppError> {
            Ok(())
        }
        async fn update_member_role(&self, _ws: Uuid, _u: Uuid, _r: Uuid) -> Result<(), AppError> {
            Ok(())
        }
        async fn remove_member(&self, _ws: Uuid, _u: Uuid) -> Result<(), AppError> {
            Ok(())
        }
        async fn find_primary_role(&self, _ws: Uuid, _u: Uuid) -> Result<Option<Uuid>, AppError> {
            Ok(self.primary_role)
        }
        async fn get_workspace_members(
            &self,
            _w: Uuid,
            _page: i64,
            _limit: i64,
        ) -> Result<(Vec<crate::models::WorkspaceMemberDetail>, i64), AppError> {
            Ok((vec![], 0))
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
            _page: i64,
            _limit: i64,
        ) -> Result<(Vec<crate::models::WorkspaceRole>, i64), AppError> {
            Ok((vec![], 0))
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
        async fn update_workspace(
            &self,
            params: crate::repositories::workspace_repository::UpdateWorkspaceParams,
        ) -> Result<Workspace, AppError> {
            let mut ws = dummy_workspace();
            if let Some(n) = params.name {
                ws.set_name(n);
            }
            Ok(ws)
        }
        async fn update_workspace_byod_url(
            &self,
            _ws: Uuid,
            _encrypted_url: Option<String>,
        ) -> Result<(), AppError> {
            Ok(())
        }
        async fn is_any_role_global(&self, _r: &[Uuid]) -> Result<bool, AppError> {
            Ok(false)
        }
        async fn get_users_with_permission(
            &self,
            _workspace_id: Uuid,
            _permission: &str,
        ) -> Result<Vec<Uuid>, AppError> {
            Ok(vec![])
        }

        async fn delete_workspace(&self, _workspace_id: Uuid) -> Result<(), AppError> {
            Ok(())
        }
    }

    struct MockUserRepo {
        user_exists: bool,
    }
    #[async_trait]
    impl crate::repositories::user_repository::UserRepository for MockUserRepo {
        async fn get_user_by_id(&self, id: Uuid) -> Result<Option<crate::models::User>, AppError> {
            if self.user_exists {
                Ok(Some(crate::models::User::new(NewUserParams {
        id: id,
        email: "test@example.com".into(),
        password_hash: "hash".into(),
        created_at: Utc::now(),
        full_name: Some("Test User".into()),
        avatar_url: None,
        bio: None,
        preferred_language: None,
        avatar_file_id: None,
    })))
            } else {
                Ok(None)
            }
        }
        async fn get_users_by_ids(
            &self,
            ids: &[Uuid],
        ) -> Result<Vec<crate::models::User>, AppError> {
            let mut users = Vec::new();
            if self.user_exists {
                for id in ids {
                    users.push(crate::models::User::new(NewUserParams {
        id: *id,
        email: "test@example.com".into(),
        password_hash: "hash".into(),
        created_at: Utc::now(),
        full_name: Some("Test User".into()),
        avatar_url: None,
        bio: None,
        preferred_language: None,
        avatar_file_id: None,
    }));
                }
            }
            Ok(users)
        }
        async fn create_user(&self, _u: &crate::models::User) -> Result<(), AppError> {
            unimplemented!()
        }
        async fn create_user_with_admin_check(
            &self,
            _u: &crate::models::User,
        ) -> Result<bool, AppError> {
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
            Ok(crate::models::User::new(NewUserParams {
        id: id,
        email: "test@example.com".into(),
        password_hash: "hash".into(),
        created_at: Utc::now(),
        full_name: full_name,
        avatar_url: None,
        bio: None,
        preferred_language: None,
        avatar_file_id: None,
    }))
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
            _auth_token: &crate::diesel_pool::TenantAuthToken,
        ) -> Result<Option<crate::handlers::dashboard_preferences::DashboardPreferences>, AppError>
        {
            Ok(None)
        }
        async fn update_dashboard_preferences(
            &self,
            _u: Uuid,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
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
        async fn create_notifications_batch(
            &self,
            _notifications: &[crate::models::Notification],
        ) -> Result<(), AppError> {
            Ok(())
        }
    }

    struct MockTenantProvisioner;
    #[async_trait]
    impl crate::services::tenant_provisioner::TenantProvisionerTrait for MockTenantProvisioner {
        async fn provision_tenant_db(
            &self,
            _workspace_id: &Uuid,
            db_url: &str,
        ) -> crate::errors::AppResult<String> {
            Ok(db_url.to_string())
        }
    }
    #[async_trait]
    impl crate::services::tenant_provisioner::DatabaseMigratorTrait for MockTenantProvisioner {
        async fn create_tenant_database(
            &self,
            _admin_url: &str,
            _db_name: &str,
        ) -> crate::errors::AppResult<()> {
            Ok(())
        }
        async fn delete_tenant_database(
            &self,
            _admin_url: &str,
            _db_name: &str,
        ) -> crate::errors::AppResult<()> {
            Ok(())
        }
    }

    fn build_service(user_exists: bool) -> WorkspaceService {
        let mut mock_config_repo =
            crate::repositories::workspace_config_repository::MockWorkspaceConfigRepository::new();
        mock_config_repo
            .expect_delete_by_key()
            .returning(|_, _, _, _| Box::pin(async move { Ok(()) }));
        mock_config_repo
            .expect_create_entry()
            .returning(|_, entry| Box::pin(async move { Ok(entry) }));
        let provisioner = std::sync::Arc::new(MockTenantProvisioner);
        WorkspaceService::new(
            std::sync::Arc::new(MockWorkspaceRepo {
                workspace: Some(dummy_workspace()),
                primary_role: Some(Uuid::new_v4()),
            }),
            std::sync::Arc::new(MockUserRepo { user_exists }),
            provisioner.clone(),
            std::sync::Arc::new(MockSystemDbClusterRepo),
            std::sync::Arc::new(mock_config_repo),
            provisioner,
        )
    }

    struct MockSystemDbClusterRepo;
    #[async_trait]
    impl crate::repositories::system_db_cluster_repository::SystemDbClusterRepository
        for MockSystemDbClusterRepo
    {
        async fn get_least_loaded_cluster(
            &self,
        ) -> Result<Option<crate::models::system_db_cluster::SystemDbCluster>, AppError> {
            Ok(Some(crate::models::system_db_cluster::SystemDbCluster {
                id: Uuid::new_v4(),
                name: "Test Cluster".to_string(),
                admin_database_url: "postgres://admin:pass@localhost:5432/postgres".to_string(),
                region: "ap-southeast-1".to_string(),
                max_tenants: 100,
                active_tenants: 0,
                is_active: true,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            }))
        }
        async fn increment_active_tenants(&self, _cluster_id: Uuid) -> Result<(), AppError> {
            Ok(())
        }
        async fn decrement_active_tenants(&self, _cluster_id: Uuid) -> Result<(), AppError> {
            Ok(())
        }
        async fn create_cluster(
            &self,
            _cluster: &crate::models::system_db_cluster::NewSystemDbCluster,
        ) -> Result<crate::models::system_db_cluster::SystemDbCluster, AppError> {
            unimplemented!()
        }
        async fn list_clusters(
            &self,
        ) -> Result<Vec<crate::models::system_db_cluster::SystemDbCluster>, AppError> {
            unimplemented!()
        }
        async fn delete_cluster(&self, _cluster_id: Uuid) -> Result<(), AppError> {
            unimplemented!()
        }
        async fn update_cluster_status(
            &self,
            _cluster_id: Uuid,
            _is_active: bool,
        ) -> Result<crate::models::system_db_cluster::SystemDbCluster, AppError> {
            unimplemented!()
        }
    }

    struct FailingMockTenantProvisioner;
    #[async_trait]
    impl crate::services::tenant_provisioner::TenantProvisionerTrait for FailingMockTenantProvisioner {
        async fn provision_tenant_db(
            &self,
            _workspace_id: &Uuid,
            _db_url: &str,
        ) -> crate::errors::AppResult<String> {
            Err(crate::errors::AppError::Internal(
                "provisioning failed".into(),
            ))
        }
    }
    #[async_trait]
    impl crate::services::tenant_provisioner::DatabaseMigratorTrait for FailingMockTenantProvisioner {
        async fn create_tenant_database(
            &self,
            _admin_url: &str,
            _db_name: &str,
        ) -> crate::errors::AppResult<()> {
            Ok(())
        }
        async fn delete_tenant_database(
            &self,
            _admin_url: &str,
            _db_name: &str,
        ) -> crate::errors::AppResult<()> {
            Ok(())
        }
    }

    fn build_service_with_failing_provisioner() -> WorkspaceService {
        let mut mock_config_repo =
            crate::repositories::workspace_config_repository::MockWorkspaceConfigRepository::new();
        mock_config_repo
            .expect_delete_by_key()
            .returning(|_, _, _, _| Box::pin(async move { Ok(()) }));
        mock_config_repo
            .expect_create_entry()
            .returning(|_, entry| Box::pin(async move { Ok(entry) }));
        let provisioner = std::sync::Arc::new(FailingMockTenantProvisioner);
        WorkspaceService::new(
            std::sync::Arc::new(MockWorkspaceRepo {
                workspace: Some(dummy_workspace()),
                primary_role: Some(Uuid::new_v4()),
            }),
            std::sync::Arc::new(MockUserRepo { user_exists: true }),
            provisioner.clone(),
            std::sync::Arc::new(MockSystemDbClusterRepo),
            std::sync::Arc::new(mock_config_repo),
            provisioner,
        )
    }

    // ── Tests ────────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_create_workspace_user_not_found() {
        let svc = build_service(false);
        let err = svc
            .create_workspace(
                Uuid::new_v4(),
                "Acme",
                Uuid::new_v4(),
                Some("postgres://u:p@h:5432/tenant_db".to_string()),
                None,
            )
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::NotFound(_)));
    }

    #[tokio::test]
    async fn test_create_workspace_fails_when_provisioning_fails() {
        let svc = build_service_with_failing_provisioner();
        let err = svc
            .create_workspace(
                Uuid::new_v4(),
                "Acme",
                Uuid::new_v4(),
                Some("postgres://u:p@h:5432/tenant_db".to_string()),
                None,
            )
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::Internal(_)));
        assert!(err.to_string().contains("provisioning failed"));
    }

    #[tokio::test]
    async fn test_create_workspace_success() {
        let svc = build_service(true);
        let ws = svc
            .create_workspace(
                Uuid::new_v4(),
                "Acme Corp",
                Uuid::new_v4(),
                Some("postgres://u:p@h:5432/tenant_db".to_string()),
                None,
            )
            .await
            .unwrap();
        assert_eq!(ws.name(), "Acme Corp");
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
        assert!(matches!(err, AppError::NotFound(_)));
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
        let auth_token = crate::diesel_pool::TenantAuthToken::system_override(args.ws_id);
        let err = svc.update_workspace(&auth_token, args).await.unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
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
        let auth_token = crate::diesel_pool::TenantAuthToken::system_override(args.ws_id);
        let ws = svc.update_workspace(&auth_token, args).await.unwrap();
        assert_eq!(ws.name(), "Updated Corp");
    }

    #[tokio::test]
    async fn test_update_workspace_with_multiplier() {
        println!("DATABASE_URL env var: {:?}", std::env::var("DATABASE_URL"));
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
        let auth_token = crate::diesel_pool::TenantAuthToken::system_override(args.ws_id);
        let ws = svc.update_workspace(&auth_token, args).await.unwrap();
        assert!(!ws.name().is_empty());
    }

    #[tokio::test]
    async fn test_create_workspace_compensation_on_failure() {
        let svc = build_service_with_failing_provisioner();
        let err = svc
            .create_workspace(
                Uuid::new_v4(),
                "Acme Failed",
                Uuid::new_v4(),
                None, // database_url = None triggers auto-provisioning database flow
                None,
            )
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::Internal(_)));
        assert!(err.to_string().contains("provisioning failed"));
    }
}
