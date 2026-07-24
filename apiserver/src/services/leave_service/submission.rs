//! Leave request submission for [`LeaveService`].
//!
//! The main write path: validates the request, invalidates the conflict cache,
//! runs conflict detection, performs a transaction-wrapped insert with optional
//! auto-cancel of conflicting leaves, and dispatches post-commit notifications.

use crate::errors::AppError;
use crate::utils::i18n::I18nKey;
use chrono::Utc;
use diesel::result::DatabaseErrorKind;
use diesel::result::Error as DieselError;
use diesel_async::AsyncConnection;
use std::sync::Arc;
use uuid::Uuid;

use crate::models::{LeaveDurationType, LeaveRequest, LeaveStatus, SubmitLeaveRequest};

/// Result of resolving submitter identity and employment for leave submission.
pub struct SubmitterInfo {
    pub target_employee_id: Uuid,
    pub caller_employee_id: Option<Uuid>,
    pub is_on_behalf: bool,
}

impl super::LeaveService {
    /// Resolve submitter identity from user/employment IDs.
    ///
    /// Looks up the employment to find the target employee, and resolves the
    /// caller's employee record to determine if this is a self-submit or
    /// on-behalf submission.  Employment lookup uses the cached path so a
    /// subsequent `submit_leave_request` call will hit cache.
    pub async fn resolve_submitter_info(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        user_id: Uuid,
        employment_id: Uuid,
    ) -> Result<SubmitterInfo, AppError> {
        let employment = self
            .get_employment_cached(auth_token, employment_id)
            .await?
            .ok_or_else(|| AppError::not_found(I18nKey::EmploymentNotFound))?;
        let target_employee_id = employment.employee_id();

        let caller_employee_id = self
            .employee_repo
            .get_employee_id_by_user_id(auth_token, user_id)
            .await?;

        let is_on_behalf = match caller_employee_id {
            Some(emp_id) => emp_id != target_employee_id,
            None => true,
        };

        Ok(SubmitterInfo {
            target_employee_id,
            caller_employee_id,
            is_on_behalf,
        })
    }

    #[tracing::instrument(skip_all, err)]
    #[allow(clippy::too_many_lines)]
    pub async fn submit_leave_request(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        user_id: Uuid,
        req: SubmitLeaveRequest,
        auto_approve: bool,
    ) -> Result<LeaveRequest, AppError> {
        // Verify employment belongs to workspace
        let employment = self
            .get_employment_cached(
                auth_token,
                req.employment_id,
            )
            .await?
            .ok_or_else(|| AppError::not_found(I18nKey::EmploymentNotFound))?;
        let employee_id = employment.employee_id();

        // Validate leave_type_id exists for this workspace
        if !self
            .check_leave_type_exists_cached(
                auth_token,
                req.leave_type_id,
            )
            .await?
        {
            return Err(AppError::validation(I18nKey::Validation));
        }

        // Validate partial day constraints
        match req.duration_type {
            LeaveDurationType::FirstHalf | LeaveDurationType::SecondHalf => {
                if req.start_date != req.end_date {
                    return Err(AppError::validation(I18nKey::LeaveHalfDaySingleDay));
                }
            }
            LeaveDurationType::CustomHours => {
                if req.start_time.is_none() || req.end_time.is_none() {
                    return Err(AppError::validation(I18nKey::LeaveCustomHoursMissingTime));
                }
                if req.start_date != req.end_date {
                    return Err(AppError::validation(I18nKey::LeaveCustomHoursSingleDay));
                }
                if let (Some(st), Some(et)) = (req.start_time, req.end_time) {
                    if st >= et {
                        return Err(AppError::validation(I18nKey::LeaveStartTimeBeforeEndTime));
                    }
                }
            }
            LeaveDurationType::FullDay => {}
        }

        let (status, approved_by) = if auto_approve {
            (LeaveStatus::PreApproved, Some(user_id))
        } else {
            (LeaveStatus::Pending, None)
        };

        let attachment_file_ids = req
            .attachment_file_ids
            .map(|ids| ids.into_iter().map(Some).collect());

        let request = LeaveRequest::new(
            Uuid::new_v4(),
            req.employment_id,
            req.leave_type_id,
            req.start_date,
            req.end_date,
            req.reason,
            status,
            req.duration_type,
            req.start_time,
            req.end_time,
            Some(user_id),
            approved_by,
            Utc::now(),
            Utc::now(),
            attachment_file_ids,
        );

        // Phase 0: Invalidate cached employee leaves to force fresh DB read.
        if let Some(ref cache) = self.cache {
            let key = format!("cache:employee_leaves:{employee_id}");
            let _ = cache.delete(&key).await;
        }

        // Phase 1: Detect conflicts (reads fresh data from DB since cache was invalidated).
        let conflict_info = self
            .detect_conflicts(auth_token, employee_id, &request)
            .await?;

        // Phase 2: Transaction-wrapped critical writes
        if let Some(ref pools) = self.diesel_pools {
            let mut conn = pools.tenant_transaction_conn(auth_token).await?;
            let request = request.clone();
            let repo = Arc::clone(&self.leave_request_repo);
            let auth_token_clone = auth_token;
            let conflicting_ids: Vec<Uuid> = conflict_info
                .conflicting_leaves
                .iter()
                .map(LeaveRequest::id)
                .collect();
            let should_cancel = conflict_info.should_auto_cancel;

            let tx_result = conn.transaction::<_, AppError, _>(|pg_conn| {
                let req = request.clone();
                let repo = Arc::clone(&repo);
                let auth = auth_token_clone;
                let ids = conflicting_ids.clone();
                let cancel = should_cancel;
                Box::pin(async move {
                    repo.create_leave_request_tx(pg_conn, auth, &req).await?;
                    if cancel && !ids.is_empty() {
                        repo.bulk_cancel_leave_requests_tx(pg_conn, auth, &ids)
                            .await?;
                    }
                    Ok(())
                })
            })
            .await;

            match tx_result {
                Ok(()) => {}
                Err(AppError::Diesel(ref e)) if is_exclusion_violation(e) => {
                    tracing::warn!(
                        employee_id = %employee_id,
                        start = %request.start_date(),
                        end = %request.end_date(),
                        "Leave overlap prevented by DB exclusion constraint"
                    );
                    return Err(AppError::conflict(I18nKey::Conflict));
                }
                Err(e) => return Err(e),
            }
        } else {
            // Fallback: no DieselPools injected — use separate non-transactional calls
            self.leave_request_repo
                .create_leave_request(
                    auth_token,
                    &request,
                )
                .await?;
        }

        // Phase 3: Handle non-cancel conflict actions
        self.handle_conflict_post_tx(auth_token, &request, &conflict_info)
            .await?;

        Ok(request)
    }
}

/// Returns true if the diesel error is a PostgreSQL exclusion-constraint violation
/// (SQLSTATE 23P01). This can happen when two concurrent leave submissions create
/// overlapping approved/pre-approved leaves despite application-level conflict checks.
pub(super) fn is_exclusion_violation(err: &DieselError) -> bool {
    matches!(
        err,
        DieselError::DatabaseError(DatabaseErrorKind::CheckViolation, _)
    ) || matches!(
        err,
        DieselError::DatabaseError(DatabaseErrorKind::UniqueViolation, _)
    )
}
