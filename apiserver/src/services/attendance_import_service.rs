use axum::http::StatusCode;
use chrono::{Utc, NaiveDateTime, DateTime};
use sqlx::PgPool;
use uuid::Uuid;
use tracing::{error, info};
use std::io::Cursor;
use calamine::{Reader, Xlsx, Xls, Data, DataType};

use crate::models::{AttendanceImportJob, TimeEntry, CreateAttendanceAdjustmentRequest};
use crate::repositories::{import_job_repository, employee_repository, time_entry_repository};
use crate::services::attendance_adjustment_service::AttendanceAdjustmentService;

pub struct AttendanceImportService;

impl AttendanceImportService {
    pub async fn batch_import_from_file(
        pool: PgPool,
        workspace_id: Uuid,
        user_id: Uuid,
        file_path: String,
        target_type: String, // "adjustment" or "log"
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
            target_type,
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

    pub async fn resume_all_pending_jobs(pool: PgPool) {
        if let Ok(jobs) = import_job_repository::get_pending_jobs(&pool).await {
            for job in jobs {
                info!("Resuming import job: {} (target: {})", job.id, job.target_type);
                let pool_clone = pool.clone();
                tokio::spawn(async move {
                    let _ = Self::process_job(pool_clone, job.id).await;
                });
            }
        }
    }

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

        // Check if Excel
        let mut is_excel = false;
        if let Ok(mut reader) = Xlsx::new(Cursor::new(&file_bytes)) {
            if let Some(Ok(range)) = reader.worksheet_range_at(0) {
                is_excel = true;
                Self::process_excel_range(&pool, job_id, &job, range, &mut processed_rows, &mut total_rows, &mut errors, Some(&date_format)).await;
            }
        } else if let Ok(mut reader) = Xls::new(Cursor::new(&file_bytes)) {
            if let Some(Ok(range)) = reader.worksheet_range_at(0) {
                is_excel = true;
                Self::process_excel_range(&pool, job_id, &job, range, &mut processed_rows, &mut total_rows, &mut errors, Some(&date_format)).await;
            }
        }

        // Try CSV if not Excel
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

                let punch_time = match AttendanceAdjustmentService::parse_datetime(emp_dt_str, Some(&date_format)) {
                    Some(dt) => dt,
                    None => {
                        errors.push(format!("NV {}: Sai định dạng ngày giờ ({})", emp_id_number, emp_dt_str));
                        continue;
                    }
                };

                match Self::process_import_row(&pool, &job, &emp_id_number, punch_time).await {
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
        info!("Import job {} ({}) finished with status {}", job_id, job.target_type, status);
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
                Some(Data::String(s)) => AttendanceAdjustmentService::parse_datetime(s, preferred_format),
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

            match Self::process_import_row(pool, job, &emp_id_number, punch_time).await {
                Ok(c) => {
                    *processed_rows += c as i32;
                    if *processed_rows % 10 == 0 {
                        let _ = import_job_repository::update_job_progress(pool, job_id, *processed_rows, "processing", None).await;
                    }
                }
                Err(_) => errors.push(format!("NV {}: Lỗi hệ thống", emp_id_number)),
            }
        }
    }

    async fn process_import_row(
        pool: &PgPool,
        job: &AttendanceImportJob,
        emp_id_number: &str,
        punch_time: NaiveDateTime,
    ) -> Result<usize, StatusCode> {
        let employee = match employee_repository::get_employee_by_id_number(pool, emp_id_number, job.workspace_id).await {
            Ok(Some(emp)) => emp,
            _ => {
                error!("EMPLOYEE_NOT_FOUND: {} in WS {}", emp_id_number, job.workspace_id);
                return Ok(0);
            }
        };

        let punch_time_utc = DateTime::<Utc>::from_naive_utc_and_offset(punch_time, Utc);

        if job.target_type == "log" {
            // Direct insertion into time_entries
            let entry = TimeEntry {
                id: Uuid::new_v4(),
                employee_id: employee.id,
                shift_id: None,
                punch_in_time: punch_time_utc,
                punch_out_time: None,
                duration_minutes: None,
                is_break_overtime: false,
                entry_type: "work".to_string(),
                note: Some("Imported directly to logs".to_string()),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };

            time_entry_repository::create_time_entry(pool, &entry)
                .await
                .map_err(|e| {
                    error!("CREATE_TIME_ENTRY_ERROR: {:?}", e);
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;
            Ok(1)
        } else {
            // Original adjustment flow
            let req = CreateAttendanceAdjustmentRequest {
                employee_id: employee.id,
                time_entry_id: None,
                requested_punch_in_time: punch_time_utc,
                requested_punch_out_time: None,
                is_break_overtime: false,
                reason: Some("Import from Excel/CSV".to_string()),
            };

            if let Ok(adjustment) = AttendanceAdjustmentService::submit_request(pool, job.workspace_id, job.user_id, req).await {
                let _ = AttendanceAdjustmentService::approve_request(pool, job.workspace_id, job.user_id, adjustment.id, "approved".to_string()).await;
                return Ok(1);
            }
            Ok(0)
        }
    }
}
