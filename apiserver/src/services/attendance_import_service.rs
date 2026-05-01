use std::sync::Arc;

use crate::errors::AppError;

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
use crate::repositories::{employee, EmployeeRepository, TimeEntryRepository};
use crate::services::attendance_adjustment_service::AttendanceAdjustmentService;
use crate::services::import_messages::{fmt_msg, AttendanceImportMessages};
use crate::utils::normalize_employee_id;
use std::collections::HashMap;

struct PendingRow {
    emp_id_number: String,
    punch_time: DateTime<Utc>,
    line_number: usize,
}

struct PendingEmployee {
    id_number: String,
    full_name: String,
    hire_date: NaiveDate,
    department_name: Option<String>,
    line_number: usize,
}

#[derive(Clone)]
pub struct AttendanceImportService {
    import_repo: Arc<dyn ImportJobRepository>,
    adj_repo: Arc<dyn AttendanceAdjustmentRepository>,
    employee_repo: Arc<dyn EmployeeRepository>,
    time_entry_repo: Arc<dyn TimeEntryRepository>,
    workspace_repo: Arc<dyn crate::repositories::workspace_repository::WorkspaceRepository>,
    pool: PgPool,
    redis: Option<crate::queue::RedisPool>,
}

impl AttendanceImportService {
    pub fn new(
        import_repo: Arc<dyn ImportJobRepository>,
        adj_repo: Arc<dyn AttendanceAdjustmentRepository>,
        employee_repo: Arc<dyn EmployeeRepository>,
        time_entry_repo: Arc<dyn TimeEntryRepository>,
        workspace_repo: Arc<dyn crate::repositories::workspace_repository::WorkspaceRepository>,
        pool: PgPool,
        redis: Option<crate::queue::RedisPool>,
    ) -> Self {
        Self {
            import_repo,
            adj_repo,
            employee_repo,
            time_entry_repo,
            workspace_repo,
            pool,
            redis,
        }
    }
    #[tracing::instrument(skip_all, err)]
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

        self.import_repo.create_job(&job).await.map_err(|e| {
            error!("CREATE_JOB_ERROR: {:?}", e);
            AppError::Internal("Internal error".into())
        })?;

        let svc = self.clone();
        tokio::spawn(async move {
            let _ = svc.process_job(job_id).await;
        });

        Ok(job_id)
    }

    #[tracing::instrument(skip_all)]
    pub async fn resume_all_pending_jobs(&self) {
        if let Ok(jobs) = self.import_repo.get_pending_jobs().await {
            for job in jobs {
                info!(
                    "Resuming import job: {} (target: {})",
                    job.id, job.target_type
                );
                let svc = self.clone();
                tokio::spawn(async move {
                    let _ = svc.process_job(job.id).await;
                });
            }
        }
    }

    async fn process_job(&self, job_id: Uuid) -> Result<(), ()> {
        let job = match self.import_repo.get_job_by_id(job_id).await {
            Ok(Some(j)) => j,
            _ => return Err(()),
        };

        let workspace = match self
            .workspace_repo
            .get_workspace_by_id(job.workspace_id)
            .await
        {
            Ok(Some(ws)) => ws,
            _ => {
                error!("WORKSPACE_NOT_FOUND for job {}", job_id);
                let _ = self
                    .import_repo
                    .update_job_progress(
                        job_id,
                        job.processed_rows,
                        ImportJobStatus::Failed,
                        Some("Workspace not found".to_string()),
                    )
                    .await;
                return Err(());
            }
        };

        let date_format = workspace.date_format.clone();
        let time_format = workspace.time_format.clone();
        let timezone = workspace.timezone.clone();
        let lang = &workspace.default_language;
        let msgs = AttendanceImportMessages::for_lang(lang);

        if job.target_type == "employee" {
            return self.process_employee_job(job_id, &job, &msgs).await;
        }

        let file_path = match &job.file_path {
            Some(p) => p,
            None => return Err(()),
        };

        let file_bytes = match tokio::fs::read(file_path).await {
            Ok(b) => b,
            Err(e) => {
                error!("READ_FILE_ERROR for job {}: {:?}", job_id, e);
                let _ = self
                    .import_repo
                    .update_job_progress(
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

        match crate::utils::file_parser::extract_file_rows(&file_bytes) {
            Ok(rows) => {
                for (current_line, cols) in rows {
                    let raw_emp_id = cols.first().map(|s| s.as_str()).unwrap_or("").trim();
                    let emp_id_number = normalize_employee_id(raw_emp_id);
                    let emp_dt_str = cols.get(2).map(|s| s.as_str()).unwrap_or("").trim();

                    if emp_id_number.is_empty() || emp_dt_str.is_empty() {
                        errors.push(fmt_msg(
                            msgs.missing_employee_or_time,
                            &[("line", &current_line.to_string())],
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
                        Err(_e) => {
                            errors.push(format!(
                                "{} [Raw: '{emp_dt_str}']",
                                fmt_msg(
                                    msgs.missing_employee_or_time,
                                    &[("line", &current_line.to_string())],
                                )
                            ));
                        }
                    };
                }
            }
            Err(e) => {
                errors.push(e);
            }
        }

        let total_rows = pending_rows.len() as i32 + errors.len() as i32;

        let _ = self
            .import_repo
            .update_job_total_rows(job_id, total_rows)
            .await;

        if !errors.is_empty() {
            let _ = self
                .import_repo
                .update_job_progress(job_id, 0, ImportJobStatus::Failed, Some(errors.join("\n")))
                .await;
            let _ = tokio::fs::remove_file(file_path).await;
            return Err(());
        }

        let unique_emp_ids: HashSet<String> = pending_rows
            .iter()
            .map(|r| r.emp_id_number.clone())
            .collect();
        let unique_emp_ids_vec: Vec<String> = unique_emp_ids.iter().cloned().collect();

        let known_emp_ids: HashSet<String> = match self
            .employee_repo
            .get_existing_employee_ids_by_numbers(job.workspace_id, &unique_emp_ids_vec)
            .await
        {
            Ok(existing_ids) => {
                let known: HashSet<String> = existing_ids.into_iter().collect();
                let missing_ids: Vec<String> = unique_emp_ids.difference(&known).cloned().collect();

                if !missing_ids.is_empty() {
                    errors.push(fmt_msg(
                        msgs.skipped_unknown_employees,
                        &[
                            ("count", &missing_ids.len().to_string()),
                            ("ids", &missing_ids.join(", ")),
                        ],
                    ));
                }
                known
            }
            Err(e) => {
                error!("VALIDATION_ERROR: {:?}", e);
                let _ = self
                    .import_repo
                    .update_job_progress(
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

            match self
                .process_import_row(&job, &row.emp_id_number, row.punch_time)
                .await
            {
                Ok(inserted) => {
                    if inserted > 0 {
                        processed_rows += 1;
                    } else {
                        skipped_rows += 1;
                    }

                    if (processed_rows + skipped_rows) % 10 == 0 {
                        let _ = self
                            .import_repo
                            .update_job_progress(
                                job_id,
                                processed_rows,
                                ImportJobStatus::Processing,
                                None,
                            )
                            .await;
                    }
                }
                Err(_) => {
                    errors.push(fmt_msg(
                        msgs.system_save_error,
                        &[
                            ("emp", &row.emp_id_number),
                            ("line", &row.line_number.to_string()),
                        ],
                    ));
                }
            }
        }

        if skipped_rows > 0 {
            errors.push(fmt_msg(
                msgs.skipped_rows,
                &[("count", &skipped_rows.to_string())],
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
        let _ = self
            .import_repo
            .update_job_progress(job_id, processed_rows, status, error_log)
            .await;

        let _ = tokio::fs::remove_file(file_path).await;
        Ok(())
    }

    async fn process_employee_job(
        &self,
        job_id: Uuid,
        job: &AttendanceImportJob,
        msgs: &AttendanceImportMessages,
    ) -> Result<(), ()> {
        let pool = self.pool.clone();

        let workspace_tz = match self
            .workspace_repo
            .get_workspace_by_id(job.workspace_id)
            .await
        {
            Ok(Some(ws)) => ws.timezone.parse::<Tz>().unwrap_or(Tz::UTC),
            _ => Tz::UTC,
        };

        let file_path = match &job.file_path {
            Some(p) => p,
            None => return Err(()),
        };

        let file_bytes = match tokio::fs::read(file_path).await {
            Ok(b) => b,
            Err(e) => {
                error!("READ_FILE_ERROR for job {}: {:?}", job_id, e);
                let _ = self
                    .import_repo
                    .update_job_progress(
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

        match crate::utils::file_parser::extract_file_rows(&file_bytes) {
            Ok(rows) => {
                for (current_line, cols) in rows {
                    let raw_id = cols.first().map(|s| s.as_str()).unwrap_or("").trim();
                    let id_number = normalize_employee_id(raw_id);
                    let full_name = cols
                        .get(1)
                        .map(|s| s.as_str())
                        .unwrap_or("")
                        .trim()
                        .to_string();
                    let hire_date_raw = cols.get(2).map(|s| s.as_str()).unwrap_or("").trim();

                    if id_number.is_empty() || full_name.is_empty() {
                        errors.push(fmt_msg(
                            msgs.missing_employee_or_id,
                            &[("line", &current_line.to_string())],
                        ));
                        continue;
                    }

                    // Extract date part if file_parser included time string (e.g. from Excel Datetime)
                    let hire_date_str = hire_date_raw.split(' ').next().unwrap_or(hire_date_raw);

                    let hire_date = if hire_date_str.is_empty() {
                        Utc::now().with_timezone(&workspace_tz).date_naive()
                    } else {
                        NaiveDate::parse_from_str(hire_date_str, "%Y-%m-%d")
                            .or_else(|_| NaiveDate::parse_from_str(hire_date_str, "%d/%m/%Y"))
                            .or_else(|_| NaiveDate::parse_from_str(hire_date_str, "%m/%d/%Y"))
                            .unwrap_or_else(|_| {
                                Utc::now().with_timezone(&workspace_tz).date_naive()
                            })
                    };

                    let department_name = cols
                        .get(3)
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty());

                    pending_employees.push(PendingEmployee {
                        id_number,
                        full_name,
                        hire_date,
                        department_name,
                        line_number: current_line,
                    });
                }
            }
            Err(e) => {
                errors.push(e);
            }
        }

        let total_rows = pending_employees.len() as i32 + errors.len() as i32;
        let _ = self
            .import_repo
            .update_job_total_rows(job_id, total_rows)
            .await;

        if !errors.is_empty() {
            let _ = self
                .import_repo
                .update_job_progress(job_id, 0, ImportJobStatus::Failed, Some(errors.join("\n")))
                .await;
            let _ = tokio::fs::remove_file(file_path).await;
            return Err(());
        }

        let mut processed_rows = 0;
        let mut skipped_rows = 0;

        // Cache department lookups by name (lowercased) -> department_id
        let mut dept_cache: HashMap<String, Uuid> = HashMap::new();
        // Pre-load existing departments
        if let Ok(existing_depts) =
            crate::repositories::department_repository::get_departments(&pool, job.workspace_id)
                .await
        {
            for dept in existing_depts {
                dept_cache.insert(dept.name.to_lowercase(), dept.id);
            }
        }

        for emp in pending_employees {
            match Self::process_employee_import_row(&pool, job, &emp).await {
                Ok(maybe_id) => {
                    let employee_id = if let Some(id) = maybe_id {
                        processed_rows += 1;
                        id
                    } else {
                        skipped_rows += 1;
                        // If already exists, we still want to try claiming logs for this ID
                        match self
                            .employee_repo
                            .get_employee_by_id_number(&emp.id_number, job.workspace_id)
                            .await
                        {
                            Ok(Some(e)) => e.id,
                            _ => continue,
                        }
                    };

                    // Auto-assign to department if column D was provided
                    if let Some(ref dept_name) = emp.department_name {
                        let dept_key = dept_name.to_lowercase();
                        let dept_id = if let Some(&cached_id) = dept_cache.get(&dept_key) {
                            cached_id
                        } else {
                            // Create the department
                            match crate::repositories::department_repository::create_department(
                                &pool,
                                job.workspace_id,
                                dept_name.clone(),
                                None,
                                None,
                                None,
                            )
                            .await
                            {
                                Ok(new_dept) => {
                                    dept_cache.insert(dept_key, new_dept.id);
                                    new_dept.id
                                }
                                Err(e) => {
                                    error!("Failed to create department '{}': {}", dept_name, e);
                                    // Skip department assignment but continue with employee
                                    Uuid::nil()
                                }
                            }
                        };
                        if !dept_id.is_nil() {
                            if let Err(e) =
                                crate::repositories::department_repository::transfer_employee(
                                    &pool,
                                    employee_id,
                                    dept_id,
                                    emp.hire_date,
                                    Some("Auto-assigned via bulk import".to_string()),
                                )
                                .await
                            {
                                error!(
                                    "Failed to assign employee {} to department '{}': {}",
                                    emp.id_number, dept_name, e
                                );
                            }
                        }
                    }

                    // Enqueue claim-staged-logs job via queue runner
                    if let Some(ref redis) = self.redis {
                        let queue = crate::queue::JobQueue::new(redis.clone());
                        let claim_job = crate::queue::Job::new(
                            Uuid::new_v4(),
                            0,
                            crate::queue::TaskType::ClaimStagedLogs,
                            job.workspace_id,
                            crate::queue::JobPayload::ClaimStagedLogs(
                                crate::queue::ClaimStagedLogsPayload {
                                    workspace_id: job.workspace_id,
                                    employee_id,
                                    employee_id_number: emp.id_number.clone(),
                                },
                            ),
                        );
                        if let Err(e) = queue.enqueue_jobs(vec![claim_job]).await {
                            error!(
                                "Failed to enqueue claim-staged-logs for {}: {}",
                                emp.id_number, e
                            );
                        }
                    } else {
                        // Fallback: claim staged logs directly when no queue runner is available
                        if let Err(e) =
                            crate::repositories::staging_log_repository::claim_staged_logs_direct(
                                &self.pool,
                                job.workspace_id,
                                employee_id,
                                &emp.id_number,
                            )
                            .await
                        {
                            error!(
                                "Failed to claim staged logs directly for {}: {}",
                                emp.id_number, e
                            );
                        }
                    }

                    if (processed_rows + skipped_rows) % 10 == 0 {
                        let _ = self
                            .import_repo
                            .update_job_progress(
                                job_id,
                                processed_rows,
                                ImportJobStatus::Processing,
                                None,
                            )
                            .await;
                    }
                }
                Err(e) => {
                    errors.push(fmt_msg(
                        msgs.db_save_error,
                        &[
                            ("line", &emp.line_number.to_string()),
                            ("emp", &emp.id_number),
                            ("error", &e.to_string()),
                        ],
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
            let skip_msg = fmt_msg(
                msgs.skipped_employees,
                &[("count", &skipped_rows.to_string())],
            );
            final_log = Some(match final_log {
                Some(log) => format!("{log}\n{skip_msg}"),
                None => skip_msg,
            });
        }

        let _ = self
            .import_repo
            .update_job_progress(job_id, processed_rows, status, final_log)
            .await;
        let _ = tokio::fs::remove_file(file_path).await;

        Ok(())
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
            department_name: None,
        };

        let inserted = employee::create_employee_tx_skip_duplicates(&mut tx, &employee)
            .await
            .map_err(|e| e.to_string())?;

        tx.commit().await.map_err(|e| e.to_string())?;

        if inserted {
            Ok(Some(employee_id))
        } else {
            Ok(None)
        }
    }

    async fn process_import_row(
        &self,
        job: &AttendanceImportJob,
        emp_id_number: &str,
        punch_time_utc: DateTime<Utc>,
    ) -> Result<usize, AppError> {
        let pool = self.pool.clone();
        let employee = match self
            .employee_repo
            .get_employee_by_id_number(emp_id_number, job.workspace_id)
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
            let ps = std::sync::Arc::new(crate::services::pairing_service::PairingService::new(
                te_repo,
                std::sync::Arc::new(crate::repositories::roster::SqlxRosterRepository::new(
                    pool.clone(),
                )),
                pool.clone(),
            ));
            let emp_repo: std::sync::Arc<dyn crate::repositories::EmployeeRepository> =
                std::sync::Arc::new(crate::repositories::SqlxEmployeeRepository::new(
                    pool.clone(),
                ));
            let rl_repo: std::sync::Arc<dyn crate::repositories::raw_attendance_log_repository::RawAttendanceLogRepository> = std::sync::Arc::new(crate::repositories::raw_attendance_log_repository::SqlxRawAttendanceLogRepository::new(pool.clone()));
            let st_repo: std::sync::Arc<
                dyn crate::repositories::staging_log_repository::StagingLogRepository,
            > = std::sync::Arc::new(
                crate::repositories::staging_log_repository::SqlxStagingLogRepository::new(
                    pool.clone(),
                ),
            );
            let er_repo: std::sync::Arc<dyn crate::repositories::attendance_log_error_repository::AttendanceLogErrorRepository> = std::sync::Arc::new(crate::repositories::attendance_log_error_repository::SqlxAttendanceLogErrorRepository::new(pool.clone()));
            let (log_svc, _h) = crate::services::AttendanceLogService::new(
                pool.clone(),
                ps,
                crate::services::attendance_log_service::AttendanceLogServiceConfig {
                    buffer_capacity: 1,
                    batch_size: 1,
                    flush_interval_ms: 1000,
                },
                crate::services::attendance_log_service::AttendanceLogServiceDeps {
                    employee_repo: emp_repo,
                    raw_log_repo: rl_repo,
                    staging_repo: st_repo,
                    error_repo: er_repo,
                },
            );
            match log_svc
                .process_parsed_log(
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
                crate::services::attendance_adjustment_service::AttendanceAdjustmentService::new(
                    adj_repo,
                    te_repo,
                    pool.clone(),
                )
            };
            if let Ok(adjustment) = adj_svc
                .submit_request(job.workspace_id, job.user_id, req)
                .await
            {
                let _ = adj_svc
                    .approve_request(
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
