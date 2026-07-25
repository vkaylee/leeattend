//! Attendance Report Handler
//!
//! Processes attendance jobs for a batch of employees.

use std::collections::HashMap;
use std::sync::Arc;

use chrono::{Duration, NaiveDate, TimeZone, Utc};
use chrono_tz::Tz;
use tracing::info;
use uuid::Uuid;

use crate::cache::{CacheManager, CachedWorkspaceConfig, RosterCache};
use crate::domain::policy::resolver::PolicyResolver;
use crate::models::{AttendanceSummary, DailyAttendanceReport, TimeEntry};
use crate::queue::{Job, JobPayload, ReportType};
use crate::repositories::{PublisherRepository, QueueRepository};
use crate::services::{AttendanceCalculator, PolicyResolverService};

pub mod detail;
pub mod summary;

#[cfg(test)]
mod tests;

pub struct AttendanceHandler;

pub(super) struct CalculateBatchArgs<'a> {
    pub schedules: &'a [crate::models::EmployeeSchedule],
    pub entries_map: &'a HashMap<Uuid, Vec<TimeEntry>>,
    pub workspace_id: Uuid,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub _workspace_config: &'a CachedWorkspaceConfig,
    pub tz: Tz,
    pub policy_repo: &'a dyn crate::repositories::AttendancePolicyRepository,
    pub condition_type_catalog: &'a HashMap<String, crate::models::ConditionTypeDefinition>,
    pub employee_repo: &'a dyn crate::repositories::EmployeeRepository,
    pub leave_repo: &'a dyn crate::repositories::leave_request_repository::LeaveRequestRepository,
    pub employee_ids: &'a [Uuid],
    pub employment_to_employee: &'a HashMap<Uuid, Uuid>,
    pub employment_to_employer: &'a HashMap<Uuid, (Uuid, Option<String>)>,
}

impl AttendanceHandler {
    /// Process an attendance job for a batch of employees.
    #[allow(
        clippy::too_many_lines,
        clippy::too_many_arguments,
        clippy::cast_possible_truncation
    )]
    pub async fn handle(
        queue: &Arc<dyn QueueRepository>,
        publisher: &Arc<dyn PublisherRepository>,
        cache: &Arc<CacheManager>,
        job: &Job,
        roster_cache: Option<&RosterCache>,
        deps: &crate::worker::WorkerDeps,
    ) -> Result<(), String> {
        let JobPayload::AttendanceReport(payload) = &job.payload else {
            return Err(format!(
                "Expected AttendanceReport payload, got {:?}",
                job.task_type
            ));
        };
        let employee_ids = &payload.employee_ids;
        let start_date = payload.date_range.start;
        let end_date = payload.date_range.end;
        let workspace_id = job.workspace_id;

        info!(
            job_id = %job.job_id,
            employees = employee_ids.len(),
            start = %start_date,
            end = %end_date,
            "Processing attendance batch"
        );

        let t0 = std::time::Instant::now();

        // 1. Get workspace config from cache first (needed for timezone)
        let workspace_config = cache
            .workspace
            .get(workspace_id)
            .await
            .map_err(|e| format!("Failed to get workspace config: {e}"))?;

        let tz: Tz = crate::utils::tz::parse_workspace_tz(&workspace_config.timezone);

        // 2. Convert dates to UTC for time entry query
        let (start_dt, end_dt) =
            crate::services::attendance_utils::date_range_to_utc_checked(tz, start_date, end_date)
                .ok_or("Invalid or ambiguous date range")?;

        let auth_token = crate::diesel_pool::TenantAuthToken::system_override(workspace_id);

        // Fetch all employers to get names
        let mut all_employers = Vec::new();
        let mut page = 1;
        loop {
            let (employers, total) = deps
                .employer_repo
                .get_employers(&auth_token, page, 1000)
                .await
                .map_err(|e| format!("Failed to fetch employers: {e}"))?;
            if employers.is_empty() {
                break;
            }
            all_employers.extend(employers);
            if u64::try_from(all_employers.len()).unwrap_or(u64::MAX)
                >= u64::try_from(total).unwrap_or(0)
            {
                break;
            }
            page += 1;
        }

        let employer_name_map: HashMap<Uuid, String> = all_employers
            .into_iter()
            .map(|emp| (emp.id, emp.name))
            .collect();

        // Fetch employments to build active employment mappings
        let employments = deps
            .employment_repo
            .get_employments_for_employees(&auth_token, employee_ids)
            .await
            .map_err(|e| format!("Failed to fetch employments: {e}"))?;

        // Map employment_id -> employee_id
        let mut employment_to_employee = HashMap::new();
        // Map employment_id -> (employer_id, employer_name)
        let mut employment_to_employer = HashMap::new();
        let mut employment_ids = Vec::new();

        for emp in &employments {
            employment_to_employee.insert(emp.id(), emp.employee_id());
            let is_overlapping =
                emp.start_date() <= end_date && emp.end_date().is_none_or(|end| end >= start_date);

            if is_overlapping {
                employment_ids.push(emp.id());
                let employer_id = emp.employer_id();
                let employer_name = employer_name_map.get(&employer_id).cloned();
                employment_to_employer.insert(emp.id(), (employer_id, employer_name));
            }
        }

        if employment_ids.is_empty() {
            for emp in &employments {
                employment_ids.push(emp.id());
                let employer_id = emp.employer_id();
                let employer_name = employer_name_map.get(&employer_id).cloned();
                employment_to_employer.insert(emp.id(), (employer_id, employer_name));
            }
        }

        // 3. Fetch schedules AND time entries sequentially (prevent DB pool exhaustion)
        let schedules_result = deps
            .roster_repo
            .get_workspace_schedules(
                roster_cache,
                &auth_token,
                start_date,
                end_date,
                Some(&employment_ids),
                None,
                None,
            )
            .await;

        let entries_result = deps
            .time_entry_repo
            .get_workspace_time_entries_batch_by_employments(
                &auth_token,
                &employment_ids,
                start_dt,
                end_dt,
            )
            .await;

        let (schedules, _) =
            schedules_result.map_err(|e| format!("Failed to fetch schedules: {e}"))?;

        let t_fetch = t0.elapsed();

        let entries_map =
            entries_result.map_err(|e| format!("Failed to fetch time entries: {e}"))?;

        let condition_type_catalog = PolicyResolverService::load_condition_type_catalog(
            deps.condition_type_repo.as_ref(),
            &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
        )
        .await
        .map_err(|e| format!("Failed to fetch condition type catalog: {e}"))?;

        // 5. Calculate based on report type
        let policy_repo = deps.policy_repo.as_ref();

        let result_json = match payload.config.report_type {
            ReportType::Summary => {
                Self::calculate_summary(CalculateBatchArgs {
                    schedules: &schedules,
                    entries_map: &entries_map,
                    workspace_id,
                    start_date,
                    end_date,
                    _workspace_config: &workspace_config,
                    tz,
                    policy_repo,
                    condition_type_catalog: &condition_type_catalog,
                    employee_repo: deps.employee_repo.as_ref(),
                    leave_repo: deps.leave_request_repo.as_ref(),
                    employee_ids,
                    employment_to_employee: &employment_to_employee,
                    employment_to_employer: &employment_to_employer,
                })
                .await?
            }
            ReportType::Detail => {
                Self::calculate_detail(CalculateBatchArgs {
                    schedules: &schedules,
                    entries_map: &entries_map,
                    workspace_id,
                    start_date,
                    end_date,
                    _workspace_config: &workspace_config,
                    tz,
                    policy_repo,
                    condition_type_catalog: &condition_type_catalog,
                    employee_repo: deps.employee_repo.as_ref(),
                    leave_repo: deps.leave_request_repo.as_ref(),
                    employee_ids,
                    employment_to_employee: &employment_to_employee,
                    employment_to_employer: &employment_to_employer,
                })
                .await?
            }
        };

        let t_calc = t0.elapsed();

        // 6. Store result in Redis
        queue
            .store_result(job.parent_job_id, job.batch_index, &result_json)
            .await
            .map_err(|e| format!("Failed to store result: {e}"))?;

        let t_store = t0.elapsed();

        // 7. Publish chunk to Redis Pub/Sub for real-time streaming
        let metadata = queue
            .get_job_metadata(job.parent_job_id)
            .await
            .map_err(|e| format!("Failed to get job metadata: {e}"))?
            .ok_or_else(|| "Job metadata not found".to_string())?;

        publisher
            .publish_chunk(job.parent_job_id, job.batch_index, metadata.total_batches)
            .await
            .map_err(|e| format!("Failed to publish chunk: {e}"))?;

        info!(
            job_id = %job.job_id,
            result_len = result_json.len(),
            "Batch result stored and published"
        );

        // Emit timing breakdown for slow batches
        let t_total = t0.elapsed();
        if t_total.as_secs() > 5 {
            tracing::warn!(
                job_id = %job.job_id,
                employees = employee_ids.len(),
                fetch_parallel_ms = t_fetch.as_millis() as u64,
                calc_ms = t_calc.checked_sub(t_fetch).unwrap_or_default().as_millis() as u64,
                store_ms = t_store.checked_sub(t_calc).unwrap_or_default().as_millis() as u64,
                publish_ms = t_total.checked_sub(t_store).unwrap_or_default().as_millis() as u64,
                result_bytes = result_json.len(),
                total_ms = t_total.as_millis() as u64,
                "Slow attendance batch breakdown"
            );
        }

        Ok(())
    }
}
