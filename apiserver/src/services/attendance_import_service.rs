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
use crate::repositories::import_job_repository::ImportJobRepository;
use crate::repositories::{EmployeeRepository, TimeEntryRepository};
use crate::services::attendance_adjustment_service::AttendanceAdjustmentService;
use crate::services::import_messages::{fmt_msg, AttendanceImportMessages};
use std::collections::HashMap;

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
    pub dept_cache: HashMap<String, Uuid>,
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
    ) -> Result<Self::RowData, String> {
        let raw_id = row_data
            .get("employee_code")
            .map(std::string::String::as_str)
            .unwrap_or("")
            .trim();
        let id_number = crate::utils::normalize_employee_id(raw_id);
        let full_name = row_data
            .get("full_name")
            .map(std::string::String::as_str)
            .unwrap_or("")
            .trim()
            .to_string();
        let hire_date_raw = row_data
            .get("hire_date")
            .map(std::string::String::as_str)
            .unwrap_or("")
            .trim();

        if id_number.is_empty() || full_name.is_empty() {
            return Err(fmt_msg(
                self.msgs.missing_employee_or_id,
                &[("line", &current_line.to_string())],
            ));
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
        _diesel: &crate::diesel_pool::DieselPools,
        job: &AttendanceImportJob,
    ) -> Result<(), String> {
        if let Ok(existing_depts) = self.department_repo.get_departments(job.workspace_id).await {
            for dept in existing_depts {
                self.dept_cache.insert(dept.name.to_lowercase(), dept.id);
            }
        }
        Ok(())
    }

    async fn process_row(
        &mut self,
        diesel: &crate::diesel_pool::DieselPools,
        job: &AttendanceImportJob,
        emp: &Self::RowData,
    ) -> Result<Option<Uuid>, String> {
        match AttendanceImportService::process_employee_import_row(&self.employee_repo, job, emp)
            .await
        {
            Ok(maybe_id) => {
                let mut was_new = true;
                let employee_id = if let Some(id) = maybe_id {
                    id
                } else {
                    was_new = false;
                    // If exists, find the ID
                    match self
                        .employee_repo
                        .get_employee_by_id_number(&emp.id_number, job.workspace_id)
                        .await
                    {
                        Ok(Some(e)) => e.id,
                        _ => return Ok(None),
                    }
                };

                // Auto-assign to department
                let mut department_updated = false;
                if let Some(ref dept_name) = emp.department_name {
                    let dept_key = dept_name.to_lowercase();
                    let dept_id = if let Some(&cached_id) = self.dept_cache.get(&dept_key) {
                        cached_id
                    } else {
                        let existing = self
                            .department_repo
                            .get_department_by_name(job.workspace_id, &dept_key)
                            .await;

                        if let Ok(Some(id)) = existing {
                            self.dept_cache.insert(dept_key.clone(), id);
                            id
                        } else {
                            match self
                                .department_repo
                                .create_department(
                                    job.workspace_id,
                                    dept_name.clone(),
                                    None,
                                    None,
                                    None,
                                )
                                .await
                            {
                                Ok(new_dept) => {
                                    self.dept_cache.insert(dept_key.clone(), new_dept.id);
                                    new_dept.id
                                }
                                Err(e) => {
                                    // Final check in case of a unique constraint race condition
                                    let retry = self
                                        .department_repo
                                        .get_department_by_name(job.workspace_id, &dept_key)
                                        .await;

                                    if let Ok(Some(id)) = retry {
                                        self.dept_cache.insert(dept_key.clone(), id);
                                        id
                                    } else {
                                        error!(
                                            "Failed to create department '{}': {}",
                                            dept_name, e
                                        );
                                        Uuid::nil()
                                    }
                                }
                            }
                        }
                    };
                    if !dept_id.is_nil()
                        && self
                            .department_repo
                            .transfer_employee(
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
                let _ = crate::repositories::staging_log_repository::claim_staged_logs_direct(
                    diesel,
                    job.workspace_id,
                    employee_id,
                    &emp.id_number,
                )
                .await;

                if was_new || department_updated {
                    Ok(Some(employee_id))
                } else {
                    Ok(None)
                }
            }
            Err(e) => Err(fmt_msg(
                self.msgs.db_save_error,
                &[
                    ("line", &emp.line_number.to_string()),
                    ("emp", &emp.id_number),
                    ("error", &e),
                ],
            )),
        }
    }
}

struct PunchImportStrategy {
    msgs: AttendanceImportMessages,
    date_format: String,
    time_format: String,
    adj_repo: Arc<dyn AttendanceAdjustmentRepository>,
    time_entry_repo: Arc<dyn TimeEntryRepository>,
    employee_repo: Arc<dyn EmployeeRepository>,
    log_svc: Option<Arc<crate::services::attendance_log_service::AttendanceLogService>>,
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
    ) -> Result<Self::RowData, String> {
        let raw_id = row_data
            .get("employee_code")
            .map(std::string::String::as_str)
            .unwrap_or("")
            .trim();
        let emp_id_number = crate::utils::normalize_employee_id(raw_id);
        let emp_dt_str = row_data
            .get("punch_time")
            .map(std::string::String::as_str)
            .unwrap_or("")
            .trim();

        if emp_id_number.is_empty() || emp_dt_str.is_empty() {
            return Err(fmt_msg(
                self.msgs.missing_employee_or_time,
                &[("line", &current_line.to_string())],
            ));
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
            Err(_) => Err(format!(
                "{} [Raw: '{emp_dt_str}']",
                fmt_msg(
                    self.msgs.missing_employee_or_time,
                    &[("line", &current_line.to_string())],
                )
            )),
        }
    }

    async fn init(
        &mut self,
        _diesel: &crate::diesel_pool::DieselPools,
        _job: &AttendanceImportJob,
    ) -> Result<(), String> {
        Ok(())
    }

    async fn process_row(
        &mut self,
        _diesel: &crate::diesel_pool::DieselPools,
        job: &AttendanceImportJob,
        row: &Self::RowData,
    ) -> Result<Option<Uuid>, String> {
        let Ok(Some(employee)) = self
            .employee_repo
            .get_employee_by_id_number(&row.emp_id_number, job.workspace_id)
            .await
        else {
            return Err(fmt_msg(
                self.msgs.system_save_error,
                &[
                    ("emp", &row.emp_id_number),
                    ("line", &row.line_number.to_string()),
                ],
            ));
        };

        if job.target_type == ImportTargetType::Log {
            let Some(log_svc) = self.log_svc.as_ref() else {
                return Err(fmt_msg(
                    self.msgs.system_save_error,
                    &[
                        ("emp", &row.emp_id_number),
                        ("line", &row.line_number.to_string()),
                    ],
                ));
            };
            match log_svc
                .process_parsed_log(
                    job.workspace_id,
                    employee.id,
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
                    Err(fmt_msg(
                        self.msgs.system_save_error,
                        &[
                            ("emp", &row.emp_id_number),
                            ("line", &row.line_number.to_string()),
                        ],
                    ))
                }
            }
        } else {
            let req = CreateAttendanceAdjustmentRequest {
                employee_id: employee.id,
                time_entry_id: None,
                requested_punch_in_time: row.punch_time,
                requested_punch_out_time: None,
                is_break_overtime: false,
                reason: Some("Import from Excel/CSV".to_string()),
            };

            let adj_svc = AttendanceAdjustmentService::new(
                Arc::clone(&self.adj_repo),
                Arc::clone(&self.time_entry_repo),
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
                            adjustment.id,
                            AdjustmentStatus::Approved,
                        )
                        .await;
                    Ok(Some(adjustment.id))
                }
                Err(e) => {
                    error!("ADJUSTMENT_SUBMIT_ERROR: {}", e);
                    Err(fmt_msg(
                        self.msgs.system_save_error,
                        &[
                            ("emp", &row.emp_id_number),
                            ("line", &row.line_number.to_string()),
                        ],
                    ))
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
    time_entry_repo: Arc<dyn TimeEntryRepository>,
    workspace_repo: Arc<dyn crate::repositories::workspace_repository::WorkspaceRepository>,
    dept_repo: Arc<dyn crate::repositories::department_repository::DepartmentRepository>,
    diesel_pools: crate::diesel_pool::DieselPools,
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
        diesel_pools: crate::diesel_pool::DieselPools,
        log_svc: Option<Arc<crate::services::attendance_log_service::AttendanceLogService>>,
    ) -> Self {
        Self {
            import_repo,
            adj_repo,
            employee_repo,
            time_entry_repo,
            workspace_repo,
            dept_repo,
            diesel_pools,
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

    /// Check if a job target type should be resumed by the API server startup routine.
    pub fn should_resume_job(target_type: &ImportTargetType) -> bool {
        !matches!(
            target_type,
            ImportTargetType::Employee | ImportTargetType::LeaveImport
        )
    }

    #[tracing::instrument(skip_all)]
    pub async fn resume_all_pending_jobs(&self) {
        if let Ok(jobs) = self.import_repo.get_pending_jobs().await {
            for job in jobs {
                if !Self::should_resume_job(&job.target_type) {
                    // Handled by JobQueue now.
                    continue;
                }

                info!(
                    "Resuming import job: {} (target: {})",
                    job.id,
                    job.target_type.to_string()
                );
                let svc = self.clone();
                tokio::spawn(async move {
                    let _ = svc.process_job(job.id).await;
                });
            }
        }
    }

    #[tracing::instrument(skip_all)]
    pub async fn process_job(&self, job_id: Uuid) -> Result<(), ()> {
        let Ok(Some(job)) = self.import_repo.get_job_by_id(job_id).await else {
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
                    job_id,
                    job.processed_rows,
                    ImportJobStatus::Failed,
                    Some("Workspace not found".to_string()),
                )
                .await;
            return Err(());
        };

        let date_format = workspace.date_format.clone();
        let time_format = workspace.time_format.clone();
        let timezone = workspace.timezone.clone();
        let lang = &workspace.default_language;
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
            time_entry_repo: Arc::clone(&self.time_entry_repo),
            employee_repo: Arc::clone(&self.employee_repo),
            log_svc: self.log_svc.clone(),
        };

        let pipeline = ImportPipeline {
            strategy,
            diesel: self.diesel_pools.clone(),
            import_repo: &*self.import_repo,
            job: &job,
            file_bytes,
            workspace_tz,
            start_index: None,
            end_index: None,
            is_chunk: false,
            redis: None,
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
            Ok(Some(ws)) => ws.timezone.parse::<Tz>().unwrap_or(Tz::UTC),
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
            employee_repo: std::sync::Arc::clone(&self.employee_repo),
            department_repo: Arc::clone(&self.dept_repo),
            dept_cache: HashMap::new(),
        };

        let pipeline = ImportPipeline {
            strategy,
            diesel: self.diesel_pools.clone(),
            import_repo: &*self.import_repo,
            job,
            file_bytes,
            workspace_tz,
            start_index: None,
            end_index: None,
            is_chunk: false,
            redis: None,
        };

        pipeline.run().await
    }

    async fn process_employee_import_row(
        employee_repo: &Arc<dyn EmployeeRepository>,
        job: &AttendanceImportJob,
        emp: &PendingEmployee,
    ) -> Result<Option<Uuid>, String> {
        let employee_id = Uuid::new_v4();
        let employee = Employee {
            id: employee_id,
            workspace_id: job.workspace_id,
            user_id: None,
            full_name: emp.full_name.clone(),
            employee_id_number: emp.id_number.clone(),
            original_hire_date: emp.hire_date,
            hire_date: emp.hire_date,
            termination_date: None,
            tags: Vec::new(),
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            department_name: None,
        };

        let inserted = employee_repo
            .create_employee_skip_duplicates(&employee)
            .await
            .map_err(|e| e.to_string())?;

        if inserted {
            Ok(Some(employee_id))
        } else {
            Ok(None)
        }
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
