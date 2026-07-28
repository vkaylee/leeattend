//! day_type resolver for [[ADR-003]] time-sliced evaluation.
//!
//! Maps a `NaiveDate` to a day-type string compatible with the L2a override
//! scope key in `attendance_policy_day_type_overrides.day_type`.
//!
//! Taxonomy: `"weekday"` | `"saturday"` | `"sunday"` | `"holiday_{type}"`.
//! `holidays` is a `HashMap<NaiveDate, Holiday>` pre-fetched by the caller
//! via `HolidayRepository::get_holidays_in_range`.

use chrono::{Datelike, NaiveDate};
use std::collections::HashMap;

use crate::models::Holiday;

/// Resolve a date to its day-type identifier for L2a cascade lookup.
///
/// Holiday dates take precedence over weekday, so a national holiday that
/// falls on a Sunday returns `"holiday_national"` not `"sunday"`.
pub fn resolve_day_type(date: NaiveDate, holidays: &HashMap<NaiveDate, Holiday>) -> String {
    if let Some(holiday) = holidays.get(&date) {
        return format!("holiday_{}", holiday.holiday_type());
    }
    match date.weekday() {
        chrono::Weekday::Sat => "saturday".to_string(),
        chrono::Weekday::Sun => "sunday".to_string(),
        _ => "weekday".to_string(),
    }
}
