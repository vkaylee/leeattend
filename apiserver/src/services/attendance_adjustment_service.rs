use axum::http::StatusCode;
use chrono::Utc;
use sqlx::PgPool;
use tracing::error;
use uuid::Uuid;

use crate::models::{
    AdjustmentStatus, AttendanceAdjustmentRequest, CreateAttendanceAdjustmentRequest, TimeEntry,
};
use crate::repositories::{attendance_adjustment_repository, time_entry_repository};
use crate::services::attendance_import_service::AttendanceImportService;

pub struct AttendanceAdjustmentService;

impl AttendanceAdjustmentService {
    /// Resume all jobs that were interrupted by a server restart
    pub async fn resume_all_pending_jobs(pool: PgPool) {
        AttendanceImportService::resume_all_pending_jobs(pool).await;
    }

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
            status: AdjustmentStatus::Pending,
            requested_by_user_id: Some(user_id),
            approved_by_user_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        attendance_adjustment_repository::create_request(pool, &adjustment)
            .await
            .map_err(|e| {
                error!("SUBMIT_ADJUSTMENT_ERROR: {:?}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        Ok(adjustment)
    }

    pub async fn approve_request(
        pool: &PgPool,
        _workspace_id: Uuid,
        approver_id: Uuid,
        request_id: Uuid,
        status: AdjustmentStatus,
    ) -> Result<(), StatusCode> {
        let request = attendance_adjustment_repository::get_request_by_id(pool, request_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::NOT_FOUND)?;

        attendance_adjustment_repository::update_adjustment_status(
            pool,
            request_id,
            status,
            Some(approver_id),
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        if status == AdjustmentStatus::Approved {
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
                let mut duration_minutes = None;
                if let Some(punch_out) = request.requested_punch_out_time {
                    let duration = punch_out.signed_duration_since(request.requested_punch_in_time);
                    duration_minutes = Some(duration.num_minutes() as i32);
                }

                let workspace_id: Uuid = sqlx::query_scalar!(
                    "SELECT workspace_id FROM employees WHERE id = $1",
                    request.employee_id
                )
                .fetch_one(pool)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
                let entry = TimeEntry {
                    id: Uuid::new_v4(),
                    employee_id: request.employee_id,
                    workspace_id,
                    shift_id: None,
                    punch_in_time: request.requested_punch_in_time,
                    punch_out_time: request.requested_punch_out_time,
                    duration_minutes,
                    is_break_overtime: request.is_break_overtime,
                    entry_type: "work".to_string(),
                    note: request.reason.map(|r| format!("Adjustment: {r}")),
                    generation_metadata: Some(serde_json::json!({ "source": "adjustment" })),
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

    pub fn parse_datetime(
        dt_str: &str,
        timezone: &str,
        preferred_date: Option<&str>,
        preferred_time: Option<&str>,
    ) -> Result<chrono::DateTime<chrono::Utc>, String> {
        crate::utils::datetime::parse_datetime_with_tz(
            dt_str,
            timezone,
            preferred_date,
            preferred_time,
        )
    }

    pub async fn get_workspace_requests(
        pool: &PgPool,
        workspace_id: Uuid,
        status: Option<AdjustmentStatus>,
    ) -> Result<Vec<AttendanceAdjustmentRequest>, StatusCode> {
        attendance_adjustment_repository::get_workspace_adjustment_requests(
            pool,
            workspace_id,
            status,
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
    }

    pub async fn get_my_requests(
        pool: &PgPool,
        employee_id: Uuid,
    ) -> Result<Vec<AttendanceAdjustmentRequest>, StatusCode> {
        attendance_adjustment_repository::get_employee_adjustment_requests(pool, employee_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Timelike;

    #[test]
    fn test_parse_datetime_ambiguity() {
        // Case 1: Preferred DD/MM/YYYY, input 01/05/2026 (May 1st)
        let dt = AttendanceAdjustmentService::parse_datetime(
            "01/05/2026 14:00",
            "Asia/Ho_Chi_Minh",
            Some("DD/MM/YYYY"),
            Some("24h"),
        );
        assert!(dt.is_ok());
        let dt = dt.expect("parsing failed");
        // 14:00 VN = 07:00 UTC
        assert_eq!(dt.hour(), 7);
    }

    #[test]
    fn test_parse_datetime_iso() {
        // ISO with Z should ignore workspace tz
        let dt = AttendanceAdjustmentService::parse_datetime(
            "2026-01-14T08:30:00Z",
            "Asia/Ho_Chi_Minh",
            None,
            None,
        );
        assert!(dt.is_ok());
        assert_eq!(dt.expect("parsing failed").hour(), 8);
    }
}
