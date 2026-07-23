use super::*;

impl AttendanceMatcher {
    /// Check whether punch times constitute a late/early violation.
    ///
    /// Returns `true` when the employee punched in late (after shift start +
    /// grace period) or punched out early (before shift end). Never flags
    /// violations on holidays.
    pub(super) fn check_late_early(
        punch_in: chrono::DateTime<chrono_tz::Tz>,
        punch_out: chrono::DateTime<chrono_tz::Tz>,
        shift_start: chrono::DateTime<chrono_tz::Tz>,
        shift_end: chrono::DateTime<chrono_tz::Tz>,
        grace_period_minutes: i32,
        is_holiday: bool,
    ) -> bool {
        if is_holiday {
            return false;
        }
        let is_late =
            punch_in > shift_start + chrono::Duration::minutes(i64::from(grace_period_minutes));
        let is_early = punch_out < shift_end;
        is_late || is_early
    }
}
