use axum::http::StatusCode;
use chrono::{Utc, NaiveDateTime, DateTime};
use sqlx::PgPool;
use uuid::Uuid;
use tracing::{error, info};
use std::io::Cursor;
use calamine::{Reader, Xlsx, Xls, Data, DataType};

use crate::models::{AttendanceAdjustmentRequest, CreateAttendanceAdjustmentRequest, TimeEntry, AttendanceImportJob};
use crate::repositories::{attendance_adjustment_repository, time_entry_repository, import_job_repository, employee_repository};

pub struct AttendanceAdjustmentService;

impl AttendanceAdjustmentService {
    /// Start a new import job from a file on disk
    pub async fn batch_import_from_file(
        pool: PgPool,
        workspace_id: Uuid,
        user_id: Uuid,
        file_path: String,
    ) -> Result<Uuid, StatusCode> {
        let job_id = Uuid::new_v4();
        
        let job = AttendanceImportJob {
            id: job_id,
            workspace_id,
            user_id,
            status: "processing".to_string(),
            total_rows: 0,
            processed_rows: 0,
            file_path: Some(file_path.clone()),
            error_log: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        import_job_repository::create_job(&pool, &job)
            .await
            .map_err(|e| {
                error!("CREATE_JOB_ERROR: {:?}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        let pool_clone = pool.clone();
        tokio::spawn(async move {
            let _ = Self::process_job(pool_clone, job_id).await;
        });

        Ok(job_id)
    }

    /// Resume all jobs that were interrupted by a server restart
    pub async fn resume_all_pending_jobs(pool: PgPool) {
        if let Ok(jobs) = import_job_repository::get_pending_jobs(&pool).await {
            for job in jobs {
                info!("Resuming import job: {}", job.id);
                let pool_clone = pool.clone();
                tokio::spawn(async move {
                    let _ = Self::process_job(pool_clone, job.id).await;
                });
            }
        }
    }

    /// Core processing logic for a job
    async fn process_job(pool: PgPool, job_id: Uuid) -> Result<(), ()> {
        let job = match import_job_repository::get_job_by_id(&pool, job_id).await {
            Ok(Some(j)) => j,
            _ => return Err(()),
        };

        let workspace = match crate::repositories::workspace_repository::WorkspaceRepository::get_workspace_by_id(
            &crate::repositories::workspace_repository::SqlxWorkspaceRepository { pool: pool.clone() },
            job.workspace_id
        ).await {
            Ok(Some(ws)) => ws,
            _ => {
                error!("WORKSPACE_NOT_FOUND for job {}", job_id);
                let _ = import_job_repository::update_job_progress(&pool, job_id, job.processed_rows, "failed", Some("Workspace not found".to_string())).await;
                return Err(());
            }
        };

        let date_format = workspace.date_format;

        let file_path = match &job.file_path {
            Some(p) => p,
            None => return Err(()),
        };

        let file_bytes = match tokio::fs::read(file_path).await {
            Ok(b) => b,
            Err(e) => {
                error!("READ_FILE_ERROR for job {}: {:?}", job_id, e);
                let _ = import_job_repository::update_job_progress(&pool, job_id, job.processed_rows, "failed", Some(format!("Could not read file: {:?}", e))).await;
                return Err(());
            }
        };

        let mut processed_rows = job.processed_rows;
        let mut total_rows = job.total_rows;
        let mut errors = Vec::new();

        // 1. Try XLSX
        let mut is_excel = false;
        if let Ok(mut reader) = Xlsx::new(Cursor::new(&file_bytes)) {
            if let Some(Ok(range)) = reader.worksheet_range_at(0) {
                is_excel = true;
                Self::process_excel_range(&pool, job_id, &job, range, &mut processed_rows, &mut total_rows, &mut errors, Some(&date_format)).await;
            }
        }

        // 2. Try XLS (Legacy)
        if !is_excel {
            if let Ok(mut reader) = Xls::new(Cursor::new(&file_bytes)) {
                if let Some(Ok(range)) = reader.worksheet_range_at(0) {
                    is_excel = true;
                    Self::process_excel_range(&pool, job_id, &job, range, &mut processed_rows, &mut total_rows, &mut errors, Some(&date_format)).await;
                }
            }
        }

        // 3. Try CSV if not Excel
        if !is_excel {
            let mut reader = csv::ReaderBuilder::new().has_headers(true).from_reader(Cursor::new(&file_bytes));
            let records: Vec<_> = reader.records().flatten().collect();
            total_rows = records.len() as i32;
            let _ = sqlx::query("UPDATE attendance_import_jobs SET total_rows = $1 WHERE id = $2")
                .bind(total_rows).bind(job_id).execute(&pool).await;

            for record in records.into_iter().skip(processed_rows as usize) {
                let emp_id_number = record.get(0).unwrap_or("").to_string();
                let emp_dt_str = record.get(2).unwrap_or("");

                if emp_id_number.is_empty() || emp_dt_str.is_empty() { continue; }

                let punch_time = match Self::parse_datetime(emp_dt_str, Some(&date_format)) {
                    Some(dt) => dt,
                    None => {
                        errors.push(format!("NV {}: Sai định dạng ngày giờ ({})", emp_id_number, emp_dt_str));
                        continue;
                    }
                };

                match Self::process_import_row(&pool, job.workspace_id, job.user_id, &emp_id_number, punch_time).await {
                    Ok(c) => {
                        processed_rows += c as i32;
                        if processed_rows % 10 == 0 {
                            let _ = import_job_repository::update_job_progress(&pool, job_id, processed_rows, "processing", None).await;
                        }
                    },
                    Err(_) => errors.push(format!("NV {}: Lỗi hệ thống", emp_id_number)),
                }
            }
        }

        let status = if errors.len() as i32 == total_rows && total_rows > 0 { "failed" } else { "completed" };
        let error_log = if errors.is_empty() { None } else { Some(errors.join("\n")) };
        let _ = import_job_repository::update_job_progress(&pool, job_id, processed_rows, status, error_log).await;
        
        let _ = tokio::fs::remove_file(file_path).await;
        info!("Import job {} finished with status {}", job_id, status);
        Ok(())
    }

    async fn process_excel_range(
        pool: &PgPool,
        job_id: Uuid,
        job: &AttendanceImportJob,
        range: calamine::Range<Data>,
        processed_rows: &mut i32,
        total_rows: &mut i32,
        errors: &mut Vec<String>,
        preferred_format: Option<&str>,
    ) {
        *total_rows = (range.rows().count() as i32 - 1).max(0);
        let _ = sqlx::query("UPDATE attendance_import_jobs SET total_rows = $1 WHERE id = $2")
            .bind(*total_rows)
            .bind(job_id)
            .execute(pool)
            .await;

        for row in range.rows().skip(1 + *processed_rows as usize) {
            let emp_id_number = match row.get(0) {
                Some(Data::String(s)) => s.clone(),
                Some(Data::Float(f)) => f.to_string(),
                _ => {
                    errors.push("Dòng thiếu mã nhân viên".to_string());
                    continue;
                }
            };
            let punch_time = match row.get(2) {
                Some(Data::String(s)) => Self::parse_datetime(s, preferred_format),
                Some(cell_data) => cell_data.as_datetime(),
                _ => None,
            };

            let punch_time = match punch_time {
                Some(dt) => dt,
                None => {
                    errors.push(format!("NV {}: Thiếu hoặc sai định dạng ngày giờ", emp_id_number));
                    continue;
                }
            };

            match Self::process_import_row(
                pool,
                job.workspace_id,
                job.user_id,
                &emp_id_number,
                punch_time,
            )
            .await
            {
                Ok(c) => {
                    *processed_rows += c as i32;
                    if *processed_rows % 10 == 0 {
                        let _ = import_job_repository::update_job_progress(
                            pool,
                            job_id,
                            *processed_rows,
                            "processing",
                            None,
                        )
                        .await;
                    }
                }
                Err(_) => errors.push(format!("NV {}: Lỗi hệ thống", emp_id_number)),
            }
        }
    }

    async fn process_import_row(
        pool: &PgPool,
        workspace_id: Uuid,
        user_id: Uuid,
        emp_id_number: &str,
        punch_time: NaiveDateTime,
    ) -> Result<usize, StatusCode> {
        let employee = match employee_repository::get_employee_by_id_number(pool, emp_id_number, workspace_id).await {
            Ok(Some(emp)) => emp,
            _ => {
                error!("EMPLOYEE_NOT_FOUND: {} in WS {}", emp_id_number, workspace_id);
                return Ok(0);
            }
        };

        let punch_time_utc = DateTime::<Utc>::from_naive_utc_and_offset(punch_time, Utc);

        let req = CreateAttendanceAdjustmentRequest {
            employee_id: employee.id,
            time_entry_id: None,
            requested_punch_in_time: punch_time_utc,
            requested_punch_out_time: None,
            is_break_overtime: false,
            reason: Some("Import from Excel/CSV".to_string()),
        };

        if let Ok(adjustment) = Self::submit_request(pool, workspace_id, user_id, req).await {
            let _ = Self::approve_request(pool, workspace_id, user_id, adjustment.id, "approved".to_string()).await;
            return Ok(1);
        }
        
        Ok(0)
    }

    fn parse_datetime(dt_str: &str, preferred_format: Option<&str>) -> Option<NaiveDateTime> {
        let mut formats = vec![
            "%d/%m/%Y %H:%M",
            "%d/%m/%Y %H:%M:%S",
            "%m/%d/%Y %H:%M",
            "%m/%d/%Y %H:%M:%S",
            "%m/%d/%Y %I:%M:%S %p",
            "%m/%d/%Y %I:%M %p",
            "%Y-%m-%d %H:%M:%S",
            "%Y-%m-%d %H:%M",
        ];

        if let Some(pref) = preferred_format {
            // Put standard variations of the preferred format at the beginning
            let mut priority = Vec::new();
            if pref == "DD/MM/YYYY" {
                priority.push("%d/%m/%Y %H:%M:%S");
                priority.push("%d/%m/%Y %H:%M");
            } else if pref == "MM/DD/YYYY" {
                priority.push("%m/%d/%Y %I:%M:%S %p");
                priority.push("%m/%d/%Y %H:%M:%S");
                priority.push("%m/%d/%Y %H:%M");
            } else if pref == "YYYY-MM-DD" {
                priority.push("%Y-%m-%d %H:%M:%S");
                priority.push("%Y-%m-%d %H:%M");
            }

            for p in priority.into_iter().rev() {
                formats.retain(|&f| f != p);
                formats.insert(0, p);
            }
        }

        for fmt in formats {
            if let Ok(dt) = NaiveDateTime::parse_from_str(dt_str, fmt) {
                return Some(dt);
            }
        }
        None
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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Datelike, Timelike};

    #[test]
    fn test_parse_datetime_robustness() {
        // Vietnamese standard
        assert!(AttendanceAdjustmentService::parse_datetime("24/11/2025 07:12", None).is_some());
        
        // US standard with AM/PM and seconds (User's report)
        let dt = AttendanceAdjustmentService::parse_datetime("11/24/2025 7:12:46 AM", None);
        assert!(dt.is_some(), "Should parse user reported format");
        let dt = dt.unwrap();
        assert_eq!(dt.month(), 11);
        assert_eq!(dt.day(), 24);
        assert_eq!(dt.hour(), 7);
        assert_eq!(dt.minute(), 12);
        assert_eq!(dt.second(), 46);

        // PM variation
        let dt_pm = AttendanceAdjustmentService::parse_datetime("11/24/2025 7:12:46 PM", None).unwrap();
        assert_eq!(dt_pm.hour(), 19);

        // ISO-ish
        assert!(AttendanceAdjustmentService::parse_datetime("2025-11-24 14:00:00", None).is_some());

        // Test priority
        let dt_ambiguous = AttendanceAdjustmentService::parse_datetime("01/02/2025 08:00", Some("MM/DD/YYYY")).unwrap();
        assert_eq!(dt_ambiguous.month(), 1);
        assert_eq!(dt_ambiguous.day(), 2);

        let dt_vn = AttendanceAdjustmentService::parse_datetime("01/02/2025 08:00", Some("DD/MM/YYYY")).unwrap();
        assert_eq!(dt_vn.day(), 1);
        assert_eq!(dt_vn.month(), 2);
    }
}