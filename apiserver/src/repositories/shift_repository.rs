use async_trait::async_trait;
use chrono::NaiveDate;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::{Shift, ShiftVersionHistoryItem};

/// Trait for shift template + version operations (Enterprise SCD Type-2).
///
/// All consumer FKs reference `shift_templates.id` (stable).
/// Version resolution is handled at read time via `current_version_id` (denormalized)
/// or by effective-date range queries for historical lookups.
#[async_trait]
#[cfg_attr(test, mockall::automock)]
pub trait ShiftRepository: Send + Sync {
    /// Create a new shift template with its first version.
    /// Returns the domain Shift populated from the new version.
    async fn create_shift(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        actor_id: Uuid,
        req: &crate::models::CreateShiftRequest,
    ) -> Result<Shift, AppError>;

    /// Get a shift by its template UUID (resolves the current version).
    async fn get_shift_by_id(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        template_id: Uuid,
    ) -> Result<Option<Shift>, AppError>;

    /// List current versions of all active shift templates in the workspace.
    async fn get_workspace_shifts(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        location_id: Option<Uuid>,
        employer_id: Option<Uuid>,
        page: Option<i64>,
        limit: Option<i64>,
    ) -> Result<(Vec<Shift>, i64), AppError>;

    /// Create a new version for an existing template.
    ///
    /// - `change_type = "correct"`: correction — keeps same `effective_from`, supersedes current version
    /// - `change_type = "revise"`: revision — uses `effective_date` as new `effective_from`, closes current version
    async fn create_new_version(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        template_id: Uuid,
        req: &crate::models::UpdateShiftRequest,
        change_type: &str,
        effective_date: NaiveDate,
        created_by: Uuid,
        change_reason: Option<String>,
    ) -> Result<Shift, AppError>;

    /// Soft-delete a template by expiring its current version.
    async fn archive_shift(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        template_id: Uuid,
    ) -> Result<(), AppError>;

    /// Assign a shift template to an employment with date range.
    /// Stable FK — template_id never changes across versions.
    async fn assign_employment_shift(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_id: Uuid,
        template_id: Uuid,
        start_date: NaiveDate,
        end_date: Option<NaiveDate>,
    ) -> Result<(), AppError>;

    /// Get the active shift template for an employment on a given date.
    async fn get_employment_active_shift(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_id: Uuid,
        date: NaiveDate,
    ) -> Result<Option<Shift>, AppError>;

    /// Get all versions of a shift template ordered by version_number DESC,
    /// including their intervals.
    async fn get_shift_versions(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        template_id: Uuid,
    ) -> Result<Vec<ShiftVersionHistoryItem>, AppError>;
}
