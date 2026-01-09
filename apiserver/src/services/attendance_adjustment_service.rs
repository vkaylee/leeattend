use axum::http::StatusCode;
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{AttendanceAdjustmentRequest, CreateAttendanceAdjustmentRequest, TimeEntry};
use crate::repositories::{attendance_adjustment_repository, time_entry_repository};

pub struct AttendanceAdjustmentService;

impl AttendanceAdjustmentService {
    pub async fn submit_request(
        pool: &PgPool,
        _workspace_id: Uuid,
        user_id: Uuid,
        req: CreateAttendanceAdjustmentRequest,
    ) -> Result<AttendanceAdjustmentRequest, StatusCode> {
        let adjustment = AttendanceAdjustmentRequest {
            id: Uuid::new_v4(),
            employee_id: req.employee_id,
            time_entry_id: req.time_entry_id,
            requested_punch_in_time: req.requested_punch_in_time,
            requested_punch_out_time: req.requested_punch_out_time,
            is_break_overtime: req.is_break_overtime,
            reason: req.reason,
            status: "pending".to_string(),
            requested_by_user_id: Some(user_id),
            approved_by_user_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        attendance_adjustment_repository::create_request(pool, &adjustment)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        Ok(adjustment)
    }

    pub async fn approve_request(
        pool: &PgPool,
        _workspace_id: Uuid,
        approver_id: Uuid,
        request_id: Uuid,
        status: String,
    ) -> Result<(), StatusCode> {
        let request = attendance_adjustment_repository::get_request_by_id(pool, request_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::NOT_FOUND)?;

        attendance_adjustment_repository::update_adjustment_status(
            pool,
            request_id,
            &status,
            Some(approver_id),
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        if status == "approved" {
            // Update or create time entry
            if let Some(time_entry_id) = request.time_entry_id {
                let mut entry = time_entry_repository::get_time_entry_by_id(pool, time_entry_id)
                    .await
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
                    .ok_or(StatusCode::NOT_FOUND)?;

                entry.punch_in_time = request.requested_punch_in_time;
                entry.punch_out_time = request.requested_punch_out_time;
                entry.is_break_overtime = request.is_break_overtime;

                if let Some(punch_out) = entry.punch_out_time {
                    let duration = punch_out.signed_duration_since(entry.punch_in_time);
                    entry.duration_minutes = Some(duration.num_minutes() as i32);
                }

                time_entry_repository::update_time_entry(pool, &entry)
                    .await
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            } else {
                // Create new entry
                let mut duration_minutes = None;
                if let Some(punch_out) = request.requested_punch_out_time {
                    let duration = punch_out.signed_duration_since(request.requested_punch_in_time);
                    duration_minutes = Some(duration.num_minutes() as i32);
                }

                let entry = TimeEntry {
                    id: Uuid::new_v4(),
                    employee_id: request.employee_id,
                    shift_id: None,
                    punch_in_time: request.requested_punch_in_time,
                    punch_out_time: request.requested_punch_out_time,
                    duration_minutes,
                    is_break_overtime: request.is_break_overtime,
                    entry_type: "work".to_string(),
                    note: request.reason.map(|r| format!("Adjustment: {}", r)),
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                };

                time_entry_repository::create_time_entry(pool, &entry)
                    .await
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            }
        }

        Ok(())
    }

    pub async fn get_workspace_requests(
        pool: &PgPool,
        workspace_id: Uuid,
        status: Option<String>,
    ) -> Result<Vec<AttendanceAdjustmentRequest>, StatusCode> {
        attendance_adjustment_repository::get_workspace_adjustment_requests(
            pool,
            workspace_id,
            status,
        )
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
