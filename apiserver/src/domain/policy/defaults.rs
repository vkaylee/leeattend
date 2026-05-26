// ---------------------------------------------------------------------------
// Policy defaults — single source of truth for all hardcoded policy values
// ---------------------------------------------------------------------------

/// Default grace period in minutes before an employee is considered late.
///
/// Used as the fallback in `PolicyResolver::resolve_attendance_rules`
/// and `PolicyResolver::build_rules_from_conditions`.
pub const DEFAULT_GRACE_PERIOD_MINUTES: i32 = 15;

/// Default standard working days per month (system-level fallback).
///
/// Used when no individual, department, or workspace override is configured.
pub const DEFAULT_STD_WORKING_DAYS: i32 = 26;

/// Default working hours per day (system-level fallback).
pub const DEFAULT_HOURS_PER_DAY: i32 = 8;

/// Default working minutes per day (derived from `DEFAULT_HOURS_PER_DAY`).
pub const DEFAULT_WORK_MINUTES_PER_DAY: i32 = DEFAULT_HOURS_PER_DAY * 60;

/// Default overtime multiplier when no OT policy is configured.
pub const DEFAULT_OT_MULTIPLIER: f64 = 1.5;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults_are_reasonable() {
        let grace = std::hint::black_box(DEFAULT_GRACE_PERIOD_MINUTES);
        let std_days = std::hint::black_box(DEFAULT_STD_WORKING_DAYS);
        let hours = std::hint::black_box(DEFAULT_HOURS_PER_DAY);
        let mins = std::hint::black_box(DEFAULT_WORK_MINUTES_PER_DAY);
        let ot = std::hint::black_box(DEFAULT_OT_MULTIPLIER);
        assert!(grace > 0);
        assert!(std_days > 0 && std_days <= 31);
        assert!(hours > 0 && hours <= 24);
        assert_eq!(mins, 480);
        assert!(ot > 1.0);
    }

    #[test]
    fn test_derived_defaults_consistent() {
        let mins = std::hint::black_box(DEFAULT_WORK_MINUTES_PER_DAY);
        let hours = std::hint::black_box(DEFAULT_HOURS_PER_DAY);
        assert_eq!(mins, hours * 60);
    }
}
