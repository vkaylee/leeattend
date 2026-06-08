use chrono::NaiveDate;
use chrono_tz::Tz;
use std::sync::Arc;
use uuid::Uuid;

use crate::models::AttendanceReportSort;
use crate::repositories::holiday_repository::HolidayRepository;
use crate::repositories::roster::RosterRepository;
use crate::repositories::workspace_repository::WorkspaceRepository;
use crate::repositories::{
    employee::EmployeeRepository, AttendanceDailySummaryRepository, AttendancePolicyRepository,
    TimeEntryRepository,
};

pub mod absences;
pub mod daily;
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
    pub rounding_policy: &'a str,
}

pub struct ReportService {
    pub(crate) time_entry_repo: Arc<dyn TimeEntryRepository>,
    pub(crate) roster_repo: Arc<dyn RosterRepository>,
    pub(crate) policy_repo: Arc<dyn AttendancePolicyRepository>,
    pub(crate) employee_repo: Arc<dyn EmployeeRepository>,
    pub(crate) employment_repo:
        Arc<dyn crate::repositories::employment_repository::EmploymentRepository>,
    pub(crate) attendance_daily_summary_repo: Arc<dyn AttendanceDailySummaryRepository>,
    pub(crate) leave_repo:
        Arc<dyn crate::repositories::leave_request_repository::LeaveRequestRepository>,
    pub(crate) workspace_repo: Arc<dyn WorkspaceRepository>,
    pub(crate) holiday_repo: Arc<dyn HolidayRepository>,
}

pub struct ReportServiceArgs {
    pub time_entry_repo: Arc<dyn TimeEntryRepository>,
    pub roster_repo: Arc<dyn RosterRepository>,
    pub policy_repo: Arc<dyn AttendancePolicyRepository>,
    pub employee_repo: Arc<dyn EmployeeRepository>,
    pub employment_repo: Arc<dyn crate::repositories::employment_repository::EmploymentRepository>,
    pub attendance_daily_summary_repo: Arc<dyn AttendanceDailySummaryRepository>,
    pub leave_repo: Arc<dyn crate::repositories::leave_request_repository::LeaveRequestRepository>,
    pub workspace_repo: Arc<dyn WorkspaceRepository>,
    pub holiday_repo: Arc<dyn HolidayRepository>,
}

impl ReportService {
    pub fn new(args: ReportServiceArgs) -> Self {
        Self {
            time_entry_repo: args.time_entry_repo,
            roster_repo: args.roster_repo,
            policy_repo: args.policy_repo,
            employee_repo: args.employee_repo,
            employment_repo: args.employment_repo,
            attendance_daily_summary_repo: args.attendance_daily_summary_repo,
            leave_repo: args.leave_repo,
            workspace_repo: args.workspace_repo,
            holiday_repo: args.holiday_repo,
        }
    }
}

#[cfg(test)]
mod tests;
