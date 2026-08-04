use crate::diesel_pool::DieselPools;
use crate::errors::AppError;
use crate::models::diesel::core::UserEntity;
use crate::models::User;
use crate::repositories::user_repository::UserRepository;
use crate::schema::{system_roles, user_system_roles, users, workspace_members};
use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::{AsyncConnection, RunQueryDsl};
use uuid::Uuid;

pub struct DieselUserRepository {
    diesel_pools: DieselPools,
}

impl DieselUserRepository {
    pub fn new(diesel_pools: DieselPools) -> Self {
        Self { diesel_pools }
    }
}

#[async_trait]
impl UserRepository for DieselUserRepository {
    async fn create_user(&self, user: &User) -> Result<(), AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .api
                .get()
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let entity = UserEntity {
            id: user.id(),
            email: user.email().to_string(),
            password_hash: user.password_hash().to_string(),
            created_at: Some(user.created_at()),
            full_name: user.full_name().map(String::from),
            avatar_url: user.avatar_url().map(String::from),
            bio: user.bio().map(String::from),
            preferred_language: user.preferred_language().map(String::from),
            avatar_file_id: user.avatar_file_id(),
        };

        diesel::insert_into(users::table)
            .values(&entity)
            .execute(&mut *conn)
            .await
            .map_err(AppError::from)?;

        Ok(())
    }

    async fn create_user_with_admin_check(&self, user: &User) -> Result<bool, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .api
                .get()
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let entity = UserEntity {
            id: user.id(),
            email: user.email().to_string(),
            password_hash: user.password_hash().to_string(),
            created_at: Some(user.created_at()),
            full_name: user.full_name().map(String::from),
            avatar_url: user.avatar_url().map(String::from),
            bio: user.bio().map(String::from),
            preferred_language: user.preferred_language().map(String::from),
            avatar_file_id: user.avatar_file_id(),
        };

        let is_first = conn
            .transaction::<bool, AppError, _>(|conn| {
                Box::pin(async move {
                    // Lock row or rely on serializability if we wanted perfect safety.
                    // Using an advisory transaction lock ensures that concurrent registrations
                    // on an empty database are serialized, preventing multiple users from
                    // simultaneously reading `count == 0` and becoming platform_admin.
                    diesel::sql_query("SELECT pg_advisory_xact_lock(1337)")
                        .execute(&mut *conn)
                        .await
                        .map_err(|e| AppError::Internal(format!("db lock: {e}")))?;

                    let count = users::table
                        .count()
                        .get_result::<i64>(&mut *conn)
                        .await
                        .map_err(AppError::from)?;

                    let is_first = count == 0;

                    diesel::insert_into(users::table)
                        .values(&entity)
                        .execute(&mut *conn)
                        .await
                        .map_err(AppError::from)?;

                    if is_first {
                        let role_id_opt: Option<Uuid> = system_roles::table
                            .filter(system_roles::name.eq("platform_admin"))
                            .select(system_roles::id)
                            .first::<Uuid>(&mut *conn)
                            .await
                            .optional()
                            .map_err(AppError::from)?;

                        if let Some(role_id) = role_id_opt {
                            diesel::insert_into(user_system_roles::table)
                                .values((
                                    user_system_roles::user_id.eq(entity.id),
                                    user_system_roles::role_id.eq(role_id),
                                ))
                                .execute(&mut *conn)
                                .await
                                .map_err(AppError::from)?;
                        } else {
                            return Err(AppError::Internal("platform_admin role not found".into()));
                        }
                    }

                    Ok(is_first)
                })
            })
            .await?;

        Ok(is_first)
    }

    async fn get_user_count(&self) -> Result<i64, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .api
                .get()
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let count = users::table
            .count()
            .get_result::<i64>(&mut *conn)
            .await
            .map_err(AppError::from)?;

        Ok(count)
    }

    async fn assign_platform_admin(&self, user_id: Uuid) -> Result<(), AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .api
                .get()
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let role_id_opt: Option<Uuid> = system_roles::table
            .filter(system_roles::name.eq("platform_admin"))
            .select(system_roles::id)
            .first::<Uuid>(&mut *conn)
            .await
            .optional()
            .map_err(AppError::from)?;

        let Some(role_id) = role_id_opt else {
            return Err(AppError::Internal("platform_admin role not found".into()));
        };

        diesel::insert_into(user_system_roles::table)
            .values((
                user_system_roles::user_id.eq(user_id),
                user_system_roles::role_id.eq(role_id),
            ))
            .execute(&mut *conn)
            .await
            .map_err(AppError::from)?;

        Ok(())
    }

    async fn get_user_by_email(&self, email: &str) -> Result<Option<User>, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .api
                .get()
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let entity = users::table
            .filter(users::email.eq(email))
            .select(UserEntity::as_select())
            .first::<UserEntity>(&mut *conn)
            .await
            .optional()
            .map_err(AppError::from)?;

        Ok(entity.map(Into::into))
    }

    async fn get_user_by_id(&self, id: Uuid) -> Result<Option<User>, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .api
                .get()
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let entity = users::table
            .find(id)
            .select(UserEntity::as_select())
            .first::<UserEntity>(&mut *conn)
            .await
            .optional()
            .map_err(AppError::from)?;

        Ok(entity.map(Into::into))
    }

    async fn get_users_by_ids(&self, ids: &[Uuid]) -> Result<Vec<User>, AppError> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .api
                .get()
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let entities = users::table
            .filter(users::id.eq_any(ids))
            .select(UserEntity::as_select())
            .load::<UserEntity>(&mut *conn)
            .await
            .map_err(AppError::from)?;

        Ok(entities.into_iter().map(Into::into).collect())
    }

    async fn update_password_hash(
        &self,
        user_id: Uuid,
        password_hash: &str,
    ) -> Result<(), AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .api
                .get()
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        diesel::update(users::table.find(user_id))
            .set(users::password_hash.eq(password_hash))
            .execute(&mut *conn)
            .await
            .map_err(AppError::from)?;

        Ok(())
    }

    async fn get_system_roles_for_user(&self, user_id: Uuid) -> Result<Vec<String>, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .api
                .get()
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let roles = user_system_roles::table
            .inner_join(system_roles::table.on(system_roles::id.eq(user_system_roles::role_id)))
            .filter(user_system_roles::user_id.eq(user_id))
            .select(system_roles::name)
            .load::<String>(&mut *conn)
            .await
            .map_err(AppError::from)?;

        Ok(roles)
    }

    async fn update_user_profile(
        &self,
        user_id: Uuid,
        full_name: Option<String>,
        avatar_url: Option<String>,
        bio: Option<String>,
        preferred_language: Option<String>,
    ) -> Result<User, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .api
                .get()
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let entity = diesel::update(users::table.find(user_id))
            .set((
                full_name.map(|f| users::full_name.eq(f)),
                avatar_url.map(|a| users::avatar_url.eq(a)),
                bio.map(|b| users::bio.eq(b)),
                preferred_language.map(|language| users::preferred_language.eq(language)),
            ))
            .get_result::<UserEntity>(&mut *conn)
            .await
            .map_err(AppError::from)?;

        Ok(entity.into())
    }

    async fn get_workspace_users(&self, workspace_id: Uuid) -> Result<Vec<User>, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .api
                .get()
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let entities = users::table
            .inner_join(workspace_members::table.on(workspace_members::user_id.eq(users::id)))
            .filter(workspace_members::workspace_id.eq(workspace_id))
            .select(users::all_columns)
            .order(users::created_at.desc())
            .load::<UserEntity>(&mut *conn)
            .await
            .map_err(AppError::from)?;

        Ok(entities.into_iter().map(Into::into).collect())
    }

    async fn get_dashboard_preferences(
        &self,
        user_id: Uuid,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Option<crate::handlers::dashboard_preferences::DashboardPreferences>, AppError>
    {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;
        let workspace_id = auth_token.workspace_id();

        use crate::schema::dashboard_preferences;
        let entity = dashboard_preferences::table
            .filter(dashboard_preferences::user_id.eq(user_id))
            .filter(dashboard_preferences::workspace_id.eq(workspace_id))
            .first::<crate::models::diesel::core::DashboardPreferenceEntity>(&mut *conn)
            .await
            .optional()
            .map_err(AppError::from)?;

        Ok(entity.map(Into::into))
    }

    async fn update_dashboard_preferences(
        &self,
        user_id: Uuid,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        req: crate::handlers::dashboard_preferences::UpdateDashboardPreferences,
    ) -> Result<crate::handlers::dashboard_preferences::DashboardPreferences, AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;
        let workspace_id = auth_token.workspace_id();

        use crate::schema::dashboard_preferences;
        let existing = self.get_dashboard_preferences(user_id, auth_token).await?;
        if let Some(mut prefs) = existing {
            // Update fields
            if let Some(av) = req.active_view {
                prefs.active_view = av;
            }
            if let Some(vw) = req.visible_widgets {
                prefs.visible_widgets = vw;
            }
            if let Some(wo) = req.widget_order {
                prefs.widget_order = wo;
            }
            if let Some(cs) = req.collapsed_sections {
                prefs.collapsed_sections = cs;
            }

            let entity = diesel::update(
                dashboard_preferences::table.filter(dashboard_preferences::id.eq(prefs.id)),
            )
            .set((
                dashboard_preferences::active_view.eq(prefs.active_view),
                dashboard_preferences::visible_widgets.eq(prefs.visible_widgets),
                dashboard_preferences::widget_order.eq(prefs.widget_order),
                dashboard_preferences::collapsed_sections.eq(prefs.collapsed_sections),
                dashboard_preferences::updated_at.eq(chrono::Utc::now()),
            ))
            .get_result::<crate::models::diesel::core::DashboardPreferenceEntity>(&mut *conn)
            .await
            .map_err(AppError::from)?;
            return Ok(entity.into());
        }
        // Insert new
        let entity = diesel::insert_into(dashboard_preferences::table)
            .values((
                dashboard_preferences::id.eq(Uuid::new_v4()),
                dashboard_preferences::user_id.eq(user_id),
                dashboard_preferences::workspace_id.eq(workspace_id),
                dashboard_preferences::active_view
                    .eq(req.active_view.unwrap_or_else(|| "overview".to_string())),
                dashboard_preferences::visible_widgets
                    .eq(req.visible_widgets.unwrap_or_else(|| serde_json::json!({}))),
                dashboard_preferences::widget_order
                    .eq(req.widget_order.unwrap_or_else(|| serde_json::json!([]))),
                dashboard_preferences::collapsed_sections.eq(req
                    .collapsed_sections
                    .unwrap_or_else(|| serde_json::json!({}))),
                dashboard_preferences::created_at.eq(chrono::Utc::now()),
                dashboard_preferences::updated_at.eq(chrono::Utc::now()),
            ))
            .get_result::<crate::models::diesel::core::DashboardPreferenceEntity>(&mut *conn)
            .await
            .map_err(AppError::from)?;
        return Ok(entity.into());
    }

    async fn get_notifications(
        &self,
        user_id: Uuid,
        limit: i64,
        cursor: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<Vec<crate::models::Notification>, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .api
                .get()
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        use crate::schema::notifications;
        let mut query = notifications::table
            .filter(notifications::user_id.eq(user_id))
            .into_boxed();

        if let Some(c) = cursor {
            query = query.filter(notifications::created_at.lt(c));
        }

        let entities = query
            .order(notifications::created_at.desc())
            .limit(limit)
            .load::<crate::models::diesel::core::NotificationEntity>(&mut *conn)
            .await
            .map_err(AppError::from)?;

        Ok(entities.into_iter().map(Into::into).collect())
    }

    async fn get_unread_notification_count(&self, user_id: Uuid) -> Result<i64, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .api
                .get()
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        use crate::schema::notifications;
        let count = notifications::table
            .filter(notifications::user_id.eq(user_id))
            .filter(notifications::is_read.eq(false))
            .count()
            .get_result::<i64>(&mut *conn)
            .await
            .map_err(AppError::from)?;

        Ok(count)
    }

    async fn mark_notification_as_read(
        &self,
        user_id: Uuid,
        notification_id: Uuid,
    ) -> Result<(), AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .api
                .get()
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        use crate::schema::notifications;
        diesel::update(
            notifications::table
                .filter(notifications::id.eq(notification_id))
                .filter(notifications::user_id.eq(user_id)),
        )
        .set(notifications::is_read.eq(true))
        .execute(&mut *conn)
        .await
        .map_err(AppError::from)?;

        Ok(())
    }

    async fn mark_all_notifications_as_read(&self, user_id: Uuid) -> Result<(), AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .api
                .get()
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        use crate::schema::notifications;
        diesel::update(notifications::table.filter(notifications::user_id.eq(user_id)))
            .set(notifications::is_read.eq(true))
            .execute(&mut *conn)
            .await
            .map_err(AppError::from)?;

        Ok(())
    }
    async fn check_recent_notification_exists(
        &self,
        user_id_val: Uuid,
        notification_type: &str,
        link_val: &str,
        since_days: i32,
    ) -> Result<bool, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .api
                .get()
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let target_date = chrono::Utc::now() - chrono::Duration::days(i64::from(since_days));

        use crate::schema::notifications;

        let exists: bool = diesel::select(diesel::dsl::exists(
            notifications::table
                .filter(notifications::user_id.eq(user_id_val))
                .filter(notifications::type_.eq(notification_type))
                .filter(notifications::link.eq(link_val))
                .filter(notifications::created_at.ge(target_date)),
        ))
        .get_result(&mut *conn)
        .await
        .unwrap_or(false);

        Ok(exists)
    }

    async fn create_notification(
        &self,
        notification: &crate::models::Notification,
    ) -> Result<(), AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .api
                .get()
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        use crate::schema::notifications;

        #[derive(diesel::Insertable)]
        #[diesel(table_name = notifications)]
        struct NewNotification<'a> {
            id: Uuid,
            user_id: Uuid,
            #[diesel(column_name = "type_")]
            type_val: &'a str,
            title: &'a str,
            content: &'a str,
            is_read: bool,
            link: Option<&'a str>,
            created_at: chrono::DateTime<chrono::Utc>,
        }

        let new_notification = NewNotification {
            id: notification.id,
            user_id: notification.user_id,
            type_val: &notification.r#type,
            title: &notification.title,
            content: &notification.content,
            is_read: notification.is_read,
            link: notification.link.as_deref(),
            created_at: notification.created_at,
        };

        diesel::insert_into(notifications::table)
            .values(&new_notification)
            .execute(&mut *conn)
            .await
            .map_err(AppError::from)?;

        Ok(())
    }

    async fn create_notifications_batch(
        &self,
        notifications: &[crate::models::Notification],
    ) -> Result<(), AppError> {
        if notifications.is_empty() {
            return Ok(());
        }

        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .api
                .get()
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        use crate::schema::notifications;

        #[derive(diesel::Insertable)]
        #[diesel(table_name = notifications)]
        struct NewNotification<'a> {
            id: Uuid,
            user_id: Uuid,
            #[diesel(column_name = "type_")]
            type_val: &'a str,
            title: &'a str,
            content: &'a str,
            is_read: bool,
            link: Option<&'a str>,
            created_at: chrono::DateTime<chrono::Utc>,
        }

        let rows: Vec<_> = notifications
            .iter()
            .map(|n| NewNotification {
                id: n.id,
                user_id: n.user_id,
                type_val: &n.r#type,
                title: &n.title,
                content: &n.content,
                is_read: n.is_read,
                link: n.link.as_deref(),
                created_at: n.created_at,
            })
            .collect();

        diesel::insert_into(notifications::table)
            .values(&rows)
            .execute(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db batch create notifications: {e}")))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diesel_pool::DieselPools;

    #[tokio::test]
    async fn test_repo_initialization() {
        let test_db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://user:pass@localhost:5432/leeattend_test_db".to_string()
        });

        let pools = DieselPools::for_test(&test_db_url);
        let _repo = DieselUserRepository::new(pools);
        // DI connection test passes if it compiles
    }

    /// Empty input must return Ok without executing any query.
    #[tokio::test]
    async fn test_create_notifications_batch_empty_input() {
        let test_db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://user:pass@localhost:5432/leeattend_test_db".to_string()
        });
        let pools = DieselPools::for_test(&test_db_url);
        let repo = DieselUserRepository::new(pools);

        let result = repo.create_notifications_batch(&[]).await;
        assert!(
            result.is_ok(),
            "Empty input should return Ok(()), got: {result:?}"
        );
    }
}
