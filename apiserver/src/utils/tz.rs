//! Unified timezone utilities for LeeAttend.
//!
//! This module is the **single source of truth** for:
//! - Parsing workspace timezone strings → `chrono_tz::Tz`
//! - Converting local date/time → UTC (with DST handling)
//! - Building start/end-of-day UTC boundaries from local dates
//!
//! All other modules should import from here instead of implementing
//! inline timezone logic.

use chrono::{DateTime, Duration, NaiveDate, TimeZone, Utc};
use chrono_tz::Tz;

// ---------------------------------------------------------------------------
// parse_workspace_tz — safe timezone string → Tz
// ---------------------------------------------------------------------------

/// Parse a workspace timezone string into `Tz`, falling back to UTC on failure.
///
/// Logs a warning when the timezone string is invalid so operators can fix
/// the data.  This replaces 12+ inline `.parse().unwrap_or(chrono_tz::UTC)`
/// patterns.
pub fn parse_workspace_tz(tz_str: &str) -> Tz {
    if tz_str.is_empty() {
        tracing::warn!("Empty timezone string, falling back to UTC");
        return chrono_tz::UTC;
    }
    tz_str.parse().unwrap_or_else(|_| {
        tracing::warn!("Invalid timezone '{}', falling back to UTC", tz_str);
        chrono_tz::UTC
    })
}

// ---------------------------------------------------------------------------
// local_to_utc — unified DST-safe local → UTC conversion
// ---------------------------------------------------------------------------

/// Convert local date/time to UTC DateTime, handling DST edge cases.
///
/// **DST strategy** (unified across the codebase):
/// - `Single` → direct conversion
/// - `Ambiguous` → pick the *first* (pre-transition / standard-time) instant
/// - `None` (gap) → shift forward by 1 hour and try again
///
/// This is the single implementation replacing:
/// - `chrono_helpers::local_to_utc`
/// - `datetime::naive_to_utc`
/// - `pairing::local_to_tz` (UTC variant)
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
        chrono::LocalResult::Single(dt) | chrono::LocalResult::Ambiguous(dt, _) => {
            Ok(dt.with_timezone(&Utc))
        }
        chrono::LocalResult::None => {
            // DST gap — shift forward by 1 hour
            let shifted = naive + Duration::hours(1);
            tz.from_local_datetime(&shifted)
                .single()
                .map(|dt| dt.with_timezone(&Utc))
                .ok_or_else(|| "Cannot resolve DST gap".to_string())
        }
    }
}

// ---------------------------------------------------------------------------
// local_to_tz — returns DateTime<Tz> instead of DateTime<Utc>
// ---------------------------------------------------------------------------

/// Convert local date/time to a timezone-aware `DateTime<Tz>`.
///
/// Uses the same unified DST strategy as [`local_to_utc`].
/// Returns a `Result<DateTime<Tz>, String>`.
///
/// Replaces the private `local_to_tz` in `pairing_service.rs`.
pub fn local_to_tz(
    tz: &Tz,
    date: NaiveDate,
    h: u32,
    m: u32,
    s: u32,
) -> Result<DateTime<Tz>, String> {
    let naive = date
        .and_hms_opt(h, m, s)
        .ok_or_else(|| format!("Invalid time: {h}:{m}:{s}"))?;
    match tz.from_local_datetime(&naive) {
        chrono::LocalResult::Single(dt) | chrono::LocalResult::Ambiguous(dt, _) => Ok(dt),
        chrono::LocalResult::None => {
            // DST gap — shift forward by 1 hour
            let shifted = naive + Duration::hours(1);
            tz.from_local_datetime(&shifted)
                .single()
                .ok_or_else(|| "Cannot resolve DST gap".to_string())
        }
    }
}

// ---------------------------------------------------------------------------
// Day boundary helpers
// ---------------------------------------------------------------------------

/// Get the UTC instant for the start of a local date (00:00:00).
///
/// Handles DST gaps at midnight (rare but possible).
pub fn start_of_day_utc(tz: &Tz, date: NaiveDate) -> Result<DateTime<Utc>, String> {
    local_to_utc(tz, date, 0, 0, 0)
}

/// Get the UTC instant for the end of a local date (23:59:59).
///
/// Handles DST gaps at end-of-day (rare but possible).
pub fn end_of_day_utc(tz: &Tz, date: NaiveDate) -> Result<DateTime<Utc>, String> {
    local_to_utc(tz, date, 23, 59, 59)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Timelike;

    // === parse_workspace_tz ===

    #[test]
    fn test_parse_workspace_tz_valid() {
        let tz = parse_workspace_tz("Asia/Ho_Chi_Minh");
        assert_eq!(tz, chrono_tz::Asia::Ho_Chi_Minh);

        let tz = parse_workspace_tz("America/New_York");
        assert_eq!(tz, chrono_tz::America::New_York);

        let tz = parse_workspace_tz("UTC");
        assert_eq!(tz, chrono_tz::UTC);

        let tz = parse_workspace_tz("Europe/London");
        assert_eq!(tz, chrono_tz::Europe::London);
    }

    #[test]
    fn test_parse_workspace_tz_invalid_falls_back() {
        let tz = parse_workspace_tz("Invalid/Zone");
        assert_eq!(tz, chrono_tz::UTC);

        let tz = parse_workspace_tz("not_a_timezone");
        assert_eq!(tz, chrono_tz::UTC);
    }

    #[test]
    fn test_parse_workspace_tz_empty() {
        let tz = parse_workspace_tz("");
        assert_eq!(tz, chrono_tz::UTC);
    }

    // === local_to_utc ===

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
        assert_eq!(dt.minute(), 0);
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
        let dt = result.expect("should be ok");
        assert_eq!(dt.hour(), 0);
        assert_eq!(dt.minute(), 0);
        assert_eq!(dt.second(), 0);
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
        let dt = result.expect("should be ok");
        assert_eq!(dt.hour(), 23);
        assert_eq!(dt.minute(), 59);
        assert_eq!(dt.second(), 59);
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
        assert!(result
            .expect_err("should be error")
            .contains("Invalid time"));
    }

    #[test]
    fn test_local_to_utc_dst_gap() {
        // America/New_York DST starts 2026-03-08 at 02:00 → clocks jump to 03:00
        // 02:30 does not exist — should be shifted to 03:30
        let tz: Tz = "America/New_York".parse().expect("valid tz");
        let result = local_to_utc(
            &tz,
            NaiveDate::from_ymd_opt(2026, 3, 8).expect("valid date"),
            2,
            30,
            0,
        );
        // Should succeed by shifting forward
        assert!(result.is_ok(), "DST gap should be resolved by shifting");
        let dt = result.expect("should be ok");
        // After shift: 03:30 EDT (UTC-4) = 07:30 UTC
        assert_eq!(dt.hour(), 7);
        assert_eq!(dt.minute(), 30);
    }

    #[test]
    fn test_local_to_utc_dst_ambiguous() {
        // America/New_York DST ends 2026-11-01 at 02:00 → clocks fall back to 01:00
        // 01:30 exists twice — should pick the first (EDT, UTC-4)
        let tz: Tz = "America/New_York".parse().expect("valid tz");
        let result = local_to_utc(
            &tz,
            NaiveDate::from_ymd_opt(2026, 11, 1).expect("valid date"),
            1,
            30,
            0,
        );
        assert!(result.is_ok(), "Ambiguous time should resolve to first");
        let dt = result.expect("should be ok");
        // First occurrence: EDT (UTC-4) → 01:30 + 4 = 05:30 UTC
        assert_eq!(dt.hour(), 5);
        assert_eq!(dt.minute(), 30);
    }

    #[test]
    fn test_local_to_utc_various_timezones() {
        // Europe/London at midnight
        let tz: Tz = "Europe/London".parse().expect("valid tz");
        let result = local_to_utc(
            &tz,
            NaiveDate::from_ymd_opt(2026, 1, 15).expect("valid date"),
            0,
            0,
            0,
        );
        assert!(result.is_ok());
        // January = GMT (UTC+0), so midnight London = midnight UTC
        assert_eq!(result.expect("should be ok").hour(), 0);

        // Asia/Tokyo (UTC+9)
        let tz: Tz = "Asia/Tokyo".parse().expect("valid tz");
        let result = local_to_utc(
            &tz,
            NaiveDate::from_ymd_opt(2026, 6, 15).expect("valid date"),
            9,
            0,
            0,
        );
        assert!(result.is_ok());
        // 09:00 JST = 00:00 UTC
        assert_eq!(result.expect("should be ok").hour(), 0);
    }

    // === local_to_tz ===

    #[test]
    fn test_local_to_tz_basic() {
        let tz: Tz = "Asia/Ho_Chi_Minh".parse().expect("valid tz");
        let result = local_to_tz(
            &tz,
            NaiveDate::from_ymd_opt(2026, 1, 16).expect("valid date"),
            8,
            0,
            0,
        );
        assert!(result.is_ok());
        let dt = result.expect("should be ok");
        assert_eq!(dt.hour(), 8);
        // Convert to UTC and verify
        assert_eq!(dt.with_timezone(&Utc).hour(), 1);
    }

    #[test]
    fn test_local_to_tz_dst_gap() {
        let tz: Tz = "America/New_York".parse().expect("valid tz");
        let result = local_to_tz(
            &tz,
            NaiveDate::from_ymd_opt(2026, 3, 8).expect("valid date"),
            2,
            30,
            0,
        );
        assert!(result.is_ok(), "DST gap should be resolved");
    }

    #[test]
    fn test_local_to_tz_invalid_time() {
        let tz: Tz = "UTC".parse().expect("valid tz");
        let result = local_to_tz(
            &tz,
            NaiveDate::from_ymd_opt(2026, 1, 1).expect("valid date"),
            25,
            0,
            0,
        );
        assert!(result.is_err());
    }

    // === Day boundary helpers ===

    #[test]
    fn test_start_of_day_utc() {
        let tz: Tz = "Asia/Ho_Chi_Minh".parse().expect("valid tz");
        let result = start_of_day_utc(
            &tz,
            NaiveDate::from_ymd_opt(2026, 3, 1).expect("valid date"),
        );
        assert!(result.is_ok());
        let dt = result.expect("should be ok");
        // 00:00 VN (UTC+7) = 17:00 previous day UTC
        assert_eq!(
            dt.date_naive(),
            NaiveDate::from_ymd_opt(2026, 2, 28).expect("valid date")
        );
        assert_eq!(dt.hour(), 17);
        assert_eq!(dt.minute(), 0);
    }

    #[test]
    fn test_end_of_day_utc() {
        let tz: Tz = "Asia/Ho_Chi_Minh".parse().expect("valid tz");
        let result = end_of_day_utc(
            &tz,
            NaiveDate::from_ymd_opt(2026, 3, 1).expect("valid date"),
        );
        assert!(result.is_ok());
        let dt = result.expect("should be ok");
        // 23:59:59 VN (UTC+7) = 16:59:59 same day UTC
        assert_eq!(
            dt.date_naive(),
            NaiveDate::from_ymd_opt(2026, 3, 1).expect("valid date")
        );
        assert_eq!(dt.hour(), 16);
        assert_eq!(dt.minute(), 59);
        assert_eq!(dt.second(), 59);
    }

    #[test]
    fn test_start_end_of_day_roundtrip() {
        let tz: Tz = "America/New_York".parse().expect("valid tz");
        let date = NaiveDate::from_ymd_opt(2026, 6, 15).expect("valid date");
        let start = start_of_day_utc(&tz, date).expect("valid start");
        let end = end_of_day_utc(&tz, date).expect("valid end");
        assert!(start < end, "Start of day must be before end of day");
    }

    #[test]
    fn test_start_end_of_day_utc_zone() {
        let tz: Tz = "UTC".parse().expect("valid tz");
        let date = NaiveDate::from_ymd_opt(2026, 6, 15).expect("valid date");
        let start = start_of_day_utc(&tz, date).expect("valid start");
        let end = end_of_day_utc(&tz, date).expect("valid end");

        assert_eq!(start.date_naive(), date);
        assert_eq!(start.hour(), 0);
        assert_eq!(end.date_naive(), date);
        assert_eq!(end.hour(), 23);
        assert_eq!(end.minute(), 59);
        assert_eq!(end.second(), 59);
    }

    // =========================================================================
    // Comprehensive edge case tests
    // =========================================================================

    // --- parse_workspace_tz edge cases ---

    #[test]
    fn test_parse_workspace_tz_whitespace_is_invalid() {
        // Whitespace-only strings are not empty but also not valid
        let tz = parse_workspace_tz("  ");
        assert_eq!(tz, chrono_tz::UTC);
    }

    #[test]
    fn test_parse_workspace_tz_case_sensitive() {
        // Timezone names are case-sensitive
        let tz = parse_workspace_tz("asia/ho_chi_minh"); // lowercase
        assert_eq!(tz, chrono_tz::UTC); // Should fail and fall back

        let tz = parse_workspace_tz("UTC"); // correct case
        assert_eq!(tz, chrono_tz::UTC);
    }

    #[test]
    fn test_parse_workspace_tz_numeric_offset_invalid() {
        // Offset-style strings are not valid IANA names
        let tz = parse_workspace_tz("+07:00");
        assert_eq!(tz, chrono_tz::UTC);
    }

    #[test]
    fn test_parse_workspace_tz_many_valid_zones() {
        // Verify a broad set of zones commonly used by our users
        let zones = [
            ("Asia/Ho_Chi_Minh", chrono_tz::Asia::Ho_Chi_Minh),
            ("Asia/Tokyo", chrono_tz::Asia::Tokyo),
            ("Asia/Singapore", chrono_tz::Asia::Singapore),
            ("Asia/Shanghai", chrono_tz::Asia::Shanghai),
            ("Asia/Kolkata", chrono_tz::Asia::Kolkata),
            ("America/New_York", chrono_tz::America::New_York),
            ("America/Los_Angeles", chrono_tz::America::Los_Angeles),
            ("America/Chicago", chrono_tz::America::Chicago),
            ("Europe/London", chrono_tz::Europe::London),
            ("Europe/Berlin", chrono_tz::Europe::Berlin),
            ("Australia/Sydney", chrono_tz::Australia::Sydney),
        ];
        for (name, expected) in zones {
            assert_eq!(parse_workspace_tz(name), expected, "Failed for {name}");
        }
    }

    // --- local_to_utc: European DST ---

    #[test]
    fn test_local_to_utc_europe_dst_gap() {
        // Europe/Berlin DST starts last Sunday of March
        // 2026-03-29 at 02:00 CET → clocks jump to 03:00 CEST
        let tz: Tz = "Europe/Berlin".parse().expect("valid tz");
        let result = local_to_utc(
            &tz,
            NaiveDate::from_ymd_opt(2026, 3, 29).expect("valid date"),
            2,
            30,
            0,
        );
        assert!(result.is_ok(), "European DST gap should be resolved");
        let dt = result.expect("should be ok");
        // 03:30 CEST (UTC+2) = 01:30 UTC
        assert_eq!(dt.hour(), 1);
        assert_eq!(dt.minute(), 30);
    }

    #[test]
    fn test_local_to_utc_europe_dst_ambiguous() {
        // Europe/Berlin DST ends last Sunday of October
        // 2026-10-25 at 03:00 CEST → clocks fall back to 02:00 CET
        let tz: Tz = "Europe/Berlin".parse().expect("valid tz");
        let result = local_to_utc(
            &tz,
            NaiveDate::from_ymd_opt(2026, 10, 25).expect("valid date"),
            2,
            30,
            0,
        );
        assert!(result.is_ok(), "Ambiguous time should resolve");
        // Should pick first occurrence (CEST, UTC+2)
        let dt = result.expect("should be ok");
        assert_eq!(dt.hour(), 0);
        assert_eq!(dt.minute(), 30);
    }

    #[test]
    fn test_local_to_utc_australia_dst_gap() {
        // Australia/Sydney DST starts first Sunday of October
        // 2026-10-04 at 02:00 AEST → clocks jump to 03:00 AEDT
        let tz: Tz = "Australia/Sydney".parse().expect("valid tz");
        let result = local_to_utc(
            &tz,
            NaiveDate::from_ymd_opt(2026, 10, 4).expect("valid date"),
            2,
            30,
            0,
        );
        assert!(result.is_ok(), "Australian DST gap should be resolved");
    }

    // --- local_to_utc: negative offset & date boundary ---

    #[test]
    fn test_local_to_utc_negative_offset_date_change() {
        // America/Los_Angeles (UTC-8 in winter)
        // 23:00 PT on Jan 15 = 07:00 UTC on Jan 16 (next day!)
        let tz: Tz = "America/Los_Angeles".parse().expect("valid tz");
        let result = local_to_utc(
            &tz,
            NaiveDate::from_ymd_opt(2026, 1, 15).expect("valid date"),
            23,
            0,
            0,
        );
        let dt = result.expect("should be ok");
        assert_eq!(
            dt.date_naive(),
            NaiveDate::from_ymd_opt(2026, 1, 16).expect("valid date")
        );
        assert_eq!(dt.hour(), 7);
    }

    #[test]
    fn test_local_to_utc_half_hour_offset() {
        // Asia/Kolkata (UTC+5:30)
        // 12:00 IST = 06:30 UTC
        let tz: Tz = "Asia/Kolkata".parse().expect("valid tz");
        let result = local_to_utc(
            &tz,
            NaiveDate::from_ymd_opt(2026, 6, 15).expect("valid date"),
            12,
            0,
            0,
        );
        let dt = result.expect("should be ok");
        assert_eq!(dt.hour(), 6);
        assert_eq!(dt.minute(), 30);
    }

    // --- local_to_utc: leap year ---

    #[test]
    fn test_local_to_utc_leap_year_feb29() {
        // 2028 is a leap year
        let tz: Tz = "Asia/Ho_Chi_Minh".parse().expect("valid tz");
        let result = local_to_utc(
            &tz,
            NaiveDate::from_ymd_opt(2028, 2, 29).expect("valid date"),
            10,
            0,
            0,
        );
        assert!(result.is_ok());
        let dt = result.expect("should be ok");
        // 10:00 VN (UTC+7) = 03:00 UTC
        assert_eq!(dt.hour(), 3);
    }

    #[test]
    fn test_local_to_utc_year_boundary() {
        // 2025-12-31 23:00 VN (UTC+7) = 2025-12-31 16:00 UTC
        let tz: Tz = "Asia/Ho_Chi_Minh".parse().expect("valid tz");
        let result = local_to_utc(
            &tz,
            NaiveDate::from_ymd_opt(2025, 12, 31).expect("valid date"),
            23,
            0,
            0,
        );
        let dt = result.expect("should be ok");
        assert_eq!(
            dt.date_naive(),
            NaiveDate::from_ymd_opt(2025, 12, 31).expect("valid date")
        );
        assert_eq!(dt.hour(), 16);
    }

    // --- local_to_utc: boundary seconds ---

    #[test]
    fn test_local_to_utc_max_valid_time() {
        let tz: Tz = "UTC".parse().expect("valid tz");
        let result = local_to_utc(
            &tz,
            NaiveDate::from_ymd_opt(2026, 1, 1).expect("valid date"),
            23,
            59,
            59,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_local_to_utc_invalid_seconds() {
        let tz: Tz = "UTC".parse().expect("valid tz");
        let result = local_to_utc(
            &tz,
            NaiveDate::from_ymd_opt(2026, 1, 1).expect("valid date"),
            12,
            0,
            60,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_local_to_utc_invalid_minutes() {
        let tz: Tz = "UTC".parse().expect("valid tz");
        let result = local_to_utc(
            &tz,
            NaiveDate::from_ymd_opt(2026, 1, 1).expect("valid date"),
            12,
            61,
            0,
        );
        assert!(result.is_err());
    }

    // --- local_to_tz: more edge cases ---

    #[test]
    fn test_local_to_tz_dst_ambiguous() {
        let tz: Tz = "America/New_York".parse().expect("valid tz");
        let result = local_to_tz(
            &tz,
            NaiveDate::from_ymd_opt(2026, 11, 1).expect("valid date"),
            1,
            30,
            0,
        );
        assert!(result.is_ok());
        let dt = result.expect("should be ok");
        // Should be local hour 1
        assert_eq!(dt.hour(), 1);
        assert_eq!(dt.minute(), 30);
        // First occurrence = EDT (UTC-4) → 05:30 UTC
        assert_eq!(dt.with_timezone(&Utc).hour(), 5);
    }

    #[test]
    fn test_local_to_tz_preserves_timezone() {
        let tz: Tz = "Europe/Berlin".parse().expect("valid tz");
        let result = local_to_tz(
            &tz,
            NaiveDate::from_ymd_opt(2026, 7, 15).expect("valid date"),
            14,
            30,
            0,
        );
        let dt = result.expect("should be ok");
        assert_eq!(dt.hour(), 14);
        assert_eq!(dt.minute(), 30);
        // July = CEST (UTC+2) → 12:30 UTC
        assert_eq!(dt.with_timezone(&Utc).hour(), 12);
    }

    // --- Consistency between local_to_utc and local_to_tz ---

    #[test]
    fn test_local_to_utc_and_local_to_tz_consistency() {
        // Both functions should produce the same UTC instant for the same input
        let tz: Tz = "Asia/Ho_Chi_Minh".parse().expect("valid tz");
        let date = NaiveDate::from_ymd_opt(2026, 6, 15).expect("valid date");

        let utc_result = local_to_utc(&tz, date, 10, 30, 0).expect("should be ok");
        let tz_result = local_to_tz(&tz, date, 10, 30, 0).expect("should be ok");

        assert_eq!(utc_result, tz_result.with_timezone(&Utc));
    }

    #[test]
    fn test_local_to_utc_and_local_to_tz_consistency_dst_gap() {
        // Both should resolve DST gaps the same way
        let tz: Tz = "America/New_York".parse().expect("valid tz");
        let date = NaiveDate::from_ymd_opt(2026, 3, 8).expect("valid date");

        let utc_result = local_to_utc(&tz, date, 2, 30, 0).expect("should resolve");
        let tz_result = local_to_tz(&tz, date, 2, 30, 0).expect("should resolve");

        assert_eq!(utc_result, tz_result.with_timezone(&Utc));
    }

    // --- Day boundary on DST transition ---

    #[test]
    fn test_start_of_day_on_dst_spring_forward() {
        // Start of day should work even when the day has a DST gap
        let tz: Tz = "America/New_York".parse().expect("valid tz");
        let date = NaiveDate::from_ymd_opt(2026, 3, 8).expect("valid date");
        let result = start_of_day_utc(&tz, date);
        assert!(
            result.is_ok(),
            "Start of day on DST spring-forward should work"
        );
        let dt = result.expect("should be ok");
        // Midnight EST (UTC-5) = 05:00 UTC
        assert_eq!(dt.hour(), 5);
    }

    #[test]
    fn test_end_of_day_on_dst_spring_forward() {
        let tz: Tz = "America/New_York".parse().expect("valid tz");
        let date = NaiveDate::from_ymd_opt(2026, 3, 8).expect("valid date");
        let result = end_of_day_utc(&tz, date);
        assert!(
            result.is_ok(),
            "End of day on DST spring-forward should work"
        );
        let dt = result.expect("should be ok");
        // 23:59:59 EDT (UTC-4) = 03:59:59 next day UTC
        assert_eq!(dt.hour(), 3);
        assert_eq!(dt.minute(), 59);
        assert_eq!(dt.second(), 59);
    }

    #[test]
    fn test_start_end_of_day_ordering_on_dst_day() {
        // Even on DST transition days, start < end must hold
        let tz: Tz = "America/New_York".parse().expect("valid tz");

        // Spring forward day
        let spring = NaiveDate::from_ymd_opt(2026, 3, 8).expect("valid date");
        let start = start_of_day_utc(&tz, spring).expect("valid");
        let end = end_of_day_utc(&tz, spring).expect("valid");
        assert!(
            start < end,
            "Start must be before end on spring-forward day"
        );

        // Fall back day
        let fall = NaiveDate::from_ymd_opt(2026, 11, 1).expect("valid date");
        let start = start_of_day_utc(&tz, fall).expect("valid");
        let end = end_of_day_utc(&tz, fall).expect("valid");
        assert!(start < end, "Start must be before end on fall-back day");
    }

    // --- Multiple timezone day boundary comparison ---

    #[test]
    fn test_start_of_day_ordering_across_timezones() {
        // For the same date, start_of_day(UTC+14) should be earlier than start_of_day(UTC-12)
        let date = NaiveDate::from_ymd_opt(2026, 6, 15).expect("valid date");

        let tokyo =
            start_of_day_utc(&"Asia/Tokyo".parse().expect("valid tz"), date).expect("valid");
        let ny =
            start_of_day_utc(&"America/New_York".parse().expect("valid tz"), date).expect("valid");

        // Tokyo midnight (UTC+9) happens much earlier than NY midnight (UTC-4)
        assert!(
            tokyo < ny,
            "Tokyo start of day should be before NY start of day"
        );
    }
}
