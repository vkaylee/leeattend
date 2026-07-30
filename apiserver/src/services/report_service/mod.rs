use chrono::NaiveDate;
use chrono_tz::Tz;
use std::sync::Arc;
use uuid::Uuid;

use crate::diesel_pool::TenantAuthToken;
use crate::errors::AppError;
use crate::models::{
    AttendanceReportSort, AttendanceRules, ConditionTypeDefinition, Employment, TimeEntry,
    WorkingCondition,
};
use crate::repositories::holiday_repository::HolidayRepository;
use crate::repositories::roster::RosterRepository;
use crate::repositories::workspace_repository::WorkspaceRepository;
use crate::repositories::{
    employee::EmployeeRepository, AttendanceDailySummaryRepository, AttendancePolicyRepository,
    TimeEntryRepository,
};
use crate::services::PolicyResolverService;

pub mod absences;
pub mod daily;
mod employment_selection;
pub mod no_show;
pub mod summary;

pub struct AttendanceReportArgs {
    pub workspace_id: Uuid,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub page: Option<i64>,
    pub limit: Option<i64>,
    pub employee_ids: Option<Vec<Uuid>>,
    pub department_id: Option<Uuid>,
    pub sort: Option<AttendanceReportSort>,
    pub employee_status: Option<String>,
    pub employer_id: Option<Uuid>,
    pub employment_id: Option<Uuid>,
    pub shift_type: Option<String>,
}

pub struct CalculateDayUnitsArgs<'a, 'b> {
    pub day_entries: &'a [&'a crate::models::TimeEntry],
    pub daily: &'b crate::models::EffectiveSchedule,
    pub date: NaiveDate,
    pub tz: Tz,
    pub employee_id: Uuid,
    pub workspace_id: Uuid,
    pub work_unit_minutes: i32,
    pub rules: crate::models::PolicyRules,
    pub rounding_policy: crate::models::RoundingPolicy,
}

/// Shared data fetched once by all three report methods.
/// Built by `fetch_preamble`, consumed by daily/segment/summary iterations.
pub(crate) struct ReportDataPreamble {
    pub workspace_tz: Tz,
    pub workspace_id: Uuid,
    pub page: usize,
    pub limit: usize,
    pub paginated_employees: Vec<crate::repositories::employee::EmployeeBasicInfo>,
    pub total_employees: i64,
    pub employments: Vec<Employment>,
    pub paginated_employment_ids: Vec<Uuid>,
    pub employment_to_employee: std::collections::HashMap<Uuid, Uuid>,
    pub emp_ids: Vec<Uuid>,
    pub entries_map: std::collections::HashMap<Uuid, Vec<TimeEntry>>,
    pub attendance_rules_map: std::collections::HashMap<Uuid, AttendanceRules>,
    pub working_conditions_map: std::collections::HashMap<Uuid, Vec<WorkingCondition>>,
    pub condition_type_catalog: std::collections::HashMap<String, ConditionTypeDefinition>,
    pub employer_name_map: std::collections::HashMap<Uuid, String>,
}

pub struct ReportService {
    pub(crate) time_entry_repo: Arc<dyn TimeEntryRepository>,
    pub(crate) roster_repo: Arc<dyn RosterRepository>,
    pub(crate) policy_repo: Arc<dyn AttendancePolicyRepository>,
    pub(crate) employee_repo: Arc<dyn EmployeeRepository>,
    pub(crate) condition_type_repo: Arc<dyn crate::repositories::ConditionTypeRepository>,
    pub(crate) employment_repo:
        Arc<dyn crate::repositories::employment_repository::EmploymentRepository>,
    pub(crate) attendance_daily_summary_repo: Arc<dyn AttendanceDailySummaryRepository>,
    pub(crate) leave_repo:
        Arc<dyn crate::repositories::leave_request_repository::LeaveRequestRepository>,
    pub(crate) workspace_repo: Arc<dyn WorkspaceRepository>,
    pub(crate) holiday_repo: Arc<dyn HolidayRepository>,
    pub(crate) employer_repo: Arc<dyn crate::repositories::EmployerRepository>,
}

pub struct ReportServiceArgs {
    pub time_entry_repo: Arc<dyn TimeEntryRepository>,
    pub roster_repo: Arc<dyn RosterRepository>,
    pub policy_repo: Arc<dyn AttendancePolicyRepository>,
    pub employee_repo: Arc<dyn EmployeeRepository>,
    pub condition_type_repo: Arc<dyn crate::repositories::ConditionTypeRepository>,
    pub employment_repo: Arc<dyn crate::repositories::employment_repository::EmploymentRepository>,
    pub attendance_daily_summary_repo: Arc<dyn AttendanceDailySummaryRepository>,
    pub leave_repo: Arc<dyn crate::repositories::leave_request_repository::LeaveRequestRepository>,
    pub workspace_repo: Arc<dyn WorkspaceRepository>,
    pub holiday_repo: Arc<dyn HolidayRepository>,
    pub employer_repo: Arc<dyn crate::repositories::EmployerRepository>,
}

impl ReportService {
    pub fn new(args: ReportServiceArgs) -> Self {
        Self {
            time_entry_repo: args.time_entry_repo,
            roster_repo: args.roster_repo,
            policy_repo: args.policy_repo,
            employee_repo: args.employee_repo,
            condition_type_repo: args.condition_type_repo,
            employment_repo: args.employment_repo,
            attendance_daily_summary_repo: args.attendance_daily_summary_repo,
            leave_repo: args.leave_repo,
            workspace_repo: args.workspace_repo,
            holiday_repo: args.holiday_repo,
            employer_repo: args.employer_repo,
        }
    }

    /// Fetch all employers for the workspace, handling pagination transparently.
    const EMPLOYER_PAGE_SIZE: i64 = 500;

    pub(crate) async fn fetch_all_employers(
        &self,
        auth_token: &TenantAuthToken,
    ) -> Result<std::collections::HashMap<Uuid, String>, AppError> {
        let mut all_employers = Vec::new();
        let mut page: i64 = 1;
        loop {
            let (mut batch, total) = self
                .employer_repo
                .get_employers(auth_token, page, Self::EMPLOYER_PAGE_SIZE)
                .await?;
            if batch.is_empty() {
                break;
            }
            let len_i64 = i64::try_from(all_employers.len()).unwrap_or(i64::MAX);
            let remaining = total - len_i64;
            all_employers.append(&mut batch);
            if remaining <= 0 {
                break;
            }
            page += 1;
        }
        Ok(all_employers.into_iter().map(|e| (e.id, e.name)).collect())
    }

    /// Fetch the policy-resolver trio (attendance rules, working conditions, condition-type
    /// catalog) in a single call. Shared between `fetch_preamble` and the computed-sort
    /// path in `get_workspace_attendance_summary` to eliminate duplication.
    async fn fetch_policy_trio(
        &self,
        auth_token: &TenantAuthToken,
        emp_ids: &[Uuid],
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<
        (
            std::collections::HashMap<Uuid, AttendanceRules>,
            std::collections::HashMap<Uuid, Vec<WorkingCondition>>,
            std::collections::HashMap<String, ConditionTypeDefinition>,
        ),
        AppError,
    > {
        let attendance_rules_map = PolicyResolverService::resolve_attendance_policy_batch(
            &*self.policy_repo,
            emp_ids,
            auth_token,
            start_date,
        )
        .await?;
        let working_conditions_map = PolicyResolverService::resolve_attendance_rules_batch(
            &*self.employee_repo,
            emp_ids,
            auth_token,
            start_date,
            end_date,
        )
        .await?;
        let condition_type_catalog = PolicyResolverService::load_condition_type_catalog(
            self.condition_type_repo.as_ref(),
            auth_token,
        )
        .await?;
        Ok((
            attendance_rules_map,
            working_conditions_map,
            condition_type_catalog,
        ))
    }

    /// Fetches all shared data needed by daily, segment, and summary reports.
    /// Extracted from the ~120-line preamble duplicated across all three report methods.
    ///
    /// `employment_id_filter`: when `Some`, narrows the paginated branch's employment
    /// selection to a single employment ID. Daily/segment pass `None` (they already
    /// branch on `args.employment_id` via the fast-path). Summary passes the filter
    /// to avoid re-filtering employments after the preamble returns.
    #[allow(clippy::too_many_lines)]
    async fn fetch_preamble(
        &self,
        auth_token: &TenantAuthToken,
        args: &AttendanceReportArgs,
        employment_id_filter: Option<Uuid>,
    ) -> Result<ReportDataPreamble, AppError> {
        // 1. Workspace + timezone
        let workspace = self
            .workspace_repo
            .get_workspace_by_id(args.workspace_id)
            .await
            .map_err(|e| {
                tracing::error!("Error fetching workspace: {}", e);
                AppError::Internal("Internal error".into())
            })?
            .ok_or_else(|| AppError::not_found(crate::utils::i18n::I18nKey::NotFound))?;
        let workspace_tz: Tz = crate::utils::tz::parse_workspace_tz(workspace.timezone());

        // 2. Pagination defaults
        let page = usize::try_from(args.page.unwrap_or(1)).unwrap_or(1);
        let limit = usize::try_from(args.limit.unwrap_or(20)).unwrap_or(20);

        // 3. Employee resolution (employment_id fast-path OR paginated fetch)
        // The fast-path is only used when `employment_id_filter` is None (daily/segment callers).
        // Summary passes the filter explicitly to avoid a 404 on non-existent employment IDs —
        // the original summary behavior returned empty data, not NotFound.
        let (paginated_employees, total_employees, employments, paginated_employment_ids) =
            if let (Some(specific_employment_id), None) = (args.employment_id, employment_id_filter)
            {
                let emp = self
                    .employment_repo
                    .get_employment_by_id(auth_token, specific_employment_id)
                    .await?
                    .ok_or_else(|| AppError::not_found(crate::utils::i18n::I18nKey::NotFound))?;
                let employee = self
                    .employee_repo
                    .get_employees_by_ids(&[emp.employee_id()], auth_token)
                    .await?
                    .into_iter()
                    .next()
                    .map(|e| crate::repositories::employee::EmployeeBasicInfo {
                        id: e.id(),
                        full_name: e.full_name().to_string(),
                        employee_id_number: e.employee_id_number().to_string(),
                    })
                    .ok_or_else(|| AppError::not_found(crate::utils::i18n::I18nKey::NotFound))?;
                (
                    vec![employee],
                    1i64,
                    vec![emp],
                    vec![specific_employment_id],
                )
            } else {
                let (paginated_employees, total_employees) = self
                    .employee_repo
                    .get_paginated_employee_ids(
                        auth_token,
                        crate::models::EmployeePaginationArgs::new(
                            args.employee_ids.as_ref(),
                            args.department_id,
                            args.page.unwrap_or(1),
                            args.limit.unwrap_or(20),
                            false,
                            Some((args.start_date, args.end_date)),
                            args.employee_status.clone(),
                        ),
                    )
                    .await
                    .map_err(|e| {
                        tracing::error!("Error fetching paginated employees: {}", e);
                        AppError::Internal("Internal error".into())
                    })?;

                if paginated_employees.is_empty() {
                    return Ok(ReportDataPreamble {
                        workspace_tz,
                        workspace_id: workspace.id(),
                        page,
                        limit,
                        paginated_employees,
                        total_employees,
                        employments: Vec::new(),
                        paginated_employment_ids: Vec::new(),
                        employment_to_employee: std::collections::HashMap::new(),
                        emp_ids: Vec::new(),
                        entries_map: std::collections::HashMap::new(),
                        attendance_rules_map: std::collections::HashMap::new(),
                        working_conditions_map: std::collections::HashMap::new(),
                        condition_type_catalog: std::collections::HashMap::new(),
                        employer_name_map: std::collections::HashMap::new(),
                    });
                }

                let emp_ids: Vec<Uuid> = paginated_employees.iter().map(|e| e.id).collect();
                let employments = self
                    .employment_repo
                    .get_employments_for_employees(auth_token, &emp_ids)
                    .await?;
                // Apply optional employment_id filter (used by summary report).
                let employments: Vec<_> = employments
                    .into_iter()
                    .filter(|emp| employment_id_filter.is_none_or(|id| emp.id() == id))
                    .collect();
                let selection = employment_selection::select_report_employments(
                    &employments,
                    &emp_ids,
                    args.start_date,
                    args.end_date,
                    args.employer_id,
                )?;
                let paginated_employment_ids = selection.employment_ids;
                (
                    paginated_employees,
                    total_employees,
                    employments,
                    paginated_employment_ids,
                )
            };

        // 4. employment_to_employee map
        let mut employment_to_employee = std::collections::HashMap::new();
        for emp in &employments {
            employment_to_employee.insert(emp.id(), emp.employee_id());
        }

        // 5. emp_ids (employee IDs from paginated set)
        let emp_ids: Vec<Uuid> = paginated_employees.iter().map(|e| e.id).collect();

        // 6. Date range to UTC
        let (start_dt, end_dt) = crate::services::attendance_utils::date_range_to_utc_checked(
            workspace_tz,
            args.start_date,
            args.end_date,
        )
        .ok_or_else(|| AppError::validation(crate::utils::i18n::I18nKey::Validation))?;

        // 7. Time entries batch
        let entries_map = self
            .time_entry_repo
            .get_workspace_time_entries_batch_by_employments(
                auth_token,
                &paginated_employment_ids,
                start_dt,
                end_dt,
            )
            .await?;

        // 8. Policy resolver trio
        let (attendance_rules_map, working_conditions_map, condition_type_catalog) = self
            .fetch_policy_trio(auth_token, &emp_ids, args.start_date, args.end_date)
            .await?;

        // 9. Employer names
        let employer_name_map = self.fetch_all_employers(auth_token).await?;

        Ok(ReportDataPreamble {
            workspace_tz,
            workspace_id: workspace.id(),
            page,
            limit,
            paginated_employees,
            total_employees,
            employments,
            paginated_employment_ids,
            employment_to_employee,
            emp_ids,
            entries_map,
            attendance_rules_map,
            working_conditions_map,
            condition_type_catalog,
            employer_name_map,
        })
    }
}

#[cfg(test)]
mod tests;
