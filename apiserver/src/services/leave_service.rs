use std::sync::Arc;

use crate::errors::AppError;
use chrono::{Datelike, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{
    AdjustBalanceRequest, ApproveLeaveRequest, EmployeeLeaveBalance, LeaveHistoryItem,
    LeaveRequest, LeaveStatus, LeaveTransactionType, ShiftRoster, SubmitLeaveRequest,
    WorkspaceLeaveBalance,
};
use crate::repositories::leave_balance_repository::LeaveBalanceRepository;
use crate::repositories::leave_request_repository::LeaveRequestRepository;
use crate::repositories::roster_repository::RosterRepository;
use crate::repositories::EmployeeRepository;
use rust_decimal::prelude::ToPrimitive;

pub struct LeaveService {
    leave_balance_repo: Arc<dyn LeaveBalanceRepository>,
    leave_request_repo: Arc<dyn LeaveRequestRepository>,
    employee_repo: Arc<dyn EmployeeRepository>,
    roster_repo: Arc<dyn RosterRepository>,
    /// Kept for raw SQL queries
    pool: PgPool,
}

impl LeaveService {
    pub fn new(
        leave_balance_repo: Arc<dyn LeaveBalanceRepository>,
        leave_request_repo: Arc<dyn LeaveRequestRepository>,
        employee_repo: Arc<dyn EmployeeRepository>,
        roster_repo: Arc<dyn RosterRepository>,
        pool: PgPool,
    ) -> Self {
        Self { leave_balance_repo, leave_request_repo, employee_repo, roster_repo, pool }
    }

    pub async fn get_workspace_balances(
        &self,
        workspace_id: Uuid,
        year: i32,
        params: crate::models::PaginationParams,
    ) -> Result<crate::models::PaginatedResponse<WorkspaceLeaveBalance>, AppError> {
        let limit = params.limit.unwrap_or(20).max(1);
        let page = params.page.unwrap_or(1).max(1);
        let offset = (page - 1) * limit;

        let (data, total) = self.leave_balance_repo.get_workspace_balances(
            workspace_id,
            year,
            limit,
            offset,
            params.q,
        )
        .await
        .map_err(|e| {
            eprintln!("SQLx Error in get_workspace_balances: {e:?}");
            AppError::Internal("Internal error".into())
        })?;

        Ok(crate::models::PaginatedResponse {
            data,
            total: total as usize,
            page,
            limit,
        })
    }

    pub async fn get_employee_leave_history(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        year: i32,
        leave_type_id: Option<Uuid>,
    ) -> Result<Vec<LeaveHistoryItem>, AppError> {
        let pool = &self.pool;
        // Get Accruals & Adjustments with leave type info via FK join
        let logs: Vec<_> = sqlx::query!(
            r#"
            SELECT lal.id, lal.accrual_date, lal.amount, lt.name as leave_type_name,
                   lal.notes, lal.created_at, lal.policy_assignment_id
            FROM leave_accrual_logs lal
            JOIN leave_types lt ON lal.leave_type_id = lt.id
            JOIN employees e ON lal.employee_id = e.id
            WHERE lal.employee_id = $1
              AND e.workspace_id = $4
              AND EXTRACT(YEAR FROM lal.accrual_date) = $2
              AND (lt.id = $3 OR $3 IS NULL)
            "#,
            employee_id,
            year as f64,
            leave_type_id,
            workspace_id
        )
        .fetch_all(pool)
        .await
        .map_err(|e| {
            eprintln!("SQLx Error in get_employee_leave_history logs: {e:?}");
            AppError::Internal("Internal error".into())
        })?;

        // Get Usage (Approved Requests) with leave type info via FK join
        let requests: Vec<_> = sqlx::query!(
            r#"
            SELECT lr.id, lr.start_date, lr.end_date, lt.name as leave_type_name,
                   lr.reason, lr.created_at
            FROM leave_requests lr
            JOIN leave_types lt ON lr.leave_type_id = lt.id
            JOIN employees e ON lr.employee_id = e.id
            WHERE lr.employee_id = $1
              AND e.workspace_id = $4
              AND lr.status = 'approved'
              AND EXTRACT(YEAR FROM lr.start_date) = $2
              AND (lt.id = $3 OR $3 IS NULL)
            "#,
            employee_id,
            year as f64,
            leave_type_id,
            workspace_id
        )
        .fetch_all(pool)
        .await
        .map_err(|e| {
            eprintln!("SQLx Error in get_employee_leave_history requests: {e:?}");
            AppError::Internal("Internal error".into())
        })?;

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
            history.push(LeaveHistoryItem {
                id: Uuid::new_v4(),
                date: req.start_date,
                transaction_type: LeaveTransactionType::Usage,
                amount: -(weekday_count as f64),
                reason: req.reason,
                reference_id: Some(req.id),
                created_at: req.created_at.unwrap_or_else(chrono::Utc::now),
            });
        }

        // Sort by date desc
        history.sort_by(|a, b| b.date.cmp(&a.date));

        Ok(history)
    }

    pub async fn adjust_balance(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        req: AdjustBalanceRequest,
    ) -> Result<(), AppError> {
        let amount_dec =
            rust_decimal::Decimal::from_f64_retain(req.amount).ok_or(AppError::Validation("Invalid request".into()))?;

        self.leave_balance_repo.adjust_balance(
            workspace_id,
            employee_id,
            req.leave_type_id,
            amount_dec,
            req.transaction_date,
            &req.reason,
        )
        .await
        .map_err(AppError::Sqlx)
    }

    pub async fn get_employee_balances(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        year: i32,
    ) -> Result<Vec<EmployeeLeaveBalance>, AppError> {
        // Verify employee belongs to workspace
        let _ = self.employee_repo.get_employee_by_id(employee_id, workspace_id)
            .await
            .map_err(AppError::Sqlx)?
            .ok_or(AppError::NotFound("Not found".into()))?;

        self.leave_balance_repo.get_employee_balances(workspace_id, employee_id, year)
            .await
            .map_err(AppError::Sqlx)
    }

    pub async fn submit_leave_request(
        &self,
        workspace_id: Uuid,
        user_id: Uuid,
        req: SubmitLeaveRequest,
    ) -> Result<LeaveRequest, AppError> {
        let pool = &self.pool;
        // Verify employee belongs to workspace
        let _ = self.employee_repo.get_employee_by_id(req.employee_id, workspace_id)
            .await
            .map_err(AppError::Sqlx)?
            .ok_or(AppError::NotFound("Not found".into()))?;

        // Validate leave_type_id exists for this workspace
        let leave_type = sqlx::query!(
            "SELECT id FROM leave_types WHERE id = $1 AND workspace_id = $2",
            req.leave_type_id,
            workspace_id
        )
        .fetch_optional(pool)
        .await
        .map_err(AppError::Sqlx)?
        .ok_or(AppError::Validation("Invalid request".into()))?;

        let request = LeaveRequest {
            id: Uuid::new_v4(),
            employee_id: req.employee_id,
            leave_type_id: leave_type.id,
            start_date: req.start_date,
            end_date: req.end_date,
            reason: req.reason,
            status: LeaveStatus::Pending,
            requested_by_user_id: Some(user_id),
            approved_by_user_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        self.leave_request_repo.create_leave_request(&request)
            .await
            .map_err(|e| {
                eprintln!("SQLx Error in get_workspace_balances: {e:?}");
                AppError::Internal("Internal error".into())
            })?;

        Ok(request)
    }

    pub async fn approve_leave_request(
        &self,
        workspace_id: Uuid,
        user_id: Uuid,
        request_id: Uuid,
        req: ApproveLeaveRequest,
    ) -> Result<(), AppError> {
        let pool = &self.pool;
        let request = self.leave_request_repo.get_leave_request_by_id(workspace_id, request_id)
            .await
            .map_err(AppError::Sqlx)?
            .ok_or(AppError::NotFound("Not found".into()))?;

        // Verify employee of this request belongs to the workspace
        let _ = self.employee_repo.get_employee_by_id(request.employee_id, workspace_id)
            .await
            .map_err(AppError::Sqlx)?
            .ok_or(AppError::Forbidden("Forbidden".into()))?;

        self.leave_request_repo.update_leave_request_status(
            workspace_id,
            request_id,
            req.status,
            Some(user_id),
        )
        .await
        .map_err(|e| {
            eprintln!("SQLx Error in get_workspace_balances: {e:?}");
            AppError::Internal("Internal error".into())
        })?;

        // If approved, update roster
        if req.status == LeaveStatus::Approved {
            // Get leave type name for roster notes
            let leave_type_name = sqlx::query_scalar!(
                "SELECT name FROM leave_types WHERE id = $1 AND workspace_id = $2",
                request.leave_type_id,
                workspace_id
            )
            .fetch_optional(pool)
            .await
            .map_err(AppError::Sqlx)?
            .unwrap_or_else(|| "Leave".to_string());

            let mut current_date = request.start_date;
            while current_date <= request.end_date {
                let roster = ShiftRoster {
                    id: Uuid::new_v4(),
                    employee_id: request.employee_id,
                    shift_id: None,
                    date: current_date,
                    status: "on_leave".to_string(),
                    notes: Some(format!("Nghỉ phép: {leave_type_name}")),
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                };

                let _ = self.roster_repo.upsert_roster(&roster).await
                    .map_err(|e| {
                        eprintln!("SQLx Error in get_workspace_balances: {e:?}");
                        AppError::Internal("Internal error".into())
                    })?;

                if let Some(next_date) = current_date.succ_opt() {
                    current_date = next_date;
                } else {
                    break;
                }
            }
        }

        Ok(())
    }

    pub async fn get_employee_leave_requests(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
    ) -> Result<Vec<LeaveRequest>, AppError> {
        // Verify employee belongs to workspace
        let _ = self.employee_repo.get_employee_by_id(employee_id, workspace_id)
            .await
            .map_err(AppError::Sqlx)?
            .ok_or(AppError::NotFound("Not found".into()))?;

        self.leave_request_repo.get_employee_leave_requests(workspace_id, employee_id)
            .await
            .map_err(AppError::Sqlx)
    }

    pub async fn get_workspace_leave_requests(
        &self,
        workspace_id: Uuid,
        status: Option<LeaveStatus>,
    ) -> Result<Vec<LeaveRequest>, AppError> {
        self.leave_request_repo.get_workspace_leave_requests(workspace_id, status)
            .await
            .map_err(AppError::Sqlx)
    }
}
