use std::sync::Arc;

use crate::errors::AppError;
use chrono::{Datelike, Utc};

use uuid::Uuid;

use crate::models::{
    AdjustBalanceRequest, ApproveLeaveRequest, EmployeeLeaveBalance, LeaveDurationType,
    LeaveHistoryItem, LeaveRequest, LeaveStatus, LeaveTransactionType, ShiftRoster,
    SubmitLeaveRequest, WorkspaceLeaveBalance,
};
use crate::repositories::leave_balance_repository::LeaveBalanceRepository;
use crate::repositories::leave_request_repository::LeaveRequestRepository;
use crate::repositories::roster::RosterRepository;
use crate::repositories::EmployeeRepository;
use rust_decimal::prelude::ToPrimitive;

pub struct LeaveService {
    leave_balance_repo: Arc<dyn LeaveBalanceRepository>,
    leave_request_repo: Arc<dyn LeaveRequestRepository>,
    employee_repo: Arc<dyn EmployeeRepository>,
    roster_repo: Arc<dyn RosterRepository>,
    leave_accrual_log_repo:
        Arc<dyn crate::repositories::leave_accrual_log_repository::LeaveAccrualLogRepository>,
}

impl LeaveService {
    pub fn new(
        leave_balance_repo: Arc<dyn LeaveBalanceRepository>,
        leave_request_repo: Arc<dyn LeaveRequestRepository>,
        employee_repo: Arc<dyn EmployeeRepository>,
        roster_repo: Arc<dyn RosterRepository>,
        leave_accrual_log_repo: Arc<
            dyn crate::repositories::leave_accrual_log_repository::LeaveAccrualLogRepository,
        >,
    ) -> Self {
        Self {
            leave_balance_repo,
            leave_request_repo,
            employee_repo,
            roster_repo,
            leave_accrual_log_repo,
        }
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn get_workspace_balances(
        &self,
        workspace_id: Uuid,
        year: i32,
        params: crate::models::PaginationParams,
    ) -> Result<crate::models::PaginatedResponse<WorkspaceLeaveBalance>, AppError> {
        let limit = params.limit.unwrap_or(20).max(1);
        let page = params.page.unwrap_or(1).max(1);
        let offset = (page - 1) * limit;

        let (data, total) = self
            .leave_balance_repo
            .get_workspace_balances(workspace_id, year, limit, offset, params.q)
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "SQLx Error in get_workspace_balances");
                AppError::Internal("Internal error".into())
            })?;

        Ok(crate::models::PaginatedResponse {
            data,
            total: total as usize,
            page,
            limit,
        })
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn get_employee_leave_history(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        year: i32,
        leave_type_id: Option<Uuid>,
    ) -> Result<Vec<LeaveHistoryItem>, AppError> {
        let logs = self
            .leave_accrual_log_repo
            .get_employee_leave_history_logs(workspace_id, employee_id, year, leave_type_id)
            .await?;

        let requests = self
            .leave_request_repo
            .get_employee_leave_history_requests(workspace_id, employee_id, year, leave_type_id)
            .await?;

        let mut history: Vec<LeaveHistoryItem> = Vec::new();

        for log in logs {
            let transaction_type = if log.policy_assignment_id.is_some() {
                LeaveTransactionType::Accrual
            } else {
                LeaveTransactionType::Adjustment
            };
            history.push(LeaveHistoryItem {
                id: log.id,
                date: log.accrual_date,
                transaction_type,
                amount: ToPrimitive::to_f64(&log.amount).unwrap_or(0.0),
                reason: log.notes,
                reference_id: None,
                created_at: log.created_at.unwrap_or_else(chrono::Utc::now),
            });
        }

        for req in requests {
            // Count only weekdays (Mon-Fri) between start_date and end_date
            let mut weekday_count = 0i64;
            let mut d = req.start_date;
            while d <= req.end_date {
                let dow = d.weekday();
                if dow != chrono::Weekday::Sat && dow != chrono::Weekday::Sun {
                    weekday_count += 1;
                }
                if let Some(next) = d.succ_opt() {
                    d = next;
                } else {
                    break;
                }
            }

            // Adjust for partial day leaves
            let amount = if weekday_count > 0 {
                // For future: join duration_type into the history query
                // to support partial-day leave calculation here.
                -(weekday_count as f64)
            } else {
                0.0
            };

            history.push(LeaveHistoryItem {
                id: Uuid::new_v4(),
                date: req.start_date,
                transaction_type: LeaveTransactionType::Usage,
                amount,
                reason: req.reason,
                reference_id: Some(req.id),
                created_at: req.created_at.unwrap_or_else(chrono::Utc::now),
            });
        }

        // Sort by date desc
        history.sort_by(|a, b| b.date.cmp(&a.date));

        Ok(history)
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn adjust_balance(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        req: AdjustBalanceRequest,
    ) -> Result<(), AppError> {
        let amount_dec = rust_decimal::Decimal::from_f64_retain(req.amount)
            .ok_or(AppError::Validation("Invalid request".into()))?;

        self.leave_balance_repo
            .adjust_balance(
                workspace_id,
                employee_id,
                req.leave_type_id,
                amount_dec,
                req.transaction_date,
                &req.reason,
            )
            .await
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn get_employee_balances(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        year: i32,
    ) -> Result<Vec<EmployeeLeaveBalance>, AppError> {
        // Verify employee belongs to workspace
        let _ = self
            .employee_repo
            .get_employee_by_id(employee_id, workspace_id)
            .await?
            .ok_or(AppError::NotFound("Not found".into()))?;

        self.leave_balance_repo
            .get_employee_balances(workspace_id, employee_id, year)
            .await
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn submit_leave_request(
        &self,
        workspace_id: Uuid,
        user_id: Uuid,
        req: SubmitLeaveRequest,
        auto_approve: bool,
    ) -> Result<LeaveRequest, AppError> {
        // Verify employee belongs to workspace
        let _ = self
            .employee_repo
            .get_employee_by_id(req.employee_id, workspace_id)
            .await?
            .ok_or(AppError::NotFound("Not found".into()))?;

        // Validate leave_type_id exists for this workspace
        if !self
            .leave_request_repo
            .check_leave_type_exists(workspace_id, req.leave_type_id)
            .await?
        {
            return Err(AppError::Validation("Invalid request".into()));
        }

        // Validate partial day constraints
        match req.duration_type {
            LeaveDurationType::FirstHalf | LeaveDurationType::SecondHalf => {
                if req.start_date != req.end_date {
                    return Err(AppError::Validation(
                        "Half-day leave must be a single day (start_date == end_date)".into(),
                    ));
                }
            }
            LeaveDurationType::CustomHours => {
                if req.start_time.is_none() || req.end_time.is_none() {
                    return Err(AppError::Validation(
                        "Custom hours leave requires both start_time and end_time".into(),
                    ));
                }
                if req.start_date != req.end_date {
                    return Err(AppError::Validation(
                        "Custom hours leave must be a single day (start_date == end_date)".into(),
                    ));
                }
                if let (Some(st), Some(et)) = (req.start_time, req.end_time) {
                    if st >= et {
                        return Err(AppError::Validation(
                            "start_time must be before end_time".into(),
                        ));
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

        let request = LeaveRequest {
            id: Uuid::new_v4(),
            employee_id: req.employee_id,
            leave_type_id: req.leave_type_id,
            start_date: req.start_date,
            end_date: req.end_date,
            reason: req.reason,
            status,
            duration_type: req.duration_type,
            start_time: req.start_time,
            end_time: req.end_time,
            requested_by_user_id: Some(user_id),
            approved_by_user_id: approved_by,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        self.leave_request_repo
            .create_leave_request(&request)
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "SQLx Error in submit_leave_request");
                AppError::Internal("Internal error".into())
            })?;

        // If auto-approved, apply roster side effects immediately
        if auto_approve {
            self.apply_leave_to_roster(workspace_id, &request).await?;
        }

        Ok(request)
    }

    /// Apply leave-to-roster side effects (create roster entries with "on_leave" status).
    /// Shared between auto-approve (submit) and manual approve flows.
    async fn apply_leave_to_roster(
        &self,
        workspace_id: Uuid,
        request: &LeaveRequest,
    ) -> Result<(), AppError> {
        let leave_type_name = self
            .leave_request_repo
            .get_leave_type_name(workspace_id, request.leave_type_id)
            .await?
            .unwrap_or_else(|| "Leave".to_string());

        let duration_label = match request.duration_type {
            LeaveDurationType::FirstHalf => " (Nửa ngày sáng)",
            LeaveDurationType::SecondHalf => " (Nửa ngày chiều)",
            LeaveDurationType::CustomHours => {
                if let (Some(st), Some(et)) = (request.start_time, request.end_time) {
                    // We'll format inline below
                    &format!(" ({} - {})", st.format("%H:%M"), et.format("%H:%M"))
                } else {
                    ""
                }
            }
            LeaveDurationType::FullDay => "",
        };
        // Clone to avoid borrow issues with format above
        let duration_label = duration_label.to_string();

        let mut current_date = request.start_date;
        while current_date <= request.end_date {
            let roster = ShiftRoster {
                id: Uuid::new_v4(),
                employee_id: request.employee_id,
                shift_id: None,
                date: current_date,
                status: "on_leave".to_string(),
                notes: Some(format!("Nghỉ phép: {leave_type_name}{duration_label}")),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };

            let _ = self.roster_repo.upsert_roster(&roster).await.map_err(|e| {
                tracing::error!(error = %e, "SQLx Error in apply_leave_to_roster");
                AppError::Internal("Internal error".into())
            })?;

            if let Some(next_date) = current_date.succ_opt() {
                current_date = next_date;
            } else {
                break;
            }
        }

        Ok(())
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn approve_leave_request(
        &self,
        workspace_id: Uuid,
        user_id: Uuid,
        request_id: Uuid,
        req: ApproveLeaveRequest,
    ) -> Result<(), AppError> {
        let request = self
            .leave_request_repo
            .get_leave_request_by_id(workspace_id, request_id)
            .await?
            .ok_or(AppError::NotFound("Not found".into()))?;

        // Verify employee of this request belongs to the workspace
        let _ = self
            .employee_repo
            .get_employee_by_id(request.employee_id, workspace_id)
            .await?
            .ok_or(AppError::Forbidden("Forbidden".into()))?;

        self.leave_request_repo
            .update_leave_request_status(workspace_id, request_id, req.status, Some(user_id))
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "SQLx Error in approve_leave_request");
                AppError::Internal("Internal error".into())
            })?;

        // If approved, update roster
        if req.status == LeaveStatus::Approved {
            self.apply_leave_to_roster(workspace_id, &request).await?;
        }

        Ok(())
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn get_employee_leave_requests(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
    ) -> Result<Vec<LeaveRequest>, AppError> {
        // Verify employee belongs to workspace
        let _ = self
            .employee_repo
            .get_employee_by_id(employee_id, workspace_id)
            .await?
            .ok_or(AppError::NotFound("Not found".into()))?;

        self.leave_request_repo
            .get_employee_leave_requests(workspace_id, employee_id)
            .await
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn get_workspace_leave_requests(
        &self,
        workspace_id: Uuid,
        status: Option<LeaveStatus>,
    ) -> Result<Vec<LeaveRequest>, AppError> {
        self.leave_request_repo
            .get_workspace_leave_requests(workspace_id, status)
            .await
    }
}
