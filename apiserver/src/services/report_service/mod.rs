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

#[derive(Clone)]
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

#[allow(dead_code)]
pub struct CalculateDayUnitsArgs<'a, 'b> {
    day_entries: &'a [&'a crate::models::TimeEntry],
    daily: &'b crate::models::EffectiveSchedule,
    date: NaiveDate,
    tz: Tz,
    #[allow(dead_code)]
    employee_id: Uuid,
    #[allow(dead_code)]
    workspace_id: Uuid,
    work_unit_minutes: i32,
    rules: crate::models::PolicyRules,
    rounding_policy: crate::models::RoundingPolicy,
}

#[allow(dead_code)]
impl<'a, 'b> CalculateDayUnitsArgs<'a, 'b> {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        day_entries: &'a [&'a crate::models::TimeEntry],
        daily: &'b crate::models::EffectiveSchedule,
        date: NaiveDate,
        tz: Tz,
        employee_id: Uuid,
        workspace_id: Uuid,
        work_unit_minutes: i32,
        rules: crate::models::PolicyRules,
        rounding_policy: crate::models::RoundingPolicy,
    ) -> Self {
        Self {
            day_entries,
            daily,
            date,
            tz,
            employee_id,
            workspace_id,
            work_unit_minutes,
            rules,
            rounding_policy,
        }
    }

    pub(crate) fn day_entries(&self) -> &'a [&'a crate::models::TimeEntry] {
        self.day_entries
    }

    pub(crate) fn daily(&self) -> &'b crate::models::EffectiveSchedule {
        self.daily
    }

    pub(crate) fn date(&self) -> NaiveDate {
        self.date
    }

    pub(crate) fn tz(&self) -> &Tz {
        &self.tz
    }

    #[allow(dead_code)]
    pub(crate) fn employee_id(&self) -> Uuid {
        self.employee_id
    }

    #[allow(dead_code)]
    pub(crate) fn workspace_id(&self) -> Uuid {
        self.workspace_id
    }

    pub(crate) fn work_unit_minutes(&self) -> i32 {
        self.work_unit_minutes
    }

    pub(crate) fn rules(&self) -> &crate::models::PolicyRules {
        &self.rules
    }

    pub(crate) fn rounding_policy(&self) -> crate::models::RoundingPolicy {
        self.rounding_policy
    }
}

/// Shared data fetched once by all three report methods.
/// Built by `fetch_preamble`, consumed by daily/segment/summary iterations.
pub(crate) struct ReportDataPreamble {
    workspace_tz: Tz,
    #[allow(dead_code)]
    workspace_id: Uuid,
    page: usize,
    limit: usize,
    paginated_employees: Vec<crate::repositories::employee::EmployeeBasicInfo>,
    total_employees: i64,
    employments: Vec<Employment>,
    paginated_employment_ids: Vec<Uuid>,
    employment_to_employee: std::collections::HashMap<Uuid, Uuid>,
    emp_ids: Vec<Uuid>,
    entries_map: std::collections::HashMap<Uuid, Vec<TimeEntry>>,
    attendance_rules_map: std::collections::HashMap<Uuid, AttendanceRules>,
    working_conditions_map: std::collections::HashMap<Uuid, Vec<WorkingCondition>>,
    condition_type_catalog: std::collections::HashMap<String, ConditionTypeDefinition>,
    employer_name_map: std::collections::HashMap<Uuid, String>,
}

impl ReportDataPreamble {
    /// Construct an empty preamble for the early-return case (no employees matched).
    pub(crate) fn empty(
        workspace_tz: Tz,
        workspace_id: Uuid,
        page: usize,
        limit: usize,
        paginated_employees: Vec<crate::repositories::employee::EmployeeBasicInfo>,
        total_employees: i64,
    ) -> Self {
        Self {
            workspace_tz,
            workspace_id,
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
        }
    }

    // ── Getters ──────────────────────────────────────────────────────────

    pub(crate) fn workspace_tz(&self) -> Tz {
        self.workspace_tz
    }

    #[allow(dead_code)]
    pub(crate) fn workspace_id(&self) -> Uuid {
        self.workspace_id
    }

    pub(crate) fn page(&self) -> usize {
        self.page
    }

    pub(crate) fn limit(&self) -> usize {
        self.limit
    }

    pub(crate) fn paginated_employees(
        &self,
    ) -> &[crate::repositories::employee::EmployeeBasicInfo] {
        &self.paginated_employees
    }

    pub(crate) fn total_employees(&self) -> i64 {
        self.total_employees
    }

    pub(crate) fn employments(&self) -> &[Employment] {
        &self.employments
    }

    pub(crate) fn paginated_employment_ids(&self) -> &Vec<Uuid> {
        &self.paginated_employment_ids
    }

    pub(crate) fn employment_to_employee(&self) -> &std::collections::HashMap<Uuid, Uuid> {
        &self.employment_to_employee
    }

    pub(crate) fn emp_ids(&self) -> &[Uuid] {
        &self.emp_ids
    }

    pub(crate) fn entries_map(&self) -> &std::collections::HashMap<Uuid, Vec<TimeEntry>> {
        &self.entries_map
    }

    pub(crate) fn attendance_rules_map(&self) -> &std::collections::HashMap<Uuid, AttendanceRules> {
        &self.attendance_rules_map
    }

    pub(crate) fn working_conditions_map(
        &self,
    ) -> &std::collections::HashMap<Uuid, Vec<WorkingCondition>> {
        &self.working_conditions_map
    }

    pub(crate) fn condition_type_catalog(
        &self,
    ) -> &std::collections::HashMap<String, ConditionTypeDefinition> {
        &self.condition_type_catalog
    }

    pub(crate) fn employer_name_map(&self) -> &std::collections::HashMap<Uuid, String> {
        &self.employer_name_map
    }
}

#[cfg(test)]
pub(crate) struct TestReportDataPreambleParams {
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

#[cfg(test)]
impl ReportDataPreamble {
    /// Build a fully-populated preamble for unit-testing extracted report functions.
    pub(crate) fn test_new(params: TestReportDataPreambleParams) -> Self {
        Self {
            workspace_tz: params.workspace_tz,
            workspace_id: params.workspace_id,
            page: params.page,
            limit: params.limit,
            paginated_employees: params.paginated_employees,
            total_employees: params.total_employees,
            employments: params.employments,
            paginated_employment_ids: params.paginated_employment_ids,
            employment_to_employee: params.employment_to_employee,
            emp_ids: params.emp_ids,
            entries_map: params.entries_map,
            attendance_rules_map: params.attendance_rules_map,
            working_conditions_map: params.working_conditions_map,
            condition_type_catalog: params.condition_type_catalog,
            employer_name_map: params.employer_name_map,
        }
    }
}

/// Intermediate result of employee/employment resolution.
/// Built by `resolve_employees`, consumed by `fetch_preamble`.
struct EmployeeResolution {
    paginated_employees: Vec<crate::repositories::employee::EmployeeBasicInfo>,
    total_employees: i64,
    employments: Vec<Employment>,
    paginated_employment_ids: Vec<Uuid>,
    employment_to_employee: std::collections::HashMap<Uuid, Uuid>,
    emp_ids: Vec<Uuid>,
}

impl EmployeeResolution {
    fn empty(
        paginated_employees: Vec<crate::repositories::employee::EmployeeBasicInfo>,
        total_employees: i64,
    ) -> Self {
        Self {
            paginated_employees,
            total_employees,
            employments: Vec::new(),
            paginated_employment_ids: Vec::new(),
            employment_to_employee: std::collections::HashMap::new(),
            emp_ids: Vec::new(),
        }
    }
}

pub struct ReportService {
    time_entry_repo: Arc<dyn TimeEntryRepository>,
    roster_repo: Arc<dyn RosterRepository>,
    policy_repo: Arc<dyn AttendancePolicyRepository>,
    employee_repo: Arc<dyn EmployeeRepository>,
    condition_type_repo: Arc<dyn crate::repositories::ConditionTypeRepository>,
    employment_repo: Arc<dyn crate::repositories::employment_repository::EmploymentRepository>,
    attendance_daily_summary_repo: Arc<dyn AttendanceDailySummaryRepository>,
    leave_repo: Arc<dyn crate::repositories::leave_request_repository::LeaveRequestRepository>,
    workspace_repo: Arc<dyn WorkspaceRepository>,
    holiday_repo: Arc<dyn HolidayRepository>,
    employer_repo: Arc<dyn crate::repositories::EmployerRepository>,
    work_segment_repo:
        Option<Arc<dyn crate::repositories::work_segment_repository::WorkSegmentRepository>>,
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
    pub work_segment_repo:
        Option<Arc<dyn crate::repositories::work_segment_repository::WorkSegmentRepository>>,
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
            work_segment_repo: args.work_segment_repo,
        }
    }
}

// Private repository accessors — submodules access fields directly (they're
// children of this module), but these getters provide a stable interface for
// potential extraction to free functions.
#[allow(dead_code)]
impl ReportService {
    fn time_entry_repo(&self) -> &dyn TimeEntryRepository {
        &*self.time_entry_repo
    }

    fn roster_repo(&self) -> &dyn RosterRepository {
        &*self.roster_repo
    }

    fn policy_repo(&self) -> &dyn AttendancePolicyRepository {
        &*self.policy_repo
    }

    fn employee_repo(&self) -> &dyn EmployeeRepository {
        &*self.employee_repo
    }

    fn condition_type_repo(&self) -> &dyn crate::repositories::ConditionTypeRepository {
        &*self.condition_type_repo
    }

    fn employment_repo(
        &self,
    ) -> &dyn crate::repositories::employment_repository::EmploymentRepository {
        &*self.employment_repo
    }

    fn attendance_daily_summary_repo(&self) -> &dyn AttendanceDailySummaryRepository {
        &*self.attendance_daily_summary_repo
    }

    fn leave_repo(
        &self,
    ) -> &dyn crate::repositories::leave_request_repository::LeaveRequestRepository {
        &*self.leave_repo
    }

    fn workspace_repo(&self) -> &dyn WorkspaceRepository {
        &*self.workspace_repo
    }

    fn holiday_repo(&self) -> &dyn HolidayRepository {
        &*self.holiday_repo
    }

    fn employer_repo(&self) -> &dyn crate::repositories::EmployerRepository {
        &*self.employer_repo
    }

    pub(crate) fn work_segment_repo(
        &self,
    ) -> Option<&dyn crate::repositories::work_segment_repository::WorkSegmentRepository> {
        self.work_segment_repo.as_deref()
    }
}

impl ReportService {
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
        tz: Tz,
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
            tz,
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

    /// Fetch workspace and parse its timezone.
    async fn fetch_workspace_tz(
        &self,
        workspace_id: Uuid,
    ) -> Result<(crate::models::Workspace, Tz), AppError> {
        let workspace = self
            .workspace_repo
            .get_workspace_by_id(workspace_id)
            .await
            .map_err(|e| {
                tracing::error!("Error fetching workspace: {}", e);
                AppError::Internal("Internal error".into())
            })?
            .ok_or_else(|| AppError::not_found(crate::utils::i18n::I18nKey::NotFound))?;
        let tz = crate::utils::tz::parse_workspace_tz(workspace.timezone());
        Ok((workspace, tz))
    }

    /// Resolve the employee/employment set for the report, handling both the
    /// single-employment fast-path and the paginated multi-employment path.
    async fn resolve_employees(
        &self,
        auth_token: &TenantAuthToken,
        args: &AttendanceReportArgs,
        employment_id_filter: Option<Uuid>,
    ) -> Result<EmployeeResolution, AppError> {
        // Fast-path: specific employment_id without an external filter.
        if let (Some(specific_employment_id), None) = (args.employment_id, employment_id_filter) {
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
            let employee_id = employee.id;
            return Ok(EmployeeResolution {
                paginated_employees: vec![employee],
                total_employees: 1,
                employments: vec![emp],
                paginated_employment_ids: vec![specific_employment_id],
                employment_to_employee: std::collections::HashMap::from([(
                    specific_employment_id,
                    employee_id,
                )]),
                emp_ids: vec![employee_id],
            });
        }

        // Paginated path
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
            return Ok(EmployeeResolution::empty(
                paginated_employees,
                total_employees,
            ));
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

        let mut employment_to_employee = std::collections::HashMap::new();
        for emp in &employments {
            employment_to_employee.insert(emp.id(), emp.employee_id());
        }

        Ok(EmployeeResolution {
            paginated_employees,
            total_employees,
            employments,
            paginated_employment_ids: selection.employment_ids,
            employment_to_employee,
            emp_ids,
        })
    }

    /// Batch-fetch time entries for the given employment IDs over a date range.
    async fn fetch_batch_entries(
        &self,
        auth_token: &TenantAuthToken,
        tz: Tz,
        employment_ids: &[Uuid],
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<std::collections::HashMap<Uuid, Vec<TimeEntry>>, AppError> {
        let (start_dt, end_dt) =
            crate::services::attendance_utils::date_range_to_utc_checked(tz, start_date, end_date)
                .ok_or_else(|| AppError::validation(crate::utils::i18n::I18nKey::Validation))?;
        self.time_entry_repo
            .get_workspace_time_entries_batch_by_employments(
                auth_token,
                employment_ids,
                start_dt,
                end_dt,
            )
            .await
    }

    /// Fetches all shared data needed by daily, segment, and summary reports.
    ///
    /// `employment_id_filter`: when `Some`, narrows the paginated branch's employment
    /// selection to a single employment ID. Daily/segment pass `None` (they already
    /// branch on `args.employment_id` via the fast-path). Summary passes the filter
    /// to avoid re-filtering employments after the preamble returns.
    async fn fetch_preamble(
        &self,
        auth_token: &TenantAuthToken,
        args: &AttendanceReportArgs,
        employment_id_filter: Option<Uuid>,
    ) -> Result<ReportDataPreamble, AppError> {
        let (workspace, workspace_tz) = self.fetch_workspace_tz(args.workspace_id).await?;
        let page = usize::try_from(args.page.unwrap_or(1)).unwrap_or(1);
        let limit = usize::try_from(args.limit.unwrap_or(20)).unwrap_or(20);

        let resolution = self
            .resolve_employees(auth_token, args, employment_id_filter)
            .await?;
        if resolution.paginated_employees.is_empty() {
            return Ok(ReportDataPreamble::empty(
                workspace_tz,
                workspace.id(),
                page,
                limit,
                resolution.paginated_employees,
                resolution.total_employees,
            ));
        }

        let entries_map = self
            .fetch_batch_entries(
                auth_token,
                workspace_tz,
                &resolution.paginated_employment_ids,
                args.start_date,
                args.end_date,
            )
            .await?;
        let (attendance_rules_map, working_conditions_map, condition_type_catalog) = self
            .fetch_policy_trio(
                auth_token,
                &resolution.emp_ids,
                args.start_date,
                args.end_date,
                workspace_tz,
            )
            .await?;
        let employer_name_map = self.fetch_all_employers(auth_token).await?;

        Ok(ReportDataPreamble {
            workspace_tz,
            workspace_id: workspace.id(),
            page,
            limit,
            paginated_employees: resolution.paginated_employees,
            total_employees: resolution.total_employees,
            employments: resolution.employments,
            paginated_employment_ids: resolution.paginated_employment_ids,
            employment_to_employee: resolution.employment_to_employee,
            emp_ids: resolution.emp_ids,
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
