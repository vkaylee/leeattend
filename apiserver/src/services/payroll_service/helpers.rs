//! Helper functions for payroll context construction.
//!
//! These utilities are used exclusively by `build_contexts.rs` and
//! `batch_orchestrator.rs` to assemble payroll data.

use chrono::NaiveDate;
use chrono_tz::Tz;
use std::collections::HashMap;

use crate::errors::AppError;
use crate::models::attendance::AttendanceRules;
use crate::models::enums::EarlyArrivalStrategy;
use crate::models::TimeEntry;
use crate::utils::i18n::I18nKey;

/// Compute the start and end `NaiveDate` for a year/month pair.
pub fn month_boundaries(year: i32, month: u32) -> Result<(NaiveDate, NaiveDate), AppError> {
    let start_date = NaiveDate::from_ymd_opt(year, month, 1)
        .ok_or_else(|| AppError::validation(I18nKey::InvalidYearMonth))?;
    let next_month_date = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1)
    }
    .ok_or_else(|| AppError::validation(I18nKey::InvalidYearMonth))?;
    let end_date = next_month_date
        .pred_opt()
        .ok_or_else(|| AppError::validation(I18nKey::InvalidYearMonth))?;
    Ok((start_date, end_date))
}

/// Fallback attendance rules used when no policy is assigned.
pub fn default_attendance_rules() -> AttendanceRules {
    AttendanceRules {
        work_unit_minutes: 30,
        rounding_policy: crate::models::RoundingPolicy::Floor,
        early_arrival_strategy: EarlyArrivalStrategy::Block,
        grace_period_minutes: None,
    }
}

/// Group time entries by local calendar date.
#[allow(clippy::trivially_copy_pass_by_ref)]
pub fn build_daily_entries_map(
    entries: Vec<TimeEntry>,
    tz: &Tz,
) -> HashMap<NaiveDate, Vec<TimeEntry>> {
    let mut map: HashMap<NaiveDate, Vec<TimeEntry>> = HashMap::new();
    for entry in entries {
        let local_date = entry.punch_in_time().with_timezone(tz).date_naive();
        map.entry(local_date).or_default().push(entry);
    }
    map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_month_boundaries_january() {
        let (start, end) = month_boundaries(2026, 1).unwrap();
        assert_eq!(start, NaiveDate::from_ymd_opt(2026, 1, 1).unwrap());
        assert_eq!(end, NaiveDate::from_ymd_opt(2026, 1, 31).unwrap());
    }

    #[test]
    fn test_month_boundaries_february_leap() {
        let (start, end) = month_boundaries(2024, 2).unwrap();
        assert_eq!(start, NaiveDate::from_ymd_opt(2024, 2, 1).unwrap());
        assert_eq!(end, NaiveDate::from_ymd_opt(2024, 2, 29).unwrap());
    }

    #[test]
    fn test_month_boundaries_december() {
        let (start, end) = month_boundaries(2026, 12).unwrap();
        assert_eq!(start, NaiveDate::from_ymd_opt(2026, 12, 1).unwrap());
        assert_eq!(end, NaiveDate::from_ymd_opt(2026, 12, 31).unwrap());
    }

    #[test]
    fn test_month_boundaries_invalid_month() {
        assert!(month_boundaries(2026, 0).is_err());
        assert!(month_boundaries(2026, 13).is_err());
    }

    #[test]
    fn test_default_attendance_rules() {
        let rules = default_attendance_rules();
        assert_eq!(rules.work_unit_minutes, 30);
        assert_eq!(rules.rounding_policy, crate::models::RoundingPolicy::Floor);
        assert_eq!(rules.early_arrival_strategy, EarlyArrivalStrategy::Block);
        assert!(rules.grace_period_minutes.is_none());
    }

    #[test]
    fn test_build_daily_entries_map_empty() {
        let map = build_daily_entries_map(vec![], &chrono_tz::UTC);
        assert!(map.is_empty());
    }
}
