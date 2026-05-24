#![allow(unused_variables)]
//! System-level authentication context.
//!
//! `SystemContext` is an Axum extractor for endpoints that require **platform-level**
//! authority, independent of any specific workspace. It resolves which system roles
//! a user holds (platform_admin, support, billing_admin) and provides typed
//! guard methods.
//!
//! # Usage
//!
//! ```rust,ignore
//! pub async fn create_workspace(
//!     ctx: SystemContext,
//!     ...
//! ) -> AppResult<...> {
//!     ctx.require_role(SystemRoleType::PlatformAdmin)?;
//!     // proceed
//! }
//! ```
//!
//! # Caching
//!
//! System roles are cached in Redis under `cache:system:roles:{user_id}` with a
//! 30-minute TTL. Invalidation occurs when system roles are assigned or revoked.

use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

use crate::repositories::CacheRepositoryExt;
use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::errors::{AppError, AppResult};
use crate::models::enums::SystemRoleType;
use crate::{AppState, UserId};

/// Resolved system-level context for a platform admin request.
pub struct SystemContext {
    pub user_id: Uuid,
    pub roles: HashSet<SystemRoleType>,
}

/// Serializable representation stored in Redis cache.
#[derive(Serialize, Deserialize)]
struct CachedSystemRoles {
    roles: Vec<String>,
}

const SYSTEM_ROLE_CACHE_TTL: Duration = Duration::from_secs(30 * 60); // 30 minutes

impl SystemContext {
    /// Returns true if the user holds the given system role.
    pub fn has_role(&self, role: SystemRoleType) -> bool {
        self.roles.contains(&role)
    }

    /// Returns `Err(Forbidden)` if the user does NOT hold the required role.
    pub fn require_role(&self, role: SystemRoleType) -> AppResult<()> {
        if self.has_role(role) {
            Ok(())
        } else {
            Err(AppError::Forbidden(format!("Requires system role: {role}")))
        }
    }

    /// Returns `Err(Forbidden)` if the user holds NONE of the listed roles.
    pub fn require_any_role(&self, roles: &[SystemRoleType]) -> AppResult<()> {
        if roles.iter().any(|r| self.has_role(*r)) {
            Ok(())
        } else {
            let names: Vec<String> = roles.iter().map(std::string::ToString::to_string).collect();
            Err(AppError::Forbidden(format!(
                "Requires one of system roles: [{}]",
                names.join(", ")
            )))
        }
    }

    /// Fetch the user's system roles from the DB (no cache layer).
    async fn fetch_system_roles_from_db(
        user_id: Uuid,
        state: &Arc<AppState>,
    ) -> AppResult<HashSet<SystemRoleType>> {
        let mut conn = state
            .infra
            .diesel
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        #[derive(diesel::QueryableByName)]
        struct RoleRow {
            #[diesel(sql_type = diesel::sql_types::Text)]
            name: String,
        }

        use diesel_async::RunQueryDsl;

        let rows: Vec<RoleRow> = crate::system_crud_query!(
            &mut conn,
            r#"
            SELECT sr.name
              FROM user_system_roles usr
              JOIN system_roles sr ON sr.id = usr.role_id
             WHERE usr.user_id = $1
            "#,
        )
        .bind::<diesel::sql_types::Uuid, _>(user_id)
        .load(&mut *conn)
        .await
        .map_err(|e| {
            tracing::error!(user_id = %user_id, error = %e, "Failed to fetch system roles");
            AppError::Internal("Database error fetching system roles".to_string())
        })?;

        let roles = rows
            .into_iter()
            .filter_map(|r| {
                use std::str::FromStr;
                SystemRoleType::from_str(&r.name).ok()
            })
            .collect();

        Ok(roles)
    }

    /// Returns cache key for this user's system roles.
    fn cache_key(user_id: Uuid) -> String {
        format!("cache:system:roles:{user_id}")
    }

    /// Invalidate the system role cache for a specific user.
    /// Call after assigning or revoking a system role.
    pub async fn invalidate_cache(user_id: Uuid, state: &Arc<AppState>) {
        if let Some(cache) = &state.infra.cache {
            let _ = cache.delete(&Self::cache_key(user_id)).await;
        }
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for SystemContext
where
    S: Send + Sync,
    Arc<AppState>: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let UserId(user_id) = UserId::from_request_parts(parts, state)
            .await
            .map_err(|(_, msg)| AppError::Auth(msg))?;

        let app_state = Arc::<AppState>::from_ref(state);
        let cache_key = Self::cache_key(user_id);

        let roles: HashSet<SystemRoleType> = if let Some(cache) = &app_state.infra.cache {
            if let Some(cached) = cache.get::<CachedSystemRoles>(&cache_key).await {
                // Deserialize from cached string names
                cached
                    .roles
                    .into_iter()
                    .filter_map(|name| {
                        use std::str::FromStr;
                        SystemRoleType::from_str(&name).ok()
                    })
                    .collect()
            } else {
                // Cache miss: hit DB and populate cache
                let fresh = Self::fetch_system_roles_from_db(user_id, &app_state).await?;
                let payload = CachedSystemRoles {
                    roles: fresh.iter().map(std::string::ToString::to_string).collect(),
                };
                let _ = cache.set(&cache_key, &payload, SYSTEM_ROLE_CACHE_TTL).await;
                fresh
            }
        } else {
            // No Redis: always go to DB
            Self::fetch_system_roles_from_db(user_id, &app_state).await?
        };

        // Any authenticated user can pass extraction — enforcement is done
        // by calling `ctx.require_role(...)` inside each handler.
        Ok(SystemContext { user_id, roles })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_ctx(roles: &[SystemRoleType]) -> SystemContext {
        SystemContext {
            user_id: Uuid::new_v4(),
            roles: roles.iter().copied().collect(),
        }
    }

    #[test]
    fn test_has_role_present() {
        let ctx = make_ctx(&[SystemRoleType::PlatformAdmin]);
        assert!(ctx.has_role(SystemRoleType::PlatformAdmin));
    }

    #[test]
    fn test_has_role_absent() {
        let ctx = make_ctx(&[SystemRoleType::Support]);
        assert!(!ctx.has_role(SystemRoleType::PlatformAdmin));
        assert!(!ctx.has_role(SystemRoleType::BillingAdmin));
    }

    #[test]
    fn test_has_role_empty_roles() {
        let ctx = make_ctx(&[]);
        assert!(!ctx.has_role(SystemRoleType::PlatformAdmin));
        assert!(!ctx.has_role(SystemRoleType::Support));
        assert!(!ctx.has_role(SystemRoleType::BillingAdmin));
    }

    #[test]
    fn test_has_role_all_three_variants() {
        let ctx = make_ctx(&[
            SystemRoleType::PlatformAdmin,
            SystemRoleType::Support,
            SystemRoleType::BillingAdmin,
        ]);
        assert!(ctx.has_role(SystemRoleType::PlatformAdmin));
        assert!(ctx.has_role(SystemRoleType::Support));
        assert!(ctx.has_role(SystemRoleType::BillingAdmin));
    }

    // ── require_role ─────────────────────────────────────────────────────────

    #[test]
    fn test_require_role_ok_when_present() {
        let ctx = make_ctx(&[SystemRoleType::PlatformAdmin]);
        assert!(ctx.require_role(SystemRoleType::PlatformAdmin).is_ok());
    }

    #[test]
    fn test_require_role_err_when_absent() {
        let ctx = make_ctx(&[SystemRoleType::Support]);
        assert!(matches!(
            ctx.require_role(SystemRoleType::PlatformAdmin),
            Err(AppError::Forbidden(_))
        ));
    }

    #[test]
    fn test_require_role_error_message_contains_role_name() {
        let ctx = make_ctx(&[]);
        let Err(AppError::Forbidden(msg)) = ctx.require_role(SystemRoleType::BillingAdmin) else {
            panic!("Expected Forbidden");
        };
        assert!(msg.contains("billing_admin"), "msg={msg}");
    }

    #[test]
    fn test_require_role_support_cannot_elevate_to_platform_admin() {
        // Core security: Support must NOT be able to call PlatformAdmin-only endpoints.
        let ctx = make_ctx(&[SystemRoleType::Support]);
        assert!(ctx.require_role(SystemRoleType::PlatformAdmin).is_err());
        assert!(ctx.require_role(SystemRoleType::BillingAdmin).is_err());
    }

    #[test]
    fn test_require_role_platform_admin_no_implicit_elevation_to_other_roles() {
        // PlatformAdmin does NOT auto-have Support/BillingAdmin unless explicitly assigned.
        let ctx = make_ctx(&[SystemRoleType::PlatformAdmin]);
        assert!(ctx.require_role(SystemRoleType::PlatformAdmin).is_ok());
        assert!(ctx.require_role(SystemRoleType::Support).is_err());
        assert!(ctx.require_role(SystemRoleType::BillingAdmin).is_err());
    }

    // ── require_any_role ─────────────────────────────────────────────────────

    #[test]
    fn test_require_any_role_ok_if_one_matches() {
        let ctx = make_ctx(&[SystemRoleType::Support]);
        assert!(ctx
            .require_any_role(&[SystemRoleType::PlatformAdmin, SystemRoleType::Support])
            .is_ok());
    }

    #[test]
    fn test_require_any_role_err_if_none_match() {
        let ctx = make_ctx(&[SystemRoleType::BillingAdmin]);
        let result =
            ctx.require_any_role(&[SystemRoleType::PlatformAdmin, SystemRoleType::Support]);
        assert!(matches!(result, Err(AppError::Forbidden(_))));
    }

    #[test]
    fn test_require_any_role_empty_list_always_fails() {
        // Calling with an empty required list is a programmer error — it must fail.
        let ctx = make_ctx(&[SystemRoleType::PlatformAdmin]);
        let result = ctx.require_any_role(&[]);
        assert!(matches!(result, Err(AppError::Forbidden(_))));
    }

    // ── Capability helpers on SystemRoleType ─────────────────────────────────

    #[test]
    fn test_can_create_workspace_only_platform_admin() {
        assert!(SystemRoleType::PlatformAdmin.can_create_workspace());
        assert!(!SystemRoleType::Support.can_create_workspace());
        assert!(!SystemRoleType::BillingAdmin.can_create_workspace());
    }

    #[test]
    fn test_can_monitor_infra_platform_and_support() {
        assert!(SystemRoleType::PlatformAdmin.can_monitor_infra());
        assert!(SystemRoleType::Support.can_monitor_infra());
        assert!(!SystemRoleType::BillingAdmin.can_monitor_infra());
    }

    #[test]
    fn test_can_manage_billing_platform_and_billing() {
        assert!(SystemRoleType::PlatformAdmin.can_manage_billing());
        assert!(SystemRoleType::BillingAdmin.can_manage_billing());
        assert!(!SystemRoleType::Support.can_manage_billing());
    }

    // ── cache_key format ─────────────────────────────────────────────────────

    #[test]
    fn test_cache_key_format() {
        let uid = Uuid::nil();
        let key = SystemContext::cache_key(uid);
        assert_eq!(key, format!("cache:system:roles:{uid}"));
        assert!(key.starts_with("cache:system:roles:"));
    }

    #[test]
    fn test_cache_key_unique_per_user() {
        let uid1 = Uuid::new_v4();
        let uid2 = Uuid::new_v4();
        assert_ne!(
            SystemContext::cache_key(uid1),
            SystemContext::cache_key(uid2)
        );
    }

    // ── SystemRoleType serde & display ───────────────────────────────────────

    #[test]
    fn test_system_role_type_serde_roundtrip() {
        for role in [
            SystemRoleType::PlatformAdmin,
            SystemRoleType::Support,
            SystemRoleType::BillingAdmin,
        ] {
            let json = serde_json::to_string(&role).unwrap();
            let back: SystemRoleType = serde_json::from_str(&json).unwrap();
            assert_eq!(role, back);
        }
    }

    #[test]
    fn test_system_role_type_json_values() {
        assert_eq!(
            serde_json::to_string(&SystemRoleType::PlatformAdmin).unwrap(),
            "\"platform_admin\""
        );
        assert_eq!(
            serde_json::to_string(&SystemRoleType::Support).unwrap(),
            "\"support\""
        );
        assert_eq!(
            serde_json::to_string(&SystemRoleType::BillingAdmin).unwrap(),
            "\"billing_admin\""
        );
    }

    #[test]
    fn test_system_role_from_string_valid() {
        assert_eq!(
            SystemRoleType::from("platform_admin".to_string()),
            SystemRoleType::PlatformAdmin
        );
        assert_eq!(
            SystemRoleType::from("support".to_string()),
            SystemRoleType::Support
        );
        assert_eq!(
            SystemRoleType::from("billing_admin".to_string()),
            SystemRoleType::BillingAdmin
        );
    }

    #[test]
    fn test_system_role_from_invalid_string_falls_back_to_support() {
        // Invalid strings must fall back to Support (lowest privilege), never PlatformAdmin.
        for bad in ["", "ADMIN", "root", "owner", "platform_admin;DROP TABLE--"] {
            let result = SystemRoleType::from(bad.to_string());
            assert_eq!(
                result,
                SystemRoleType::Support,
                "Bad input '{bad}' must fall back to Support, not PlatformAdmin"
            );
        }
    }

    #[test]
    fn test_system_role_display_is_snake_case() {
        assert_eq!(SystemRoleType::PlatformAdmin.to_string(), "platform_admin");
        assert_eq!(SystemRoleType::Support.to_string(), "support");
        assert_eq!(SystemRoleType::BillingAdmin.to_string(), "billing_admin");
    }
}
