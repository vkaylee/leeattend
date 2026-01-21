use axum::http::StatusCode;
use calamine::{Data, DataType, Reader, Xls, Xlsx};
use chrono::{DateTime, NaiveDate, TimeZone, Utc};
use chrono_tz::Tz;
use sqlx::PgPool;
use std::collections::HashSet;
use std::io::Cursor;
use tracing::{error, info};
use uuid::Uuid;

use crate::models::{
    AdjustmentStatus, AttendanceImportJob, CreateAttendanceAdjustmentRequest, Employee,
    ImportJobStatus, LogSource, RawAttendanceLog,
};
use crate::repositories::{
    employee_repository, import_job_repository, raw_attendance_log_repository,
};
use crate::services::attendance_adjustment_service::AttendanceAdjustmentService;
use crate::services::pairing_service::PairingService;

struct PendingRow {
    emp_id_number: String,
    punch_time: DateTime<Utc>,
    line_number: usize,
}

struct PendingEmployee {
    id_number: String,
    first_name: String,
    last_name: String,
    hire_date: NaiveDate,
    line_number: usize,
}

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
            status: ImportJobStatus::Processing,
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
                info!(
                    "Resuming import job: {} (target: {})",
                    job.id, job.target_type
                );
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
                let _ = import_job_repository::update_job_progress(&pool, job_id, job.processed_rows, ImportJobStatus::Failed, Some("Workspace not found".to_string())).await;
                return Err(());
            }
        };

        let date_format = workspace.date_format.clone();
        let time_format = workspace.time_format.clone();
        let timezone = workspace.timezone.clone();

        if job.target_type == "employee" {
            return Self::process_employee_job(pool, job_id, &job).await;
        }

        let file_path = match &job.file_path {
            Some(p) => p,
            None => return Err(()),
        };

        let file_bytes = match tokio::fs::read(file_path).await {
            Ok(b) => b,
            Err(e) => {
                error!("READ_FILE_ERROR for job {}: {:?}", job_id, e);
                let _ = import_job_repository::update_job_progress(
                    &pool,
                    job_id,
                    job.processed_rows,
                    ImportJobStatus::Failed,
                    Some(format!("Could not read file: {:?}", e)),
                )
                .await;
                return Err(());
            }
        };

        let mut processed_rows = job.processed_rows; // This might be > 0 if resumed, but strictly validating means we might re-validate or fail.
                                                     // For simplicity, if we are resuming, we assume validation passed previously?
                                                     // Or better: just re-process from start if it's strict. But `processed_rows` implies we skip.
                                                     // Given strict validation requirement, if we resume, we probably should re-validate everything remaining?
                                                     // Let's assume for now we parse EVERYTHING to validate, but only insert what's needed?
                                                     // Actually, if `processed_rows > 0`, it means some were inserted. If we failed halfway, we might have partial data.
                                                     // But "Strict Validation" (Approach A) implies "Atomic Batch" conceptually (don't start if bad).
                                                     // So if we are resuming a CRASHED job, we should probably re-validate.

        let mut errors = Vec::new();
        let mut pending_rows: Vec<PendingRow> = Vec::new();

        // 1. PARSE PHASE
        let mut is_excel = false;
        if let Ok(mut reader) = Xlsx::new(Cursor::new(&file_bytes)) {
            if let Some(Ok(range)) = reader.worksheet_range_at(0) {
                is_excel = true;
                Self::parse_excel_range(
                    range,
                    &mut pending_rows,
                    &mut errors,
                    Some(&date_format),
                    Some(&time_format),
                    &timezone,
                );
            }
        } else if let Ok(mut reader) = Xls::new(Cursor::new(&file_bytes)) {
            if let Some(Ok(range)) = reader.worksheet_range_at(0) {
                is_excel = true;
                Self::parse_excel_range(
                    range,
                    &mut pending_rows,
                    &mut errors,
                    Some(&date_format),
                    Some(&time_format),
                    &timezone,
                );
            }
        }

        if !is_excel {
            let mut reader = csv::ReaderBuilder::new()
                .has_headers(true)
                .from_reader(Cursor::new(&file_bytes));
            let records: Vec<_> = reader.records().flatten().collect();
            // In CSV, first row is header, so rows are records.
            // But parse_excel_range handles header skipping (row 0).
            // For CSV, we iterate records which are already data rows.
            for (idx, record) in records.into_iter().enumerate() {
                let current_line = idx + 2; // +1 header, +1 for 1-based
                let emp_id_number = record.get(0).unwrap_or("").trim().to_string();
                let emp_dt_str = record.get(2).unwrap_or("").trim();

                if emp_id_number.is_empty() || emp_dt_str.is_empty() {
                    errors.push(format!(
                        "Dòng {}: Thiếu mã nhân viên hoặc thời gian",
                        current_line
                    ));
                    continue;
                }

                match AttendanceAdjustmentService::parse_datetime(
                    emp_dt_str,
                    &timezone,
                    Some(&date_format),
                    Some(&time_format),
                ) {
                    Ok(dt) => {
                        pending_rows.push(PendingRow {
                            emp_id_number,
                            punch_time: dt,
                            line_number: current_line,
                        });
                    }
                    Err(e) => {
                        errors.push(format!(
                            "Dòng {} (NV {}): {} [Dữ liệu gốc: '{}']",
                            current_line, emp_id_number, e, emp_dt_str
                        ));
                    }
                };
            }
        }

        let total_rows = pending_rows.len() as i32 + errors.len() as i32;

        let _ = sqlx::query("UPDATE attendance_import_jobs SET total_rows = $1 WHERE id = $2")
            .bind(total_rows)
            .bind(job_id)
            .execute(&pool)
            .await;

        if !errors.is_empty() {
            let _ = import_job_repository::update_job_progress(
                &pool,
                job_id,
                0,
                ImportJobStatus::Failed,
                Some(errors.join("\n")),
            )
            .await;
            let _ = tokio::fs::remove_file(file_path).await;
            return Err(());
        }

        // 2. VALIDATION PHASE (Strict)
        let unique_emp_ids: HashSet<String> = pending_rows
            .iter()
            .map(|r| r.emp_id_number.clone())
            .collect();
        let unique_emp_ids_vec: Vec<String> = unique_emp_ids.iter().cloned().collect();

        match employee_repository::get_existing_employee_ids_by_numbers(
            &pool,
            job.workspace_id,
            &unique_emp_ids_vec,
        )
        .await
        {
            Ok(existing_ids) => {
                let existing_set: HashSet<String> = existing_ids.into_iter().collect();
                let missing_ids: Vec<String> =
                    unique_emp_ids.difference(&existing_set).cloned().collect();

                if !missing_ids.is_empty() {
                    let err_msg = format!(
                        "Found {} unknown employees: {}",
                        missing_ids.len(),
                        missing_ids.join(", ")
                    );
                    let _ = import_job_repository::update_job_progress(
                        &pool,
                        job_id,
                        0,
                        ImportJobStatus::Failed,
                        Some(err_msg),
                    )
                    .await;
                    let _ = tokio::fs::remove_file(file_path).await;
                    return Err(());
                }
            }
            Err(e) => {
                error!("VALIDATION_ERROR: {:?}", e);
                let _ = import_job_repository::update_job_progress(
                    &pool,
                    job_id,
                    0,
                    ImportJobStatus::Failed,
                    Some("Database error during validation".to_string()),
                )
                .await;
                return Err(());
            }
        }

        // 3. PROCESSING PHASE
        // If we are resuming, skip already processed rows?
        // With strict validation, we might just re-process everything that hasn't been saved?
        // But if `processed_rows` > 0, it means we did some.
        // The loop below will try to process all pending_rows.
        // We should skip the first `processed_rows` if the order is deterministic.
        // Since `pending_rows` is built sequentially from file, it is deterministic.

        let rows_to_process = pending_rows.into_iter().skip(processed_rows as usize);

        for row in rows_to_process {
            match Self::process_import_row(&pool, &job, &row.emp_id_number, row.punch_time).await {
                Ok(c) => {
                    processed_rows += c as i32;
                    if processed_rows % 10 == 0 {
                        let _ = import_job_repository::update_job_progress(
                            &pool,
                            job_id,
                            processed_rows,
                            ImportJobStatus::Processing,
                            None,
                        )
                        .await;
                    }
                }
                Err(_) => {
                    // This should be rare if validation passed, maybe DB error
                    errors.push(format!(
                        "NV {}: Lỗi hệ thống khi lưu (Dòng {})",
                        row.emp_id_number, row.line_number
                    ));
                }
            }
        }

        let status = if !errors.is_empty() {
            ImportJobStatus::Failed
        } else {
            ImportJobStatus::Completed
        };
        let error_log = if errors.is_empty() {
            None
        } else {
            Some(errors.join("\n"))
        };
        let _ = import_job_repository::update_job_progress(
            &pool,
            job_id,
            processed_rows,
            status,
            error_log,
        )
        .await;

        let _ = tokio::fs::remove_file(file_path).await;
        info!(
            "Import job {} ({}) finished with status {}",
            job_id, job.target_type, status
        );
        Ok(())
    }

    async fn process_employee_job(
        pool: PgPool,
        job_id: Uuid,
        job: &AttendanceImportJob,
    ) -> Result<(), ()> {
        let file_path = match &job.file_path {
            Some(p) => p,
            None => return Err(()),
        };

        let file_bytes = match tokio::fs::read(file_path).await {
            Ok(b) => b,
            Err(e) => {
                error!("READ_FILE_ERROR for job {}: {:?}", job_id, e);
                let _ = import_job_repository::update_job_progress(
                    &pool,
                    job_id,
                    0,
                    ImportJobStatus::Failed,
                    Some(format!("Could not read file: {:?}", e)),
                )
                .await;
                return Err(());
            }
        };

        let mut errors = Vec::new();
        let mut pending_employees: Vec<PendingEmployee> = Vec::new();

        // 1. PARSE PHASE
        let mut is_excel = false;
        if let Ok(mut reader) = Xlsx::new(Cursor::new(&file_bytes)) {
            if let Some(Ok(range)) = reader.worksheet_range_at(0) {
                is_excel = true;
                Self::parse_employee_excel_range(range, &mut pending_employees, &mut errors);
            }
        } else if let Ok(mut reader) = Xls::new(Cursor::new(&file_bytes)) {
            if let Some(Ok(range)) = reader.worksheet_range_at(0) {
                is_excel = true;
                Self::parse_employee_excel_range(range, &mut pending_employees, &mut errors);
            }
        }

        if !is_excel {
            let mut reader = csv::ReaderBuilder::new()
                .has_headers(true)
                .from_reader(Cursor::new(&file_bytes));
            for (idx, record) in reader.records().flatten().enumerate() {
                let current_line = idx + 2;
                let id_number = record.get(0).unwrap_or("").trim().to_string();
                let first_name = record.get(1).unwrap_or("").trim().to_string();
                let last_name = record.get(2).unwrap_or("").trim().to_string();
                let hire_date_str = record.get(3).unwrap_or("").trim();

                if id_number.is_empty() || first_name.is_empty() || last_name.is_empty() {
                    errors.push(format!(
                        "Dòng {}: Thiếu ID, Họ hoặc Tên nhân viên",
                        current_line
                    ));
                    continue;
                }

                let hire_date = if hire_date_str.is_empty() {
                    Utc::now().with_timezone(&Tz::Asia__Ho_Chi_Minh).date_naive()
                } else {
                    // Try some common formats
                    NaiveDate::parse_from_str(hire_date_str, "%Y-%m-%d")
                        .or_else(|_| NaiveDate::parse_from_str(hire_date_str, "%d/%m/%Y"))
                        .or_else(|_| NaiveDate::parse_from_str(hire_date_str, "%m/%d/%Y"))
                        .unwrap_or_else(|_| {
                            Utc::now().with_timezone(&Tz::Asia__Ho_Chi_Minh).date_naive()
                        })
                };

                pending_employees.push(PendingEmployee {
                    id_number,
                    first_name,
                    last_name,
                    hire_date,
                    line_number: current_line,
                });
            }
        }

        let total_rows = pending_employees.len() as i32 + errors.len() as i32;
        let _ = sqlx::query("UPDATE attendance_import_jobs SET total_rows = $1 WHERE id = $2")
            .bind(total_rows)
            .bind(job_id)
            .execute(&pool)
            .await;

        if !errors.is_empty() {
            let _ = import_job_repository::update_job_progress(
                &pool,
                job_id,
                0,
                ImportJobStatus::Failed,
                Some(errors.join("\n")),
            )
            .await;
            let _ = tokio::fs::remove_file(file_path).await;
            return Err(());
        }

        // 2. VALIDATION PHASE (Duplicate ID check)
        let id_numbers: Vec<String> = pending_employees.iter().map(|e| e.id_number.clone()).collect();
        match employee_repository::get_existing_employee_ids_by_numbers(&pool, job.workspace_id, &id_numbers).await {
            Ok(existing) => {
                if !existing.is_empty() {
                    let err_msg = format!("Các ID nhân viên sau đã tồn tại: {}", existing.join(", "));
                    let _ = import_job_repository::update_job_progress(&pool, job_id, 0, ImportJobStatus::Failed, Some(err_msg)).await;
                    let _ = tokio::fs::remove_file(file_path).await;
                    return Err(());
                }
            },
            Err(_) => {
                let _ = import_job_repository::update_job_progress(&pool, job_id, 0, ImportJobStatus::Failed, Some("Lỗi kiểm tra trùng lặp".to_string())).await;
                return Err(());
            }
        }

        // 3. PROCESSING PHASE
        let mut processed_rows = 0;
        for emp in pending_employees {
            match Self::process_employee_import_row(&pool, job, &emp).await {
                Ok(_) => {
                    processed_rows += 1;
                    if processed_rows % 10 == 0 {
                        let _ = import_job_repository::update_job_progress(&pool, job_id, processed_rows, ImportJobStatus::Processing, None).await;
                    }
                },
                Err(e) => {
                    errors.push(format!("Dòng {} (ID {}): Lỗi khi lưu vào DB: {}", emp.line_number, emp.id_number, e));
                }
            }
        }

        let status = if !errors.is_empty() { ImportJobStatus::Failed } else { ImportJobStatus::Completed };
        let _ = import_job_repository::update_job_progress(&pool, job_id, processed_rows, status, if errors.is_empty() { None } else { Some(errors.join("\n")) }).await;
        let _ = tokio::fs::remove_file(file_path).await;
        
        Ok(())
    }

    fn parse_employee_excel_range(
        range: calamine::Range<Data>,
        pending: &mut Vec<PendingEmployee>,
        errors: &mut Vec<String>,
    ) {
        for (idx, row) in range.rows().skip(1).enumerate() {
            let current_line = idx + 2;
            let id_number = match row.get(0) {
                Some(Data::String(s)) => s.trim().to_string(),
                Some(Data::Float(f)) => f.to_string(),
                Some(Data::Int(i)) => i.to_string(),
                _ => { errors.push(format!("Dòng {}: Thiếu ID nhân viên", current_line)); continue; }
            };
            let first_name = match row.get(1) {
                Some(Data::String(s)) => s.trim().to_string(),
                _ => { errors.push(format!("Dòng {}: Thiếu Tên nhân viên", current_line)); continue; }
            };
            let last_name = match row.get(2) {
                Some(Data::String(s)) => s.trim().to_string(),
                _ => { errors.push(format!("Dòng {}: Thiếu Họ nhân viên", current_line)); continue; }
            };
            let hire_date = match row.get(3) {
                Some(Data::String(s)) => {
                    NaiveDate::parse_from_str(s.trim(), "%Y-%m-%d")
                        .or_else(|_| NaiveDate::parse_from_str(s.trim(), "%d/%m/%Y"))
                        .or_else(|_| NaiveDate::parse_from_str(s.trim(), "%m/%d/%Y"))
                        .unwrap_or_else(|_| Utc::now().with_timezone(&Tz::Asia__Ho_Chi_Minh).date_naive())
                },
                Some(cell_data) => {
                    cell_data.as_datetime().map(|ndt| ndt.date()).unwrap_or_else(|| Utc::now().with_timezone(&Tz::Asia__Ho_Chi_Minh).date_naive())
                },
                _ => Utc::now().with_timezone(&Tz::Asia__Ho_Chi_Minh).date_naive()
            };

            pending.push(PendingEmployee {
                id_number,
                first_name,
                last_name,
                hire_date,
                line_number: current_line,
            });
        }
    }

    async fn process_employee_import_row(
        pool: &PgPool,
        job: &AttendanceImportJob,
        emp: &PendingEmployee,
    ) -> Result<(), String> {
        let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

        let employee = Employee {
            id: Uuid::new_v4(),
            workspace_id: job.workspace_id,
            user_id: None,
            first_name: emp.first_name.clone(),
            last_name: emp.last_name.clone(),
            employee_id_number: emp.id_number.clone(),
            hire_date: emp.hire_date,
            termination_date: None,
            tags: Vec::new(),
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        employee_repository::create_employee_tx(&mut tx, &employee)
            .await
            .map_err(|e| e.to_string())?;

        tx.commit().await.map_err(|e| e.to_string())?;

        Ok(())
    }

    fn parse_excel_range(
        range: calamine::Range<Data>,
        pending_rows: &mut Vec<PendingRow>,
        errors: &mut Vec<String>,
        preferred_date: Option<&str>,
        preferred_time: Option<&str>,
        timezone: &str,
    ) {
        // Skip header row (index 0)
        for (idx, row) in range.rows().skip(1).enumerate() {
            let current_line = idx + 2; // +1 for skip, +1 for 1-based indexing
            let emp_id_number = match row.first() {
                Some(Data::String(s)) => s.trim().to_string(),
                Some(Data::Float(f)) => f.to_string(),
                Some(Data::Int(i)) => i.to_string(),
                _ => {
                    errors.push(format!("Dòng {}: Thiếu mã nhân viên", current_line));
                    continue;
                }
            };

            // Column 2 (index 2) is usually Time/Date
            let punch_time = match row.get(2) {
                Some(Data::String(s)) => match AttendanceAdjustmentService::parse_datetime(
                    s.trim(),
                    timezone,
                    preferred_date,
                    preferred_time,
                ) {
                    Ok(dt) => Some(dt),
                    Err(e) => {
                        errors.push(format!(
                            "Dòng {} (NV {}): {} [Dữ liệu gốc: '{}']",
                            current_line, emp_id_number, e, s
                        ));
                        None
                    }
                },
                Some(cell_data) => {
                    if let Some(naive) = cell_data.as_datetime() {
                        let tz: Tz = timezone.parse().unwrap_or(chrono_tz::Asia::Ho_Chi_Minh);
                        match tz.from_local_datetime(&naive) {
                            chrono::LocalResult::Single(dt) => Some(dt.with_timezone(&Utc)),
                            _ => Some(DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc)),
                        }
                    } else {
                        errors.push(format!("Dòng {} (NV {}): Không phải định dạng ngày giờ hợp lệ [Dữ liệu gốc: '{}']", current_line, emp_id_number, cell_data));
                        None
                    }
                }
                _ => {
                    errors.push(format!(
                        "Dòng {} (NV {}): Thiếu dữ liệu thời gian",
                        current_line, emp_id_number
                    ));
                    None
                }
            };

            if let Some(dt) = punch_time {
                pending_rows.push(PendingRow {
                    emp_id_number,
                    punch_time: dt,
                    line_number: current_line,
                });
            }
        }
    }

    async fn process_import_row(
        pool: &PgPool,
        job: &AttendanceImportJob,
        emp_id_number: &str,
        punch_time_utc: DateTime<Utc>,
    ) -> Result<usize, StatusCode> {
        // We still fetch the employee here to get the UUID, or we could have cached it in validation map.
        // Fetching again is safer/easier for now.
        let employee = match employee_repository::get_employee_by_id_number(
            pool,
            emp_id_number,
            job.workspace_id,
        )
        .await
        {
            Ok(Some(emp)) => emp,
            _ => {
                // Should not happen due to validation
                error!(
                    "EMPLOYEE_NOT_FOUND: {} in WS {}",
                    emp_id_number, job.workspace_id
                );
                return Ok(0);
            }
        };

        if job.target_type == "log" {
            // 1. Save to Raw Attendance Logs
            let raw_log = RawAttendanceLog {
                id: Uuid::new_v4(),
                workspace_id: job.workspace_id,
                employee_id: employee.id,
                punch_time: punch_time_utc,
                source: LogSource::Import,
                device_id: None,
                raw_data: None,
                created_at: Utc::now(),
            };

            raw_attendance_log_repository::create_raw_log(pool, &raw_log)
                .await
                .map_err(|e| {
                    error!("CREATE_RAW_LOG_ERROR: {:?}", e);
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;

            // 2. Trigger Pairing Logic
            if let Err(e) = PairingService::process_new_raw_log(pool, &raw_log).await {
                error!("PAIRING_ERROR for log {}: {:?}", raw_log.id, e);
                // Return error to propagate
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }

            Ok(1)
        } else {
            let req = CreateAttendanceAdjustmentRequest {
                employee_id: employee.id,
                time_entry_id: None,
                requested_punch_in_time: punch_time_utc,
                requested_punch_out_time: None,
                is_break_overtime: false,
                reason: Some("Import from Excel/CSV".to_string()),
            };

            if let Ok(adjustment) = AttendanceAdjustmentService::submit_request(
                pool,
                job.workspace_id,
                job.user_id,
                req,
            )
            .await
            {
                let _ = AttendanceAdjustmentService::approve_request(
                    pool,
                    job.workspace_id,
                    job.user_id,
                    adjustment.id,
                    AdjustmentStatus::Approved,
                )
                .await;

                return Ok(1);
            }
            Ok(0)
        }
    }
}
