use chrono::{DateTime, NaiveDate, TimeZone, Utc};
use chrono_tz::Tz;

/// Convert local date/time to UTC DateTime, handling DST edge cases.
///
/// This is the **single shared implementation** for local-to-UTC conversion,
/// replacing the duplicated versions in payroll_service, attendance_summary_service,
/// and leave_accrual_service.
pub fn local_to_utc(
    tz: &Tz,
    date: NaiveDate,
    h: u32,
    m: u32,
    s: u32,
) -> Result<DateTime<Utc>, String> {
    let naive = date
        .and_hms_opt(h, m, s)
        .ok_or_else(|| format!("Invalid time: {h}:{m}:{s}"))?;
    match tz.from_local_datetime(&naive) {
        chrono::LocalResult::Single(dt) => Ok(dt.with_timezone(&Utc)),
        chrono::LocalResult::Ambiguous(dt, _) => Ok(dt.with_timezone(&Utc)),
        chrono::LocalResult::None => {
            // DST gap - use next valid time
            let shifted = naive + chrono::Duration::hours(1);
            tz.from_local_datetime(&shifted)
                .single()
                .map(|dt| dt.with_timezone(&Utc))
                .ok_or_else(|| "Cannot resolve DST gap".to_string())
        }
    }
}

/// Variant that returns `sqlx::Error` for use in repository/service contexts
/// where the error type is `sqlx::Error`.
pub fn local_to_utc_sqlx(
    tz: &Tz,
    date: NaiveDate,
    h: u32,
    m: u32,
    s: u32,
) -> Result<DateTime<Utc>, sqlx::Error> {
    local_to_utc(tz, date, h, m, s)
        .map_err(sqlx::Error::Protocol)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Timelike;

    #[test]
    fn test_local_to_utc_basic() {
        let tz: Tz = "Asia/Ho_Chi_Minh".parse().unwrap();
        // 08:00 VN = 01:00 UTC (UTC+7)
        let result = local_to_utc(&tz, NaiveDate::from_ymd_opt(2026, 1, 16).unwrap(), 8, 0, 0);
        assert!(result.is_ok());
        let dt = result.unwrap();
        assert_eq!(dt.hour(), 1);
    }

    #[test]
    fn test_local_to_utc_midnight() {
        let tz: Tz = "UTC".parse().unwrap();
        let result = local_to_utc(&tz, NaiveDate::from_ymd_opt(2026, 3, 1).unwrap(), 0, 0, 0);
        assert!(result.is_ok());
    }

    #[test]
    fn test_local_to_utc_end_of_day() {
        let tz: Tz = "UTC".parse().unwrap();
        let result = local_to_utc(&tz, NaiveDate::from_ymd_opt(2026, 3, 1).unwrap(), 23, 59, 59);
        assert!(result.is_ok());
    }

    #[test]
    fn test_local_to_utc_invalid_time() {
        let tz: Tz = "UTC".parse().unwrap();
        let result = local_to_utc(&tz, NaiveDate::from_ymd_opt(2026, 3, 1).unwrap(), 25, 0, 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_local_to_utc_sqlx_variant() {
        let tz: Tz = "Asia/Ho_Chi_Minh".parse().unwrap();
        let result = local_to_utc_sqlx(&tz, NaiveDate::from_ymd_opt(2026, 1, 16).unwrap(), 8, 0, 0);
        assert!(result.is_ok());
    }
}
