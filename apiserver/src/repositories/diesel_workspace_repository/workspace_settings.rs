use super::DieselWorkspaceRepository;
use crate::errors::AppError;
use crate::models::diesel::core::WorkspaceEntity;
use crate::models::Workspace;
use crate::schema::{workspace_roles, workspaces};
use crate::utils::i18n::I18nKey;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

impl DieselWorkspaceRepository {
    pub(super) async fn update_workspace_impl(
        &self,
        id: Uuid,
        name: Option<String>,
        date_format: Option<String>,
        time_format: Option<String>,
        timezone: Option<String>,
        theme_primary_color: Option<String>,
        default_language: Option<String>,
        logo_url: Option<String>,
        tagline: Option<String>,
    ) -> Result<Workspace, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .api
                .get()
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

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
            logo_url: Option<Option<String>>,
            tagline: Option<Option<String>>,
        }

        let has_changes = name.is_some()
            || date_format.is_some()
            || time_format.is_some()
            || timezone.is_some()
            || theme_primary_color.is_some()
            || default_language.is_some()
            || logo_url.is_some()
            || tagline.is_some();

        if !has_changes {
            // Note: calls the trait method, which delegates to _impl — we call the _impl directly
            // to avoid going through the trait dispatcher.
            return self
                .get_workspace_by_id_impl(id)
                .await?
                .ok_or_else(|| AppError::not_found(I18nKey::WorkspaceNotFound));
        }

        let update = WorkspaceUpdate {
            name,
            date_format,
            time_format,
            timezone,
            theme_primary_color: theme_primary_color.map(Some),
            default_language,
            logo_url: logo_url.map(Some),
            tagline: tagline.map(Some),
        };

        let result: WorkspaceEntity = diesel::update(workspaces::table.find(id))
            .set(&update)
            .get_result(&mut *conn)
            .await
            .map_err(AppError::from)?;

        if let Some(ref new_tz) = update.timezone {
            let mut tenant_conn = crate::diesel_pool::TransactionConn(
                self.diesel_pools
                    .tenants_tx
                    .get_connection(&id)
                    .await
                    .map_err(|e| AppError::Internal(format!("Failed to get tenant conn: {e}")))?,
            );
            use crate::schema::work_locations;
            diesel::update(
                work_locations::table
                    .filter(work_locations::workspace_id.eq(id))
                    .filter(work_locations::is_headquarters.eq(true)),
            )
            .set(work_locations::timezone.eq(new_tz))
            .execute(&mut *tenant_conn)
            .await
            .map_err(|e| AppError::Internal(format!("db tenant: {e}")))?;
        }

        Ok(result.into())
    }

    pub(super) async fn is_any_role_global_impl(
        &self,
        role_ids: &[Uuid],
    ) -> Result<bool, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .api
                .get()
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let query = workspace_roles::table
            .filter(workspace_roles::id.eq_any(role_ids))
            .filter(workspace_roles::scope_type.eq("global"));

        let is_global: bool = diesel::select(diesel::dsl::exists(query))
            .get_result(&mut *conn)
            .await
            .map_err(AppError::from)?;

        Ok(is_global)
    }

    pub(super) async fn update_workspace_byod_url_impl(
        &self,
        workspace_id: Uuid,
        encrypted_url: Option<String>,
    ) -> Result<(), AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .api
                .get()
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        diesel::update(workspaces::table.find(workspace_id))
            .set(workspaces::custom_db_url_encrypted.eq(encrypted_url))
            .execute(&mut *conn)
            .await
            .map_err(AppError::from)?;

        Ok(())
    }

    pub(super) async fn delete_workspace_impl(
        &self,
        workspace_id: Uuid,
    ) -> Result<(), AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .api
                .get()
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        diesel::delete(workspaces::table.find(workspace_id))
            .execute(&mut *conn)
            .await
            .map_err(AppError::from)?;

        Ok(())
    }
}
