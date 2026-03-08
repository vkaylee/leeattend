use std::sync::Arc;

use crate::errors::AppError;
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
    ImportJobStatus, LogSource,
};
use crate::repositories::attendance_adjustment_repository::AttendanceAdjustmentRepository;
use crate::repositories::import_job_repository::ImportJobRepository;
use crate::repositories::{employee_repository, EmployeeRepository, TimeEntryRepository};
use crate::services::attendance_adjustment_service::AttendanceAdjustmentService;
use crate::utils::normalize_employee_id;

struct PendingRow {
    emp_id_number: String,
    punch_time: DateTime<Utc>,
    line_number: usize,
}

struct PendingEmployee {
    id_number: String,
    full_name: String,
    hire_date: NaiveDate,
    line_number: usize,
}

pub struct AttendanceImportService {
    import_repo: Arc<dyn ImportJobRepository>,
    adj_repo: Arc<dyn AttendanceAdjustmentRepository>,
    employee_repo: Arc<dyn EmployeeRepository>,
    time_entry_repo: Arc<dyn TimeEntryRepository>,
    pool: PgPool,
}

impl AttendanceImportService {
    pub fn new(
        import_repo: Arc<dyn ImportJobRepository>,
        adj_repo: Arc<dyn AttendanceAdjustmentRepository>,
        employee_repo: Arc<dyn EmployeeRepository>,
        time_entry_repo: Arc<dyn TimeEntryRepository>,
        pool: PgPool,
    ) -> Self {
        Self { import_repo, adj_repo, employee_repo, time_entry_repo, pool }
    }
    pub async fn batch_import_from_file(
        &self,
        workspace_id: Uuid,
        user_id: Uuid,
        file_path: String,
        target_type: String, // "adjustment" or "log"
    ) -> Result<Uuid, AppError> {
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

        self.import_repo.create_job(&job)
            .await
            .map_err(|e| {
                error!("CREATE_JOB_ERROR: {:?}", e);
                AppError::Internal("Internal error".into())
            })?;

        let pool_clone = self.pool.clone();
        let import_repo = self.import_repo.clone();
        let adj_repo = self.adj_repo.clone();
        let employee_repo = self.employee_repo.clone();
        let time_entry_repo = self.time_entry_repo.clone();
        tokio::spawn(async move {
            let svc = Self { import_repo, adj_repo, employee_repo, time_entry_repo, pool: pool_clone };
            let _ = svc.process_job(job_id).await;
        });

        Ok(job_id)
    }

    pub async fn resume_all_pending_jobs(&self) {
        if let Ok(jobs) = self.import_repo.get_pending_jobs().await {
            for job in jobs {
                info!(
                    "Resuming import job: {} (target: {})",
                    job.id, job.target_type
                );
                let pool_clone = self.pool.clone();
                let import_repo = self.import_repo.clone();
                let adj_repo = self.adj_repo.clone();
                let employee_repo = self.employee_repo.clone();
                let time_entry_repo = self.time_entry_repo.clone();
                tokio::spawn(async move {
                    let svc = Self { import_repo, adj_repo, employee_repo, time_entry_repo, pool: pool_clone };
                    let _ = svc.process_job(job.id).await;
                });
            }
        }
    }

    async fn process_job(&self, job_id: Uuid) -> Result<(), ()> {
        let pool = self.pool.clone();
        let job = match self.import_repo.get_job_by_id(job_id).await {
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
                let _ = self.import_repo.update_job_progress(job_id, job.processed_rows, ImportJobStatus::Failed, Some("Workspace not found".to_string())).await;
                return Err(());
            }
        };

        let date_format = workspace.date_format.clone();
        let time_format = workspace.time_format.clone();
        let timezone = workspace.timezone.clone();

        if job.target_type == "employee" {
            return self.process_employee_job(job_id, &job).await;
        }

        let file_path = match &job.file_path {
            Some(p) => p,
            None => return Err(()),
        };

        let file_bytes = match tokio::fs::read(file_path).await {
            Ok(b) => b,
            Err(e) => {
                error!("READ_FILE_ERROR for job {}: {:?}", job_id, e);
                let _ = self.import_repo.update_job_progress(
                    job_id,
                    job.processed_rows,
                    ImportJobStatus::Failed,
                    Some(format!("Could not read file: {e:?}")),
                )
                .await;
                return Err(());
            }
        };

        let mut processed_rows = job.processed_rows;
        let mut errors = Vec::new();
        let mut pending_rows: Vec<PendingRow> = Vec::new();

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
            for (idx, record) in records.into_iter().enumerate() {
                let current_line = idx + 2;
                let raw_emp_id = record.get(0).unwrap_or("").trim();
                let emp_id_number = normalize_employee_id(raw_emp_id);
                let emp_dt_str = record.get(2).unwrap_or("").trim();

                if emp_id_number.is_empty() || emp_dt_str.is_empty() {
                    errors.push(format!(
                        "Dòng {current_line}: Thiếu mã nhân viên hoặc thời gian"
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
                            "Dòng {current_line} (NV {emp_id_number}): {e} [Dữ liệu gốc: '{emp_dt_str}']"
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
            let _ = self.import_repo.update_job_progress(
                job_id,
                0,
                ImportJobStatus::Failed,
                Some(errors.join("\n")),
            )
            .await;
            let _ = tokio::fs::remove_file(file_path).await;
            return Err(());
        }

        let unique_emp_ids: HashSet<String> = pending_rows
            .iter()
            .map(|r| r.emp_id_number.clone())
            .collect();
        let unique_emp_ids_vec: Vec<String> = unique_emp_ids.iter().cloned().collect();

        let known_emp_ids: HashSet<String> =
            match self.employee_repo.get_existing_employee_ids_by_numbers(
                job.workspace_id,
                &unique_emp_ids_vec,
            )
            .await
            {
                Ok(existing_ids) => {
                    let known: HashSet<String> = existing_ids.into_iter().collect();
                    let missing_ids: Vec<String> =
                        unique_emp_ids.difference(&known).cloned().collect();

                    if !missing_ids.is_empty() {
                        errors.push(format!(
                            "Bỏ qua {} mã nhân viên không tồn tại: {}",
                            missing_ids.len(),
                            missing_ids.join(", ")
                        ));
                    }
                    known
                }
                Err(e) => {
                    error!("VALIDATION_ERROR: {:?}", e);
                    let _ = self.import_repo.update_job_progress(
                        job_id,
                        0,
                        ImportJobStatus::Failed,
                        Some("Database error during validation".to_string()),
                    )
                    .await;
                    let _ = tokio::fs::remove_file(file_path).await;
                    return Err(());
                }
            };

        let mut skipped_rows = 0;
        let rows_to_process = pending_rows.into_iter().skip(processed_rows as usize);

        for row in rows_to_process {
            if !known_emp_ids.contains(&row.emp_id_number) {
                skipped_rows += 1;
                continue;
            }

            match self.process_import_row(&job, &row.emp_id_number, row.punch_time).await {
                Ok(inserted) => {
                    if inserted > 0 {
                        processed_rows += 1;
                    } else {
                        skipped_rows += 1;
                    }

                    if (processed_rows + skipped_rows) % 10 == 0 {
                        let _ = self.import_repo.update_job_progress(
                            job_id,
                            processed_rows,
                            ImportJobStatus::Processing,
                            None,
                        )
                        .await;
                    }
                }
                Err(_) => {
                    errors.push(format!(
                        "NV {}: Lỗi hệ thống khi lưu (Dòng {})",
                        row.emp_id_number, row.line_number
                    ));
                }
            }
        }

        if skipped_rows > 0 {
            errors.push(format!(
                "Đã bỏ qua {skipped_rows} dòng (trùng lặp hoặc NV không tồn tại)."
            ));
        }

        let status = if processed_rows > 0 || errors.is_empty() {
            ImportJobStatus::Completed
        } else {
            ImportJobStatus::Failed
        };

        let error_log = if errors.is_empty() {
            None
        } else {
            Some(errors.join("\n"))
        };
        let _ = self.import_repo.update_job_progress(
            job_id,
            processed_rows,
            status,
            error_log,
        )
        .await;

        let _ = tokio::fs::remove_file(file_path).await;
        Ok(())
    }

    async fn process_employee_job(
        &self,
        job_id: Uuid,
        job: &AttendanceImportJob,
    ) -> Result<(), ()> {
        let pool = self.pool.clone();
        let file_path = match &job.file_path {
            Some(p) => p,
            None => return Err(()),
        };

        let file_bytes = match tokio::fs::read(file_path).await {
            Ok(b) => b,
            Err(e) => {
                error!("READ_FILE_ERROR for job {}: {:?}", job_id, e);
                let _ = self.import_repo.update_job_progress(
                    job_id,
                    0,
                    ImportJobStatus::Failed,
                    Some(format!("Could not read file: {e:?}")),
                )
                .await;
                return Err(());
            }
        };

        let mut errors = Vec::new();
        let mut pending_employees: Vec<PendingEmployee> = Vec::new();

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
                let raw_id = record.get(0).unwrap_or("").trim();
                let id_number = normalize_employee_id(raw_id);
                let full_name = record.get(1).unwrap_or("").trim().to_string();
                let hire_date_str = record.get(2).unwrap_or("").trim();

                if id_number.is_empty() || full_name.is_empty() {
                    errors.push(format!("Dòng {current_line}: Thiếu ID hoặc Tên nhân viên"));
                    continue;
                }

                let hire_date = if hire_date_str.is_empty() {
                    Utc::now()
                        .with_timezone(&Tz::Asia__Ho_Chi_Minh)
                        .date_naive()
                } else {
                    NaiveDate::parse_from_str(hire_date_str, "%Y-%m-%d")
                        .or_else(|_| NaiveDate::parse_from_str(hire_date_str, "%d/%m/%Y"))
                        .or_else(|_| NaiveDate::parse_from_str(hire_date_str, "%m/%d/%Y"))
                        .unwrap_or_else(|_| {
                            Utc::now()
                                .with_timezone(&Tz::Asia__Ho_Chi_Minh)
                                .date_naive()
                        })
                };

                pending_employees.push(PendingEmployee {
                    id_number,
                    full_name,
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
            let _ = self.import_repo.update_job_progress(
                job_id,
                0,
                ImportJobStatus::Failed,
                Some(errors.join("\n")),
            )
            .await;
            let _ = tokio::fs::remove_file(file_path).await;
            return Err(());
        }

        let mut processed_rows = 0;
        let mut skipped_rows = 0;

        for emp in pending_employees {
            match Self::process_employee_import_row(&pool, job, &emp).await {
                Ok(maybe_id) => {
                    let employee_id = if let Some(id) = maybe_id {
                        processed_rows += 1;
                        id
                    } else {
                        skipped_rows += 1;
                        // If already exists, we still want to try claiming logs for this ID
                        match self.employee_repo.get_employee_by_id_number(
                            &emp.id_number,
                            job.workspace_id,
                        )
                        .await
                        {
                            Ok(Some(e)) => e.id,
                            _ => continue,
                        }
                    };

                    // Auto-claim any staged logs for this employee
                    if let Err(e) = {
                        let te_repo = self.time_entry_repo.clone();
                        let ps = std::sync::Arc::new(crate::services::pairing_service::PairingService::new(te_repo, std::sync::Arc::new(crate::repositories::roster_repository::SqlxRosterRepository::new(pool.clone())), pool.clone()));
                        let emp_repo: std::sync::Arc<dyn crate::repositories::EmployeeRepository> = std::sync::Arc::new(crate::repositories::SqlxEmployeeRepository::new(pool.clone()));
                        let rl_repo: std::sync::Arc<dyn crate::repositories::raw_attendance_log_repository::RawAttendanceLogRepository> = std::sync::Arc::new(crate::repositories::raw_attendance_log_repository::SqlxRawAttendanceLogRepository::new(pool.clone()));
                        let st_repo: std::sync::Arc<dyn crate::repositories::staging_log_repository::StagingLogRepository> = std::sync::Arc::new(crate::repositories::staging_log_repository::SqlxStagingLogRepository::new(pool.clone()));
                        let er_repo: std::sync::Arc<dyn crate::repositories::attendance_log_error_repository::AttendanceLogErrorRepository> = std::sync::Arc::new(crate::repositories::attendance_log_error_repository::SqlxAttendanceLogErrorRepository::new(pool.clone()));
                        let (log_svc, _h) = crate::services::AttendanceLogService::new(pool.clone(), ps, 1, 1, 1000, emp_repo, rl_repo, st_repo, er_repo);
                        log_svc.claim_staged_logs(job.workspace_id, employee_id, &emp.id_number).await
                    } {
                        error!("Failed to auto-claim logs for employee {}: {}", emp.id_number, e);
                    }

                    if (processed_rows + skipped_rows) % 10 == 0 {
                        let _ = self.import_repo.update_job_progress(
                            job_id,
                            processed_rows,
                            ImportJobStatus::Processing,
                            None,
                        )
                        .await;
                    }
                }
                Err(e) => {
                    errors.push(format!(
                        "Dòng {} (ID {}): Lỗi khi lưu vào DB: {}",
                        emp.line_number, emp.id_number, e
                    ));
                }
            }
        }

        let status = if !errors.is_empty() {
            ImportJobStatus::Failed
        } else {
            ImportJobStatus::Completed
        };

        let mut final_log = if errors.is_empty() {
            None
        } else {
            Some(errors.join("\n"))
        };
        if skipped_rows > 0 {
            let skip_msg = format!("Đã bỏ qua {skipped_rows} nhân viên trùng mã.");
            final_log = Some(match final_log {
                Some(log) => format!("{log}\n{skip_msg}"),
                None => skip_msg,
            });
        }

        let _ = self.import_repo.update_job_progress(
            job_id,
            processed_rows,
            status,
            final_log,
        )
        .await;
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
            let raw_id = match row.first() {
                Some(Data::String(s)) => s.clone(),
                Some(Data::Float(f)) => f.to_string(),
                Some(Data::Int(i)) => i.to_string(),
                _ => {
                    errors.push(format!("Dòng {current_line}: Thiếu ID nhân viên"));
                    continue;
                }
            };
            let id_number = normalize_employee_id(&raw_id);
            let full_name = match row.get(1) {
                Some(Data::String(s)) => s.trim().to_string(),
                _ => {
                    errors.push(format!("Dòng {current_line}: Thiếu Tên nhân viên"));
                    continue;
                }
            };
            let hire_date = match row.get(2) {
                Some(Data::String(s)) => NaiveDate::parse_from_str(s.trim(), "%Y-%m-%d")
                    .or_else(|_| NaiveDate::parse_from_str(s.trim(), "%d/%m/%Y"))
                    .or_else(|_| NaiveDate::parse_from_str(s.trim(), "%m/%d/%Y"))
                    .unwrap_or_else(|_| {
                        Utc::now()
                            .with_timezone(&Tz::Asia__Ho_Chi_Minh)
                            .date_naive()
                    }),
                Some(cell_data) => cell_data
                    .as_datetime()
                    .map(|ndt| ndt.date())
                    .unwrap_or_else(|| {
                        Utc::now()
                            .with_timezone(&Tz::Asia__Ho_Chi_Minh)
                            .date_naive()
                    }),
                _ => Utc::now()
                    .with_timezone(&Tz::Asia__Ho_Chi_Minh)
                    .date_naive(),
            };

            pending.push(PendingEmployee {
                id_number,
                full_name,
                hire_date,
                line_number: current_line,
            });
        }
    }

    async fn process_employee_import_row(
        pool: &PgPool,
        job: &AttendanceImportJob,
        emp: &PendingEmployee,
    ) -> Result<Option<Uuid>, String> {
        let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

        let employee_id = Uuid::new_v4();
        let employee = Employee {
            id: employee_id,
            workspace_id: job.workspace_id,
            user_id: None,
            full_name: emp.full_name.clone(),
            employee_id_number: emp.id_number.clone(),
            hire_date: emp.hire_date,
            termination_date: None,
            tags: Vec::new(),
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let inserted = employee_repository::create_employee_tx_skip_duplicates(&mut tx, &employee)
            .await
            .map_err(|e| e.to_string())?;

        tx.commit().await.map_err(|e| e.to_string())?;

        if inserted {
            Ok(Some(employee_id))
        } else {
            Ok(None)
        }
    }

    fn parse_excel_range(
        range: calamine::Range<Data>,
        pending_rows: &mut Vec<PendingRow>,
        errors: &mut Vec<String>,
        preferred_date: Option<&str>,
        preferred_time: Option<&str>,
        timezone: &str,
    ) {
        for (idx, row) in range.rows().skip(1).enumerate() {
            let current_line = idx + 2;
            let raw_id = match row.first() {
                Some(Data::String(s)) => s.clone(),
                Some(Data::Float(f)) => f.to_string(),
                Some(Data::Int(i)) => i.to_string(),
                _ => {
                    errors.push(format!("Dòng {current_line}: Thiếu mã nhân viên"));
                    continue;
                }
            };
            let emp_id_number = normalize_employee_id(&raw_id);

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
                            "Dòng {current_line} (NV {emp_id_number}): {e} [Dữ liệu gốc: '{s}']"
                        ));
                        None
                    }
                },
                Some(cell_data) => {
                    if let Some(naive) = cell_data.as_datetime() {
                        let tz: Tz = timezone.parse().unwrap_or(chrono_tz::Asia::Ho_Chi_Minh);
                        match tz.from_local_datetime(&naive) {
                            chrono::LocalResult::Single(dt) => Some(dt.with_timezone::<Utc>(&Utc)),
                            _ => Some(DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc)),
                        }
                    } else {
                        errors.push(format!("Dòng {current_line} (NV {emp_id_number}): Không phải định dạng ngày giờ hợp lệ [Dữ liệu gốc: '{cell_data}']"));
                        None
                    }
                }
                _ => {
                    errors.push(format!(
                        "Dòng {current_line} (NV {emp_id_number}): Thiếu dữ liệu thời gian"
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
        &self,
        job: &AttendanceImportJob,
        emp_id_number: &str,
        punch_time_utc: DateTime<Utc>,
    ) -> Result<usize, AppError> {
        let pool = self.pool.clone();
        let employee = match self.employee_repo.get_employee_by_id_number(
            emp_id_number,
            job.workspace_id,
        )
        .await
        {
            Ok(Some(emp)) => emp,
            _ => {
                error!(
                    "EMPLOYEE_NOT_FOUND: {} in WS {}",
                    emp_id_number, job.workspace_id
                );
                return Ok(0);
            }
        };

        if job.target_type == "log" {
            let te_repo = self.time_entry_repo.clone();
            let ps = std::sync::Arc::new(crate::services::pairing_service::PairingService::new(te_repo, std::sync::Arc::new(crate::repositories::roster_repository::SqlxRosterRepository::new(pool.clone())), pool.clone()));
            let emp_repo: std::sync::Arc<dyn crate::repositories::EmployeeRepository> = std::sync::Arc::new(crate::repositories::SqlxEmployeeRepository::new(pool.clone()));
            let rl_repo: std::sync::Arc<dyn crate::repositories::raw_attendance_log_repository::RawAttendanceLogRepository> = std::sync::Arc::new(crate::repositories::raw_attendance_log_repository::SqlxRawAttendanceLogRepository::new(pool.clone()));
            let st_repo: std::sync::Arc<dyn crate::repositories::staging_log_repository::StagingLogRepository> = std::sync::Arc::new(crate::repositories::staging_log_repository::SqlxStagingLogRepository::new(pool.clone()));
            let er_repo: std::sync::Arc<dyn crate::repositories::attendance_log_error_repository::AttendanceLogErrorRepository> = std::sync::Arc::new(crate::repositories::attendance_log_error_repository::SqlxAttendanceLogErrorRepository::new(pool.clone()));
            let (log_svc, _h) = crate::services::AttendanceLogService::new(pool.clone(), ps, 1, 1, 1000, emp_repo, rl_repo, st_repo, er_repo);
            match log_svc.process_parsed_log(
                job.workspace_id,
                employee.id,
                punch_time_utc,
                LogSource::Import,
                None,
                None,
            )
            .await
            {
                Ok(true) => Ok(1),
                Ok(false) => Ok(0),
                Err(e) => {
                    error!("UNIFIED_LOG_SERVICE_ERROR: {}", e);
                    Err(AppError::Internal("Internal error".into()))
                }
            }
        } else {
            let req = CreateAttendanceAdjustmentRequest {
                employee_id: employee.id,
                time_entry_id: None,
                requested_punch_in_time: punch_time_utc,
                requested_punch_out_time: None,
                is_break_overtime: false,
                reason: Some("Import from Excel/CSV".to_string()),
            };

            let adj_svc = {
                let adj_repo = self.adj_repo.clone();
                let te_repo = self.time_entry_repo.clone();
                crate::services::attendance_adjustment_service::AttendanceAdjustmentService::new(adj_repo, te_repo, pool.clone())
            };
            if let Ok(adjustment) = adj_svc.submit_request(
                job.workspace_id,
                job.user_id,
                req,
            )
            .await
            {
                let _ = adj_svc.approve_request(
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
