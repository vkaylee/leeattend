use sqlx::PgPool;
use uuid::Uuid;
use chrono::Utc;
use axum::http::StatusCode;

use crate::models::{LeaveRequest, SubmitLeaveRequest, ApproveLeaveRequest};
use crate::repositories::leave_request_repository;
use crate::repositories::employee_repository;

pub struct LeaveService;

impl LeaveService {
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

        let request = LeaveRequest {
            id: Uuid::new_v4(),
            employee_id: req.employee_id,
            leave_type: req.leave_type,
            start_date: req.start_date,
            end_date: req.end_date,
            reason: req.reason,
            status: "pending".to_string(),
            requested_by_user_id: Some(user_id),
            approved_by_user_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        leave_request_repository::create_leave_request(pool, &request)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

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

        leave_request_repository::update_leave_request_status(pool, request_id, &req.status, Some(user_id))
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

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
        status: Option<String>,
    ) -> Result<Vec<LeaveRequest>, StatusCode> {
        leave_request_repository::get_workspace_leave_requests(pool, workspace_id, status)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
    }
}
