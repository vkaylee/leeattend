//! Conflict detection and resolution for [`LeaveService`].
//!
//! Handles pre-transaction conflict detection (leave overlap, shift overlap),
//! post-transaction notification dispatch, and the approve-time conflict handler.
//! The conflict policy (`auto_cancel` vs `notify`) is read from workspace config.

use crate::errors::AppError;
use crate::utils::i18n::I18nKey;
use chrono::Utc;
use uuid::Uuid;

use crate::models::{LeaveRequest, LeaveStatus};

/// Pre-transaction conflict detection result used by submit_leave_request
/// to separate detection (read-only) from atomic write handling.
pub(super) struct ConflictInfo {
    pub conflicting_leaves: Vec<LeaveRequest>,
    pub conflicting_shifts: Vec<crate::models::ShiftRoster>,
    pub should_auto_cancel: bool,
}

impl super::LeaveService {
    /// Resolve the `leave.conflict_policy` config value for a workspace,
    /// falling back to "notify" when the config is missing or unreadable.
    /// Shared by `detect_conflicts` and `check_and_handle_conflicts_inner`.
    async fn resolve_conflict_policy(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        workspace_id: Uuid,
        reference_date: chrono::NaiveDate,
    ) -> Result<String, AppError> {
        let config_entry = if let Some(ref cache) = self.config_cache {
            match cache
                .get_effective_config(
                    auth_token,
                    workspace_id,
                    "leave.conflict_policy",
                    "workspace",
                    None,
                    reference_date,
                )
                .await
            {
                Ok(entry) => entry,
                Err(e) => {
                    tracing::warn!(error = %e, "Failed to get leave.conflict_policy from cache, falling back to DB");
                    self.workspace_config_repo
                        .get_effective_config(
                            auth_token,
                            workspace_id,
                            "leave.conflict_policy",
                            "workspace",
                            None,
                            reference_date,
                        )
                        .await?
                }
            }
        } else {
            self.workspace_config_repo
                .get_effective_config(
                    auth_token,
                    workspace_id,
                    "leave.conflict_policy",
                    "workspace",
                    None,
                    reference_date,
                )
                .await?
        };

        Ok(config_entry
            .and_then(|entry| entry.value_text().map(ToString::to_string))
            .unwrap_or_else(|| "notify".to_string()))
    }

    /// Detect conflicts for a leave request (read-only, no transaction needed).
    /// Returns conflict info used by both the transaction phase and post-tx notification phase.
    #[allow(clippy::too_many_lines)]
    pub(super) async fn detect_conflicts(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        req: &LeaveRequest,
    ) -> Result<ConflictInfo, AppError> {
        let auth_token = crate::diesel_pool::TenantAuthToken::system_override(workspace_id);

        // Fetch other leave requests of the employee
        let other_requests = self
            .get_employee_leave_requests_cached(&auth_token, employee_id)
            .await?;

        let mut conflicting_leaves = Vec::new();
        for r in other_requests {
            if r.id() == req.id() {
                continue;
            }
            if (r.status() == LeaveStatus::Approved || r.status() == LeaveStatus::PreApproved)
                && r.start_date() <= req.end_date()
                && r.end_date() >= req.start_date()
            {
                conflicting_leaves.push(r);
            }
        }

        // Fetch rosters for the date range
        let (rosters, _) = self
            .roster_repo
            .get_employment_rosters(
                &auth_token,
                req.employment_id(),
                req.start_date(),
                req.end_date(),
                None,
                None,
            )
            .await?;

        let mut conflicting_shifts = Vec::new();
        for roster in rosters {
            if roster.shift_id.is_some() {
                conflicting_shifts.push(roster);
            }
        }

        if conflicting_leaves.is_empty() && conflicting_shifts.is_empty() {
            return Ok(ConflictInfo {
                conflicting_leaves: Vec::new(),
                conflicting_shifts: Vec::new(),
                should_auto_cancel: false,
            });
        }

        // Get workspace conflict policy
        let policy = self
            .resolve_conflict_policy(&auth_token, workspace_id, req.start_date())
            .await?;

        let should_auto_cancel = policy == "auto_cancel"
            && (req.status() == LeaveStatus::Approved || req.status() == LeaveStatus::PreApproved);

        Ok(ConflictInfo {
            conflicting_leaves,
            conflicting_shifts,
            should_auto_cancel,
        })
    }

    /// Post-transaction conflict handling: notifications for shift conflicts
    /// or full notify policy path (after the leave request is already committed).
    pub(super) async fn handle_conflict_post_tx(
        &self,
        workspace_id: Uuid,
        req: &LeaveRequest,
        info: &ConflictInfo,
    ) -> Result<(), AppError> {
        if info.conflicting_leaves.is_empty() && info.conflicting_shifts.is_empty() {
            return Ok(());
        }

        if info.should_auto_cancel {
            // Leaves were already cancelled in the transaction.
            // If there's a shift conflict, we still notify because we can't cancel the shift.
            if !info.conflicting_shifts.is_empty() {
                self.send_conflict_notifications(
                    workspace_id,
                    req,
                    Vec::new(),
                    info.conflicting_shifts.clone(),
                )
                .await?;
            }
        } else {
            // Policy is notify, or request is Pending: send notifications
            self.send_conflict_notifications(
                workspace_id,
                req,
                info.conflicting_leaves.clone(),
                info.conflicting_shifts.clone(),
            )
            .await?;
        }

        Ok(())
    }

    #[allow(clippy::too_many_lines)]
    pub(super) async fn check_and_handle_conflicts(
        &self,
        workspace_id: Uuid,
        req: &LeaveRequest,
    ) -> Result<(), AppError> {
        self.check_and_handle_conflicts_inner(workspace_id, req, None)
            .await
    }

    #[allow(clippy::too_many_lines)]
    pub(super) async fn check_and_handle_conflicts_inner(
        &self,
        workspace_id: Uuid,
        req: &LeaveRequest,
        tx_conn: Option<&mut crate::diesel_pool::TransactionConn>,
    ) -> Result<(), AppError> {
        let auth_token = crate::diesel_pool::TenantAuthToken::system_override(workspace_id);

        // 1. Get employee ID from employment
        let employment = self
            .get_employment_cached(&auth_token, req.employment_id())
            .await?
            .ok_or_else(|| AppError::not_found(I18nKey::EmploymentNotFound))?;
        let employee_id = employment.employee_id();

        // 2. Fetch other leave requests of the employee
        let other_requests = self
            .get_employee_leave_requests_cached(&auth_token, employee_id)
            .await?;

        // Filter for conflicting leave requests (Approved or PreApproved, overlapping dates, excluding self)
        let mut conflicting_leaves = Vec::new();
        for r in other_requests {
            if r.id() == req.id() {
                continue;
            }
            if r.status() == LeaveStatus::Approved || r.status() == LeaveStatus::PreApproved {
                // Overlap check
                if r.start_date() <= req.end_date() && r.end_date() >= req.start_date() {
                    conflicting_leaves.push(r);
                }
            }
        }

        // 3. Fetch rosters for the date range
        let (rosters, _) = self
            .roster_repo
            .get_employment_rosters(
                &auth_token,
                req.employment_id(),
                req.start_date(),
                req.end_date(),
                None,
                None,
            )
            .await?;

        // Filter for conflicting shift rosters (has shift_id scheduled)
        let mut conflicting_shifts = Vec::new();
        for roster in rosters {
            if roster.shift_id.is_some() {
                conflicting_shifts.push(roster);
            }
        }

        // If no conflict, we are done
        if conflicting_leaves.is_empty() && conflicting_shifts.is_empty() {
            return Ok(());
        }

        // 4. Get workspace conflict policy
        let policy = self
            .resolve_conflict_policy(&auth_token, workspace_id, req.start_date())
            .await?;

        // 5. Handle conflict based on policy and current request status
        if policy == "auto_cancel"
            && (req.status() == LeaveStatus::Approved || req.status() == LeaveStatus::PreApproved)
        {
            let conflicting_ids: Vec<Uuid> =
                conflicting_leaves.iter().map(LeaveRequest::id).collect();
            if let Some(conn) = tx_conn {
                self.leave_request_repo
                    .bulk_cancel_leave_requests_tx(conn, &auth_token, &conflicting_ids)
                    .await?;
            } else {
                self.leave_request_repo
                    .bulk_cancel_leave_requests(&auth_token, &conflicting_ids)
                    .await?;
            }
            if !conflicting_shifts.is_empty() {
                self.send_conflict_notifications(workspace_id, req, Vec::new(), conflicting_shifts)
                    .await?;
            }
        } else {
            // Policy is notify, or request is Pending: send notifications
            self.send_conflict_notifications(
                workspace_id,
                req,
                conflicting_leaves,
                conflicting_shifts,
            )
            .await?;
        }

        Ok(())
    }

    pub(super) async fn send_conflict_notifications(
        &self,
        workspace_id: Uuid,
        req: &LeaveRequest,
        conflicting_leaves: Vec<LeaveRequest>,
        conflicting_shifts: Vec<crate::models::ShiftRoster>,
    ) -> Result<(), AppError> {
        let mut user_ids_to_notify = std::collections::HashSet::new();

        // 1. Notify the submitter of the request
        if let Some(user_id) = req.requested_by_user_id() {
            user_ids_to_notify.insert(user_id);
        }

        // 2. Notify all admins/HR
        let admins = self
            .workspace_repo
            .get_admins_with_permissions(workspace_id, &["leave.approve", "leave.manage"])
            .await
            .unwrap_or_else(|e| {
                tracing::error!(
                    workspace_id = %workspace_id,
                    error = %e,
                    "Leave notification: get_admins_with_permissions failed — no admins will be notified"
                );
                Vec::new()
            });

        for admin_id in admins {
            user_ids_to_notify.insert(admin_id);
        }

        // 3. Format message content
        let mut content_parts = Vec::new();
        if !conflicting_leaves.is_empty() {
            let count = conflicting_leaves.len();
            content_parts.push(format!("trùng với {count} đơn nghỉ phép khác đã duyệt"));
        }
        if !conflicting_shifts.is_empty() {
            let count = conflicting_shifts.len();
            content_parts.push(format!("trùng với {count} ca làm việc đã phân công"));
        }

        let details = content_parts.join(" và ");
        let title = "Xung đột lịch nghỉ phép".to_string();
        let content = format!(
            "Yêu cầu nghỉ phép từ ngày {} đến {} {details}",
            req.start_date().format("%d/%m/%Y"),
            req.end_date().format("%d/%m/%Y")
        );
        let link = "/workspace/leaves".to_string();

        // 4. Create notification records in batch to avoid N+1
        let notifications: Vec<crate::models::Notification> = user_ids_to_notify
            .iter()
            .map(|&target_user_id| crate::models::Notification {
                id: Uuid::new_v4(),
                user_id: target_user_id,
                r#type: "leave_conflict".to_string(),
                title: title.clone(),
                content: content.clone(),
                is_read: false,
                link: Some(link.clone()),
                created_at: Utc::now(),
            })
            .collect();

        self.user_repo
            .create_notifications_batch(&notifications)
            .await?;

        Ok(())
    }
}
