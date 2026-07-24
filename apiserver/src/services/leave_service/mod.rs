mod caching;
mod balance;
mod approval;
pub(super) mod conflict;
pub(super) mod submission;
mod workflow;
#[cfg(test)]
mod tests;

use std::sync::Arc;
use uuid::Uuid;

// These re-exports exist only so the test module's `use super::*;` can find
// items that were previously in this file before decomposition.
#[cfg(test)]
pub(super) use chrono::Utc;
#[cfg(test)]
pub(super) use crate::models::{LeaveDurationType, LeaveStatus};

use crate::repositories::leave_balance_repository::LeaveBalanceRepository;
use crate::repositories::leave_request_repository::LeaveRequestRepository;
use crate::repositories::roster::RosterRepository;
use crate::repositories::workspace_config_repository::WorkspaceConfigRepository;
use crate::repositories::workspace_repository::WorkspaceRepository;
use crate::repositories::EmployeeRepository;
use crate::repositories::UserRepository;

pub struct LeaveService {
    leave_balance_repo: Arc<dyn LeaveBalanceRepository>,
    leave_request_repo: Arc<dyn LeaveRequestRepository>,
    employee_repo: Arc<dyn EmployeeRepository>,
    employment_repo: Arc<dyn crate::repositories::EmploymentRepository>,
    roster_repo: Arc<dyn RosterRepository>,
    leave_accrual_log_repo:
        Arc<dyn crate::repositories::leave_accrual_log_repository::LeaveAccrualLogRepository>,
    workspace_config_repo: Arc<dyn WorkspaceConfigRepository>,
    user_repo: Arc<dyn UserRepository>,
    workspace_repo: Arc<dyn WorkspaceRepository>,
    config_cache: Option<crate::cache::ConfigCache>,
    cache: Option<Arc<dyn crate::repositories::CacheRepository>>,
    diesel_pools: Option<crate::diesel_pool::DieselPools>,
}

use crate::errors::AppError;
use crate::models::{LeaveRequest, SubmitLeaveRequest};

/// Minimal interface for submitting leave requests — used to decouple
/// the import worker from the concrete [`LeaveService`] constructor.
#[async_trait::async_trait]
pub trait LeaveSubmission: Send + Sync {
    async fn submit_leave_request(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        user_id: Uuid,
        req: SubmitLeaveRequest,
        auto_approve: bool,
    ) -> Result<LeaveRequest, AppError>;
}

impl LeaveService {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        leave_balance_repo: Arc<dyn LeaveBalanceRepository>,
        leave_request_repo: Arc<dyn LeaveRequestRepository>,
        employee_repo: Arc<dyn EmployeeRepository>,
        employment_repo: Arc<dyn crate::repositories::EmploymentRepository>,
        roster_repo: Arc<dyn RosterRepository>,
        leave_accrual_log_repo: Arc<
            dyn crate::repositories::leave_accrual_log_repository::LeaveAccrualLogRepository,
        >,
        workspace_config_repo: Arc<dyn WorkspaceConfigRepository>,
        user_repo: Arc<dyn UserRepository>,
        workspace_repo: Arc<dyn WorkspaceRepository>,
        cache: Option<Arc<dyn crate::repositories::CacheRepository>>,
    ) -> Self {
        let config_cache = cache.as_ref().map(|c| {
            crate::cache::ConfigCache::new(Arc::clone(c), Arc::clone(&workspace_config_repo))
        });
        Self {
            leave_balance_repo,
            leave_request_repo,
            employee_repo,
            employment_repo,
            roster_repo,
            leave_accrual_log_repo,
            workspace_config_repo,
            user_repo,
            workspace_repo,
            config_cache,
            cache,
            diesel_pools: None,
        }
    }

    /// Inject DieselPools for methods that need manual transaction control.
    #[must_use]
    pub fn with_diesel_pools(mut self, pools: crate::diesel_pool::DieselPools) -> Self {
        self.diesel_pools = Some(pools);
        self
    }
}

#[async_trait::async_trait]
impl LeaveSubmission for LeaveService {
    async fn submit_leave_request(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        user_id: Uuid,
        req: SubmitLeaveRequest,
        auto_approve: bool,
    ) -> Result<LeaveRequest, AppError> {
        self.submit_leave_request(auth_token, user_id, req, auto_approve).await
    }
}
