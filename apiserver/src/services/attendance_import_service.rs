use std::sync::Arc;

use crate::errors::AppError;

use chrono::{DateTime, NaiveDate, Utc};
use chrono_tz::Tz;

use tracing::{error, info};
use uuid::Uuid;

use crate::models::{
    enums::ImportTargetType, AdjustmentStatus, AttendanceImportJob,
    CreateAttendanceAdjustmentRequest, Employee, ImportJobStatus, LogSource,
};
use crate::repositories::attendance_adjustment_repository::AttendanceAdjustmentRepository;
use crate::repositories::employee::{EmployeeCoreRepository, EmployeeRepository};
use crate::repositories::import_job_repository::ImportJobRepository;
use crate::repositories::TimeEntryRepository;
use crate::services::attendance_adjustment_service::AttendanceAdjustmentService;
use crate::services::import_messages::{fmt_msg, AttendanceImportMessages};
use std::collections::HashMap;

#[derive(Clone)]
struct PendingRow {
    emp_id_number: String,
    punch_time: DateTime<Utc>,
    line_number: usize,
}

#[derive(Debug, Clone)]
pub struct PendingEmployee {
    pub id_number: String,
    pub full_name: String,
    pub hire_date: chrono::NaiveDate,
    pub department_name: Option<String>,
    pub line_number: usize,
}

use crate::services::import_pipeline::{ImportPipeline, ImportStrategy};

pub struct EmployeeImportStrategy {
    pub msgs: AttendanceImportMessages,
    pub employee_repo: Arc<dyn EmployeeRepository>,
    pub department_repo: Arc<dyn crate::repositories::department_repository::DepartmentRepository>,
    pub staging_repo: Arc<dyn crate::repositories::staging_log_repository::StagingLogRepository>,
    pub dept_cache: HashMap<String, Uuid>,
    pub emp_cache: HashMap<String, Uuid>,
    pub newly_added_emps: std::collections::HashSet<String>,
}

#[allow(clippy::too_many_lines)]
#[async_trait::async_trait]
impl ImportStrategy for EmployeeImportStrategy {
    type RowData = PendingEmployee;

    fn expected_headers(&self) -> &'static [&'static str] {
        &["employee_code", "full_name", "hire_date", "department_name"]
    }

    fn map_row(
        &mut self,
        row_data: &HashMap<String, String>,
        current_line: usize,
        workspace_tz: Tz,
    ) -> Result<Self::RowData, AppError> {
        let raw_id = row_data
            .get("employee_code")
            .map_or("", std::string::String::as_str)
            .trim();
        let id_number = crate::utils::normalize_employee_id(raw_id);
        let full_name = row_data
            .get("full_name")
            .map_or("", std::string::String::as_str)
            .trim()
            .to_string();
        let hire_date_raw = row_data
            .get("hire_date")
            .map_or("", std::string::String::as_str)
            .trim();

        if id_number.is_empty() || full_name.is_empty() {
            return Err(AppError::Validation(fmt_msg(
                self.msgs.missing_employee_or_id,
                &[("line", &current_line.to_string())],
            ).into()));
        }

        let hire_date_str = hire_date_raw.split(' ').next().unwrap_or(hire_date_raw);
        let hire_date = if hire_date_str.is_empty() {
            Utc::now().with_timezone(&workspace_tz).date_naive()
        } else {
            NaiveDate::parse_from_str(hire_date_str, "%Y-%m-%d")
                .or_else(|_| NaiveDate::parse_from_str(hire_date_str, "%d/%m/%Y"))
                .or_else(|_| NaiveDate::parse_from_str(hire_date_str, "%m/%d/%Y"))
                .unwrap_or_else(|_| Utc::now().with_timezone(&workspace_tz).date_naive())
        };

        let department_name = row_data
            .get("department_name")
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());

        Ok(PendingEmployee {
            id_number,
            full_name,
            hire_date,
            department_name,
            line_number: current_line,
        })
    }

    async fn init(
        &mut self,
        _deps: &crate::worker::deps::WorkerDeps,
        job: &AttendanceImportJob,
        _rows: &[Self::RowData],
    ) -> Result<(), AppError> {
        if let Ok(existing_depts) = self
            .department_repo
            .get_departments(
                &crate::diesel_pool::TenantAuthToken::system_override(job.workspace_id),
                1,
                3000,
            )
            .await
        {
            for dept in existing_depts.0 {
                self.dept_cache
                    .insert(dept.name().to_lowercase(), dept.id());
            }
        }
        if let Ok(existing_emps) = self
            .employee_repo
            .get_active_employees(&crate::diesel_pool::TenantAuthToken::system_override(
                job.workspace_id,
            ))
            .await
        {
            for e in existing_emps {
                self.emp_cache
                    .insert(e.employee_id_number().to_lowercase(), e.id());
            }
        }
        Ok(())
    }

    async fn pre_process_chunk(
        &mut self,
        _deps: &crate::worker::deps::WorkerDeps,
        job: &AttendanceImportJob,
        rows: &[Self::RowData],
    ) -> Result<(), AppError> {
        if rows.is_empty() {
            return Ok(());
        }

        let auth_token = crate::diesel_pool::TenantAuthToken::system_override(job.workspace_id);

        let mut emp_ids = Vec::new();
        let mut dept_names = Vec::new();
        for row in rows {
            emp_ids.push(row.id_number.clone());
            if let Some(dept) = &row.department_name {
                dept_names.push(dept.clone());
            }
        }

        if let Ok(existing_depts) = self
            .department_repo
            .get_department_ids_by_names(&auth_token, &dept_names)
            .await
        {
            for (name, id) in existing_depts {
                self.dept_cache.insert(name, id);
            }
        }

        if let Ok(existing_emps) = self
            .employee_repo
            .get_employee_ids_by_numbers(&auth_token, &emp_ids)
            .await
        {
            for (num, id) in existing_emps {
                self.emp_cache.insert(num.to_lowercase(), id);
            }
        }

        let mut new_depts = Vec::new();
        let mut unique_dept_names = std::collections::HashSet::new();
        for name in dept_names {
            let key = name.to_lowercase();
            if !self.dept_cache.contains_key(&key) && unique_dept_names.insert(key.clone()) {
                let dept_id = Uuid::new_v4();
                new_depts.push(crate::models::Department::new(
                    dept_id,
                    job.workspace_id,
                    name.clone(),
                    None,
                    None,
                    Utc::now(),
                    Utc::now(),
                    0,
                ));
                self.dept_cache.insert(key, dept_id);
            }
        }
        if !new_depts.is_empty() {
            let _ = self
                .department_repo
                .bulk_create_departments_skip_duplicates(&auth_token, &new_depts)
                .await;
            let newly_inserted_names: Vec<String> = new_depts
                .into_iter()
                .map(|d| d.name().to_string())
                .collect();
            if let Ok(existing_depts) = self
                .department_repo
                .get_department_ids_by_names(&auth_token, &newly_inserted_names)
                .await
            {
                for (name, id) in existing_depts {
                    self.dept_cache.insert(name, id);
                }
            }
        }

        let Ok(Some(_)) =
            EmployeeCoreRepository::get_headquarters_location_id(&*self.employee_repo, &auth_token)
                .await
        else {
            return Err(AppError::Internal(
                "Workspace headquarters location not found".into(),
            ));
        };

        let mut new_emps = Vec::new();
        let mut unique_emp_ids = std::collections::HashSet::new();
        for row in rows {
            let key = row.id_number.to_lowercase();
            if !self.emp_cache.contains_key(&key) && unique_emp_ids.insert(key.clone()) {
                let emp_id = Uuid::new_v4();
                let hire_date_dt = row
                    .hire_date
                    .and_hms_opt(0, 0, 0)
                    .and_then(|t| t.and_local_timezone(Utc).single())
                    .unwrap_or_else(Utc::now);

                new_emps.push(crate::models::Employee::new(
                    emp_id,
                    job.workspace_id,
                    None,
                    row.full_name.clone(),
                    row.id_number.clone(),
                    Vec::new(),
                    hire_date_dt,
                    Utc::now(),
                    None,
                ));
                self.emp_cache.insert(key.clone(), emp_id);
                self.newly_added_emps.insert(key);
            }
        }
        if !new_emps.is_empty() {
            self.employee_repo
                .bulk_create_employees_skip_duplicates(&auth_token, &new_emps)
                .await?;
            let newly_inserted_ids: Vec<String> = new_emps
                .into_iter()
                .map(|e| e.employee_id_number().to_string())
                .collect();
            if let Ok(existing_emps) = self
                .employee_repo
                .get_employee_ids_by_numbers(&auth_token, &newly_inserted_ids)
                .await
            {
                for (num, id) in existing_emps {
                    self.emp_cache.insert(num.to_lowercase(), id);
                }
            }
        }

        Ok(())
    }

    async fn process_row(
        &mut self,
        _deps: &crate::worker::deps::WorkerDeps,
        job: &AttendanceImportJob,
        emp: &Self::RowData,
    ) -> Result<Option<Uuid>, AppError> {
        let emp_key = emp.id_number.to_lowercase();
        // pre_process_chunk has already populated emp_cache and inserted new employees to DB.
        // We just fetch from cache.
        let Some(&employee_id) = self.emp_cache.get(&emp_key) else {
            return Err(AppError::Internal(fmt_msg(
                self.msgs.db_save_error,
                &[
                    ("line", &emp.line_number.to_string()),
                    ("emp", &emp.id_number),
                    (
                        "error",
                        "Employee could not be loaded from cache after bulk insert",
                    ),
                ],
            )));
        };

        // For this refactor, we still need to determine if it "was_new" to return Ok(Some(id))
        // We track newly inserted employees during pre_process_chunk in self.newly_added_emps
        let was_new = self.newly_added_emps.contains(&emp_key);

        let mut department_updated = false;
        if let Some(ref dept_name) = emp.department_name {
            let dept_key = dept_name.to_lowercase();
            let dept_id = if let Some(&cached_id) = self.dept_cache.get(&dept_key) {
                cached_id
            } else {
                Uuid::nil()
            };

            if !dept_id.is_nil()
                && self
                    .department_repo
                    .transfer_employee(
                        &crate::diesel_pool::TenantAuthToken::system_override(job.workspace_id),
                        employee_id,
                        dept_id,
                        emp.hire_date,
                        Some("Auto-assigned via bulk import".to_string()),
                    )
                    .await
                    .is_ok()
            {
                department_updated = true;
            }
        }

        // Claim staged logs directly
        let _ = self
            .staging_repo
            .claim_staged_logs(job.workspace_id, &emp.id_number, employee_id)
            .await;

        if was_new || department_updated {
            Ok(Some(employee_id))
        } else {
            Ok(None)
        }
    }
}

struct PunchImportStrategy {
    msgs: AttendanceImportMessages,
    date_format: String,
    time_format: String,
    adj_repo: Arc<dyn AttendanceAdjustmentRepository>,
    employee_repo: Arc<dyn EmployeeRepository>,
    log_svc: Option<Arc<crate::services::attendance_log_service::AttendanceLogService>>,
    emp_cache: std::collections::HashMap<String, Employee>,
}

#[async_trait::async_trait]
impl ImportStrategy for PunchImportStrategy {
    type RowData = PendingRow;

    fn expected_headers(&self) -> &'static [&'static str] {
        &["employee_code", "punch_time"]
    }

    fn map_row(
        &mut self,
        row_data: &HashMap<String, String>,
        current_line: usize,
        workspace_tz: Tz,
    ) -> Result<Self::RowData, AppError> {
        let raw_id = row_data
            .get("employee_code")
            .map_or("", std::string::String::as_str)
            .trim();
        let emp_id_number = crate::utils::normalize_employee_id(raw_id);
        let emp_dt_str = row_data
            .get("punch_time")
            .map_or("", std::string::String::as_str)
            .trim();

        if emp_id_number.is_empty() || emp_dt_str.is_empty() {
            return Err(AppError::Validation(fmt_msg(
                self.msgs.missing_employee_or_time,
                &[("line", &current_line.to_string())],
            ).into()));
        }

        match AttendanceAdjustmentService::parse_datetime(
            emp_dt_str,
            workspace_tz.name(),
            Some(&self.date_format),
            Some(&self.time_format),
        ) {
            Ok(dt) => Ok(PendingRow {
                emp_id_number,
                punch_time: dt,
                line_number: current_line,
            }),
            Err(_) => Err(AppError::Validation(format!(
                "{} [Raw: '{emp_dt_str}']",
                fmt_msg(
                    self.msgs.missing_employee_or_time,
                    &[("line", &current_line.to_string())],
                )
            ).into())),
        }
    }

    async fn init(
        &mut self,
        _deps: &crate::worker::deps::WorkerDeps,
        _job: &AttendanceImportJob,
        _rows: &[Self::RowData],
    ) -> Result<(), AppError> {
        Ok(())
    }

    #[allow(clippy::too_many_lines)]
    async fn process_row(
        &mut self,
        deps: &crate::worker::deps::WorkerDeps,
        job: &AttendanceImportJob,
        row: &Self::RowData,
    ) -> Result<Option<Uuid>, AppError> {
        let employee = if let Some(e) = self.emp_cache.get(&row.emp_id_number) {
            e.clone()
        } else {
            let Ok(Some(emp)) = self
                .employee_repo
                .get_employee_by_id_number(
                    &row.emp_id_number,
                    &crate::diesel_pool::TenantAuthToken::system_override(job.workspace_id),
                )
                .await
            else {
                return Err(AppError::Internal(fmt_msg(
                    self.msgs.system_save_error,
                    &[
                        ("emp", &row.emp_id_number),
                        ("line", &row.line_number.to_string()),
                    ],
                )));
            };
            self.emp_cache
                .insert(row.emp_id_number.clone(), emp.clone());
            emp
        };

        if job.target_type == ImportTargetType::Log {
            let Some(log_svc) = self.log_svc.as_ref() else {
                return Err(AppError::Internal(fmt_msg(
                    self.msgs.system_save_error,
                    &[
                        ("emp", &row.emp_id_number),
                        ("line", &row.line_number.to_string()),
                    ],
                )));
            };
            match log_svc
                .process_parsed_log(
                    job.workspace_id,
                    employee.id(),
                    row.punch_time,
                    LogSource::Import,
                    None,
                    None,
                )
                .await
            {
                Ok(true) => Ok(Some(Uuid::new_v4())),
                Ok(false) => Ok(None),
                Err(e) => {
                    error!("UNIFIED_LOG_SERVICE_ERROR: {}", e);
                    Err(AppError::Internal(fmt_msg(
                        self.msgs.system_save_error,
                        &[
                            ("emp", &row.emp_id_number),
                            ("line", &row.line_number.to_string()),
                        ],
                    )))
                }
            }
        } else {
            let Some(employment_id) = employee.active_employment_id() else {
                return Err(AppError::NotFound(
                    "error.active_employment_not_found".into(),
                ));
            };

            let req = CreateAttendanceAdjustmentRequest {
                employment_id,
                time_entry_id: None,
                requested_punch_time: row.punch_time,
                reason: Some("Import from Excel/CSV".to_string()),
                attachment_file_ids: None,
            };

            let adj_svc = AttendanceAdjustmentService::new(
                Arc::clone(&self.adj_repo),
                Arc::clone(&deps.raw_log_repo),
                deps.pairing_service(),
            );

            match adj_svc
                .submit_request(job.workspace_id, job.user_id, req)
                .await
            {
                Ok(adjustment) => {
                    let _ = adj_svc
                        .approve_request(
                            job.workspace_id,
                            job.user_id,
                            adjustment.id(),
                            AdjustmentStatus::Approved,
                        )
                        .await;
                    Ok(Some(adjustment.id()))
                }
                Err(e) => {
                    error!("ADJUSTMENT_SUBMIT_ERROR: {}", e);
                    Err(AppError::Internal(fmt_msg(
                        self.msgs.system_save_error,
                        &[
                            ("emp", &row.emp_id_number),
                            ("line", &row.line_number.to_string()),
                        ],
                    )))
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct AttendanceImportService {
    import_repo: Arc<dyn ImportJobRepository>,
    adj_repo: Arc<dyn AttendanceAdjustmentRepository>,
    employee_repo: Arc<dyn EmployeeRepository>,
    _time_entry_repo: Arc<dyn TimeEntryRepository>,
    workspace_repo: Arc<dyn crate::repositories::workspace_repository::WorkspaceRepository>,
    dept_repo: Arc<dyn crate::repositories::department_repository::DepartmentRepository>,
    staging_repo: Arc<dyn crate::repositories::staging_log_repository::StagingLogRepository>,
    deps: crate::worker::deps::WorkerDeps,
    log_svc: Option<Arc<crate::services::attendance_log_service::AttendanceLogService>>,
}

impl AttendanceImportService {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        import_repo: Arc<dyn ImportJobRepository>,
        adj_repo: Arc<dyn AttendanceAdjustmentRepository>,
        employee_repo: Arc<dyn EmployeeRepository>,
        time_entry_repo: Arc<dyn TimeEntryRepository>,
        workspace_repo: Arc<dyn crate::repositories::workspace_repository::WorkspaceRepository>,
        dept_repo: Arc<dyn crate::repositories::department_repository::DepartmentRepository>,
        staging_repo: Arc<dyn crate::repositories::staging_log_repository::StagingLogRepository>,
        deps: crate::worker::deps::WorkerDeps,
        log_svc: Option<Arc<crate::services::attendance_log_service::AttendanceLogService>>,
    ) -> Self {
        Self {
            import_repo,
            adj_repo,
            employee_repo,
            _time_entry_repo: time_entry_repo,
            workspace_repo,
            dept_repo,
            staging_repo,
            deps,
            log_svc,
        }
    }
    #[tracing::instrument(skip_all, err)]
    pub async fn batch_import_from_file(
        &self,
        workspace_id: Uuid,
        user_id: Uuid,
        file_path: String,
        target_type: ImportTargetType,
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

        self.import_repo
            .create_job(
                &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
                &job,
            )
            .await
            .map_err(|e| {
                error!("CREATE_JOB_ERROR: {:?}", e);
                AppError::Internal("Internal error".into())
            })?;

        let svc = self.clone();
        tokio::spawn(async move {
            let _ = svc.process_job(workspace_id, job_id).await;
        });

        Ok(job_id)
    }

    /// Check if a job target type should be resumed by the API server startup routine.
    pub fn should_resume_job(target_type: &ImportTargetType) -> bool {
        !matches!(
            target_type,
            ImportTargetType::Employee | ImportTargetType::LeaveImport
        )
    }

    #[tracing::instrument(skip_all)]
    pub async fn resume_all_pending_jobs(&self) {
        tracing::info!("resume_all_pending_jobs: starting...");
        // Get all workspaces from system DB, then query each tenant for pending jobs
        let workspaces = match self.workspace_repo.get_all_workspaces().await {
            Ok(ws) => ws,
            Err(e) => {
                error!("Failed to list workspaces for resume: {:?}", e);
                return;
            }
        };

        for workspace in workspaces {
            let Ok(jobs) = self
                .import_repo
                .get_pending_jobs(&crate::diesel_pool::TenantAuthToken::system_override(
                    workspace.id(),
                ))
                .await
            else {
                continue;
            };

            let mut failed_jobs = Vec::new();

            for job in jobs {
                if !Self::should_resume_job(&job.target_type) {
                    continue;
                }

                let Some(file_path) = &job.file_path else {
                    failed_jobs.push((
                        job.id,
                        job.processed_rows,
                        ImportJobStatus::Failed,
                        Some("Missing file path".to_string()),
                    ));
                    continue;
                };

                if !tokio::fs::try_exists(file_path).await.unwrap_or(false) {
                    failed_jobs.push((
                        job.id,
                        job.processed_rows,
                        ImportJobStatus::Failed,
                        Some("Could not read file: Not found or inaccessible".to_string()),
                    ));
                    continue;
                }

                info!(
                    "Resuming import job: {} (target: {})",
                    job.id,
                    job.target_type.to_string()
                );
                let svc = self.clone();
                let ws_id = workspace.id();
                tokio::spawn(async move {
                    let _ = svc.process_job(ws_id, job.id).await;
                });
            }

            if !failed_jobs.is_empty() {
                let _ = self
                    .import_repo
                    .batch_update_job_progress(
                        &crate::diesel_pool::TenantAuthToken::system_override(workspace.id()),
                        failed_jobs,
                    )
                    .await;
            }
        }
    }

    #[tracing::instrument(skip(self))]
    pub async fn process_job(&self, workspace_id: Uuid, job_id: Uuid) -> Result<(), ()> {
        let Ok(Some(job)) = self
            .import_repo
            .get_job_by_id(
                &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
                job_id,
            )
            .await
        else {
            return Err(());
        };

        let Ok(Some(workspace)) = self
            .workspace_repo
            .get_workspace_by_id(job.workspace_id)
            .await
        else {
            error!("WORKSPACE_NOT_FOUND for job {}", job_id);
            let _ = self
                .import_repo
                .update_job_progress(
                    &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
                    job_id,
                    job.processed_rows,
                    ImportJobStatus::Failed,
                    Some("Workspace not found".to_string()),
                )
                .await;
            return Err(());
        };

        let date_format = workspace.date_format().to_string();
        let time_format = workspace.time_format().to_string();
        let timezone = workspace.timezone().to_string();
        let lang = workspace.default_language();
        let msgs = AttendanceImportMessages::for_lang(lang);

        if job.target_type == ImportTargetType::Employee {
            return self.process_employee_job(job_id, &job, &msgs).await;
        }

        let Some(file_path) = &job.file_path else {
            return Err(());
        };

        let file_bytes = match tokio::fs::read(file_path).await {
            Ok(b) => b,
            Err(e) => {
                error!("READ_FILE_ERROR for job {}: {:?}", job_id, e);
                let _ = self
                    .import_repo
                    .update_job_progress(
                        &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
                        job_id,
                        job.processed_rows,
                        ImportJobStatus::Failed,
                        Some(format!("Could not read file: {e:?}")),
                    )
                    .await;
                return Err(());
            }
        };

        let workspace_tz = timezone.parse::<Tz>().unwrap_or(Tz::UTC);

        let strategy = PunchImportStrategy {
            msgs: msgs.clone(),
            date_format,
            time_format,
            adj_repo: Arc::clone(&self.adj_repo),
            employee_repo: Arc::clone(&self.employee_repo),
            log_svc: self.log_svc.clone(),
            emp_cache: std::collections::HashMap::new(),
        };

        let pipeline = ImportPipeline {
            strategy,
            deps: self.deps.clone(),
            import_repo: &*self.import_repo,
            job: &job,
            file_bytes,
            workspace_tz,
            start_index: None,
            end_index: None,
            is_chunk: false,
            queue_repo: None,
        };

        pipeline.run().await
    }

    async fn process_employee_job(
        &self,
        job_id: Uuid,
        job: &AttendanceImportJob,
        msgs: &AttendanceImportMessages,
    ) -> Result<(), ()> {
        let workspace_tz = match self
            .workspace_repo
            .get_workspace_by_id(job.workspace_id)
            .await
        {
            Ok(Some(ws)) => ws.timezone().parse::<Tz>().unwrap_or(Tz::UTC),
            _ => Tz::UTC,
        };

        let Some(file_path) = &job.file_path else {
            return Err(());
        };

        let file_bytes = match tokio::fs::read(file_path).await {
            Ok(b) => b,
            Err(e) => {
                error!("READ_FILE_ERROR for job {}: {:?}", job_id, e);
                let _ = self
                    .import_repo
                    .update_job_progress(
                        &crate::diesel_pool::TenantAuthToken::system_override(job.workspace_id),
                        job_id,
                        0,
                        ImportJobStatus::Failed,
                        Some(format!("Could not read file: {e:?}")),
                    )
                    .await;
                return Err(());
            }
        };

        let strategy = EmployeeImportStrategy {
            msgs: msgs.clone(),
            employee_repo: Arc::clone(&self.employee_repo),
            department_repo: Arc::clone(&self.dept_repo),
            staging_repo: Arc::clone(&self.staging_repo),
            dept_cache: HashMap::new(),
            emp_cache: HashMap::new(),
            newly_added_emps: std::collections::HashSet::new(),
        };

        let pipeline = ImportPipeline {
            strategy,
            deps: self.deps.clone(),
            import_repo: &*self.import_repo,
            job,
            file_bytes,
            workspace_tz,
            start_index: None,
            end_index: None,
            is_chunk: false,
            queue_repo: None,
        };

        pipeline.run().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_resume_job() {
        // Jobs that should be resumed by the API server
        assert!(AttendanceImportService::should_resume_job(
            &ImportTargetType::Log
        ));
        assert!(AttendanceImportService::should_resume_job(
            &ImportTargetType::Adjustment
        ));

        // Jobs handled by Worker Queue (should NOT be resumed by API server)
        assert!(!AttendanceImportService::should_resume_job(
            &ImportTargetType::Employee
        ));
        assert!(!AttendanceImportService::should_resume_job(
            &ImportTargetType::LeaveImport
        ));
    }
}
