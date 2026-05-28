pub mod agent;
pub mod assignment_rule;
pub mod attendance_adjustment;
pub mod attendance_daily_summary;
pub mod attendance_log;
pub mod attendance_policy;
pub mod audit_log;
pub mod core;
pub mod department;
pub mod employee;
pub mod employee_contract;
pub mod employment_phase;
pub mod feature_flag;
pub mod holiday;
pub mod import_job;
pub mod leave;
pub mod overtime_policy;
pub mod payroll;
pub mod payroll_rule;
pub mod roster;
pub mod time_entry;
pub mod work_location;
pub mod working_pattern;

use diesel::sql_types::BigInt;

#[derive(diesel::QueryableByName)]
pub struct BigIntCount {
    #[diesel(sql_type = BigInt)]
    pub count: i64,
}
