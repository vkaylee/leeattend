use sqlx::PgPool;
use uuid::Uuid;
use chrono::Utc;
use axum::http::StatusCode;

use crate::models::{AttendanceAdjustmentRequest, SubmitAttendanceAdjustmentRequest, ApproveAttendanceAdjustmentRequest, TimeEntry};
use crate::repositories::{attendance_adjustment_repository, employee_repository, time_entry_repository};

pub struct AttendanceAdjustmentService;

impl AttendanceAdjustmentService {
    pub async fn submit_request(
        pool: &PgPool,
        workspace_id: Uuid,
        user_id: Uuid,
        req: SubmitAttendanceAdjustmentRequest,
    ) -> Result<AttendanceAdjustmentRequest, StatusCode> {
        // Verify employee belongs to workspace
        let _ = employee_repository::get_employee_by_id(pool, req.employee_id, workspace_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::NOT_FOUND)?;

        let request = AttendanceAdjustmentRequest {
            id: Uuid::new_v4(),
            employee_id: req.employee_id,
            time_entry_id: req.time_entry_id,
            requested_punch_in_time: req.requested_punch_in_time,
            requested_punch_out_time: req.requested_punch_out_time,
            reason: req.reason,
            status: "pending".to_string(),
            requested_by_user_id: Some(user_id),
            approved_by_user_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        attendance_adjustment_repository::create_adjustment_request(pool, &request)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        Ok(request)
    }

    pub async fn approve_request(
        pool: &PgPool,
        workspace_id: Uuid,
        user_id: Uuid,
        request_id: Uuid,
        req: ApproveAttendanceAdjustmentRequest,
    ) -> Result<(), StatusCode> {
        let request = attendance_adjustment_repository::get_adjustment_request_by_id(pool, request_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::NOT_FOUND)?;

        // Verify employee of this request belongs to the workspace
        let _ = employee_repository::get_employee_by_id(pool, request.employee_id, workspace_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::FORBIDDEN)?;

        if req.status == "approved" {
            // Update or Create Time Entry
            if let Some(time_entry_id) = request.time_entry_id {
                // Update existing
                let mut entry = time_entry_repository::get_time_entry_by_id(pool, time_entry_id)
                    .await
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
                    .ok_or(StatusCode::NOT_FOUND)?;

                entry.punch_in_time = request.requested_punch_in_time;
                entry.punch_out_time = request.requested_punch_out_time;
                
                if let Some(po) = entry.punch_out_time {
                    let duration = po.signed_duration_since(entry.punch_in_time);
                    entry.duration_minutes = Some(duration.num_minutes() as i32);
                }
                
                entry.note = Some(format!("{}\n[Adjustment Approved by {}]", entry.note.unwrap_or_default(), user_id).trim().to_string());
                entry.updated_at = Utc::now();

                time_entry_repository::update_time_entry(pool, &entry)
                    .await
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            } else {
                // Create new
                let mut duration_minutes = None;
                if let Some(po) = request.requested_punch_out_time {
                    let duration = po.signed_duration_since(request.requested_punch_in_time);
                    duration_minutes = Some(duration.num_minutes() as i32);
                }

                let entry = TimeEntry {
                    id: Uuid::new_v4(),
                    employee_id: request.employee_id,
                    punch_in_time: request.requested_punch_in_time,
                    punch_out_time: request.requested_punch_out_time,
                    duration_minutes,
                    entry_type: "work".to_string(),
                    note: Some(format!("[Manual Entry Approved by {}] Reason: {}", user_id, request.reason.unwrap_or_default())),
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                };

                time_entry_repository::create_time_entry(pool, &entry)
                    .await
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            }
        }

        attendance_adjustment_repository::update_adjustment_status(pool, request_id, &req.status, Some(user_id))
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        Ok(())
    }

    pub async fn get_workspace_requests(
        pool: &PgPool,
        workspace_id: Uuid,
        status: Option<String>,
    ) -> Result<Vec<AttendanceAdjustmentRequest>, StatusCode> {
        attendance_adjustment_repository::get_workspace_adjustment_requests(pool, workspace_id, status)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
    }

    pub async fn get_employee_requests(
        pool: &PgPool,
        employee_id: Uuid,
    ) -> Result<Vec<AttendanceAdjustmentRequest>, StatusCode> {
        attendance_adjustment_repository::get_employee_adjustment_requests(pool, employee_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
    }
}
