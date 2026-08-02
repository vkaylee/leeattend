use std::sync::Arc;

use crate::errors::AppError;
use crate::utils::i18n::I18nKey;

use crate::utils::datetime::parse_date;
use chrono::{DateTime, Utc};
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

/// Dependency injection struct for AttendanceImportService.
/// Follows the pattern established by AttendanceLogServiceDeps.
pub struct ImportServiceDeps {
    pub import_repo: Arc<dyn ImportJobRepository>,
    pub adj_repo: Arc<dyn AttendanceAdjustmentRepository>,
    pub employee_repo: Arc<dyn EmployeeRepository>,
    pub time_entry_repo: Arc<dyn TimeEntryRepository>,
    pub workspace_repo: Arc<dyn crate::repositories::workspace_repository::WorkspaceRepository>,
    pub dept_repo: Arc<dyn crate::repositories::department_repository::DepartmentRepository>,
    pub staging_repo: Arc<dyn crate::repositories::staging_log_repository::StagingLogRepository>,
}

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

impl EmployeeImportStrategy {
    /// Collect employee IDs and department names from rows for bulk lookup.
    fn collect_row_identifiers(rows: &[PendingEmployee]) -> (Vec<String>, Vec<String>) {
        let mut emp_ids = Vec::with_capacity(rows.len());
        let mut dept_names = Vec::new();
        for row in rows {
            emp_ids.push(row.id_number.clone());
            if let Some(dept) = &row.department_name {
                dept_names.push(dept.clone());
            }
        }
        (emp_ids, dept_names)
    }

    /// Load existing departments and employees into caches.
    async fn load_existing_caches(
        &mut self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        emp_ids: &[String],
        dept_names: &[String],
    ) {
        if let Ok(existing_depts) = self
            .department_repo
            .get_department_ids_by_names(auth_token, dept_names)
            .await
        {
            for (name, id) in existing_depts {
                self.dept_cache.insert(name, id);
            }
        }

        if let Ok(existing_emps) = self
            .employee_repo
            .get_employee_ids_by_numbers(auth_token, emp_ids)
            .await
        {
            for (num, id) in existing_emps {
                self.emp_cache.insert(num.to_lowercase(), id);
            }
        }
    }

    /// Bulk create departments that don't exist and update cache.
    async fn bulk_create_departments(
        &mut self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        workspace_id: Uuid,
        dept_names: Vec<String>,
    ) {
        let mut new_depts = Vec::new();
        let mut unique_dept_names = std::collections::HashSet::new();
        for name in dept_names {
            let key = name.to_lowercase();
            if !self.dept_cache.contains_key(&key) && unique_dept_names.insert(key.clone()) {
                let dept_id = Uuid::new_v4();
                new_depts.push(crate::models::Department::new(
                    dept_id,
                    workspace_id,
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
                .bulk_create_departments_skip_duplicates(auth_token, &new_depts)
                .await;
            let newly_inserted_names: Vec<String> = new_depts
                .into_iter()
                .map(|d| d.name().to_string())
                .collect();
            if let Ok(existing_depts) = self
                .department_repo
                .get_department_ids_by_names(auth_token, &newly_inserted_names)
                .await
            {
                for (name, id) in existing_depts {
                    self.dept_cache.insert(name, id);
                }
            }
        }
    }

    /// Bulk create employees that don't exist and update cache.
    async fn bulk_create_employees(
        &mut self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        workspace_id: Uuid,
        rows: &[PendingEmployee],
    ) -> Result<(), AppError> {
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
                    workspace_id,
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
                .bulk_create_employees_skip_duplicates(auth_token, &new_emps)
                .await?;
            let newly_inserted_ids: Vec<String> = new_emps
                .into_iter()
                .map(|e| e.employee_id_number().to_string())
                .collect();
            if let Ok(existing_emps) = self
                .employee_repo
                .get_employee_ids_by_numbers(auth_token, &newly_inserted_ids)
                .await
            {
                for (num, id) in existing_emps {
                    self.emp_cache.insert(num.to_lowercase(), id);
                }
            }
        }
        Ok(())
    }

    /// Attempt to transfer employee to department if specified.
    async fn try_transfer_department(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
        emp: &PendingEmployee,
    ) -> bool {
        let Some(ref dept_name) = emp.department_name else {
            return false;
        };
        let dept_key = dept_name.to_lowercase();
        let Some(&dept_id) = self.dept_cache.get(&dept_key) else {
            return false;
        };
        if dept_id.is_nil() {
            return false;
        }
        self.department_repo
            .transfer_employee(
                auth_token,
                employee_id,
                dept_id,
                emp.hire_date,
                Some("Auto-assigned via bulk import".to_string()),
            )
            .await
            .is_ok()
    }
}

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
            let msg = fmt_msg(
                self.msgs.missing_employee_or_id,
                &[("line", &current_line.to_string())],
            );
            return Err(AppError::validation_args(
                I18nKey::Validation,
                [("details", msg)],
            ));
        }

        let hire_date_str = hire_date_raw.split(' ').next().unwrap_or(hire_date_raw);
        let hire_date = if hire_date_str.is_empty() {
            Utc::now().with_timezone(&workspace_tz).date_naive()
        } else {
            parse_date(hire_date_str)
                .unwrap_or_else(|| Utc::now().with_timezone(&workspace_tz).date_naive())
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
        let (emp_ids, dept_names) = Self::collect_row_identifiers(rows);

        // Load existing departments and employees into caches
        self.load_existing_caches(&auth_token, &emp_ids, &dept_names)
            .await;

        // Bulk create any missing departments
        self.bulk_create_departments(&auth_token, job.workspace_id, dept_names)
            .await;

        // Verify workspace headquarters location exists
        let Ok(Some(_)) =
            EmployeeCoreRepository::get_headquarters_location_id(&*self.employee_repo, &auth_token)
                .await
        else {
            return Err(AppError::Internal(
                "Workspace headquarters location not found".into(),
            ));
        };

        // Bulk create any missing employees
        self.bulk_create_employees(&auth_token, job.workspace_id, rows)
            .await?;

        Ok(())
    }

    async fn process_row(
        &mut self,
        _deps: &crate::worker::deps::WorkerDeps,
        job: &AttendanceImportJob,
        emp: &Self::RowData,
    ) -> Result<Option<Uuid>, AppError> {
        let emp_key = emp.id_number.to_lowercase();
        let auth_token = crate::diesel_pool::TenantAuthToken::system_override(job.workspace_id);

        // pre_process_chunk has already populated emp_cache and inserted new employees to DB.
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

        let was_new = self.newly_added_emps.contains(&emp_key);
        let department_updated = self
            .try_transfer_department(&auth_token, employee_id, emp)
            .await;

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

impl PunchImportStrategy {
    /// Look up employee from cache or fetch from database.
    /// Returns the employee or an error if not found.
    async fn validate_employee_mapping(
        &mut self,
        workspace_id: Uuid,
        row: &PendingRow,
    ) -> Result<Employee, AppError> {
        if let Some(e) = self.emp_cache.get(&row.emp_id_number) {
            return Ok(e.clone());
        }

        let Ok(Some(emp)) = self
            .employee_repo
            .get_employee_by_id_number(
                &row.emp_id_number,
                &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
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
        Ok(emp)
    }

    /// Process a punch as an attendance log entry.
    async fn process_log_punch(
        &self,
        workspace_id: Uuid,
        employee: &Employee,
        row: &PendingRow,
    ) -> Result<Option<Uuid>, AppError> {
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
                workspace_id,
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
    }

    /// Process a punch as an attendance adjustment request.
    async fn process_adjustment_punch(
        &self,
        deps: &crate::worker::deps::WorkerDeps,
        job: &AttendanceImportJob,
        employee: &Employee,
        row: &PendingRow,
    ) -> Result<Option<Uuid>, AppError> {
        let Some(employment_id) = employee.active_employment_id() else {
            return Err(AppError::not_found(I18nKey::NoActiveEmployment));
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
            let msg = fmt_msg(
                self.msgs.missing_employee_or_time,
                &[("line", &current_line.to_string())],
            );
            return Err(AppError::validation_args(
                I18nKey::Validation,
                [("details", msg)],
            ));
        }

        if let Ok(dt) = AttendanceAdjustmentService::parse_datetime(
            emp_dt_str,
            workspace_tz.name(),
            Some(&self.date_format),
            Some(&self.time_format),
        ) {
            Ok(PendingRow {
                emp_id_number,
                punch_time: dt,
                line_number: current_line,
            })
        } else {
            let msg = format!(
                "{} [Raw: '{emp_dt_str}']",
                fmt_msg(
                    self.msgs.missing_employee_or_time,
                    &[("line", &current_line.to_string())],
                )
            );
            Err(AppError::validation_args(
                I18nKey::Validation,
                [("details", msg)],
            ))
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

    async fn process_row(
        &mut self,
        deps: &crate::worker::deps::WorkerDeps,
        job: &AttendanceImportJob,
        row: &Self::RowData,
    ) -> Result<Option<Uuid>, AppError> {
        let employee = self
            .validate_employee_mapping(job.workspace_id, row)
            .await?;

        if job.target_type == ImportTargetType::Log {
            self.process_log_punch(job.workspace_id, &employee, row)
                .await
        } else {
            self.process_adjustment_punch(deps, job, &employee, row)
                .await
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
    pub fn new(
        repo_deps: ImportServiceDeps,
        deps: crate::worker::deps::WorkerDeps,
        log_svc: Option<Arc<crate::services::attendance_log_service::AttendanceLogService>>,
    ) -> Self {
        Self {
            import_repo: repo_deps.import_repo,
            adj_repo: repo_deps.adj_repo,
            employee_repo: repo_deps.employee_repo,
            _time_entry_repo: repo_deps.time_entry_repo,
            workspace_repo: repo_deps.workspace_repo,
            dept_repo: repo_deps.dept_repo,
            staging_repo: repo_deps.staging_repo,
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

        use futures_util::stream::StreamExt;

        futures_util::stream::iter(workspaces)
            .map(|workspace| async move {
                let ws_id = workspace.id();
                let Ok(jobs) = self
                    .import_repo
                    .get_pending_jobs(&crate::diesel_pool::TenantAuthToken::system_override(ws_id))
                    .await
                else {
                    return;
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
                    tokio::spawn(async move {
                        let _ = svc.process_job(ws_id, job.id).await;
                    });
                }

                if !failed_jobs.is_empty() {
                    let _ = self
                        .import_repo
                        .batch_update_job_progress(
                            &crate::diesel_pool::TenantAuthToken::system_override(ws_id),
                            failed_jobs,
                        )
                        .await;
                }
            })
            .buffer_unordered(5)
            .collect::<Vec<()>>()
            .await;
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

        let workspace_tz = crate::utils::tz::parse_workspace_tz(&timezone);

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
            Ok(Some(ws)) => crate::utils::tz::parse_workspace_tz(ws.timezone()),
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
    use crate::repositories::attendance_adjustment_repository::MockAttendanceAdjustmentRepository;
    use crate::repositories::department_repository::MockDepartmentRepository;
    use crate::repositories::employee::MockEmployeeRepository;
    use crate::repositories::import_job_repository::MockImportJobRepository;
    use crate::repositories::staging_log_repository::MockStagingLogRepository;
    use crate::repositories::workspace_repository::MockWorkspaceRepository;
    use crate::repositories::time_entry_repository::MockTimeEntryRepository;
    use crate::worker::test_utils::MockWorkerDepsBuilder;

    fn make_job(target_type: ImportTargetType) -> AttendanceImportJob {
        AttendanceImportJob {
            id: Uuid::new_v4(),
            workspace_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            status: ImportJobStatus::Processing,
            total_rows: 0,
            processed_rows: 0,
            file_path: None,
            error_log: None,
            target_type,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn build_service(
        import_repo: MockImportJobRepository,
        workspace_repo: MockWorkspaceRepository,
    ) -> AttendanceImportService {
        let repo_deps = ImportServiceDeps {
            import_repo: Arc::new(import_repo),
            adj_repo: Arc::new(MockAttendanceAdjustmentRepository::new()),
            employee_repo: Arc::new(MockEmployeeRepository::new()),
            time_entry_repo: Arc::new(MockTimeEntryRepository::new()),
            workspace_repo: Arc::new(workspace_repo),
            dept_repo: Arc::new(MockDepartmentRepository::new()),
            staging_repo: Arc::new(MockStagingLogRepository::new()),
        };
        let worker_deps = MockWorkerDepsBuilder::new().build();
        AttendanceImportService::new(repo_deps, worker_deps, None)
    }

    fn mock_workspace(id: Uuid) -> crate::models::Workspace {
        crate::models::Workspace::new(
            id,
            "Test Workspace".into(),
            "YYYY-MM-DD".into(),
            "HH:mm:ss".into(),
            "UTC".into(),
            "#000000".into(),
            "vi".into(),
            None,
            None,
            "split".into(),
            rust_decimal::Decimal::new(15, 1),
            Utc::now(),
            None,
        )
    }

    #[test]
    fn test_should_resume_job() {
        assert!(AttendanceImportService::should_resume_job(
            &ImportTargetType::Log
        ));
        assert!(AttendanceImportService::should_resume_job(
            &ImportTargetType::Adjustment
        ));
        assert!(!AttendanceImportService::should_resume_job(
            &ImportTargetType::Employee
        ));
        assert!(!AttendanceImportService::should_resume_job(
            &ImportTargetType::LeaveImport
        ));
    }

    #[tokio::test]
    async fn test_process_job_not_found_returns_err() {
        let mut mock_import_repo = MockImportJobRepository::new();
        mock_import_repo
            .expect_get_job_by_id()
            .returning(|_, _| Ok(None));

        let mock_workspace_repo = MockWorkspaceRepository::new();
        let svc = build_service(mock_import_repo, mock_workspace_repo);

        let result = svc.process_job(Uuid::new_v4(), Uuid::new_v4()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_process_job_workspace_not_found_marks_failed() {
        let job = make_job(ImportTargetType::Log);
        let ws_id = job.workspace_id;
        let job_id = job.id;

        let mut mock_import_repo = MockImportJobRepository::new();
        let job_clone = job.clone();
        mock_import_repo
            .expect_get_job_by_id()
            .returning(move |_, _| Ok(Some(job_clone.clone())));
        mock_import_repo
            .expect_update_job_progress()
            .with(
                mockall::predicate::always(),
                mockall::predicate::eq(job_id),
                mockall::predicate::always(),
                mockall::predicate::eq(ImportJobStatus::Failed),
                mockall::predicate::always(),
            )
            .returning(|_, _, _, _, _| Ok(()));

        let mut mock_workspace_repo = MockWorkspaceRepository::new();
        mock_workspace_repo
            .expect_get_workspace_by_id()
            .returning(|_| Box::pin(async move { Ok(None) }));

        let svc = build_service(mock_import_repo, mock_workspace_repo);
        let result = svc.process_job(ws_id, job_id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_process_job_missing_file_path_returns_err() {
        let mut job = make_job(ImportTargetType::Log);
        job.file_path = None;
        let ws_id = job.workspace_id;
        let job_id = job.id;

        let mut mock_import_repo = MockImportJobRepository::new();
        let job_clone = job.clone();
        mock_import_repo
            .expect_get_job_by_id()
            .returning(move |_, _| Ok(Some(job_clone.clone())));

        let mut mock_workspace_repo = MockWorkspaceRepository::new();
        let ws_id_clone = ws_id;
        mock_workspace_repo
            .expect_get_workspace_by_id()
            .returning(move |_| {
                let ws = mock_workspace(ws_id_clone);
                Box::pin(async move { Ok(Some(ws)) })
            });

        let svc = build_service(mock_import_repo, mock_workspace_repo);
        let result = svc.process_job(ws_id, job_id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_process_job_file_read_error_marks_failed() {
        let mut job = make_job(ImportTargetType::Adjustment);
        job.file_path = Some("/nonexistent/path/file.xlsx".to_string());
        let ws_id = job.workspace_id;
        let job_id = job.id;

        let mut mock_import_repo = MockImportJobRepository::new();
        let job_clone = job.clone();
        mock_import_repo
            .expect_get_job_by_id()
            .returning(move |_, _| Ok(Some(job_clone.clone())));
        mock_import_repo
            .expect_update_job_progress()
            .with(
                mockall::predicate::always(),
                mockall::predicate::eq(job_id),
                mockall::predicate::always(),
                mockall::predicate::eq(ImportJobStatus::Failed),
                mockall::predicate::always(),
            )
            .returning(|_, _, _, _, _| Ok(()));

        let mut mock_workspace_repo = MockWorkspaceRepository::new();
        let ws_id_clone = ws_id;
        mock_workspace_repo
            .expect_get_workspace_by_id()
            .returning(move |_| {
                let ws = mock_workspace(ws_id_clone);
                Box::pin(async move { Ok(Some(ws)) })
            });

        let svc = build_service(mock_import_repo, mock_workspace_repo);
        let result = svc.process_job(ws_id, job_id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_process_job_employee_target_missing_file_path() {
        let mut job = make_job(ImportTargetType::Employee);
        job.file_path = None;
        let ws_id = job.workspace_id;
        let job_id = job.id;

        let mut mock_import_repo = MockImportJobRepository::new();
        let job_clone = job.clone();
        mock_import_repo
            .expect_get_job_by_id()
            .returning(move |_, _| Ok(Some(job_clone.clone())));

        let mut mock_workspace_repo = MockWorkspaceRepository::new();
        let ws_id_clone = ws_id;
        mock_workspace_repo
            .expect_get_workspace_by_id()
            .returning(move |_| {
                let ws = mock_workspace(ws_id_clone);
                Box::pin(async move { Ok(Some(ws)) })
            });

        let svc = build_service(mock_import_repo, mock_workspace_repo);
        let result = svc.process_job(ws_id, job_id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_process_job_employee_file_read_error_marks_failed() {
        let mut job = make_job(ImportTargetType::Employee);
        job.file_path = Some("/nonexistent/employee_file.xlsx".to_string());
        let ws_id = job.workspace_id;
        let job_id = job.id;

        let mut mock_import_repo = MockImportJobRepository::new();
        let job_clone = job.clone();
        mock_import_repo
            .expect_get_job_by_id()
            .returning(move |_, _| Ok(Some(job_clone.clone())));
        mock_import_repo
            .expect_update_job_progress()
            .with(
                mockall::predicate::always(),
                mockall::predicate::eq(job_id),
                mockall::predicate::eq(0),
                mockall::predicate::eq(ImportJobStatus::Failed),
                mockall::predicate::always(),
            )
            .returning(|_, _, _, _, _| Ok(()));

        let mut mock_workspace_repo = MockWorkspaceRepository::new();
        let ws_id_clone = ws_id;
        mock_workspace_repo
            .expect_get_workspace_by_id()
            .returning(move |_| {
                let ws = mock_workspace(ws_id_clone);
                Box::pin(async move { Ok(Some(ws)) })
            });

        let svc = build_service(mock_import_repo, mock_workspace_repo);
        let result = svc.process_job(ws_id, job_id).await;
        assert!(result.is_err());
    }
}
