use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use chrono_tz::Tz;

/// Per-day cell data for the timesheet matrix.
pub struct DayCell {
    pub std_work_units: i32,
    pub ot_work_units: i32,
    pub punch_in: Option<DateTime<Utc>>,
    pub punch_out: Option<DateTime<Utc>>,
    pub scheduled_start: Option<NaiveTime>,
    pub scheduled_end: Option<NaiveTime>,
    pub shift_symbol: Option<String>,
    pub is_holiday: bool,
}

/// Convert work units to hours: units × work_unit_minutes / 60
#[inline]
pub fn units_to_hours(units: i32, work_unit_minutes: i32) -> f64 {
    (f64::from(units) * f64::from(work_unit_minutes)) / 60.0
}

/// Ceil-round minutes to work units, then convert to hours: ⌈diff / wum⌉ × wum / 60
#[inline]
pub fn ceil_units_to_hours(diff_minutes: i64, work_unit_minutes: i64) -> f64 {
    let units = (diff_minutes + work_unit_minutes - 1) / work_unit_minutes;
    (units * work_unit_minutes) as f64 / 60.0
}

/// Format period label from template with start/end dates.
pub fn format_period_label(tpl: &str, date_range: (NaiveDate, NaiveDate)) -> String {
    tpl.replace("{start}", &date_range.0.to_string())
        .replace("{end}", &date_range.1.to_string())
}

/// Format optional NaiveTime to HH:MM string.
/// Format hours: show as integer if whole number (8.0 → "8"), otherwise 1 decimal (0.5 → "0.5").
pub fn fmt_h(h: f64) -> String {
    if h.fract() == 0.0 {
        format!("{}", h as i64)
    } else {
        format!("{h:.1}")
    }
}

pub fn format_time(time: Option<NaiveTime>) -> String {
    time.map(|t| t.format("%H:%M").to_string())
        .unwrap_or_default()
}

/// Format optional DateTime to HH:MM string in the given timezone.
pub fn format_datetime(dt: Option<DateTime<Utc>>, tz: Tz) -> String {
    dt.map(|t| t.with_timezone(&tz).format("%H:%M").to_string())
        .unwrap_or_default()
}
