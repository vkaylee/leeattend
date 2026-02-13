use axum::http::StatusCode;
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{
    AdjustBalanceRequest, ApproveLeaveRequest, EmployeeLeaveBalance, LeaveHistoryItem,
    LeaveRequest, LeaveStatus, LeaveTransactionType, ShiftRoster, SubmitLeaveRequest,
    WorkspaceLeaveBalance,
};
use crate::repositories::employee_repository;
use crate::repositories::leave_balance_repository;
use crate::repositories::leave_request_repository;
use crate::repositories::roster_repository;
use rust_decimal::prelude::ToPrimitive;

pub struct LeaveService;

impl LeaveService {
    pub async fn get_workspace_balances(
        pool: &PgPool,
        workspace_id: Uuid,
        year: i32,
        params: crate::models::PaginationParams,
    ) -> Result<crate::models::PaginatedResponse<WorkspaceLeaveBalance>, StatusCode> {
        let limit = params.limit.unwrap_or(20).max(1);
        let page = params.page.unwrap_or(1).max(1);
        let offset = (page - 1) * limit;

        let (data, total) = leave_balance_repository::get_workspace_balances(
            pool,
            workspace_id,
            year,
            limit,
            offset,
            params.q,
        )
        .await
        .map_err(|e| {
            eprintln!("SQLx Error in get_workspace_balances: {e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        Ok(crate::models::PaginatedResponse {
            data,
            total: total as usize,
            page,
            limit,
        })
    }

    pub async fn get_employee_leave_history(
        pool: &PgPool,
        _workspace_id: Uuid,
        employee_id: Uuid,
        year: i32,
        leave_type_id: Option<Uuid>,
    ) -> Result<Vec<LeaveHistoryItem>, StatusCode> {
        // Get Accruals & Adjustments with leave type info via FK join
        let logs: Vec<_> = sqlx::query!(
            r#"
            SELECT lal.id, lal.accrual_date, lal.amount, lt.name as leave_type_name,
                   lal.notes, lal.created_at, lal.policy_assignment_id
            FROM leave_accrual_logs lal
            JOIN leave_types lt ON lal.leave_type_id = lt.id
            WHERE lal.employee_id = $1
              AND EXTRACT(YEAR FROM lal.accrual_date) = $2
              AND (lt.id = $3 OR $3 IS NULL)
            "#,
            employee_id,
            year as f64,
            leave_type_id
        )
        .fetch_all(pool)
        .await
        .map_err(|e| {
            eprintln!("SQLx Error in get_employee_leave_history logs: {e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        // Get Usage (Approved Requests) with leave type info via FK join
        let requests: Vec<_> = sqlx::query!(
            r#"
            SELECT lr.id, lr.start_date, lr.end_date, lt.name as leave_type_name,
                   lr.reason, lr.created_at
            FROM leave_requests lr
            JOIN leave_types lt ON lr.leave_type_id = lt.id
            WHERE lr.employee_id = $1
              AND lr.status = 'approved'
              AND EXTRACT(YEAR FROM lr.start_date) = $2
              AND (lt.id = $3 OR $3 IS NULL)
            "#,
            employee_id,
            year as f64,
            leave_type_id
        )
        .fetch_all(pool)
        .await
        .map_err(|e| {
            eprintln!("SQLx Error in get_employee_leave_history requests: {e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
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
            let days = (req.end_date - req.start_date).num_days() + 1;
            history.push(LeaveHistoryItem {
                id: Uuid::new_v4(),
                date: req.start_date,
                transaction_type: LeaveTransactionType::Usage,
                amount: -(days as f64),
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
        pool: &PgPool,
        workspace_id: Uuid,
        employee_id: Uuid,
        req: AdjustBalanceRequest,
    ) -> Result<(), StatusCode> {
        let amount_dec =
            rust_decimal::Decimal::from_f64_retain(req.amount).ok_or(StatusCode::BAD_REQUEST)?;

        leave_balance_repository::adjust_balance(
            pool,
            workspace_id,
            employee_id,
            req.leave_type_id,
            amount_dec,
            req.transaction_date,
            &req.reason,
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
    }

    pub async fn get_employee_balances(
        pool: &PgPool,
        workspace_id: Uuid,
        employee_id: Uuid,
        year: i32,
    ) -> Result<Vec<EmployeeLeaveBalance>, StatusCode> {
        // Verify employee belongs to workspace
        let _ = employee_repository::get_employee_by_id(pool, employee_id, workspace_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::NOT_FOUND)?;

        leave_balance_repository::get_employee_balances(pool, employee_id, year)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
    }

    pub async fn submit_leave_request(
        pool: &PgPool,
        workspace_id: Uuid,
        user_id: Uuid,
        req: SubmitLeaveRequest,
    ) -> Result<LeaveRequest, StatusCode> {
        // Verify employee belongs to workspace
        let _ = employee_repository::get_employee_by_id(pool, req.employee_id, workspace_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::NOT_FOUND)?;

        // Validate leave_type_id exists for this workspace
        let leave_type = sqlx::query!(
            "SELECT id FROM leave_types WHERE id = $1 AND workspace_id = $2",
            req.leave_type_id,
            workspace_id
        )
        .fetch_optional(pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::BAD_REQUEST)?;

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

        leave_request_repository::create_leave_request(pool, &request)
            .await
            .map_err(|e| {
            eprintln!("SQLx Error in get_workspace_balances: {e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        Ok(request)
    }

    pub async fn approve_leave_request(
        pool: &PgPool,
        workspace_id: Uuid,
        user_id: Uuid,
        request_id: Uuid,
        req: ApproveLeaveRequest,
    ) -> Result<(), StatusCode> {
        let request = leave_request_repository::get_leave_request_by_id(pool, request_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::NOT_FOUND)?;

        // Verify employee of this request belongs to the workspace
        let _ = employee_repository::get_employee_by_id(pool, request.employee_id, workspace_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::FORBIDDEN)?;

        leave_request_repository::update_leave_request_status(
            pool,
            request_id,
            req.status,
            Some(user_id),
        )
        .await
        .map_err(|e| {
            eprintln!("SQLx Error in get_workspace_balances: {e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        // If approved, update roster
        if req.status == LeaveStatus::Approved {
            // Get leave type name for roster notes
            let leave_type_name = sqlx::query_scalar!(
                "SELECT name FROM leave_types WHERE id = $1",
                request.leave_type_id
            )
            .fetch_optional(pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
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

                let _ = roster_repository::upsert_roster(pool, &roster)
                    .await
                    .map_err(|e| {
            eprintln!("SQLx Error in get_workspace_balances: {e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
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
        pool: &PgPool,
        workspace_id: Uuid,
        employee_id: Uuid,
    ) -> Result<Vec<LeaveRequest>, StatusCode> {
        // Verify employee belongs to workspace
        let _ = employee_repository::get_employee_by_id(pool, employee_id, workspace_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::NOT_FOUND)?;

        leave_request_repository::get_employee_leave_requests(pool, employee_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
    }

    pub async fn get_workspace_leave_requests(
        pool: &PgPool,
        workspace_id: Uuid,
        status: Option<LeaveStatus>,
    ) -> Result<Vec<LeaveRequest>, StatusCode> {
        leave_request_repository::get_workspace_leave_requests(pool, workspace_id, status)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
    }
}
