pub mod agent;
pub mod assignment_rule;
pub mod attendance_adjustment;
pub mod attendance_daily_summary;
pub mod attendance_log;
pub mod attendance_policy;
pub mod audit_log;
pub mod behalf_assignment;
pub mod biometric_template;
pub mod core;
pub mod custom_field;
pub mod department;
pub mod device_enrollment;
pub mod document;
pub mod employee;
pub mod employee_contract;
pub mod employer;
pub mod employment;
pub mod employment_phase;
pub mod feature_flag;
pub mod file;
pub mod holiday;
pub mod import_job;
pub mod leave;
pub mod lifecycle_checklist;
pub mod outbox_job;
pub mod overtime_policy;
pub mod payroll;
pub mod payroll_rule;
pub mod roster;
pub mod signature;
pub mod time_entry;
pub mod work_location;
pub mod working_pattern;

use diesel::sql_types::BigInt;

#[derive(diesel::QueryableByName)]
pub struct BigIntCount {
    #[diesel(sql_type = BigInt)]
    pub count: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_big_int_count() {
        let count = BigIntCount { count: 42 };
        assert_eq!(count.count, 42);
    }
}
