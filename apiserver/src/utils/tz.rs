//! Unified timezone utilities for LeeAttend.
//!
//! This module is the **single source of truth** for:
//! - Parsing workspace timezone strings → `chrono_tz::Tz`
//! - Converting local date/time → UTC (with DST handling)
//! - Building start/end-of-day UTC boundaries from local dates
//!
//! All other modules should import from here instead of implementing
//! inline timezone logic.

use chrono::{DateTime, Duration, NaiveDate, NaiveTime, TimeZone, Timelike, Utc};
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
// validate_tz — strict timezone validation for write paths
// ---------------------------------------------------------------------------

/// Validate an IANA timezone string, rejecting anything `Tz` cannot parse.
///
/// Use this on **write** paths (create/update device, workspace settings) where
/// silently degrading to UTC would corrupt data. `parse_workspace_tz` is the
/// lenient counterpart for **read** paths, where existing rows must still load.
///
/// A device timezone is what the agent uses to reconstruct the true instant from
/// the clock's naive wall-clock reading, so a typo here shifts every punch from
/// that device by hours.
pub fn validate_tz(tz_str: &str) -> Result<Tz, crate::errors::AppError> {
    tz_str.parse::<Tz>().map_err(|e| {
        tracing::warn!("Rejected invalid timezone '{}': {}", tz_str, e);
        crate::errors::AppError::validation(crate::utils::i18n::I18nKey::InvalidTimezone)
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
// naive_time_to_tz — NaiveTime → DateTime<Tz> with DST handling
// ---------------------------------------------------------------------------

/// Convert a `NaiveTime` on a given `NaiveDate` to a timezone-aware `DateTime<Tz>`.
///
/// Uses the same unified DST strategy as [`local_to_tz`]:
/// - `Single` → direct conversion
/// - `Ambiguous` → pick the first (pre-transition) instant
/// - `None` (gap) → shift forward by 1 hour
///
/// Returns `None` only when the time components are invalid (should not happen
/// with well-formed `NaiveTime` values).
///
/// Replaces the inline `tz.from_local_datetime(&date.and_time(t)).single()`
/// pattern that silently returned `None` on DST ambiguity.
pub fn naive_time_to_tz(time: NaiveTime, date: NaiveDate, tz: Tz) -> Option<DateTime<Tz>> {
    local_to_tz(&tz, date, time.hour(), time.minute(), time.second()).ok()
}

/// Like [`naive_time_to_tz`], but resolves on the next *calendar* day when
/// `time < ref_time` (overnight shift).
///
/// Typical usage: converting shift-end or break-end times where the end may
/// cross midnight (e.g., shift 22:00–06:00). Pass the shift's start time as
/// `ref_time` so the function knows to advance the date.
///
/// **DST-safe:** Uses `NaiveDate` arithmetic to advance by exactly 1 calendar
/// day, then resolves the wall-clock time on that date. This preserves the
/// intended wall-clock time (e.g., 06:00) regardless of DST transitions
/// (which would make the day 23 or 25 hours long).
pub fn naive_time_to_tz_overnight(
    time: NaiveTime,
    ref_time: NaiveTime,
    date: NaiveDate,
    tz: Tz,
) -> Option<DateTime<Tz>> {
    if time < ref_time {
        // Overnight: resolve the same wall-clock time on the next *calendar* day.
        // Using NaiveDate arithmetic ensures exactly 1 calendar day is added,
        // regardless of DST (which would make Duration::days(1) on DateTime<Tz>
        // overshoot/undershoot by the DST offset).
        let next_date = date + Duration::days(1);
        naive_time_to_tz(time, next_date, tz)
    } else {
        naive_time_to_tz(time, date, tz)
    }
}

// ---------------------------------------------------------------------------
// resolve_shift_times — unified shift + break window resolution
// ---------------------------------------------------------------------------

/// Resolved shift times from [`resolve_shift_times`].
pub type ShiftTimeResolution = (
    Option<DateTime<Tz>>,
    Option<DateTime<Tz>>,
    Option<DateTime<Tz>>,
    Option<DateTime<Tz>>,
);

/// Resolve shift (and optionally break) start/end times to timezone-aware
/// `DateTime<Tz>`, handling overnight shifts in one place.
///
/// This is the **single source of truth** for the "convert NaiveTime shift
/// boundaries to DateTime" pattern used across report generation, worker
/// handlers, and attendance calculation.
///
/// **Overnight detection:** `end < start` → end is resolved on the next
/// *calendar* day (via `NaiveDate + Duration::days(1)`), preserving the
/// wall-clock time regardless of DST transitions. Same for breaks.
///
/// Returns `(shift_start, shift_end, break_start, break_end)` — all `None`
/// when the corresponding input is `None`.
pub fn resolve_shift_times(
    start: Option<NaiveTime>,
    end: Option<NaiveTime>,
    break_start: Option<NaiveTime>,
    break_end: Option<NaiveTime>,
    date: NaiveDate,
    tz: Tz,
) -> ShiftTimeResolution {
    let shift_start = start.and_then(|t| naive_time_to_tz(t, date, tz));
    let shift_end = if let (Some(s), Some(e)) = (start, end) {
        if e < s {
            // Overnight: resolve on next calendar day (DST-safe via NaiveDate arithmetic)
            let next_date = date + Duration::days(1);
            naive_time_to_tz(e, next_date, tz)
        } else {
            naive_time_to_tz(e, date, tz)
        }
    } else {
        end.and_then(|t| naive_time_to_tz(t, date, tz))
    };

    // Detect overnight shift: break times < shift_start belong to next calendar day
    let is_overnight_shift = match (start, end) {
        (Some(s), Some(e)) => e < s,
        _ => false,
    };

    let brk_start = break_start.and_then(|t| {
        if is_overnight_shift && start.is_some_and(|s| t < s) {
            let next_date = date + Duration::days(1);
            naive_time_to_tz(t, next_date, tz)
        } else {
            naive_time_to_tz(t, date, tz)
        }
    });
    let brk_end = break_end.and_then(|t| {
        if is_overnight_shift && start.is_some_and(|s| t < s) {
            // In overnight shift context, time < shift_start → next calendar day
            let next_date = date + Duration::days(1);
            naive_time_to_tz(t, next_date, tz)
        } else if break_start.is_some_and(|bs| t < bs) {
            // Overnight break in non-overnight shift (e.g., break 23:00–01:00 in 06:00–22:00)
            let next_date = date + Duration::days(1);
            naive_time_to_tz(t, next_date, tz)
        } else {
            naive_time_to_tz(t, date, tz)
        }
    });

    (shift_start, shift_end, brk_start, brk_end)
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

/// Get the UTC instant for the start of the *next* local calendar day (00:00:00).
///
/// Uses `NaiveDate` arithmetic to advance by exactly 1 calendar day, then
/// resolves midnight of that date through the timezone — making it correct
/// on DST transition days (23h or 25h days).
///
/// Replaces the `(start_of_day + Duration::hours(24))` and
/// `(start_of_day_local + Duration::days(1)).with_timezone(&Utc)` patterns
/// that produce wrong boundaries when DST changes the day length.
pub fn start_of_next_day_utc(tz: &Tz, date: NaiveDate) -> Result<DateTime<Utc>, String> {
    let next_date = date + Duration::days(1);
    local_to_utc(tz, next_date, 0, 0, 0)
}

// ---------------------------------------------------------------------------
// Current-date helper
// ---------------------------------------------------------------------------

/// Get the current calendar date in the given timezone.
///
/// Replaces the widespread `Utc::now().date_naive()` anti-pattern which
/// returns the UTC date — potentially one day behind the local date for
/// workspaces east of UTC (e.g., Vietnam at 01:00 local is still 18:00 UTC
/// the previous day).
pub fn utc_now_date_in_tz(tz: &Tz) -> NaiveDate {
    Utc::now().with_timezone(tz).date_naive()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Timelike;

    // === validate_tz — strict write-path validation ===

    #[test]
    fn test_validate_tz_accepts_valid_iana_names() {
        for (name, expected) in [
            ("Asia/Ho_Chi_Minh", chrono_tz::Asia::Ho_Chi_Minh),
            ("Asia/Bangkok", chrono_tz::Asia::Bangkok),
            ("America/New_York", chrono_tz::America::New_York),
            ("Europe/London", chrono_tz::Europe::London),
            ("UTC", chrono_tz::UTC),
        ] {
            let parsed = validate_tz(name).unwrap_or_else(|_| panic!("expected {name} to parse"));
            assert_eq!(parsed, expected, "wrong zone for {name}");
        }
    }

    #[test]
    fn test_validate_tz_rejects_invalid_instead_of_defaulting_to_utc() {
        // This is the whole point of the strict variant: `parse_workspace_tz`
        // would silently return UTC for each of these, which on a device row
        // shifts every punch by the real zone's offset.
        for bad in [
            "",
            "  ",
            "Invalid/Zone",
            "not_a_timezone",
            "+07:00",
            "asia/ho_chi_minh", // wrong case
            "Asia/Ho_Chi_Minh ",
        ] {
            assert!(
                validate_tz(bad).is_err(),
                "expected {bad:?} to be rejected, but it was accepted"
            );
            assert_eq!(
                parse_workspace_tz(bad),
                chrono_tz::UTC,
                "sanity: lenient parser should degrade {bad:?} to UTC"
            );
        }
    }

    #[test]
    fn test_validate_tz_error_is_validation_not_internal() {
        let err = validate_tz("Invalid/Zone").expect_err("must reject");
        assert_eq!(
            err.status_code(),
            axum::http::StatusCode::BAD_REQUEST,
            "invalid client input must surface as 400, not 500"
        );
    }

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

    // =========================================================================
    // naive_time_to_tz
    // =========================================================================

    #[test]
    fn test_naive_time_to_tz_basic() {
        use chrono::NaiveTime;
        let tz: Tz = "Asia/Ho_Chi_Minh".parse().expect("valid tz");
        let date = NaiveDate::from_ymd_opt(2026, 6, 15).expect("valid date");
        let time = NaiveTime::from_hms_opt(8, 30, 0).expect("valid time");

        let dt = naive_time_to_tz(time, date, tz).expect("should resolve");
        assert_eq!(dt.hour(), 8);
        assert_eq!(dt.minute(), 30);
        // 08:30 ICT (UTC+7) = 01:30 UTC
        assert_eq!(dt.with_timezone(&Utc).hour(), 1);
    }

    #[test]
    fn test_naive_time_to_tz_dst_gap() {
        use chrono::NaiveTime;
        // America/New_York DST starts 2026-03-08 at 02:00 → clocks jump to 03:00
        // 02:30 does not exist — should be shifted forward
        let tz: Tz = "America/New_York".parse().expect("valid tz");
        let date = NaiveDate::from_ymd_opt(2026, 3, 8).expect("valid date");
        let time = NaiveTime::from_hms_opt(2, 30, 0).expect("valid time");

        let dt = naive_time_to_tz(time, date, tz);
        assert!(
            dt.is_some(),
            "naive_time_to_tz should resolve DST gap, not return None"
        );
    }

    #[test]
    fn test_naive_time_to_tz_dst_ambiguous() {
        use chrono::NaiveTime;
        // America/New_York DST ends 2026-11-01 at 02:00 → clocks fall back to 01:00
        // 01:30 exists twice — should pick first occurrence (EDT)
        let tz: Tz = "America/New_York".parse().expect("valid tz");
        let date = NaiveDate::from_ymd_opt(2026, 11, 1).expect("valid date");
        let time = NaiveTime::from_hms_opt(1, 30, 0).expect("valid time");

        let dt = naive_time_to_tz(time, date, tz);
        assert!(
            dt.is_some(),
            "naive_time_to_tz should resolve DST ambiguity, not return None"
        );
        // First occurrence: EDT (UTC-4) → 01:30 + 4 = 05:30 UTC
        let dt = dt.expect("should resolve");
        assert_eq!(dt.with_timezone(&Utc).hour(), 5);
        assert_eq!(dt.with_timezone(&Utc).minute(), 30);
    }

    #[test]
    fn test_naive_time_to_tz_consistency_with_local_to_tz() {
        use chrono::NaiveTime;
        let tz: Tz = "Europe/Berlin".parse().expect("valid tz");
        let date = NaiveDate::from_ymd_opt(2026, 7, 15).expect("valid date");
        let time = NaiveTime::from_hms_opt(14, 30, 0).expect("valid time");

        let from_helper = naive_time_to_tz(time, date, tz).expect("helper");
        let from_direct = local_to_tz(&tz, date, 14, 30, 0).expect("direct");

        assert_eq!(
            from_helper, from_direct,
            "naive_time_to_tz should produce same result as local_to_tz"
        );
    }

    // =========================================================================
    // naive_time_to_tz_overnight
    // =========================================================================

    #[test]
    fn test_naive_time_to_tz_overnight_no_rollover() {
        use chrono::NaiveTime;
        let tz: Tz = "UTC".parse().expect("valid tz");
        let date = NaiveDate::from_ymd_opt(2026, 6, 15).expect("valid date");
        let start = NaiveTime::from_hms_opt(8, 0, 0).expect("valid");
        let end = NaiveTime::from_hms_opt(17, 0, 0).expect("valid");

        let dt = naive_time_to_tz_overnight(end, start, date, tz).expect("should resolve");
        // 17:00 > 08:00 → no overnight → same date
        assert_eq!(dt.date_naive(), date);
        assert_eq!(dt.hour(), 17);
    }

    #[test]
    fn test_naive_time_to_tz_overnight_rollover() {
        use chrono::NaiveTime;
        let tz: Tz = "UTC".parse().expect("valid tz");
        let date = NaiveDate::from_ymd_opt(2026, 6, 15).expect("valid date");
        let start = NaiveTime::from_hms_opt(22, 0, 0).expect("valid");
        let end = NaiveTime::from_hms_opt(6, 0, 0).expect("valid");

        let dt = naive_time_to_tz_overnight(end, start, date, tz).expect("should resolve");
        // 06:00 < 22:00 → overnight → next day
        assert_eq!(
            dt.date_naive(),
            NaiveDate::from_ymd_opt(2026, 6, 16).expect("valid"),
            "overnight end should advance to next day"
        );
        assert_eq!(dt.hour(), 6);
    }

    #[test]
    fn test_naive_time_to_tz_overnight_equal_times() {
        use chrono::NaiveTime;
        let tz: Tz = "UTC".parse().expect("valid tz");
        let date = NaiveDate::from_ymd_opt(2026, 6, 15).expect("valid date");
        let time = NaiveTime::from_hms_opt(12, 0, 0).expect("valid");

        let dt = naive_time_to_tz_overnight(time, time, date, tz).expect("should resolve");
        // Equal times → not overnight → same date
        assert_eq!(dt.date_naive(), date);
    }

    #[test]
    fn test_naive_time_to_tz_overnight_dst_gap() {
        use chrono::NaiveTime;
        // Overnight shift crossing a DST gap should still resolve
        let tz: Tz = "America/New_York".parse().expect("valid tz");
        let date = NaiveDate::from_ymd_opt(2026, 3, 8).expect("valid date");
        let start = NaiveTime::from_hms_opt(22, 0, 0).expect("valid");
        // End at 03:30 — but 02:00-03:00 is a DST gap on this date
        // The overnight adjustment adds 1 day, then DST gap is handled
        let end = NaiveTime::from_hms_opt(3, 30, 0).expect("valid");

        let dt = naive_time_to_tz_overnight(end, start, date, tz);
        assert!(dt.is_some(), "overnight + DST gap should resolve");
    }

    // =========================================================================
    // Roundtrip consistency: local → UTC → local
    // =========================================================================

    #[test]
    fn test_roundtrip_local_to_utc_to_local_vn() {
        let tz: Tz = "Asia/Ho_Chi_Minh".parse().expect("valid");
        let date = NaiveDate::from_ymd_opt(2026, 6, 15).expect("valid");
        let h = 14;
        let m = 30;
        let s = 0;

        let utc = local_to_utc(&tz, date, h, m, s).expect("valid");
        let back = utc.with_timezone(&tz);
        assert_eq!(back.hour(), h);
        assert_eq!(back.minute(), m);
        assert_eq!(back.second(), s);
        assert_eq!(back.date_naive(), date);
    }

    #[test]
    fn test_roundtrip_local_to_utc_to_local_negative_offset() {
        let tz: Tz = "America/Los_Angeles".parse().expect("valid");
        let date = NaiveDate::from_ymd_opt(2026, 7, 4).expect("valid");
        let h = 9;
        let m = 15;
        let s = 30;

        let utc = local_to_utc(&tz, date, h, m, s).expect("valid");
        let back = utc.with_timezone(&tz);
        assert_eq!(back.hour(), h);
        assert_eq!(back.minute(), m);
        assert_eq!(back.second(), s);
    }

    #[test]
    fn test_roundtrip_local_to_utc_to_local_half_hour_offset() {
        let tz: Tz = "Asia/Kolkata".parse().expect("valid");
        let date = NaiveDate::from_ymd_opt(2026, 3, 10).expect("valid");
        let h = 18;
        let m = 45;
        let s = 0;

        let utc = local_to_utc(&tz, date, h, m, s).expect("valid");
        let back = utc.with_timezone(&tz);
        assert_eq!(back.hour(), h);
        assert_eq!(back.minute(), m);
    }

    // =========================================================================
    // DST day length: 23-hour day (spring-forward) and 25-hour day (fall-back)
    // =========================================================================

    #[test]
    fn test_spring_forward_day_is_23_hours() {
        // 2026-03-08 in America/New_York: clocks skip 02:00→03:00 (23 hours)
        let tz: Tz = "America/New_York".parse().expect("valid");
        let date = NaiveDate::from_ymd_opt(2026, 3, 8).expect("valid");
        let start = start_of_day_utc(&tz, date).expect("valid start");
        let end = end_of_day_utc(&tz, date).expect("valid end");
        let duration = end.signed_duration_since(start);
        // 23 hours + 59 minutes + 59 seconds = 86399 seconds
        // (end_of_day is 23:59:59, so the span is just under 24h on a normal day,
        //  but on a 23-hour day it's just under 23h)
        assert!(
            duration.num_hours() == 22,
            "Spring-forward day should span ~23h (got {}h {}m)",
            duration.num_hours(),
            duration.num_minutes() % 60
        );
    }

    #[test]
    fn test_fall_back_day_is_25_hours() {
        // 2026-11-01 in America/New_York: clocks repeat 01:00→02:00 (25 hours)
        let tz: Tz = "America/New_York".parse().expect("valid");
        let date = NaiveDate::from_ymd_opt(2026, 11, 1).expect("valid");
        let start = start_of_day_utc(&tz, date).expect("valid start");
        let end = end_of_day_utc(&tz, date).expect("valid end");
        let duration = end.signed_duration_since(start);
        // On a 25-hour day, the span should be ~25h
        assert!(
            duration.num_hours() == 24,
            "Fall-back day should span ~25h (got {}h {}m)",
            duration.num_hours(),
            duration.num_minutes() % 60
        );
    }

    #[test]
    fn test_normal_day_is_24_hours() {
        let tz: Tz = "America/New_York".parse().expect("valid");
        let date = NaiveDate::from_ymd_opt(2026, 6, 15).expect("valid");
        let start = start_of_day_utc(&tz, date).expect("valid start");
        let end = end_of_day_utc(&tz, date).expect("valid end");
        let duration = end.signed_duration_since(start);
        assert_eq!(duration.num_hours(), 23);
        assert_eq!(duration.num_minutes() % 60, 59);
    }

    // =========================================================================
    // Extreme timezones: UTC+14 and UTC-12
    // =========================================================================

    #[test]
    fn test_extreme_positive_offset_kiritimati() {
        // Pacific/Kiritimati is UTC+14 — the earliest timezone
        let tz: Tz = "Pacific/Kiritimati".parse().expect("valid");
        let date = NaiveDate::from_ymd_opt(2026, 1, 1).expect("valid");
        let utc = local_to_utc(&tz, date, 0, 0, 0).expect("valid");
        // 00:00 UTC+14 = 10:00 UTC previous day (Dec 31)
        assert_eq!(
            utc.date_naive(),
            NaiveDate::from_ymd_opt(2025, 12, 31).expect("valid")
        );
        assert_eq!(utc.hour(), 10);
    }

    #[test]
    fn test_midway_atoll_utc_minus_11() {
        // Pacific/Midway is UTC-11
        let tz: Tz = "Pacific/Midway".parse().expect("valid");
        let date = NaiveDate::from_ymd_opt(2026, 1, 1).expect("valid");
        let utc = local_to_utc(&tz, date, 23, 0, 0).expect("valid");
        // 23:00 UTC-11 = 10:00 UTC next day (Jan 2)
        assert_eq!(
            utc.date_naive(),
            NaiveDate::from_ymd_opt(2026, 1, 2).expect("valid")
        );
        assert_eq!(utc.hour(), 10);
    }

    // =========================================================================
    // naive_time_to_tz_overnight: DST fall-back during overnight
    // =========================================================================

    #[test]
    fn test_naive_time_to_tz_overnight_dst_fall_back() {
        use chrono::NaiveTime;
        // Overnight shift on DST fall-back day: 22:00 → 01:30
        // Nov 1 is DST fall-back day in America/New_York (02:00 EDT → 01:00 EST).
        // After fix: resolves 01:30 on next *calendar* day (Nov 2), preserving
        // the wall-clock time regardless of DST offset.
        let tz: Tz = "America/New_York".parse().expect("valid");
        let date = NaiveDate::from_ymd_opt(2026, 11, 1).expect("valid");
        let start = NaiveTime::from_hms_opt(22, 0, 0).expect("valid");
        let end = NaiveTime::from_hms_opt(1, 30, 0).expect("valid");

        let dt = naive_time_to_tz_overnight(end, start, date, tz);
        assert!(dt.is_some(), "overnight + DST fall-back should resolve");
        let dt = dt.expect("should resolve");
        assert_eq!(
            dt.date_naive(),
            NaiveDate::from_ymd_opt(2026, 11, 2).expect("valid")
        );
        // Wall-clock preserved: 01:30 on Nov 2 (EST, UTC-5)
        assert_eq!(dt.hour(), 1);
        assert_eq!(dt.minute(), 30);

        // Verify UTC: 01:30 EST = 06:30 UTC on Nov 2
        let utc = dt.with_timezone(&Utc);
        assert_eq!(utc.hour(), 6);
        assert_eq!(utc.minute(), 30);
    }

    // =========================================================================
    // Day boundary ordering across all timezone extremes
    // =========================================================================

    #[test]
    fn test_start_of_day_ordering_extreme_east_vs_west() {
        let date = NaiveDate::from_ymd_opt(2026, 6, 15).expect("valid");

        let earliest =
            start_of_day_utc(&"Pacific/Kiritimati".parse().expect("valid"), date).expect("valid");
        let latest =
            start_of_day_utc(&"Pacific/Midway".parse().expect("valid"), date).expect("valid");

        // Kiritimati (UTC+14) starts its day 25 hours before Midway (UTC-11)
        assert!(
            earliest < latest,
            "Extreme-east start of day should be before extreme-west"
        );
        let gap = latest.signed_duration_since(earliest);
        assert_eq!(gap.num_hours(), 25);
    }

    // =========================================================================
    // local_to_tz vs local_to_utc: both produce same UTC instant
    // =========================================================================

    #[test]
    fn test_consistency_across_all_representative_zones() {
        let zones = [
            "UTC",
            "Asia/Ho_Chi_Minh",
            "Asia/Kolkata",
            "Asia/Tokyo",
            "America/New_York",
            "America/Los_Angeles",
            "Europe/London",
            "Europe/Berlin",
            "Australia/Sydney",
            "Pacific/Auckland",
        ];
        let date = NaiveDate::from_ymd_opt(2026, 6, 15).expect("valid");

        for zone in &zones {
            let tz: Tz = zone.parse().expect("valid tz");
            for h in [0, 6, 12, 18, 23] {
                let utc_result = local_to_utc(&tz, date, h, 0, 0);
                let tz_result = local_to_tz(&tz, date, h, 0, 0);
                assert!(
                    utc_result.is_ok(),
                    "local_to_utc failed for {zone} at {h}:00"
                );
                assert!(tz_result.is_ok(), "local_to_tz failed for {zone} at {h}:00");
                assert_eq!(
                    utc_result.unwrap(),
                    tz_result.unwrap().with_timezone(&Utc),
                    "Mismatch for {zone} at {h}:00"
                );
            }
        }
    }

    // =========================================================================
    // parse_workspace_tz: edge cases
    // =========================================================================

    #[test]
    fn test_parse_workspace_tz_offset_string_fails() {
        // Offset-style strings like "+07:00" are not valid IANA zone names
        let tz = parse_workspace_tz("+07:00");
        assert_eq!(tz, chrono_tz::UTC);
    }

    #[test]
    fn test_parse_workspace_tz_abbreviation_resolves() {
        // chrono-tz accepts some abbreviations like "EST", "UTC" as valid
        let tz = parse_workspace_tz("EST");
        // EST is a valid chrono-tz zone (fixed UTC-5, no DST)
        assert_ne!(tz, chrono_tz::UTC, "EST should resolve to a non-UTC zone");
    }

    #[test]
    fn test_parse_workspace_tz_trailing_whitespace_fails() {
        let tz = parse_workspace_tz("Asia/Ho_Chi_Minh ");
        assert_eq!(tz, chrono_tz::UTC);
    }

    // =========================================================================
    // start_of_day_utc / end_of_day_utc: start < end invariant for all zones
    // =========================================================================

    #[test]
    fn test_start_before_end_for_many_zones_and_dates() {
        let zones = [
            "UTC",
            "Asia/Ho_Chi_Minh",
            "America/New_York",
            "Europe/Berlin",
            "Australia/Sydney",
        ];
        // Include DST transition dates and normal dates
        let dates = [
            NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
            NaiveDate::from_ymd_opt(2026, 3, 8).unwrap(), // US DST spring
            NaiveDate::from_ymd_opt(2026, 3, 29).unwrap(), // EU DST spring
            NaiveDate::from_ymd_opt(2026, 6, 15).unwrap(),
            NaiveDate::from_ymd_opt(2026, 10, 4).unwrap(), // AU DST spring
            NaiveDate::from_ymd_opt(2026, 11, 1).unwrap(), // US DST fall
            NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
        ];

        for zone in &zones {
            let tz: Tz = zone.parse().expect("valid tz");
            for date in &dates {
                let start = start_of_day_utc(&tz, *date)
                    .unwrap_or_else(|_| panic!("start_of_day failed for {zone} on {date}"));
                let end = end_of_day_utc(&tz, *date)
                    .unwrap_or_else(|_| panic!("end_of_day failed for {zone} on {date}"));
                assert!(
                    start < end,
                    "start_of_day must be before end_of_day for {zone} on {date}"
                );
            }
        }
    }

    // =========================================================================
    // resolve_shift_times
    // =========================================================================

    #[test]
    fn test_resolve_shift_times_normal_day_shift() {
        let tz: Tz = "Asia/Ho_Chi_Minh".parse().expect("valid");
        let date = NaiveDate::from_ymd_opt(2026, 6, 15).expect("valid");
        let start = Some(NaiveTime::from_hms_opt(8, 0, 0).expect("valid"));
        let end = Some(NaiveTime::from_hms_opt(17, 0, 0).expect("valid"));

        let (s, e, bs, be) = resolve_shift_times(start, end, None, None, date, tz);
        assert!(s.is_some());
        assert!(e.is_some());
        assert!(bs.is_none());
        assert!(be.is_none());
        assert_eq!(s.unwrap().date_naive(), date);
        assert_eq!(e.unwrap().date_naive(), date);
    }

    #[test]
    fn test_resolve_shift_times_overnight() {
        let tz: Tz = "Asia/Ho_Chi_Minh".parse().expect("valid");
        let date = NaiveDate::from_ymd_opt(2026, 6, 15).expect("valid");
        let start = Some(NaiveTime::from_hms_opt(22, 0, 0).expect("valid"));
        let end = Some(NaiveTime::from_hms_opt(6, 0, 0).expect("valid"));

        let (s, e, _, _) = resolve_shift_times(start, end, None, None, date, tz);
        let s = s.expect("start");
        let e = e.expect("end");

        assert_eq!(s.date_naive(), date);
        assert_eq!(
            e.date_naive(),
            NaiveDate::from_ymd_opt(2026, 6, 16).expect("valid")
        );
        // Duration should be 8 hours
        assert_eq!(e.signed_duration_since(s).num_hours(), 8);
    }

    #[test]
    fn test_resolve_shift_times_with_break() {
        let tz: Tz = "UTC".parse().expect("valid");
        let date = NaiveDate::from_ymd_opt(2026, 6, 15).expect("valid");
        let start = Some(NaiveTime::from_hms_opt(9, 0, 0).expect("valid"));
        let end = Some(NaiveTime::from_hms_opt(18, 0, 0).expect("valid"));
        let bs = Some(NaiveTime::from_hms_opt(12, 0, 0).expect("valid"));
        let be = Some(NaiveTime::from_hms_opt(13, 0, 0).expect("valid"));

        let (s, e, brk_s, brk_e) = resolve_shift_times(start, end, bs, be, date, tz);
        assert!(s.is_some());
        assert!(e.is_some());
        assert!(brk_s.is_some());
        assert!(brk_e.is_some());
        assert_eq!(brk_s.unwrap().hour(), 12);
        assert_eq!(brk_e.unwrap().hour(), 13);
    }

    #[test]
    fn test_resolve_shift_times_overnight_break() {
        let tz: Tz = "UTC".parse().expect("valid");
        let date = NaiveDate::from_ymd_opt(2026, 6, 15).expect("valid");
        let start = Some(NaiveTime::from_hms_opt(22, 0, 0).expect("valid"));
        let end = Some(NaiveTime::from_hms_opt(6, 0, 0).expect("valid"));
        let bs = Some(NaiveTime::from_hms_opt(1, 0, 0).expect("valid"));
        let be = Some(NaiveTime::from_hms_opt(1, 30, 0).expect("valid"));

        let (_, _, brk_s, brk_e) = resolve_shift_times(start, end, bs, be, date, tz);
        let brk_s = brk_s.expect("break start");
        let brk_e = brk_e.expect("break end");
        // Break 01:00-01:30 within overnight shift 22:00-06:00 — both on next calendar day
        let next_day = date + Duration::days(1);
        assert_eq!(brk_s.date_naive(), next_day);
        assert_eq!(brk_e.date_naive(), next_day);
    }

    #[test]
    fn test_resolve_shift_times_none_inputs() {
        let tz: Tz = "UTC".parse().expect("valid");
        let date = NaiveDate::from_ymd_opt(2026, 6, 15).expect("valid");

        let (s, e, bs, be) = resolve_shift_times(None, None, None, None, date, tz);
        assert!(s.is_none());
        assert!(e.is_none());
        assert!(bs.is_none());
        assert!(be.is_none());
    }

    #[test]
    fn test_resolve_shift_times_dst_gap() {
        // America/New_York DST starts 2026-03-08 at 02:00 → 03:00
        let tz: Tz = "America/New_York".parse().expect("valid");
        let date = NaiveDate::from_ymd_opt(2026, 3, 8).expect("valid");
        let start = Some(NaiveTime::from_hms_opt(2, 0, 0).expect("valid"));
        let end = Some(NaiveTime::from_hms_opt(10, 0, 0).expect("valid"));

        let (s, e, _, _) = resolve_shift_times(start, end, None, None, date, tz);
        // DST gap at 02:00 should resolve (not return None)
        assert!(s.is_some(), "DST gap at shift start should resolve");
        assert!(e.is_some(), "shift end should resolve");
    }

    #[test]
    fn test_resolve_shift_times_consistency_with_individual_calls() {
        // Verify resolve_shift_times produces same results as calling
        // naive_time_to_tz + manual overnight check individually
        let tz: Tz = "Asia/Ho_Chi_Minh".parse().expect("valid");
        let date = NaiveDate::from_ymd_opt(2026, 6, 15).expect("valid");
        let start = NaiveTime::from_hms_opt(22, 0, 0).expect("valid");
        let end = NaiveTime::from_hms_opt(6, 0, 0).expect("valid");

        // Individual approach (DST-safe: use NaiveDate arithmetic, not DateTime += days)
        let ind_start = naive_time_to_tz(start, date, tz).expect("start");
        let ind_end = if end < start {
            let next_date = date + Duration::days(1);
            naive_time_to_tz(end, next_date, tz).expect("end next day")
        } else {
            naive_time_to_tz(end, date, tz).expect("end same day")
        };

        // Helper approach
        let (s, e, _, _) = resolve_shift_times(Some(start), Some(end), None, None, date, tz);
        assert_eq!(s.unwrap(), ind_start);
        assert_eq!(e.unwrap(), ind_end);
    }

    // =========================================================================
    // Overnight shifts on DST transition days — wall-clock preservation
    // =========================================================================

    /// DST date constants for America/New_York 2026
    const NY_SPRING_FORWARD_DATE: NaiveDate = match NaiveDate::from_ymd_opt(2026, 3, 8) {
        Some(d) => d,
        None => unreachable!(),
    };
    const NY_FALL_BACK_DATE: NaiveDate = match NaiveDate::from_ymd_opt(2026, 11, 1) {
        Some(d) => d,
        None => unreachable!(),
    };
    /// Day before DST transitions — for overnight shift tests
    const NY_DAY_BEFORE_SPRING_FORWARD: NaiveDate = match NaiveDate::from_ymd_opt(2026, 3, 7) {
        Some(d) => d,
        None => unreachable!(),
    };
    const NY_DAY_BEFORE_FALL_BACK: NaiveDate = match NaiveDate::from_ymd_opt(2026, 10, 31) {
        Some(d) => d,
        None => unreachable!(),
    };

    fn ny() -> Tz {
        "America/New_York".parse().expect("valid tz")
    }

    fn sydney() -> Tz {
        "Australia/Sydney".parse().expect("valid tz")
    }

    #[test]
    fn test_overnight_shift_spring_forward() {
        use chrono::NaiveTime;
        // Overnight shift 22:00-06:00 crossing spring-forward day.
        // Date: Mar 7 (day before DST). DST transition: Mar 8 at 02:00 EST → 03:00 EDT.
        // Expected: shift end = 06:00 EDT on Mar 8 (wall-clock preserved).
        // Duration: 7 real hours (1 hour lost to spring-forward).
        let tz = ny();
        let date = NY_DAY_BEFORE_SPRING_FORWARD;
        let start = NaiveTime::from_hms_opt(22, 0, 0).expect("valid");
        let end = NaiveTime::from_hms_opt(6, 0, 0).expect("valid");

        let dt = naive_time_to_tz_overnight(end, start, date, tz);
        let dt = dt.expect("should resolve");

        // Wall-clock: 06:00 on Mar 8
        assert_eq!(
            dt.date_naive(),
            NaiveDate::from_ymd_opt(2026, 3, 8).expect("valid")
        );
        assert_eq!(dt.hour(), 6);
        assert_eq!(dt.minute(), 0);

        // Duration: 7 hours (spring-forward lost 1h)
        let shift_start = naive_time_to_tz(start, date, tz).expect("start");
        let hours = dt.signed_duration_since(shift_start).num_hours();
        assert_eq!(
            hours, 7,
            "spring-forward overnight shift should be 7h, got {hours}h"
        );
    }

    #[test]
    fn test_overnight_shift_fall_back() {
        use chrono::NaiveTime;
        // Overnight shift 22:00-06:00 crossing fall-back day.
        // Date: Oct 31 (day before DST). DST transition: Nov 1 at 02:00 EDT → 01:00 EST.
        // Expected: shift end = 06:00 EST on Nov 1 (wall-clock preserved).
        // Duration: 9 real hours (1 extra hour from fall-back).
        let tz = ny();
        let date = NY_DAY_BEFORE_FALL_BACK;
        let start = NaiveTime::from_hms_opt(22, 0, 0).expect("valid");
        let end = NaiveTime::from_hms_opt(6, 0, 0).expect("valid");

        let dt = naive_time_to_tz_overnight(end, start, date, tz);
        let dt = dt.expect("should resolve");

        // Wall-clock: 06:00 on Nov 1
        assert_eq!(
            dt.date_naive(),
            NaiveDate::from_ymd_opt(2026, 11, 1).expect("valid")
        );
        assert_eq!(dt.hour(), 6);
        assert_eq!(dt.minute(), 0);

        // Duration: 9 hours (fall-back gained 1h)
        let shift_start = naive_time_to_tz(start, date, tz).expect("start");
        let hours = dt.signed_duration_since(shift_start).num_hours();
        assert_eq!(
            hours, 9,
            "fall-back overnight shift should be 9h, got {hours}h"
        );
    }

    #[test]
    fn test_overnight_shift_no_dst() {
        use chrono::NaiveTime;
        // Normal overnight shift 22:00-06:00, no DST transition.
        let tz = ny();
        let date = NaiveDate::from_ymd_opt(2026, 6, 15).expect("valid");
        let start = NaiveTime::from_hms_opt(22, 0, 0).expect("valid");
        let end = NaiveTime::from_hms_opt(6, 0, 0).expect("valid");

        let dt = naive_time_to_tz_overnight(end, start, date, tz);
        let dt = dt.expect("should resolve");

        assert_eq!(
            dt.date_naive(),
            NaiveDate::from_ymd_opt(2026, 6, 16).expect("valid")
        );
        assert_eq!(dt.hour(), 6);

        let shift_start = naive_time_to_tz(start, date, tz).expect("start");
        assert_eq!(dt.signed_duration_since(shift_start).num_hours(), 8);
    }

    #[test]
    fn test_resolve_shift_times_overnight_spring_forward() {
        // Full resolve_shift_times with overnight shift on spring-forward day
        let tz = ny();
        let date = NY_DAY_BEFORE_SPRING_FORWARD;
        let start = Some(NaiveTime::from_hms_opt(22, 0, 0).expect("valid"));
        let end = Some(NaiveTime::from_hms_opt(6, 0, 0).expect("valid"));

        let (s, e, _, _) = resolve_shift_times(start, end, None, None, date, tz);
        let s = s.expect("start");
        let e = e.expect("end");

        // Shift start: 22:00 EST Mar 7
        assert_eq!(s.date_naive(), date);
        assert_eq!(s.hour(), 22);

        // Shift end: 06:00 EDT Mar 8 (wall-clock preserved, not 07:00)
        assert_eq!(
            e.date_naive(),
            NaiveDate::from_ymd_opt(2026, 3, 8).expect("valid")
        );
        assert_eq!(e.hour(), 6);

        // Duration: 7 hours (spring-forward)
        assert_eq!(e.signed_duration_since(s).num_hours(), 7);
    }

    #[test]
    fn test_resolve_shift_times_overnight_fall_back() {
        // Full resolve_shift_times with overnight shift on fall-back day
        let tz = ny();
        let date = NY_DAY_BEFORE_FALL_BACK;
        let start = Some(NaiveTime::from_hms_opt(22, 0, 0).expect("valid"));
        let end = Some(NaiveTime::from_hms_opt(6, 0, 0).expect("valid"));

        let (s, e, _, _) = resolve_shift_times(start, end, None, None, date, tz);
        let s = s.expect("start");
        let e = e.expect("end");

        // Shift start: 22:00 EDT Oct 31
        assert_eq!(s.date_naive(), date);
        assert_eq!(s.hour(), 22);

        // Shift end: 06:00 EST Nov 1 (wall-clock preserved, not 05:00)
        assert_eq!(
            e.date_naive(),
            NaiveDate::from_ymd_opt(2026, 11, 1).expect("valid")
        );
        assert_eq!(e.hour(), 6);

        // Duration: 9 hours (fall-back)
        assert_eq!(e.signed_duration_since(s).num_hours(), 9);
    }

    // =========================================================================
    // start_of_next_day_utc — DST-safe next-day boundary
    // =========================================================================

    #[test]
    fn test_start_of_next_day_utc_spring_forward() {
        let tz = ny();
        // Mar 8 is spring-forward day (23h day). Next day starts at normal midnight.
        let date = NY_SPRING_FORWARD_DATE;
        let next = start_of_next_day_utc(&tz, date).expect("valid");

        // Mar 9 midnight EDT = 04:00 UTC
        assert_eq!(
            next.date_naive(),
            NaiveDate::from_ymd_opt(2026, 3, 9).expect("valid")
        );
        let utc = next.with_timezone(&Utc);
        assert_eq!(utc.hour(), 4); // EDT = UTC-4
        assert_eq!(utc.minute(), 0);
    }

    #[test]
    fn test_start_of_next_day_utc_fall_back() {
        let tz = ny();
        // Nov 1 is fall-back day (25h day). Next day starts at normal midnight.
        let date = NY_FALL_BACK_DATE;
        let next = start_of_next_day_utc(&tz, date).expect("valid");

        // Nov 2 midnight EST = 05:00 UTC
        let utc = next.with_timezone(&Utc);
        assert_eq!(utc.hour(), 5); // EST = UTC-5
        assert_eq!(utc.minute(), 0);
    }

    #[test]
    fn test_start_of_next_day_utc_normal() {
        let tz: Tz = "Asia/Ho_Chi_Minh".parse().expect("valid");
        let date = NaiveDate::from_ymd_opt(2026, 6, 15).expect("valid");
        let next = start_of_next_day_utc(&tz, date).expect("valid");

        // Jun 16 midnight ICT (UTC+7) = 17:00 UTC Jun 15
        let utc = next.with_timezone(&Utc);
        assert_eq!(
            utc.date_naive(),
            NaiveDate::from_ymd_opt(2026, 6, 15).expect("valid")
        );
        assert_eq!(utc.hour(), 17);
    }

    // =========================================================================
    // utc_now_date_in_tz
    // =========================================================================

    #[test]
    fn test_utc_now_date_in_tz() {
        // Just verify it returns a valid date — the exact value depends on
        // when the test runs. We verify the function doesn't panic and returns
        // a date within a reasonable range.
        let tz: Tz = "Asia/Ho_Chi_Minh".parse().expect("valid");
        let date = utc_now_date_in_tz(&tz);
        let utc_date = Utc::now().date_naive();

        // Vietnam (UTC+7) date should be same as UTC or 1 day ahead
        let diff = date.signed_duration_since(utc_date).num_days();
        assert!(
            (0..=1).contains(&diff),
            "Vietnam date should be UTC date or UTC+1, got diff={diff}"
        );
    }

    // =========================================================================
    // Overnight shift in extreme timezones
    // =========================================================================

    #[test]
    fn test_overnight_shift_extreme_east() {
        use chrono::NaiveTime;
        // Pacific/Kiritimati (UTC+14, no DST) — overnight shift
        let tz: Tz = "Pacific/Kiritimati".parse().expect("valid");
        let date = NaiveDate::from_ymd_opt(2026, 6, 15).expect("valid");
        let start = NaiveTime::from_hms_opt(22, 0, 0).expect("valid");
        let end = NaiveTime::from_hms_opt(6, 0, 0).expect("valid");

        let dt = naive_time_to_tz_overnight(end, start, date, tz);
        let dt = dt.expect("should resolve");

        assert_eq!(
            dt.date_naive(),
            NaiveDate::from_ymd_opt(2026, 6, 16).expect("valid")
        );
        assert_eq!(dt.hour(), 6);

        let shift_start = naive_time_to_tz(start, date, tz).expect("start");
        assert_eq!(dt.signed_duration_since(shift_start).num_hours(), 8);
    }

    // =========================================================================
    // Southern hemisphere DST (Australia/Sydney)
    // =========================================================================

    #[test]
    fn test_overnight_shift_sydney_spring_forward() {
        use chrono::NaiveTime;
        // Sydney DST starts first Sunday of October: 2026-10-04 at 02:00 AEST → 03:00 AEDT
        let tz = sydney();
        let date = NaiveDate::from_ymd_opt(2026, 10, 3).expect("valid");
        let start = NaiveTime::from_hms_opt(22, 0, 0).expect("valid");
        let end = NaiveTime::from_hms_opt(6, 0, 0).expect("valid");

        let dt = naive_time_to_tz_overnight(end, start, date, tz);
        let dt = dt.expect("should resolve");

        // Wall-clock: 06:00 AEDT on Oct 4
        assert_eq!(
            dt.date_naive(),
            NaiveDate::from_ymd_opt(2026, 10, 4).expect("valid")
        );
        assert_eq!(dt.hour(), 6);

        // Duration: 7 hours (spring-forward lost 1h)
        let shift_start = naive_time_to_tz(start, date, tz).expect("start");
        assert_eq!(dt.signed_duration_since(shift_start).num_hours(), 7);
    }

    // =========================================================================
    // Overnight break on DST transition day
    // =========================================================================

    #[test]
    fn test_resolve_shift_times_overnight_break_spring_forward() {
        // Shift 22:00-06:00 with break 01:00-01:30, crossing spring-forward day.
        // DST gap at 02:00 on Mar 8 in NY. Break at 01:00 is before the gap.
        let tz = ny();
        let date = NY_DAY_BEFORE_SPRING_FORWARD;
        let start = Some(NaiveTime::from_hms_opt(22, 0, 0).expect("valid"));
        let end = Some(NaiveTime::from_hms_opt(6, 0, 0).expect("valid"));
        let bs = Some(NaiveTime::from_hms_opt(1, 0, 0).expect("valid"));
        let be = Some(NaiveTime::from_hms_opt(1, 30, 0).expect("valid"));

        let (s, e, brk_s, brk_e) = resolve_shift_times(start, end, bs, be, date, tz);
        let s = s.expect("shift start");
        let e = e.expect("shift end");
        let brk_s = brk_s.expect("break start");
        let brk_e = brk_e.expect("break end");

        // Shift: 22:00 EST Mar 7 → 06:00 EDT Mar 8 (7h due to spring-forward)
        assert_eq!(e.signed_duration_since(s).num_hours(), 7);

        // Break: 01:00-01:30 EST on Mar 8 (before DST gap)
        assert_eq!(
            brk_s.date_naive(),
            NaiveDate::from_ymd_opt(2026, 3, 8).expect("valid")
        );
        assert_eq!(brk_s.hour(), 1);
        assert_eq!(brk_e.hour(), 1);
        assert_eq!(brk_e.minute(), 30);

        // Break duration: 30 minutes
        assert_eq!(brk_e.signed_duration_since(brk_s).num_minutes(), 30);
    }
}
