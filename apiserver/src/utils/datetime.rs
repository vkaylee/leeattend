use chrono::{DateTime, NaiveDateTime, Timelike, Utc};
use chrono_tz::Tz;

/// Parses a datetime string into Utc.
///
/// Priority:
/// 1. If string contains a timezone/offset (e.g. ISO8601 with Z or +HH:mm), parse it directly.
/// 2. Otherwise, parse as NaiveDateTime using provided or fallback formats,
///    and convert to Utc using the provided workspace timezone.
pub fn parse_datetime_with_tz(
    dt_str: &str,
    workspace_tz: &str,
    preferred_date_fmt: Option<&str>,
    preferred_time_fmt: Option<&str>,
) -> Result<DateTime<Utc>, String> {
    let dt_str = dt_str.trim();
    if dt_str.is_empty() {
        return Err("Dữ liệu thời gian bị trống".to_string());
    }

    // 1. Try parsing as full DateTime with offset (ISO 8601)
    if let Ok(dt) = DateTime::parse_from_rfc3339(dt_str) {
        return Ok(dt.with_timezone(&Utc));
    }

    // Try other ISO formats that DateTime::parse_from_str might handle if they have offsets
    // e.g. "2023-01-01 12:00:00 +07:00"
    if let Ok(dt) = DateTime::parse_from_str(dt_str, "%Y-%m-%d %H:%M:%S %z") {
        return Ok(dt.with_timezone(&Utc));
    }

    // 2. Parse as Naive if no offset is found
    let tz: Tz = workspace_tz
        .parse()
        .map_err(|_e| format!("Múi giờ không hợp lệ: {workspace_tz}"))?;

    let date_pref = preferred_date_fmt.unwrap_or("DD/MM/YYYY").to_uppercase();
    let time_pref = preferred_time_fmt.unwrap_or("24h");

    let mut priority_fmts = Vec::new();
    let date_fmts = match date_pref.as_str() {
        "MM/DD/YYYY" => vec!["%m/%d/%Y", "%m-%d-%Y", "%m.%d.%Y"],
        "YYYY-MM-DD" => vec!["%Y-%m-%d", "%Y/%m/%d", "%Y.%m.%d"],
        _ => vec!["%d/%m/%Y", "%d-%m-%Y", "%d.%m.%Y"], // Default DD/MM/YYYY
    };

    let time_fmts = if time_pref == "12h" {
        vec!["%I:%M:%S %p", "%I:%M %p"]
    } else {
        vec!["%H:%M:%S", "%H:%M"]
    };

    for df in &date_fmts {
        for tf in &time_fmts {
            priority_fmts.push(format!("{df} {tf}"));
        }
    }

    // Try strict priority formats first
    for fmt in &priority_fmts {
        if let Ok(naive) = NaiveDateTime::parse_from_str(dt_str, fmt) {
            return naive_to_utc(naive, tz);
        }
    }

    // Fallbacks for common naive formats
    let fallbacks = vec![
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%dT%H:%M:%S",
        "%d/%m/%Y %H:%M",
        "%Y-%m-%d %H:%M",
    ];

    for fmt in fallbacks {
        if let Ok(naive) = NaiveDateTime::parse_from_str(dt_str, fmt) {
            return naive_to_utc(naive, tz);
        }
    }

    Err(format!(
        "Không thể nhận diện ngày giờ: '{}'. Múi giờ: {}, Định dạng yêu cầu: {} ({}).",
        dt_str,
        workspace_tz,
        date_pref,
        if time_pref == "12h" { "12h" } else { "24h" }
    ))
}

fn naive_to_utc(naive: NaiveDateTime, tz: Tz) -> Result<DateTime<Utc>, String> {
    // Delegate to the unified DST-handling implementation in tz.rs
    let date = naive.date();
    let h = naive.hour();
    let m = naive.minute();
    let s = naive.second();
    super::tz::local_to_utc(&tz, date, h, m, s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_with_z() {
        let dt = parse_datetime_with_tz("2026-01-16T10:00:00Z", "Asia/Ho_Chi_Minh", None, None)
            .expect("valid ISO datetime");
        assert_eq!(dt.to_rfc3339(), "2026-01-16T10:00:00+00:00");
    }

    #[test]
    fn test_parse_naive_vn() {
        // 08:00 VN is 01:00 UTC
        let dt = parse_datetime_with_tz(
            "16/01/2026 08:00",
            "Asia/Ho_Chi_Minh",
            Some("DD/MM/YYYY"),
            Some("24h"),
        )
        .expect("valid VN datetime");
        assert_eq!(dt.to_rfc3339(), "2026-01-16T01:00:00+00:00");
    }

    #[test]
    fn test_parse_us_format() {
        // 12/31/2025 11:00 PM (US) -> 2026-01-01 04:00 AM UTC (if NY)
        let dt = parse_datetime_with_tz(
            "12/31/2025 11:00 PM",
            "America/New_York",
            Some("MM/DD/YYYY"),
            Some("12h"),
        )
        .expect("valid US datetime");
        assert_eq!(dt.to_rfc3339(), "2026-01-01T04:00:00+00:00");
    }

    #[test]
    fn test_parse_iso_naive() {
        // 2026-01-16 08:00 (Naive ISO) in VN -> 01:00 UTC
        let dt = parse_datetime_with_tz(
            "2026-01-16 08:00:00",
            "Asia/Ho_Chi_Minh",
            Some("YYYY-MM-DD"),
            Some("24h"),
        )
        .expect("valid ISO naive datetime");
        assert_eq!(dt.to_rfc3339(), "2026-01-16T01:00:00+00:00");
    }

    #[test]
    fn test_parse_invalid_timezone() {
        let res = parse_datetime_with_tz("16/01/2026 08:00", "Invalid/Zone", None, None);
        assert!(res.is_err());
        assert!(res
            .expect_err("should be error")
            .contains("Múi giờ không hợp lệ"));
    }

    #[test]
    fn test_parse_dst_gap() {
        // America/New_York DST starts 2026-03-08 02:00:00
        // 02:30:00 does not exist — unified DST strategy resolves by shifting +1h
        let res = parse_datetime_with_tz(
            "08/03/2026 02:30:00",
            "America/New_York",
            Some("DD/MM/YYYY"),
            Some("24h"),
        );
        // With unified DST handling, gaps are resolved (shifted to 03:30 EDT)
        assert!(res.is_ok(), "DST gap should now be resolved, not rejected");
        let dt = res.expect("should be ok");
        // 03:30 EDT (UTC-4) = 07:30 UTC
        assert_eq!(dt.hour(), 7);
        assert_eq!(dt.minute(), 30);
    }

    #[test]
    fn test_parse_empty() {
        let res = parse_datetime_with_tz("", "UTC", None, None);
        assert!(res.is_err());
    }

    // --- Additional delegation chain tests ---

    #[test]
    fn test_parse_dst_ambiguous_via_delegation() {
        // Fall-back: America/New_York 2026-11-01 01:30:00 exists twice
        // The delegation to tz::local_to_utc should pick the first (EDT, UTC-4)
        let res = parse_datetime_with_tz(
            "01/11/2026 01:30:00",
            "America/New_York",
            Some("DD/MM/YYYY"),
            Some("24h"),
        );
        assert!(
            res.is_ok(),
            "Ambiguous time should be resolved via delegation"
        );
        let dt = res.expect("should be ok");
        // First occurrence: EDT (UTC-4) → 05:30 UTC
        assert_eq!(dt.hour(), 5);
        assert_eq!(dt.minute(), 30);
    }

    #[test]
    fn test_parse_12h_format_am() {
        let res = parse_datetime_with_tz(
            "16/01/2026 08:30 AM",
            "Asia/Ho_Chi_Minh",
            Some("DD/MM/YYYY"),
            Some("12h"),
        );
        assert!(res.is_ok());
        let dt = res.expect("should be ok");
        // 08:30 VN (UTC+7) = 01:30 UTC
        assert_eq!(dt.hour(), 1);
        assert_eq!(dt.minute(), 30);
    }

    #[test]
    fn test_parse_midnight_24h() {
        let res = parse_datetime_with_tz(
            "16/01/2026 00:00:00",
            "Asia/Ho_Chi_Minh",
            Some("DD/MM/YYYY"),
            Some("24h"),
        );
        assert!(res.is_ok());
        let dt = res.expect("should be ok");
        // 00:00 VN (UTC+7) = 17:00 previous day UTC
        assert_eq!(dt.hour(), 17);
    }

    #[test]
    fn test_parse_with_offset_ignores_workspace_tz() {
        // When the string already has an offset, workspace tz should be ignored
        let dt = parse_datetime_with_tz(
            "2026-01-16T10:00:00+05:00",
            "UTC", // This should be ignored
            None,
            None,
        )
        .expect("valid offset datetime");
        // 10:00+05:00 = 05:00 UTC
        assert_eq!(dt.hour(), 5);
    }

    #[test]
    fn test_parse_rfc3339_format() {
        let dt = parse_datetime_with_tz(
            "2026-06-15T14:30:00Z",
            "Asia/Ho_Chi_Minh", // Should be ignored for RFC3339
            None,
            None,
        )
        .expect("valid RFC3339");
        assert_eq!(dt.hour(), 14);
        assert_eq!(dt.minute(), 30);
    }

    #[test]
    fn test_parse_gibberish_returns_error() {
        let res = parse_datetime_with_tz("not-a-date", "UTC", None, None);
        assert!(res.is_err());
    }
}
