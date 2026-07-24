//! Caching helpers for LeaveService.
//!
//! Uses [`crate::repositories::CacheRepositoryExt`] to wrap database lookups
//! with Redis-backed caching (get-or-fetch pattern), falling back to direct DB
//! queries when no cache is configured.

use crate::errors::AppError;

impl super::LeaveService {
    pub(super) async fn get_employment_cached(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_id: uuid::Uuid,
    ) -> Result<Option<crate::models::employment::Employment>, AppError> {
        use crate::repositories::CacheRepositoryExt;

        if let Some(ref cache) = self.cache {
            let key = format!("cache:employment:{employment_id}");
            let repo = std::sync::Arc::clone(&self.employment_repo);
            let token = *auth_token;

            cache
                .get_or_fetch(&key, crate::cache::DEFAULT_CACHE_TTL, || async move {
                    repo.get_employment_by_id(&token, employment_id)
                        .await
                        .map_err(|e| format!("Failed to fetch employment: {e}"))
                })
                .await
                .map_err(AppError::Internal)
        } else {
            self.employment_repo
                .get_employment_by_id(auth_token, employment_id)
                .await
        }
    }

    pub(super) async fn check_leave_type_exists_cached(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        leave_type_id: uuid::Uuid,
    ) -> Result<bool, AppError> {
        use crate::repositories::CacheRepositoryExt;

        if let Some(ref cache) = self.cache {
            let key = format!("cache:leave_type_exists:{leave_type_id}");
            let repo = std::sync::Arc::clone(&self.leave_request_repo);
            let token = *auth_token;

            cache
                .get_or_fetch(&key, crate::cache::DEFAULT_CACHE_TTL, || async move {
                    repo.check_leave_type_exists(&token, leave_type_id)
                        .await
                        .map_err(|e| format!("Failed to check leave type exists: {e}"))
                })
                .await
                .map_err(AppError::Internal)
        } else {
            self.leave_request_repo
                .check_leave_type_exists(auth_token, leave_type_id)
                .await
        }
    }

    pub(super) async fn get_employee_leave_requests_cached(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: uuid::Uuid,
    ) -> Result<Vec<crate::models::LeaveRequest>, AppError> {
        use crate::repositories::CacheRepositoryExt;

        if let Some(ref cache) = self.cache {
            let key = format!("cache:employee_leaves:{employee_id}");
            let repo = std::sync::Arc::clone(&self.leave_request_repo);
            let token = *auth_token;

            cache
                .get_or_fetch(&key, crate::cache::DEFAULT_CACHE_TTL, || async move {
                    repo.get_employee_leave_requests(&token, employee_id)
                        .await
                        .map_err(|e| format!("Failed to fetch employee leaves: {e}"))
                })
                .await
                .map_err(AppError::Internal)
        } else {
            self.leave_request_repo
                .get_employee_leave_requests(auth_token, employee_id)
                .await
        }
    }
}
