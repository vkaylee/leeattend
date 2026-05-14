//! Backward-compatible re-exports from [`super::tz`].
//!
//! New code should import from `crate::utils::tz` directly.

pub use super::tz::local_to_utc;

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{NaiveDate, Timelike};
    use chrono_tz::Tz;

    #[test]
    fn test_local_to_utc_basic() {
        let tz: Tz = "Asia/Ho_Chi_Minh".parse().expect("valid tz");
        // 08:00 VN = 01:00 UTC (UTC+7)
        let result = local_to_utc(
            &tz,
            NaiveDate::from_ymd_opt(2026, 1, 16).expect("valid date"),
            8,
            0,
            0,
        );
        assert!(result.is_ok());
        let dt = result.expect("should be ok");
        assert_eq!(dt.hour(), 1);
    }

    #[test]
    fn test_local_to_utc_midnight() {
        let tz: Tz = "UTC".parse().expect("valid tz");
        let result = local_to_utc(
            &tz,
            NaiveDate::from_ymd_opt(2026, 3, 1).expect("valid date"),
            0,
            0,
            0,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_local_to_utc_end_of_day() {
        let tz: Tz = "UTC".parse().expect("valid tz");
        let result = local_to_utc(
            &tz,
            NaiveDate::from_ymd_opt(2026, 3, 1).expect("valid date"),
            23,
            59,
            59,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_local_to_utc_invalid_time() {
        let tz: Tz = "UTC".parse().expect("valid tz");
        let result = local_to_utc(
            &tz,
            NaiveDate::from_ymd_opt(2026, 3, 1).expect("valid date"),
            25,
            0,
            0,
        );
        assert!(result.is_err());
    }
}
