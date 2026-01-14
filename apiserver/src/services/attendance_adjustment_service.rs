use axum::http::StatusCode;
use chrono::{Utc, NaiveDateTime};
use sqlx::PgPool;
use uuid::Uuid;
use tracing::error;

use crate::models::{AttendanceAdjustmentRequest, CreateAttendanceAdjustmentRequest, TimeEntry};
use crate::repositories::{attendance_adjustment_repository, time_entry_repository};
use crate::services::attendance_import_service::AttendanceImportService;

pub struct AttendanceAdjustmentService;

impl AttendanceAdjustmentService {
    /// Start a new import job from a file on disk (Delegated to AttendanceImportService)
    pub async fn batch_import_from_file(
        pool: PgPool,
        workspace_id: Uuid,
        user_id: Uuid,
        file_path: String,
    ) -> Result<Uuid, StatusCode> {
        AttendanceImportService::batch_import_from_file(pool, workspace_id, user_id, file_path, "adjustment".to_string()).await
    }

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
            status: "pending".to_string(),
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

    pub fn parse_datetime(
        dt_str: &str, 
        preferred_date: Option<&str>,
        preferred_time: Option<&str>
    ) -> Result<NaiveDateTime, String> {
        let dt_str = dt_str.trim();
        if dt_str.is_empty() { 
            return Err("Dữ liệu thời gian bị trống".to_string()); 
        }

        let date_pref = preferred_date.unwrap_or("DD/MM/YYYY").to_uppercase();
        let time_pref = preferred_time.unwrap_or("24h");

        // 1. Build strict priority list based on Workspace Settings
        let mut priority_fmts = Vec::new();
        let date_fmts = match date_pref.as_str() {
            "MM/DD/YYYY" => vec!["%m/%d/%Y", "%m-%d-%Y", "%m.%d.%Y"],
            "YYYY-MM-DD" => vec!["%Y-%m-%d", "%Y/%m/%d", "%Y.%m.%d"],
            _ => vec!["%d/%m/%Y", "%d-%m-%Y", "%d.%m.%Y"], // Default DD/MM/YYYY
        };

        let time_fmts = if time_pref == "12h" {
            vec!["%I:%M:%S %p", "%I:%M %p"]
        } else {
            vec!["%H:%M:%S", "%H:%M"]
        };

        for df in &date_fmts {
            for tf in &time_fmts {
                priority_fmts.push(format!("{} {}", df, tf));
            }
        }

        // Try strict priority formats first
        for fmt in &priority_fmts {
            if let Ok(dt) = NaiveDateTime::parse_from_str(dt_str, fmt) {
                return Ok(dt);
            }
        }

        // 2. Fallback for ISO 8601 (Machine format)
        if let Ok(dt) = NaiveDateTime::parse_from_str(dt_str, "%Y-%m-%d %H:%M:%S") {
            return Ok(dt);
        }
        if let Ok(dt) = NaiveDateTime::parse_from_str(dt_str, "%Y-%m-%dT%H:%M:%S") {
            return Ok(dt);
        }

        // 3. If strict fails, check if it matches ANY common format to give a better hint
        let common_fmts = vec![
            "%d/%m/%Y %H:%M:%S", "%m/%d/%Y %H:%M:%S", "%Y-%m-%d %H:%M:%S",
            "%d/%m/%Y %H:%M", "%m/%d/%Y %H:%M", "%Y-%m-%d %H:%M",
            "%d-%m-%Y %H:%M", "%Y/%m/%d %H:%M", "%Y-%m-%dT%H:%M:%S"
        ];
        
        for fmt in common_fmts {
            if let Ok(_) = NaiveDateTime::parse_from_str(dt_str, fmt) {
                return Err(format!(
                    "Giá trị '{}' sai định dạng. Cài đặt hiện tại yêu cầu: {} ({}).",
                    dt_str, 
                    date_pref, 
                    if time_pref == "12h" { "12 giờ AM/PM" } else { "24 giờ" }
                ));
            }
        }

        Err(format!("Không thể nhận diện ngày giờ: '{}'. Vui lòng kiểm tra lại file.", dt_str))
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
    use chrono::{Datelike, Timelike};

    #[test]
    fn test_parse_datetime_ambiguity() {
        // Case 1: Preferred DD/MM/YYYY, input 01/05/2026 (May 1st)
        let dt = AttendanceAdjustmentService::parse_datetime("01/05/2026 14:00", Some("DD/MM/YYYY"), Some("24h"));
        assert!(dt.is_ok());
        let dt = dt.unwrap();
        assert_eq!(dt.day(), 1);
        assert_eq!(dt.month(), 5);

        // Case 2: Preferred MM/DD/YYYY, input 01/05/2026 (Jan 5th)
        let dt = AttendanceAdjustmentService::parse_datetime("01/05/2026 14:00", Some("MM/DD/YYYY"), Some("24h"));
        assert!(dt.is_ok());
        let dt = dt.unwrap();
        assert_eq!(dt.day(), 5);
        assert_eq!(dt.month(), 1);
    }

    #[test]
    fn test_parse_datetime_strict_failure_detailed() {
        // Preference is DD/MM/YYYY but input is ISO YYYY-MM-DD
        let res = AttendanceAdjustmentService::parse_datetime("2026-01-14 08:00", Some("DD/MM/YYYY"), Some("24h"));
        assert!(res.is_err());
        let err = res.unwrap_err();
        
        // Should contain the faulty data
        assert!(err.contains("'2026-01-14 08:00'"));
        // Should contain the expected format hint
        assert!(err.contains("DD/MM/YYYY"));
        assert!(err.contains("24 giờ"));
    }

    #[test]
    fn test_parse_datetime_empty() {
        let res = AttendanceAdjustmentService::parse_datetime("  ", None, None);
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), "Dữ liệu thời gian bị trống");
    }

    #[test]
    fn test_parse_datetime_12h() {
        let dt = AttendanceAdjustmentService::parse_datetime("01/05/2026 02:00 PM", Some("DD/MM/YYYY"), Some("12h"));
        assert!(dt.is_ok());
        assert_eq!(dt.unwrap().hour(), 14);
    }

    #[test]
    fn test_parse_datetime_iso() {
        // When no preferences, it should still work with ISO as fallback
        let dt = AttendanceAdjustmentService::parse_datetime("2026-01-14 08:30:00", None, None);
        // Note: With my new logic, if no preferences, it defaults to DD/MM/YYYY.
        // ISO is now a fallback ONLY if it matches strict formats. 
        // Actually ISO is at the end of the list.
        assert!(dt.is_ok());
        assert_eq!(dt.unwrap().hour(), 8);
    }
}