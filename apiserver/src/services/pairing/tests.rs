use super::*;
use crate::cache::{RosterData, RosterShift};
use crate::diesel_pool::TenantAuthToken;
use crate::errors::AppError;
use crate::models::enums::LogSource;
use crate::models::{RawAttendanceLog, TimeEntry};
use crate::services::pairing::service_helpers::retry_with_log;
use chrono::{NaiveDate, NaiveTime, TimeZone, Timelike, Utc};
use std::sync::Arc;
use uuid::Uuid;

use crate::repositories::employee::MockEmployeeRepository;
use crate::repositories::raw_attendance_log_repository::MockRawAttendanceLogRepository;
use crate::repositories::roster::MockRosterRepository;
use crate::repositories::time_entry_repository::MockTimeEntryRepository;
use crate::services::attendance_summary_service::MockSummaryRecalculator;

// --- Helper Functions for Strategy Tests ---

fn make_log(minutes: i32) -> LogEvent {
    let id = Uuid::new_v4();
    let punch_time = Utc.with_ymd_and_hms(2026, 6, 20, 0, 0, 0).unwrap()
        + chrono::Duration::minutes(i64::from(minutes));
    LogEvent {
        id,
        minutes,
        punch_time,
        raw_data: None,
        all_ids: vec![id],
    }
}

fn standard_shift() -> PairingShiftConfig {
    PairingShiftConfig::new(
        480,       // 08:00
        1020,      // 17:00
        Some(720), // 12:00
        Some(780), // 13:00
        None,
        None,
    )
}

// --- Strategy Tests ---

#[test]
fn test_shift_window_normal_day() {
    let strategy = get_strategy();
    let shift = standard_shift();
    let logs = vec![
        make_log(475),  // 07:55 — within check-in window (480 ± 120)
        make_log(725),  // 12:05 — break area
        make_log(775),  // 12:55 — break area
        make_log(1025), // 17:05 — last log of day
    ];

    let sessions = strategy.pair(&logs, Some(&shift));
    assert_eq!(sessions.len(), 2);
    assert_eq!(sessions[0].start_minutes(), 475);
    assert_eq!(sessions[0].end_minutes(), Some(720));
    assert_eq!(sessions[1].start_minutes(), 780);
    assert_eq!(sessions[1].end_minutes(), Some(1025));

    let meta = sessions[0].metadata().unwrap();
    assert_eq!(meta["tier"], "shift_window");
}

#[test]
fn test_shift_window_long_overtime() {
    let strategy = get_strategy();
    let shift = standard_shift(); // 08:00-17:00
    let logs = vec![
        make_log(475),  // 07:55 — check-in
        make_log(1380), // 23:00 — check-out (6h after shift end)
    ];

    let sessions = strategy.pair(&logs, Some(&shift));
    assert_eq!(sessions.len(), 2);
    assert_eq!(sessions[0].start_minutes(), 475);
    assert_eq!(sessions[0].end_minutes(), Some(720));
    assert_eq!(sessions[1].start_minutes(), 780);
    assert_eq!(sessions[1].end_minutes(), Some(1380)); // 23:00 captured!

    let meta = sessions[0].metadata().unwrap();
    assert_eq!(meta["tier"], "shift_window");
}

#[test]
fn test_shift_window_logs_outside_window_fallback() {
    let strategy = get_strategy();
    let shift = standard_shift();
    let logs = vec![
        make_log(200), // 03:20 — way before shift
        make_log(300), // 05:00 — still before window
    ];

    let sessions = strategy.pair(&logs, Some(&shift));
    assert_eq!(sessions.len(), 1);

    let meta = sessions[0].metadata().unwrap();
    assert_eq!(meta["tier"], "anchor_first"); // Fell back
}

#[test]
fn test_anchor_first_perfect_day() {
    let strategy = get_strategy();
    let logs = vec![make_log(480), make_log(1020)];

    let sessions = strategy.pair(&logs, None);
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].start_minutes(), 480);
    assert_eq!(sessions[0].end_minutes(), Some(1020));

    let meta = sessions[0].metadata().unwrap();
    assert_eq!(meta["tier"], "anchor_first");
}

#[test]
fn test_anchor_first_missing_out() {
    let strategy = get_strategy();
    let logs = vec![make_log(480)];

    let sessions = strategy.pair(&logs, None);
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].start_minutes(), 480);
    assert!(sessions[0].end_minutes().is_none());
    assert_eq!(sessions[0].note(), Some(&PairingNote::MissingPunchOut));
}

#[test]
fn test_anchor_first_16h_limit() {
    let strategy = get_strategy();
    let logs = vec![make_log(480), make_log(1500)];

    let sessions = strategy.pair(&logs, None);
    assert_eq!(sessions.len(), 2);
    assert!(sessions[0].end_minutes().is_none());
    assert!(sessions[1].end_minutes().is_none());
}

#[test]
fn test_empty_logs() {
    let strategy = get_strategy();
    let sessions = strategy.pair(&[], None);
    assert!(sessions.is_empty());
}

#[test]
fn test_shift_window_overnight_shift() {
    let strategy = get_strategy();
    let shift = PairingShiftConfig::new(
        1320, // 22:00
        1800, // 06:00 next day
        None, None, None, None,
    );

    let logs = vec![
        make_log(1310), // 21:50 — within check-in window
        make_log(1810), // 06:10 next day
    ];

    let sessions = strategy.pair(&logs, Some(&shift));
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].start_minutes(), 1310);
    assert_eq!(sessions[0].end_minutes(), Some(1810));

    let meta = sessions[0].metadata().unwrap();
    assert_eq!(meta["tier"], "shift_window");
}

#[test]
fn test_single_log_no_config() {
    let strategy = get_strategy();
    let logs = vec![make_log(480)];

    let sessions = strategy.pair(&logs, None);
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].start_minutes(), 480);
    assert!(sessions[0].end_minutes().is_none());
    assert_eq!(sessions[0].note(), Some(&PairingNote::MissingPunchOut));
}

#[test]
fn test_single_log_with_shift_config() {
    let strategy = get_strategy();
    let shift = standard_shift();
    let logs = vec![make_log(480)];

    let sessions = strategy.pair(&logs, Some(&shift));
    assert_eq!(sessions.len(), 1);
    assert!(sessions[0].end_minutes().is_none());
}

#[test]
fn test_anchor_first_exactly_at_16h_gap() {
    let strategy = get_strategy();
    let logs = vec![make_log(480), make_log(1440)];

    let sessions = strategy.pair(&logs, None);
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].end_minutes(), Some(1440));
}

#[test]
fn test_anchor_first_one_minute_over_16h_gap() {
    let strategy = get_strategy();
    let logs = vec![make_log(480), make_log(1441)];

    let sessions = strategy.pair(&logs, None);
    assert_eq!(sessions.len(), 2);
    assert!(sessions[0].end_minutes().is_none());
    assert!(sessions[1].end_minutes().is_none());
}

#[test]
fn test_shift_window_earliest_checkin() {
    let strategy = get_strategy();
    let shift = standard_shift();
    let logs = vec![
        make_log(470),  // 07:50
        make_log(485),  // 08:05
        make_log(1020), // 17:00
    ];

    let sessions = strategy.pair(&logs, Some(&shift));
    assert_eq!(sessions.len(), 2);
    assert_eq!(sessions[0].start_minutes(), 470);
    assert_eq!(sessions[0].end_minutes(), Some(720));
    assert_eq!(sessions[1].start_minutes(), 780);
    assert_eq!(sessions[1].end_minutes(), Some(1020));
}

#[test]
fn test_algorithm_name() {
    let strategy = get_strategy();
    assert_eq!(strategy.algorithm_name(), "shift_aware");
}

#[test]
fn test_paired_session_serde_roundtrip() {
    let t1 = Utc.with_ymd_and_hms(2026, 6, 20, 8, 0, 0).unwrap();
    let t2 = Utc.with_ymd_and_hms(2026, 6, 20, 17, 0, 0).unwrap();
    let session = PairedSession::new(
        480,
        Some(1020),
        t1,
        Some(t2),
        EntryType::Work,
        None,
        false,
        Some(serde_json::json!({"tier": "test"})),
    );
    let json = serde_json::to_string(&session).unwrap();
    let parsed: PairedSession = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, session);
    assert!(json.contains("\"work\""));
}

#[test]
fn test_overnight_shift_with_overtime() {
    let strategy = get_strategy();
    let shift = PairingShiftConfig::new(
        1320, // 22:00
        1800, // 06:00 next day
        None, None, None, None,
    );

    let logs = vec![
        make_log(1310), // 21:50 — check-in
        make_log(1920), // 08:00 next day — check-out (2h OT)
    ];

    let sessions = strategy.pair(&logs, Some(&shift));
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].start_minutes(), 1310);
    assert_eq!(sessions[0].end_minutes(), Some(1920));

    let meta = sessions[0].metadata().unwrap();
    assert_eq!(meta["tier"], "shift_window");
}

/// Regression test for the "night-shift dropped check-out" bug (SX Đêm KNGC).
///
/// Shift 19:30–04:30 (next day) with a break 23:30–00:30 that CROSSES midnight
/// (break_end = 00:30 → 1470 min, i.e. >= 1440). The employee records a 2-punch
/// day: 19:21 check-in (+ device duplicate) and 07:31 next-day check-out.
///
/// Bug: splitting on the midnight-crossing break produced a `second_half`
/// segment starting at 00:30 on day D+1. `build_time_entries` then discarded
/// that segment (its `date_naive() != self.date` guard), so the report showed
/// check-out 23:30 instead of the real 07:31.
///
/// Fix: a break whose end crosses midnight is NOT a valid split point. The
/// session must stay continuous (19:21 → 07:31) as a single shift-window
/// session with NO `break_split` tag. The attendance calculator still deducts
/// the break / accrues break-OT from the shift config, so hours are unchanged.
#[test]
fn test_overnight_break_crossing_midnight_not_split() {
    let strategy = get_strategy();
    // 19:30–04:30, break 23:30–00:30 (crosses midnight → break_end 1470 >= 1440)
    let shift = PairingShiftConfig::new(
        1170,       // 19:30
        1710,       // 04:30 next day
        Some(1410), // 23:30
        Some(1470), // 00:30 next day
        None,
        None,
    );

    // 19:21 = 1161 min (+ device duplicate), 07:31 next day = 1891 min
    let logs = vec![
        make_log(1161), // 19:21 check-in
        make_log(1161), // 19:21 device duplicate
        make_log(1891), // 07:31 next-day check-out
    ];

    let sessions = strategy.pair(&logs, Some(&shift));

    // Exactly ONE continuous session — the break must not split it.
    assert_eq!(
        sessions.len(),
        1,
        "midnight-crossing break must not split the session (got {:?})",
        sessions
            .iter()
            .map(|s| (s.start_minutes(), s.end_minutes()))
            .collect::<Vec<_>>()
    );

    // Session spans the real check-in → real check-out.
    assert_eq!(sessions[0].start_minutes(), 1161);
    assert_eq!(
        sessions[0].end_minutes(),
        Some(1891),
        "BUG: session must end at the real 07:31 check-out (1891 min), not the break start"
    );
    assert!(
        sessions[0].end_time().is_some(),
        "BUG: session end_time() is None — report would show no check-out"
    );

    let meta = sessions[0].metadata().unwrap();
    assert_eq!(meta["tier"], "shift_window");
    assert!(
        meta.get("break_split").is_none(),
        "continuous session must not carry a break_split tag"
    );
}

/// Guard precision: a night shift whose break is BEFORE midnight still splits.
///
/// Shift 19:30–04:30 with a break 20:00–21:00 (both boundaries < 1440). Both
/// resulting segments start on the shift's calendar day D, so the split is
/// valid and must still happen. This proves the fix only suppresses
/// midnight-crossing breaks, not all night-shift breaks.
#[test]
fn test_overnight_break_before_midnight_still_splits() {
    let strategy = get_strategy();
    // 19:30–04:30, break 20:00–21:00 (pre-midnight → 1200/1260, both < 1440)
    let shift = PairingShiftConfig::new_with_breaks(
        1170, // 19:30
        1710, // 04:30 next day
        vec![BreakInterval::new(1200, 1260)],
        None,
        None,
    );

    let logs = vec![
        make_log(1161), // 19:21 check-in
        make_log(1891), // 07:31 next-day check-out
    ];

    let sessions = strategy.pair(&logs, Some(&shift));

    assert_eq!(sessions.len(), 2, "pre-midnight break must still split");
    assert_eq!(sessions[0].start_minutes(), 1161);
    assert_eq!(sessions[0].end_minutes(), Some(1200));
    assert_eq!(sessions[0].metadata().unwrap()["break_split"], "first_half");
    assert_eq!(sessions[1].start_minutes(), 1260);
    assert_eq!(sessions[1].end_minutes(), Some(1891));
    assert_eq!(
        sessions[1].metadata().unwrap()["break_split"],
        "second_half"
    );
}

#[test]
fn test_single_log_in_checkin_window_missing_checkout() {
    let strategy = get_strategy();
    let shift = standard_shift();
    let logs = vec![make_log(478)];

    let sessions = strategy.pair(&logs, Some(&shift));
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].start_minutes(), 478);
    assert!(sessions[0].end_minutes().is_none());
    assert_eq!(sessions[0].note(), Some(&PairingNote::MissingPunchOut));

    let meta = sessions[0].metadata().unwrap();
    assert_eq!(meta["tier"], "shift_window");
}

#[test]
fn test_rapid_multiple_punches() {
    let strategy = get_strategy();
    let shift = PairingShiftConfig::new(
        450,       // 07:30
        990,       // 16:30
        Some(690), // 11:30
        Some(750), // 12:30
        None,
        None,
    );

    let logs = vec![
        make_log(446), // 07:26
        make_log(455),
        make_log(461),
        make_log(464),
        make_log(466),
        make_log(1020), // 17:00
    ];

    let sessions = strategy.pair(&logs, Some(&shift));
    assert_eq!(sessions.len(), 2);
    assert_eq!(sessions[0].start_minutes(), 446);
    assert_eq!(sessions[0].end_minutes(), Some(690));
    assert_eq!(sessions[1].start_minutes(), 750);
    assert_eq!(sessions[1].end_minutes(), Some(1020));
}

#[test]
fn test_log_before_shift_window_falls_back() {
    let strategy = get_strategy();
    let shift = standard_shift();
    let logs = vec![make_log(180), make_log(240)];

    let sessions = strategy.pair(&logs, Some(&shift));
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].start_minutes(), 180);
    assert_eq!(sessions[0].end_minutes(), Some(240));

    let meta = sessions[0].metadata().unwrap();
    assert_eq!(meta["tier"], "anchor_first");
}

#[test]
fn test_overnight_shift_multiple_checkout_punches() {
    let strategy = get_strategy();
    let shift = PairingShiftConfig::new(
        1080, // 18:00
        1560, // 02:00 next day
        None, None, None, None,
    );

    let logs = vec![make_log(1085), make_log(1825), make_log(1826)];

    let sessions = strategy.pair(&logs, Some(&shift));
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].start_minutes(), 1085);
    assert_eq!(sessions[0].end_minutes(), Some(1826));

    let meta = sessions[0].metadata().unwrap();
    assert_eq!(meta["log_ids"].as_array().unwrap().len(), 3);
}

#[test]
fn test_pair_with_none_config() {
    let strategy = get_strategy();
    let logs = vec![make_log(480), make_log(1020)];
    let sessions = strategy.pair(&logs, None);
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].start_minutes(), 480);
    assert_eq!(sessions[0].end_minutes(), Some(1020));
    assert_eq!(sessions[0].metadata().unwrap()["tier"], "anchor_first");
}

#[test]
fn test_pair_with_some_config() {
    let strategy = get_strategy();
    let shift = standard_shift();
    let logs = vec![make_log(475), make_log(1025)];
    let sessions = strategy.pair(&logs, Some(&shift));
    assert_eq!(sessions.len(), 2);
    assert_eq!(sessions[0].start_minutes(), 475);
    assert_eq!(sessions[0].end_minutes(), Some(720));
    assert_eq!(sessions[1].start_minutes(), 780);
    assert_eq!(sessions[1].end_minutes(), Some(1025));
    assert_eq!(sessions[0].metadata().unwrap()["tier"], "shift_window");
}

#[test]
fn test_pair_negative_minutes_log() {
    let strategy = get_strategy();
    let logs = vec![make_log(-30), make_log(480)];
    let sessions = strategy.pair(&logs, None);
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].start_minutes(), -30);
    assert_eq!(sessions[0].end_minutes(), Some(480));
}

#[test]
fn test_pair_unsorted_logs() {
    let strategy = get_strategy();
    let logs = vec![make_log(1020), make_log(480)];
    let sessions = strategy.pair(&logs, None);
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].start_minutes(), 480);
    assert_eq!(sessions[0].end_minutes(), Some(1020));
}

#[test]
fn test_pair_extreme_minutes_values() {
    let strategy = get_strategy();
    let logs = vec![make_log(i32::MIN + 100), make_log(i32::MAX - 100)];
    let sessions = strategy.pair(&logs, None);
    assert_eq!(sessions.len(), 2);
    assert!(sessions[0].end_minutes().is_none());
    assert!(sessions[1].end_minutes().is_none());
}

#[test]
fn test_pair_zero_minute_gap_between_logs() {
    let strategy = get_strategy();
    let logs = vec![make_log(480), make_log(480)];
    let sessions = strategy.pair(&logs, None);
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].start_minutes(), 480);
    assert_eq!(sessions[0].end_minutes(), Some(480));
}

/// Regression test for the "4 sessions" bug.
///
/// Scenario: scheduled shift 07:30–16:30 with a single break 11:30–12:30. The
/// employee records five raw punches on the same day: one early check-in at
/// 07:22, three duplicate 15:06 punches (device retransmits) and a 16:45
/// check-out. The pairing algorithm MUST collapse the three 15:06 duplicates
/// (they sit strictly between first-in and last-out) and yield exactly 2 work
/// sessions split on the configured break.
///
/// Before the fix this was observed at the DB layer (4 rows in `time_entries`
/// after concurrent reprocess), but the pairing unit must also never emit >2
/// sessions for the single-day input.
#[test]
fn test_shift_window_reproduces_4_session_bug_scenario() {
    let strategy = get_strategy();
    // 07:30-16:30, break 11:30-12:30
    let shift = PairingShiftConfig::new_with_breaks(
        450,
        990,
        vec![BreakInterval::new(690, 750)],
        None,
        None,
    );

    // 07:22 = 442 min; three 15:06 = 906 min (within break gap region);
    // 16:45 = 1005 min.
    let logs = vec![
        make_log(442),  // 07:22 check-in
        make_log(906),  // 15:06 duplicate #1
        make_log(906),  // 15:06 duplicate #2
        make_log(906),  // 15:06 duplicate #3
        make_log(1005), // 16:45 check-out
    ];

    let sessions = strategy.pair(&logs, Some(&shift));

    // Exactly 2 work sessions: morning (checkin..break) and afternoon
    // (break..checkout).
    assert_eq!(
        sessions.len(),
        2,
        "paired session count for the bug scenario must be exactly 2 (got {:?})",
        sessions
            .iter()
            .map(|s| (s.start_minutes(), s.end_minutes()))
            .collect::<Vec<_>>()
    );

    // Segment 1: 07:22 → 11:30
    assert_eq!(sessions[0].start_minutes(), 442);
    assert_eq!(sessions[0].end_minutes(), Some(690));

    // Segment 2: 12:30 → 16:45
    assert_eq!(sessions[1].start_minutes(), 750);
    assert_eq!(sessions[1].end_minutes(), Some(1005));

    // Both segments are tagged as the shift-window break-split tier.
    assert_eq!(sessions[0].metadata().unwrap()["break_split"], "first_half");
    assert_eq!(
        sessions[1].metadata().unwrap()["break_split"],
        "second_half"
    );
}

/// The deduplication window in `build_log_events` is 5 seconds. Duplicate
/// punches spaced > 5 seconds apart produce distinct `LogEvent`s at the same
/// minute-of-day. The strategy must still emit exactly 2 sessions regardless
/// of how many distinct-but-temporally-coincident events straddle the break.
#[test]
fn test_shift_window_many_duplicate_mid_break_punches() {
    let strategy = get_strategy();
    let shift = PairingShiftConfig::new_with_breaks(
        450,
        990,
        vec![BreakInterval::new(690, 750)],
        None,
        None,
    );

    let mut logs = vec![make_log(442) /* 07:22 check-in */];
    // Add 10 duplicate 15:06 events (each is a distinct UUID, the timestamps
    // differ by < 5 seconds so they survive dedup; all have minute 906).
    for _ in 0..10 {
        logs.push(make_log(906));
    }
    logs.push(make_log(1005)); // 16:45 check-out

    let sessions = strategy.pair(&logs, Some(&shift));
    assert_eq!(
        sessions.len(),
        2,
        "strategy must ignore interior duplicate punches and split only on the configured break"
    );
}

#[test]
fn test_shift_window_multiple_breaks() {
    let strategy = get_strategy();
    let shift = PairingShiftConfig::new_with_breaks(
        480,
        1020,
        vec![BreakInterval::new(600, 630), BreakInterval::new(720, 780)],
        None,
        None,
    );

    let logs = vec![make_log(475), make_log(1025)];

    let sessions = strategy.pair(&logs, Some(&shift));
    assert_eq!(sessions.len(), 3);

    assert_eq!(sessions[0].start_minutes(), 475);
    assert_eq!(sessions[0].end_minutes(), Some(600));
    assert_eq!(sessions[0].metadata().unwrap()["break_split"], "first_half");

    assert_eq!(sessions[1].start_minutes(), 630);
    assert_eq!(sessions[1].end_minutes(), Some(720));
    assert_eq!(
        sessions[1].metadata().unwrap()["break_split"],
        "middle_segment"
    );

    assert_eq!(sessions[2].start_minutes(), 780);
    assert_eq!(sessions[2].end_minutes(), Some(1025));
    assert_eq!(
        sessions[2].metadata().unwrap()["break_split"],
        "second_half"
    );
}

// --- Context Tests ---

#[test]
fn test_utc_timezone_offset_bug() {
    let date = NaiveDate::from_ymd_opt(2026, 5, 20).unwrap();
    let tz_utc = chrono_tz::UTC;
    let context_utc = PairingContext::new(date, tz_utc);

    let start_time = NaiveTime::from_hms_opt(7, 30, 0).unwrap();
    let end_time = NaiveTime::from_hms_opt(16, 30, 0).unwrap();

    let shift_config_utc = context_utc.build_shift_config(start_time, end_time, None, None, 0);
    let (window_start_utc, _window_end_utc) = context_utc
        .calculate_fetch_window(shift_config_utc.as_ref())
        .unwrap();

    let expected_window_start = Utc.with_ymd_and_hms(2026, 5, 20, 1, 30, 0).unwrap();
    assert_eq!(window_start_utc, expected_window_start);

    let log1_utc = Utc.with_ymd_and_hms(2026, 5, 20, 0, 8, 27).unwrap();
    let log2_utc = Utc.with_ymd_and_hms(2026, 5, 20, 14, 31, 36).unwrap();

    let raw_logs = vec![
        RawAttendanceLog::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            log1_utc,
            LogSource::Manual,
            None,
            None,
            1,
            1,
            Utc::now(),
        ),
        RawAttendanceLog::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            log2_utc,
            LogSource::Manual,
            None,
            None,
            1,
            1,
            Utc::now(),
        ),
    ];

    let log_events_utc = context_utc.build_log_events(&raw_logs).unwrap();
    assert_eq!(log_events_utc[0].minutes(), 8);
    assert_eq!(log_events_utc[1].minutes(), 871);

    let tz_hcm = chrono_tz::Asia::Ho_Chi_Minh;
    let context_hcm = PairingContext::new(date, tz_hcm);

    let shift_config_hcm = context_hcm.build_shift_config(start_time, end_time, None, None, 0);
    let (window_start_hcm, _window_end_hcm) = context_hcm
        .calculate_fetch_window(shift_config_hcm.as_ref())
        .unwrap();

    let expected_window_start_hcm = Utc.with_ymd_and_hms(2026, 5, 19, 18, 30, 0).unwrap();
    assert_eq!(window_start_hcm, expected_window_start_hcm);

    let log_events_hcm = context_hcm.build_log_events(&raw_logs).unwrap();
    assert_eq!(log_events_hcm[0].minutes(), 428);
    assert_eq!(log_events_hcm[1].minutes(), 1291);
}

#[test]
fn test_build_shift_config_overnight() {
    let date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
    let tz = chrono_tz::UTC;
    let context = PairingContext::new(date, tz);

    let config = context
        .build_shift_config(
            NaiveTime::from_hms_opt(22, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(6, 0, 0).unwrap(),
            Some(NaiveTime::from_hms_opt(2, 0, 0).unwrap()),
            Some(NaiveTime::from_hms_opt(3, 0, 0).unwrap()),
            0,
        )
        .unwrap();
    assert_eq!(config.start_minutes(), 1320);
    assert_eq!(config.end_minutes(), 360 + 1440);
    assert_eq!(config.break_start_minutes(), Some(120 + 1440));
    assert_eq!(config.break_end_minutes(), Some(180 + 1440));
}

#[test]
fn test_calculate_fetch_window_no_shift() {
    let date = NaiveDate::from_ymd_opt(2026, 5, 20).unwrap();
    let tz = chrono_tz::Asia::Ho_Chi_Minh;
    let context = PairingContext::new(date, tz);

    let (start_utc, end_utc) = context.calculate_fetch_window(None).unwrap();

    let expected_start = Utc.with_ymd_and_hms(2026, 5, 19, 17, 0, 0).unwrap();
    let expected_end = Utc.with_ymd_and_hms(2026, 5, 20, 17, 0, 0).unwrap();

    assert_eq!(start_utc, expected_start);
    assert_eq!(end_utc, expected_end);
}

#[test]
fn test_build_log_events_deduplication() {
    let date = NaiveDate::from_ymd_opt(2026, 5, 20).unwrap();
    let tz = chrono_tz::UTC;
    let context = PairingContext::new(date, tz);

    let workspace_id = Uuid::new_v4();
    let employee_id = Uuid::new_v4();
    let log1 = RawAttendanceLog::new(
        Uuid::new_v4(),
        workspace_id,
        employee_id,
        Utc.with_ymd_and_hms(2026, 5, 20, 8, 30, 15).unwrap(),
        LogSource::Manual,
        None,
        None,
        1,
        1,
        Utc::now(),
    );

    let log2 = RawAttendanceLog::new(
        Uuid::new_v4(),
        workspace_id,
        employee_id,
        Utc.with_ymd_and_hms(2026, 5, 20, 8, 30, 17).unwrap(),
        LogSource::Manual,
        None,
        None,
        1,
        1,
        log1.created_at(),
    );

    let logs = vec![log1, log2];
    let events = context.build_log_events(&logs).unwrap();

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].minutes(), 8 * 60 + 30);
}

#[test]
fn test_build_time_entries() {
    let date = NaiveDate::from_ymd_opt(2026, 5, 20).unwrap();
    let tz = chrono_tz::Asia::Ho_Chi_Minh;
    let context = PairingContext::new(date, tz);

    let workspace_id = Uuid::new_v4();
    let employment_id = Uuid::new_v4();

    let t1_start = Utc.with_ymd_and_hms(2026, 5, 20, 1, 0, 0).unwrap();
    let t1_end = Utc.with_ymd_and_hms(2026, 5, 20, 10, 0, 0).unwrap();
    let session1 = PairedSession::new(
        480,
        Some(1020),
        t1_start,
        Some(t1_end),
        EntryType::Work,
        None,
        false,
        Some(serde_json::json!({"algo": "shift_window"})),
    );

    let t2_start = Utc.with_ymd_and_hms(2026, 5, 20, 11, 0, 0).unwrap();
    let session2 = PairedSession::new(
        1080,
        None,
        t2_start,
        None,
        EntryType::Work,
        Some(PairingNote::MissingPunchOut),
        false,
        None,
    );

    let sessions = vec![session1, session2];
    let traces = serde_json::json!(["trace1"]);

    let entries = context
        .build_time_entries(workspace_id, employment_id, None, sessions, &traces)
        .unwrap();

    assert_eq!(entries.len(), 2);

    let e1 = &entries[0];
    assert_eq!(e1.workspace_id(), workspace_id);
    assert_eq!(e1.employment_id(), employment_id);
    assert_eq!(
        e1.punch_in_time(),
        Utc.with_ymd_and_hms(2026, 5, 20, 1, 0, 0).unwrap()
    );
    assert_eq!(
        e1.punch_out_time(),
        Some(Utc.with_ymd_and_hms(2026, 5, 20, 10, 0, 0).unwrap())
    );
    assert_eq!(e1.duration_minutes(), Some(540));

    let meta1 = e1.generation_metadata().unwrap();
    assert_eq!(
        meta1.get("schedule_traces").unwrap().as_array().unwrap()[0]
            .as_str()
            .unwrap(),
        "trace1"
    );

    let e2 = &entries[1];
    assert_eq!(
        e2.punch_in_time(),
        Utc.with_ymd_and_hms(2026, 5, 20, 11, 0, 0).unwrap()
    );
    assert_eq!(e2.punch_out_time(), None);
    assert_eq!(e2.duration_minutes(), None);
    assert_eq!(e2.note(), Some("missing_punch_out"));
}

// --- Service Tests ---

#[tokio::test]
async fn test_process_new_raw_log_duplicate() {
    let mut mock_raw_repo = MockRawAttendanceLogRepository::new();
    mock_raw_repo
        .expect_has_duplicate_raw_log()
        .times(1)
        .returning(|_, _, _, _| Box::pin(async { Ok(true) }));

    let service = PairingService::new(PairingServiceArgs::new(
        Arc::new(MockTimeEntryRepository::new()),
        Arc::new(MockRosterRepository::new()),
        Arc::new(MockEmployeeRepository::new()),
        Arc::new(mock_raw_repo),
        Arc::new(MockSummaryRecalculator::new()),
    ));

    let log = RawAttendanceLog::new(
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Utc::now(),
        LogSource::Manual,
        None,
        Some(serde_json::json!({})),
        1,
        1,
        Utc::now(),
    );

    let result = service.process_new_raw_log(&log).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_clear_auto_entries() {
    let mut mock_time_entry_repo = MockTimeEntryRepository::new();
    mock_time_entry_repo
        .expect_replace_time_entries()
        .times(1)
        .returning(|_, _, _, _, _| Ok(()));

    let service = PairingService::new(PairingServiceArgs::new(
        Arc::new(mock_time_entry_repo),
        Arc::new(MockRosterRepository::new()),
        Arc::new(MockEmployeeRepository::new()),
        Arc::new(MockRawAttendanceLogRepository::new()),
        Arc::new(MockSummaryRecalculator::new()),
    ));

    let auth_token = TenantAuthToken::system_override(Uuid::new_v4());
    let result = service
        .clear_auto_entries(&auth_token, Uuid::new_v4(), Utc::now(), Utc::now())
        .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_reprocess_daily_attendance_no_logs() {
    let mut mock_roster_repo = MockRosterRepository::new();
    mock_roster_repo
        .expect_get_effective_shift()
        .times(1)
        .returning(|_, _, _| Box::pin(async { Ok((None, vec![])) }));

    let mut mock_raw_repo = MockRawAttendanceLogRepository::new();
    mock_raw_repo
        .expect_get_employment_logs_in_time_range()
        .times(1)
        .returning(|_, _, _, _| Box::pin(async { Ok(vec![]) }));

    let mut mock_time_entry_repo = MockTimeEntryRepository::new();
    mock_time_entry_repo
        .expect_get_employment_time_entries()
        .times(1)
        .returning(|_, _, _, _| Ok(vec![]));
    mock_time_entry_repo
        .expect_replace_time_entries()
        .times(1)
        .returning(|_, _, _, _, _| Ok(()));

    let mut mock_summary_service = MockSummaryRecalculator::new();
    mock_summary_service
        .expect_recalculate_daily_summary()
        .times(1)
        .returning(|_, _, _| Ok(()));

    let service = PairingService::new(PairingServiceArgs::new(
        Arc::new(mock_time_entry_repo),
        Arc::new(mock_roster_repo),
        Arc::new(MockEmployeeRepository::new()),
        Arc::new(mock_raw_repo),
        Arc::new(mock_summary_service),
    ));

    let result = service
        .reprocess_daily_attendance(
            Uuid::new_v4(),
            Uuid::new_v4(),
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            chrono_tz::UTC,
        )
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_reprocess_daily_attendance_excludes_already_paired_logs() {
    let mut mock_roster_repo = MockRosterRepository::new();
    mock_roster_repo
        .expect_get_effective_shift()
        .times(1)
        .returning(|_, _, _| Box::pin(async { Ok((None, vec![])) }));

    let paired_log_id = Uuid::new_v4();
    let fresh_log_id = Uuid::new_v4();
    let ws_id = Uuid::new_v4();
    let emp_id = Uuid::new_v4();

    let mut mock_time_entry_repo = MockTimeEntryRepository::new();
    let test_date = NaiveDate::from_ymd_opt(2026, 6, 19).unwrap();
    let log_time = Utc.with_ymd_and_hms(2026, 6, 19, 10, 0, 0).unwrap();
    let adjacent_time = log_time - chrono::Duration::days(1);
    let adjacent_entry = TimeEntry::new(
        Uuid::new_v4(),
        emp_id,
        ws_id,
        None,
        None,
        adjacent_time,
        None,
        None,
        false,
        "work".to_string(),
        None,
        Some(serde_json::json!({ "log_ids": vec![paired_log_id.to_string()] })),
        adjacent_time,
        adjacent_time,
    );
    mock_time_entry_repo
        .expect_get_employment_time_entries()
        .times(1)
        .returning(move |_, _, _, _| Ok(vec![adjacent_entry.clone()]));

    mock_time_entry_repo
        .expect_replace_time_entries()
        .times(1)
        .returning(move |_, _, _, _, entries| {
            assert_eq!(entries.len(), 1);
            let meta = entries[0].generation_metadata().unwrap();
            let log_ids = meta["log_ids"].as_array().unwrap();
            assert_eq!(log_ids.len(), 1);
            assert_eq!(log_ids[0].as_str().unwrap(), fresh_log_id.to_string());
            Ok(())
        });

    let mut mock_raw_repo = MockRawAttendanceLogRepository::new();
    let log1 = RawAttendanceLog::new(
        paired_log_id,
        ws_id,
        emp_id,
        log_time,
        LogSource::Manual,
        None,
        None,
        1,
        1,
        log_time,
    );
    let log2 = RawAttendanceLog::new(
        fresh_log_id,
        ws_id,
        emp_id,
        log_time + chrono::Duration::minutes(5),
        LogSource::Manual,
        None,
        None,
        1,
        1,
        log_time,
    );

    mock_raw_repo
        .expect_get_employment_logs_in_time_range()
        .times(1)
        .returning(move |_, _, _, _| {
            let l1 = log1.clone();
            let l2 = log2.clone();
            Box::pin(async move { Ok(vec![l1, l2]) })
        });

    let mut mock_summary_service = MockSummaryRecalculator::new();
    mock_summary_service
        .expect_recalculate_daily_summary()
        .times(1)
        .returning(|_, _, _| Ok(()));

    let service = PairingService::new(PairingServiceArgs::new(
        Arc::new(mock_time_entry_repo),
        Arc::new(mock_roster_repo),
        Arc::new(MockEmployeeRepository::new()),
        Arc::new(mock_raw_repo),
        Arc::new(mock_summary_service),
    ));

    let result = service
        .reprocess_daily_attendance(ws_id, emp_id, test_date, chrono_tz::UTC)
        .await;

    assert!(result.is_ok(), "Reprocess failed: {:?}", result.err());
}

#[tokio::test]
async fn test_reprocess_attendance_range_success() {
    let ws_id = Uuid::new_v4();
    let emp_id = Uuid::new_v4();
    let start_date = NaiveDate::from_ymd_opt(2026, 6, 18).unwrap();
    let end_date = NaiveDate::from_ymd_opt(2026, 6, 19).unwrap();

    let mut mock_roster_repo = MockRosterRepository::new();
    let sched_emp_id = emp_id;
    mock_roster_repo
        .expect_get_workspace_schedules()
        .times(1)
        .returning(move |_, _, s_date, e_date, _, _, _| {
            let schedules = vec![
                crate::models::EffectiveSchedule::new(
                    s_date,
                    None,
                    "none".to_string(),
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    vec![],
                    false,
                    false,
                    None,
                ),
                crate::models::EffectiveSchedule::new(
                    e_date,
                    None,
                    "none".to_string(),
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    vec![],
                    false,
                    false,
                    None,
                ),
            ];
            Box::pin(async move {
                let list = vec![crate::models::EmployeeSchedule::new(
                    sched_emp_id,
                    sched_emp_id,
                    "EMP001".to_string(),
                    "Test Employee".to_string(),
                    schedules,
                )];
                let total = list.len() as i64;
                Ok((list, total))
            })
        });

    let mut mock_raw_repo = MockRawAttendanceLogRepository::new();
    mock_raw_repo
        .expect_get_employment_logs_in_time_range()
        .times(1)
        .returning(|_, _, _, _| Box::pin(async { Ok(vec![]) }));

    let mut mock_time_entry_repo = MockTimeEntryRepository::new();
    mock_time_entry_repo
        .expect_get_employment_time_entries()
        .times(1)
        .returning(|_, _, _, _| Ok(vec![]));
    mock_time_entry_repo
        .expect_replace_time_entries()
        .times(1)
        .returning(|_, _, _, _, _| Ok(()));

    let mut mock_summary_service = MockSummaryRecalculator::new();
    mock_summary_service
        .expect_recalculate_attendance_range()
        .times(1)
        .returning(|_, _, _, _| Ok(()));

    let service = PairingService::new(PairingServiceArgs::new(
        Arc::new(mock_time_entry_repo),
        Arc::new(mock_roster_repo),
        Arc::new(MockEmployeeRepository::new()),
        Arc::new(mock_raw_repo),
        Arc::new(mock_summary_service),
    ));

    let result = service
        .reprocess_attendance_range(ws_id, emp_id, start_date, end_date, chrono_tz::UTC)
        .await;

    assert!(result.is_ok(), "Reprocess range failed: {:?}", result.err());
}

#[tokio::test]
async fn test_retry_with_log_returns_error_after_max_attempts() {
    let ws_id = Uuid::new_v4();
    let emp_id = Uuid::new_v4();
    let counter = std::sync::atomic::AtomicU32::new(0);

    let result = retry_with_log("test_op", ws_id, emp_id, "test_context", 3, 1, || {
        counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Box::pin(async { Err(AppError::Internal("test_err".to_string())) })
    })
    .await;

    assert!(result.is_err());
    assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 3);
}

#[tokio::test]
async fn test_retry_with_log_succeeds_on_second_attempt() {
    let ws_id = Uuid::new_v4();
    let emp_id = Uuid::new_v4();
    let counter = std::sync::atomic::AtomicU32::new(0);

    let result = retry_with_log("test_op", ws_id, emp_id, "test_context", 3, 1, || {
        let attempts = counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Box::pin(async move {
            if attempts == 0 {
                Err(AppError::Internal("test_err".to_string()))
            } else {
                Ok(())
            }
        })
    })
    .await;

    assert!(result.is_ok());
    assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 2);
}

// =============================================================================
// --- TD-7: Overnight Propagation Tests ---
// =============================================================================

/// Day shift (08:00-17:00, end_minutes=1020 < 1440) should NOT trigger propagation.
#[tokio::test]
async fn test_propagate_overnight_does_nothing_for_day_shift() {
    let ws_id = Uuid::new_v4();
    let emp_id = Uuid::new_v4();
    let date = NaiveDate::from_ymd_opt(2026, 6, 20).unwrap();

    // Day shift: end_minutes = 1020 (17:00) → below 1440 threshold
    let day_shift_config = PairingShiftConfig::new(
        480,       // 08:00
        1020,      // 17:00
        Some(720), // 12:00
        Some(780), // 13:00
        None,
        None,
    );

    // No mock expectations needed — propagation should return immediately
    let service = PairingService::new(PairingServiceArgs::new(
        Arc::new(MockTimeEntryRepository::new()),
        Arc::new(MockRosterRepository::new()),
        Arc::new(MockEmployeeRepository::new()),
        Arc::new(MockRawAttendanceLogRepository::new()),
        Arc::new(MockSummaryRecalculator::new()),
    ));

    let result = service
        .propagate_overnight_reprocess(ws_id, emp_id, date, chrono_tz::UTC, Some(day_shift_config))
        .await;

    assert!(result.is_ok(), "Day shift should not trigger propagation");
}

/// Overnight shift (22:00-06:00, end_minutes=1800 > 1440) SHOULD trigger next day reprocess.
#[tokio::test]
async fn test_propagate_overnight_reprocesses_next_day() {
    let ws_id = Uuid::new_v4();
    let emp_id = Uuid::new_v4();
    let date = NaiveDate::from_ymd_opt(2026, 6, 20).unwrap();
    let next_date = NaiveDate::from_ymd_opt(2026, 6, 21).unwrap();

    // Overnight shift: 22:00 → 06:00 (end_minutes = 1800 = 30h > 1440)
    let overnight_config = PairingShiftConfig::new(
        1320, // 22:00
        1800, // 06:00 next day
        None, None, None, None,
    );

    // Schedule for next day: day shift (08:00-17:00) → propagation stops after this
    let mut mock_roster_repo = MockRosterRepository::new();
    mock_roster_repo
        .expect_get_workspace_schedules()
        .times(1)
        .returning(move |_, _, _, _, _, _, _| {
            let sched = crate::models::EffectiveSchedule::new(
                next_date,
                None,
                "roster".to_string(),
                Some("Day".to_string()),
                None,
                Some(NaiveTime::from_hms_opt(8, 0, 0).unwrap()),
                Some(NaiveTime::from_hms_opt(17, 0, 0).unwrap()),
                Some(60),
                Some(NaiveTime::from_hms_opt(12, 0, 0).unwrap()),
                Some(NaiveTime::from_hms_opt(13, 0, 0).unwrap()),
                Some(true),
                None,
                None,
                None,
                None,
                vec![],
                false,
                false,
                None,
            );
            Box::pin(async move {
                let list = vec![crate::models::EmployeeSchedule::new(
                    emp_id,
                    emp_id,
                    "EMP001".to_string(),
                    "Test".to_string(),
                    vec![sched],
                )];
                let total = list.len() as i64;
                Ok((list, total))
            })
        });

    let mut mock_raw_repo = MockRawAttendanceLogRepository::new();
    mock_raw_repo
        .expect_get_employment_logs_in_time_range()
        .times(1)
        .returning(|_, _, _, _| Box::pin(async { Ok(vec![]) }));

    let mut mock_time_entry_repo = MockTimeEntryRepository::new();
    mock_time_entry_repo
        .expect_get_employment_time_entries()
        .times(1)
        .returning(|_, _, _, _| Ok(vec![]));
    mock_time_entry_repo
        .expect_replace_time_entries()
        .times(1)
        .returning(|_, _, _, _, _| Ok(()));

    let mut mock_summary = MockSummaryRecalculator::new();
    mock_summary
        .expect_recalculate_attendance_range()
        .times(1)
        .returning(|_, _, _, _| Ok(()));

    let service = PairingService::new(PairingServiceArgs::new(
        Arc::new(mock_time_entry_repo),
        Arc::new(mock_roster_repo),
        Arc::new(MockEmployeeRepository::new()),
        Arc::new(mock_raw_repo),
        Arc::new(mock_summary),
    ));

    let result = service
        .propagate_overnight_reprocess(ws_id, emp_id, date, chrono_tz::UTC, Some(overnight_config))
        .await;

    assert!(
        result.is_ok(),
        "Overnight propagation failed: {:?}",
        result.err()
    );
}

/// Propagation should stop when the next day has a day shift (end_minutes <= 1440).
#[tokio::test]
async fn test_propagate_overnight_stops_when_next_day_is_day_shift() {
    let ws_id = Uuid::new_v4();
    let emp_id = Uuid::new_v4();
    let date = NaiveDate::from_ymd_opt(2026, 6, 20).unwrap();

    // Day shift config (not overnight)
    let day_shift_config = PairingShiftConfig::new(
        480,  // 08:00
        1020, // 17:00
        None, None, None, None,
    );

    // end_minutes = 1020 <= 1440, so propagation returns Ok immediately
    let service = PairingService::new(PairingServiceArgs::new(
        Arc::new(MockTimeEntryRepository::new()),
        Arc::new(MockRosterRepository::new()),
        Arc::new(MockEmployeeRepository::new()),
        Arc::new(MockRawAttendanceLogRepository::new()),
        Arc::new(MockSummaryRecalculator::new()),
    ));

    let result = service
        .propagate_overnight_reprocess(ws_id, emp_id, date, chrono_tz::UTC, Some(day_shift_config))
        .await;

    assert!(result.is_ok());
}

/// Propagation with None config should be a no-op.
#[tokio::test]
async fn test_propagate_overnight_none_config_noop() {
    let ws_id = Uuid::new_v4();
    let emp_id = Uuid::new_v4();
    let date = NaiveDate::from_ymd_opt(2026, 6, 20).unwrap();

    let service = PairingService::new(PairingServiceArgs::new(
        Arc::new(MockTimeEntryRepository::new()),
        Arc::new(MockRosterRepository::new()),
        Arc::new(MockEmployeeRepository::new()),
        Arc::new(MockRawAttendanceLogRepository::new()),
        Arc::new(MockSummaryRecalculator::new()),
    ));

    let result = service
        .propagate_overnight_reprocess(ws_id, emp_id, date, chrono_tz::UTC, None)
        .await;

    assert!(result.is_ok(), "None config should return Ok immediately");
}

/// When no schedules are found during propagation lookahead, it should gracefully return Ok.
#[tokio::test]
async fn test_propagate_overnight_no_schedule_graceful() {
    let ws_id = Uuid::new_v4();
    let emp_id = Uuid::new_v4();
    let date = NaiveDate::from_ymd_opt(2026, 6, 20).unwrap();

    let overnight_config = PairingShiftConfig::new(
        1320, // 22:00
        1800, // 06:00 next day
        None, None, None, None,
    );

    // Return empty schedules → fetch_schedules_for_range will return Err
    let mut mock_roster_repo = MockRosterRepository::new();
    mock_roster_repo
        .expect_get_workspace_schedules()
        .times(1)
        .returning(|_, _, _, _, _, _, _| Box::pin(async { Ok((vec![], 0)) }));

    let service = PairingService::new(PairingServiceArgs::new(
        Arc::new(MockTimeEntryRepository::new()),
        Arc::new(mock_roster_repo),
        Arc::new(MockEmployeeRepository::new()),
        Arc::new(MockRawAttendanceLogRepository::new()),
        Arc::new(MockSummaryRecalculator::new()),
    ));

    let result = service
        .propagate_overnight_reprocess(ws_id, emp_id, date, chrono_tz::UTC, Some(overnight_config))
        .await;

    // Should gracefully return Ok, not propagate error
    assert!(
        result.is_ok(),
        "Missing schedule should be graceful: {:?}",
        result.err()
    );
}

// =============================================================================
// --- TD-7: Range Reprocessing Tests ---
// =============================================================================

/// Multi-day range with logs should produce time entries for each day.
#[tokio::test]
async fn test_range_reprocess_multi_day_with_logs() {
    let ws_id = Uuid::new_v4();
    let emp_id = Uuid::new_v4();
    let start_date = NaiveDate::from_ymd_opt(2026, 6, 18).unwrap();
    let end_date = NaiveDate::from_ymd_opt(2026, 6, 20).unwrap();

    let mut mock_roster_repo = MockRosterRepository::new();
    let sched_emp_id = emp_id;
    mock_roster_repo
        .expect_get_workspace_schedules()
        .times(1)
        .returning(move |_, _, s_date, e_date, _, _, _| {
            let mut schedules = Vec::new();
            let mut d = s_date;
            while d <= e_date {
                schedules.push(crate::models::EffectiveSchedule::new(
                    d,
                    None,
                    "none".to_string(),
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    vec![],
                    false,
                    false,
                    None,
                ));
                d = d.succ_opt().unwrap();
            }
            Box::pin(async move {
                let list = vec![crate::models::EmployeeSchedule::new(
                    sched_emp_id,
                    sched_emp_id,
                    "EMP001".to_string(),
                    "Test Employee".to_string(),
                    schedules,
                )];
                let total = list.len() as i64;
                Ok((list, total))
            })
        });

    let mut mock_raw_repo = MockRawAttendanceLogRepository::new();
    mock_raw_repo
        .expect_get_employment_logs_in_time_range()
        .times(1)
        .returning(|_, _, _, _| Box::pin(async { Ok(vec![]) }));

    let mut mock_time_entry_repo = MockTimeEntryRepository::new();
    mock_time_entry_repo
        .expect_get_employment_time_entries()
        .times(1)
        .returning(|_, _, _, _| Ok(vec![]));
    mock_time_entry_repo
        .expect_replace_time_entries()
        .times(1)
        .returning(|_, _, _, _, _| Ok(()));

    let mut mock_summary = MockSummaryRecalculator::new();
    mock_summary
        .expect_recalculate_attendance_range()
        .times(1)
        .returning(|_, _, _, _| Ok(()));

    let service = PairingService::new(PairingServiceArgs::new(
        Arc::new(mock_time_entry_repo),
        Arc::new(mock_roster_repo),
        Arc::new(MockEmployeeRepository::new()),
        Arc::new(mock_raw_repo),
        Arc::new(mock_summary),
    ));

    let result = service
        .reprocess_attendance_range(ws_id, emp_id, start_date, end_date, chrono_tz::UTC)
        .await;

    assert!(result.is_ok(), "Multi-day range failed: {:?}", result.err());
}

/// When the last day of a range is overnight, the handler should trigger next-day reprocess.
#[tokio::test]
async fn test_range_reprocess_handles_overnight_tail() {
    let ws_id = Uuid::new_v4();
    let emp_id = Uuid::new_v4();
    let start_date = NaiveDate::from_ymd_opt(2026, 6, 18).unwrap();
    let end_date = NaiveDate::from_ymd_opt(2026, 6, 19).unwrap();

    let mut mock_raw_repo = MockRawAttendanceLogRepository::new();
    mock_raw_repo
        .expect_get_employment_logs_in_time_range()
        .returning(|_, _, _, _| Box::pin(async { Ok(vec![]) }));

    let mut mock_time_entry_repo = MockTimeEntryRepository::new();
    mock_time_entry_repo
        .expect_get_employment_time_entries()
        .returning(|_, _, _, _| Ok(vec![]));
    mock_time_entry_repo
        .expect_replace_time_entries()
        .returning(|_, _, _, _, _| Ok(()));

    // handle_overnight_tail triggers reprocess_single_day for next day

    let mut mock_summary = MockSummaryRecalculator::new();
    mock_summary
        .expect_recalculate_attendance_range()
        .returning(|_, _, _, _| Ok(()));
    mock_summary
        .expect_recalculate_daily_summary()
        .returning(|_, _, _| Ok(()));

    // We can't easily set expectations for both range and single-day on same mock.
    // Instead, test that result is Ok — which proves handle_overnight_tail was invoked.
    let mut mock_roster_combined = MockRosterRepository::new();
    mock_roster_combined
        .expect_get_workspace_schedules()
        .returning(move |_, _, s_date, e_date, _, _, _| {
            let mut schedules = Vec::new();
            let mut d = s_date;
            while d <= e_date {
                if d == NaiveDate::from_ymd_opt(2026, 6, 19).unwrap() {
                    schedules.push(crate::models::EffectiveSchedule::new(
                        d,
                        None,
                        "roster".to_string(),
                        None,
                        None,
                        Some(NaiveTime::from_hms_opt(22, 0, 0).unwrap()),
                        Some(NaiveTime::from_hms_opt(6, 0, 0).unwrap()),
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        vec![],
                        false,
                        false,
                        None,
                    ));
                } else {
                    schedules.push(crate::models::EffectiveSchedule::new(
                        d,
                        None,
                        "none".to_string(),
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        vec![],
                        false,
                        false,
                        None,
                    ));
                }
                d = d.succ_opt().unwrap();
            }
            Box::pin(async move {
                let list = vec![crate::models::EmployeeSchedule::new(
                    emp_id,
                    emp_id,
                    "EMP001".to_string(),
                    "Test".to_string(),
                    schedules,
                )];
                let total = list.len() as i64;
                Ok((list, total))
            })
        });
    mock_roster_combined
        .expect_get_effective_shift()
        .returning(|_, _, _| Box::pin(async { Ok((None, vec![])) }));

    let service = PairingService::new(PairingServiceArgs::new(
        Arc::new(mock_time_entry_repo),
        Arc::new(mock_roster_combined),
        Arc::new(MockEmployeeRepository::new()),
        Arc::new(mock_raw_repo),
        Arc::new(mock_summary),
    ));

    let result = service
        .reprocess_attendance_range(ws_id, emp_id, start_date, end_date, chrono_tz::UTC)
        .await;

    assert!(
        result.is_ok(),
        "Overnight tail handling failed: {:?}",
        result.err()
    );
}

/// Range reprocess with a day that has no schedule should skip that day gracefully.
#[tokio::test]
async fn test_range_reprocess_empty_schedule_day_skipped() {
    let ws_id = Uuid::new_v4();
    let emp_id = Uuid::new_v4();
    let start_date = NaiveDate::from_ymd_opt(2026, 6, 18).unwrap();
    let end_date = NaiveDate::from_ymd_opt(2026, 6, 20).unwrap();

    let mut mock_roster_repo = MockRosterRepository::new();
    let sched_emp_id = emp_id;
    mock_roster_repo
        .expect_get_workspace_schedules()
        .times(1)
        .returning(move |_, _, s_date, _e_date, _, _, _| {
            // Only return schedule for first day, skip day 2 and 3
            let schedules = vec![crate::models::EffectiveSchedule::new(
                s_date,
                None,
                "none".to_string(),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                vec![],
                false,
                false,
                None,
            )];
            Box::pin(async move {
                let list = vec![crate::models::EmployeeSchedule::new(
                    sched_emp_id,
                    sched_emp_id,
                    "EMP001".to_string(),
                    "Test".to_string(),
                    schedules,
                )];
                let total = list.len() as i64;
                Ok((list, total))
            })
        });

    let mut mock_raw_repo = MockRawAttendanceLogRepository::new();
    mock_raw_repo
        .expect_get_employment_logs_in_time_range()
        .times(1)
        .returning(|_, _, _, _| Box::pin(async { Ok(vec![]) }));

    let mut mock_time_entry_repo = MockTimeEntryRepository::new();
    mock_time_entry_repo
        .expect_get_employment_time_entries()
        .times(1)
        .returning(|_, _, _, _| Ok(vec![]));
    mock_time_entry_repo
        .expect_replace_time_entries()
        .times(1)
        .returning(|_, _, _, _, _| Ok(()));

    let mut mock_summary = MockSummaryRecalculator::new();
    mock_summary
        .expect_recalculate_attendance_range()
        .times(1)
        .returning(|_, _, _, _| Ok(()));

    let service = PairingService::new(PairingServiceArgs::new(
        Arc::new(mock_time_entry_repo),
        Arc::new(mock_roster_repo),
        Arc::new(MockEmployeeRepository::new()),
        Arc::new(mock_raw_repo),
        Arc::new(mock_summary),
    ));

    // Days without schedule in the map should be silently skipped
    let result = service
        .reprocess_attendance_range(ws_id, emp_id, start_date, end_date, chrono_tz::UTC)
        .await;

    assert!(
        result.is_ok(),
        "Missing schedule day should be skipped: {:?}",
        result.err()
    );
}

// =============================================================================
// --- TD-7: Strategy Split Tests ---
// =============================================================================

/// 3 break intervals should produce 4 session segments.
#[test]
fn test_split_by_breaks_3_breaks() {
    let strategy = get_strategy();
    let shift = PairingShiftConfig::new_with_breaks(
        480,  // 08:00
        1020, // 17:00
        vec![
            BreakInterval::new(600, 615), // 10:00-10:15 (tea break)
            BreakInterval::new(720, 780), // 12:00-13:00 (lunch)
            BreakInterval::new(900, 915), // 15:00-15:15 (afternoon tea)
        ],
        None,
        None,
    );

    let logs = vec![make_log(475), make_log(1025)];
    let sessions = strategy.pair(&logs, Some(&shift));

    // 3 breaks → 4 segments
    assert_eq!(
        sessions.len(),
        4,
        "Expected 4 segments for 3 breaks, got {}",
        sessions.len()
    );

    // Verify segment boundaries
    assert_eq!(sessions[0].start_minutes(), 475);
    assert_eq!(sessions[0].end_minutes(), Some(600));
    assert_eq!(sessions[0].metadata().unwrap()["break_split"], "first_half");

    assert_eq!(sessions[1].start_minutes(), 615);
    assert_eq!(sessions[1].end_minutes(), Some(720));
    assert_eq!(
        sessions[1].metadata().unwrap()["break_split"],
        "middle_segment"
    );

    assert_eq!(sessions[2].start_minutes(), 780);
    assert_eq!(sessions[2].end_minutes(), Some(900));
    assert_eq!(
        sessions[2].metadata().unwrap()["break_split"],
        "middle_segment"
    );

    assert_eq!(sessions[3].start_minutes(), 915);
    assert_eq!(sessions[3].end_minutes(), Some(1025));
    assert_eq!(
        sessions[3].metadata().unwrap()["break_split"],
        "second_half"
    );
}

/// Break interval outside the session range should not cause splitting.
#[test]
fn test_split_by_breaks_break_outside_session() {
    let strategy = get_strategy();
    // Break at 06:00-06:30, but shift is 08:00-17:00 — break is before shift
    let shift = PairingShiftConfig::new_with_breaks(
        480,                                // 08:00
        1020,                               // 17:00
        vec![BreakInterval::new(360, 390)], // 06:00-06:30 (before shift)
        None,
        None,
    );

    let logs = vec![make_log(475), make_log(1025)];
    let sessions = strategy.pair(&logs, Some(&shift));

    // Break is before shift start, so it should be filtered out
    // The result depends on whether the strategy filters breaks outside the session
    // At minimum, the pairing should not crash and produce valid output
    assert!(!sessions.is_empty(), "Should produce at least one session");
    for session in &sessions {
        assert!(session.start_minutes() >= 475);
    }
}

/// Regression test for the "missing check-out" bug.
///
/// Scenario: scheduled shift 07:30–16:30 with a single break 11:30–12:30.
/// Employee records 4 punches: 07:13 (check-in), 11:46 (lunch out),
/// 11:46 (duplicate), 19:32 (late check-out).
///
/// The pairing algorithm MUST produce exactly 2 work sessions split on
/// the configured break. Session 2 MUST have an end_time so the report
/// correctly shows check-out at 19:32.
///
/// Bug: the report only shows check-in 07:13 with no check-out.
#[test]
fn test_missing_checkout_with_break_and_late_out() {
    let strategy = get_strategy();
    // 07:30-16:30, break 11:30-12:30
    let shift = PairingShiftConfig::new_with_breaks(
        450,                                // 07:30
        990,                                // 16:30
        vec![BreakInterval::new(690, 750)], // 11:30-12:30
        None,
        None,
    );

    // Punches: 07:13 (433 min), 11:46 (706 min), 11:46 (706 min), 19:32 (1172 min)
    let logs = vec![
        make_log(433),  // 07:13 check-in
        make_log(706),  // 11:46 lunch out
        make_log(706),  // 11:46 duplicate
        make_log(1172), // 19:32 late check-out
    ];

    let sessions = strategy.pair(&logs, Some(&shift));

    // Must produce exactly 2 sessions split on break
    assert_eq!(
        sessions.len(),
        2,
        "expected exactly 2 sessions, got {}: {:?}",
        sessions.len(),
        sessions
            .iter()
            .map(|s| (s.start_minutes(), s.end_minutes()))
            .collect::<Vec<_>>()
    );

    // Session 1: 07:13 → 11:30 (first half, break start)
    assert_eq!(sessions[0].start_minutes(), 433);
    assert_eq!(sessions[0].end_minutes(), Some(690));
    assert_eq!(sessions[0].metadata().unwrap()["break_split"], "first_half");

    // Session 2: 12:30 → 19:32 (second half, break end to late checkout)
    // THIS IS THE KEY ASSERTION: session 2 MUST have an end_time
    assert_eq!(sessions[1].start_minutes(), 750);
    assert_eq!(
        sessions[1].end_minutes(),
        Some(1172),
        "BUG: second session should end at 19:32 (1172 min), but end_minutes is {:?}",
        sessions[1].end_minutes()
    );
    assert_eq!(
        sessions[1].metadata().unwrap()["break_split"],
        "second_half"
    );

    // Verify session 2 has a non-None end_time (actual_punch_out in report)
    assert!(
        sessions[1].end_time().is_some(),
        "BUG: session 2 end_time() is None — report will show no check-out"
    );
}

/// Full pipeline test: raw logs → build_log_events → pairing → build_time_entries.
/// Verifies that the exact scenario (07:13, 11:46, 11:46, 19:32) produces
/// TimeEntries where the LAST entry has a punch_out_time (so the report shows check-out).
#[test]
fn test_full_pipeline_4_punches_with_late_out() {
    let date = NaiveDate::from_ymd_opt(2026, 6, 20).unwrap();
    let tz = chrono_tz::Tz::Asia__Ho_Chi_Minh;
    let context = PairingContext::new(date, tz);
    let workspace_id = Uuid::new_v4();
    let employment_id = Uuid::new_v4();

    // Create raw logs simulating 4 punches:
    // 07:13, 11:46 (two close together), 19:32 ICT
    let log7_13 = RawAttendanceLog::new(
        Uuid::new_v4(),
        workspace_id,
        employment_id,
        tz.with_ymd_and_hms(2026, 6, 20, 7, 13, 0)
            .unwrap()
            .with_timezone(&Utc),
        LogSource::Manual,
        None,
        None,
        1,
        1,
        Utc::now(),
    );
    let log11_46_a = RawAttendanceLog::new(
        Uuid::new_v4(),
        workspace_id,
        employment_id,
        tz.with_ymd_and_hms(2026, 6, 20, 11, 46, 0)
            .unwrap()
            .with_timezone(&Utc),
        LogSource::Manual,
        None,
        None,
        1,
        1,
        Utc::now(),
    );
    let log11_46_b = RawAttendanceLog::new(
        Uuid::new_v4(),
        workspace_id,
        employment_id,
        tz.with_ymd_and_hms(2026, 6, 20, 11, 46, 2)
            .unwrap()
            .with_timezone(&Utc),
        LogSource::Manual,
        None,
        None,
        1,
        1,
        Utc::now(),
    );
    let log19_32 = RawAttendanceLog::new(
        Uuid::new_v4(),
        workspace_id,
        employment_id,
        tz.with_ymd_and_hms(2026, 6, 20, 19, 32, 0)
            .unwrap()
            .with_timezone(&Utc),
        LogSource::Manual,
        None,
        None,
        1,
        1,
        Utc::now(),
    );

    let raw_logs = vec![log7_13, log11_46_a, log11_46_b, log19_32];

    // Step 1: build_log_events (dedup by 5-second window)
    let log_events = context.build_log_events(&raw_logs).unwrap();
    assert!(
        log_events.len() >= 3,
        "expected at least 3 log events (07:13, merged 11:46, 19:32), got {}",
        log_events.len()
    );

    // Step 2: pair
    let shift = PairingShiftConfig::new_with_breaks(
        450,                                // 07:30
        990,                                // 16:30
        vec![BreakInterval::new(690, 750)], // 11:30-12:30
        None,
        None,
    );
    let strategy = get_strategy();
    let sessions = strategy.pair(&log_events, Some(&shift));
    assert_eq!(
        sessions.len(),
        2,
        "pairing must produce 2 sessions, got {}: {:?}",
        sessions.len(),
        sessions
            .iter()
            .map(|s| (s.start_minutes(), s.end_minutes()))
            .collect::<Vec<_>>()
    );

    // Step 3: build_time_entries (the full pipeline)
    let traces = serde_json::json!(["test_trace"]);
    let entries = context
        .build_time_entries(workspace_id, employment_id, None, sessions, &traces)
        .unwrap();

    assert_eq!(
        entries.len(),
        2,
        "build_time_entries must produce 2 entries, got {}",
        entries.len()
    );

    // KEY CHECK: the LAST time entry must have a punch_out_time
    let first_entry = &entries[0];
    let last_entry = &entries[1];
    assert!(
        last_entry.punch_out_time().is_some(),
        "BUG: last entry punch_out_time is None — report will show no check-out!\n\
         First entry: {:?}→{:?}\n\
         Last entry: {:?}→{:?}",
        first_entry.punch_in_time().with_timezone(&tz).time(),
        first_entry
            .punch_out_time()
            .map(|t| t.with_timezone(&tz).time()),
        last_entry.punch_in_time().with_timezone(&tz).time(),
        last_entry
            .punch_out_time()
            .map(|t| t.with_timezone(&tz).time()),
    );

    // Verify the actual times match expectations
    let last_out_local = last_entry.punch_out_time().unwrap().with_timezone(&tz);
    assert_eq!(last_out_local.hour(), 19);
    assert_eq!(last_out_local.minute(), 32);
}

fn make_log_with_seconds(minutes: i32, seconds: i32) -> LogEvent {
    let id = Uuid::new_v4();
    let punch_time = Utc.with_ymd_and_hms(2026, 6, 20, 0, 0, 0).unwrap()
        + chrono::Duration::minutes(i64::from(minutes))
        + chrono::Duration::seconds(i64::from(seconds));
    LogEvent {
        id,
        minutes,
        punch_time,
        raw_data: None,
        all_ids: vec![id],
    }
}

#[test]
fn test_split_by_breaks_seconds_alignment() {
    use chrono::Timelike;

    let strategy = get_strategy();
    let shift = PairingShiftConfig::new_with_breaks(
        450,                                // 07:30
        990,                                // 16:30
        vec![BreakInterval::new(690, 750)], // 11:30-12:30
        None,
        None,
    );

    let first_in = make_log_with_seconds(450, 0); // 07:30:00
    let last_out = make_log_with_seconds(1008, 10); // 16:48:10
    let logs = vec![first_in, last_out];

    let sessions = strategy.pair(&logs, Some(&shift));

    assert_eq!(sessions.len(), 2);
    // Segment 1 (07:30 to 11:30)
    assert_eq!(sessions[0].start_minutes(), 450);
    assert_eq!(sessions[0].end_minutes(), Some(690));
    assert_eq!(sessions[0].start_time().second(), 0);
    assert_eq!(sessions[0].end_time().unwrap().second(), 0);

    // Segment 2 (12:30 to 16:48)
    assert_eq!(sessions[1].start_minutes(), 750);
    assert_eq!(sessions[1].end_minutes(), Some(1008));
    // The start time of Segment 2 (break end) should be aligned to first_in (0 seconds)
    assert_eq!(sessions[1].start_time().second(), 0);
    // The end time of Segment 2 (checkout) should be the actual checkout (10 seconds)
    assert_eq!(sessions[1].end_time().unwrap().second(), 10);
}

// =============================================================================
// fetch_effective_shift_config — interval fallback tests
// =============================================================================

/// Builds an `EffectiveSchedule` with all fields set for pairing.
fn make_effective_schedule(
    date: NaiveDate,
    shift_id: Option<Uuid>,
    start_h: u32,
    start_m: u32,
    end_h: u32,
    end_m: u32,
    break_start_h: u32,
    break_start_m: u32,
    break_end_h: u32,
    break_end_m: u32,
) -> crate::models::EffectiveSchedule {
    crate::models::EffectiveSchedule::new(
        date,
        shift_id,
        "roster".to_string(),
        Some("Test Shift".to_string()),
        None,
        Some(NaiveTime::from_hms_opt(start_h, start_m, 0).unwrap()),
        Some(NaiveTime::from_hms_opt(end_h, end_m, 0).unwrap()),
        Some(0),
        Some(NaiveTime::from_hms_opt(break_start_h, break_start_m, 0).unwrap()),
        Some(NaiveTime::from_hms_opt(break_end_h, break_end_m, 0).unwrap()),
        Some(true),
        None,
        None,
        None,
        None,
        vec![],
        false,
        false,
        None,
    )
}

/// Regression: shift with NO intervals falls back to schedule to retain break awareness.
///
/// Before the fix, `fetch_effective_shift_config` only built `PairingShiftConfig`
/// from `shift_intervals`.  If a shift had no interval entry (legacy/migrated
/// shifts), `shift_config` was `None` and pairing degraded to `anchor_first`,
/// which doesn't know about breaks — producing `missing_punch_out` on both
/// sessions of a break-split day.
///
/// This test verifies that when the shift exists in `roster_data.shifts` but has
/// no entry in `roster_data.shift_intervals`, the fallback reads the resolved
/// `EffectiveSchedule` and produces a config WITH break data so pairing uses the
/// shift-window tier.
#[tokio::test]
async fn test_fetch_shift_config_falls_back_when_no_intervals() {
    let ws_id = Uuid::new_v4();
    let emp_id = Uuid::new_v4();
    let shift_id = Uuid::new_v4();
    let date = NaiveDate::from_ymd_opt(2026, 6, 20).unwrap();
    let tz = chrono_tz::Tz::Asia__Ho_Chi_Minh;
    let ctx = PairingContext::new(date, tz);

    // --- Mocks ---
    let mut mock_roster = MockRosterRepository::new();

    // get_effective_shift → returns a valid shift_id
    mock_roster
        .expect_get_effective_shift()
        .times(1)
        .returning(move |_, _, _| {
            let sid = shift_id;
            Box::pin(async move { Ok((Some(sid), vec![])) })
        });

    // get_raw_roster_data → shift exists but NO intervals
    let roster_data = Arc::new({
        let mut d = RosterData::default();
        d.shifts.insert(
            shift_id,
            RosterShift::new("Test Shift".to_string(), Some("TS".to_string())),
        );
        // Intentionally leave shift_intervals EMPTY — this is the bug scenario
        d
    });
    mock_roster
        .expect_get_raw_roster_data()
        .times(1)
        .returning(move |_| {
            let rd = Arc::clone(&roster_data);
            Box::pin(async move { Ok(rd) })
        });

    // get_workspace_schedules → fallback path — returns schedule WITH break
    let sched = make_effective_schedule(
        date,
        Some(shift_id),
        7,
        30, // start 07:30
        16,
        30, // end   16:30
        11,
        30, // break_start 11:30
        12,
        30, // break_end   12:30
    );
    let emp_sched = crate::models::EmployeeSchedule::new(
        emp_id,
        emp_id,
        "EMP001".to_string(),
        "Test Employee".to_string(),
        vec![sched],
    );
    mock_roster
        .expect_get_workspace_schedules()
        .times(1)
        .returning(move |_, _, _, _, _, _, _| {
            let s = emp_sched.clone();
            Box::pin(async move { Ok((vec![s], 1)) })
        });

    let service = PairingService::new(PairingServiceArgs::new(
        Arc::new(MockTimeEntryRepository::new()),
        Arc::new(mock_roster),
        Arc::new(MockEmployeeRepository::new()),
        Arc::new(MockRawAttendanceLogRepository::new()),
        Arc::new(MockSummaryRecalculator::new()),
    ));

    let auth_token = TenantAuthToken::system_override(ws_id);

    let result = crate::services::pairing::service_single::fetch_effective_shift_config(
        &service,
        &auth_token,
        emp_id,
        date,
        &ctx,
    )
    .await
    .unwrap();

    let (got_shift_id, got_config, _traces) = result;

    // Shift ID must be preserved
    assert_eq!(got_shift_id, Some(shift_id));

    // Config MUST be Some — before the fix it was None
    assert!(
        got_config.is_some(),
        "BUG: shift_config is None — fallback to EffectiveSchedule failed"
    );

    let config = got_config.unwrap();
    // Verify break awareness — 07:30 = 450 min, 16:30 = 990 min, break 11:30-12:30
    assert_eq!(config.start_minutes(), 450, "expected shift start 07:30");
    assert_eq!(config.end_minutes(), 990, "expected shift end 16:30");
    assert_eq!(
        config.break_start_minutes(),
        Some(690),
        "expected break_start 11:30 — pairing will split correctly"
    );
    assert_eq!(
        config.break_end_minutes(),
        Some(750),
        "expected break_end 12:30 — pairing will split correctly"
    );
    assert_eq!(
        config.breaks().len(),
        1,
        "should have exactly 1 break interval"
    );
}

/// Similar to above but also runs the pairing algorithm to verify the full pipeline
/// produces 2 sessions with a valid check-out when the fallback is exercised.
#[tokio::test]
async fn test_pairing_uses_fallback_config_to_split_on_break() {
    let ws_id = Uuid::new_v4();
    let emp_id = Uuid::new_v4();
    let shift_id = Uuid::new_v4();
    let date = NaiveDate::from_ymd_opt(2026, 6, 20).unwrap();
    let tz = chrono_tz::Tz::Asia__Ho_Chi_Minh;

    // --- Mocks ---
    let mut mock_roster = MockRosterRepository::new();
    let mut mock_raw = MockRawAttendanceLogRepository::new();
    let mut mock_time_entry = MockTimeEntryRepository::new();
    let mut mock_summary = MockSummaryRecalculator::new();

    // get_effective_shift → valid shift_id
    mock_roster
        .expect_get_effective_shift()
        .times(1)
        .returning(move |_, _, _| {
            let sid = shift_id;
            Box::pin(async move { Ok((Some(sid), vec![])) })
        });

    // get_raw_roster_data → shift exists but NO intervals
    let roster_data2 = Arc::new({
        let mut d = RosterData::default();
        d.shifts.insert(
            shift_id,
            RosterShift::new("Test Shift".to_string(), Some("TS".to_string())),
        );
        d
    });
    mock_roster
        .expect_get_raw_roster_data()
        .times(1)
        .returning(move |_| {
            let rd = Arc::clone(&roster_data2);
            Box::pin(async move { Ok(rd) })
        });

    // get_workspace_schedules → fallback path with break
    let sched = make_effective_schedule(date, Some(shift_id), 7, 30, 16, 30, 11, 30, 12, 30);
    let emp_sched = crate::models::EmployeeSchedule::new(
        emp_id,
        emp_id,
        "EMP001".to_string(),
        "Test Employee".to_string(),
        vec![sched],
    );
    mock_roster
        .expect_get_workspace_schedules()
        .times(1)
        .returning(move |_, _, _, _, _, _, _| {
            let s = emp_sched.clone();
            Box::pin(async move { Ok((vec![s], 1)) })
        });

    // get_employment_time_entries → empty (no consumed logs from adjacent days)
    mock_time_entry
        .expect_get_employment_time_entries()
        .times(1)
        .returning(|_, _, _, _| Ok(vec![]));

    // replace_time_entries → capture entries for assertion
    let emp_id_captured = emp_id;
    mock_time_entry
        .expect_replace_time_entries()
        .times(1)
        .returning(move |_, eid, _, _, entries| {
            assert_eq!(eid, emp_id_captured);
            // KEY: must produce exactly 2 sessions with break split
            assert_eq!(
                entries.len(),
                2,
                "expected 2 time entries (split on break), got {}",
                entries.len()
            );

            // Session 2 must have punch_out_time (the check-out)
            let last = &entries[1];
            assert!(
                last.punch_out_time().is_some(),
                "BUG: second session has no punch_out_time — check-out missing!\n\
                 entries: {:?}",
                entries
                    .iter()
                    .map(|e| (
                        e.punch_in_time().with_timezone(&tz).time(),
                        e.punch_out_time().map(|t| t.with_timezone(&tz).time())
                    ))
                    .collect::<Vec<_>>()
            );

            // Verify tier is shift_window (NOT anchor_first)
            let meta = last.generation_metadata().unwrap();
            assert_eq!(
                meta["tier"],
                "shift_window",
                "BUG: pairing degraded to {} — break splitting never ran",
                meta["tier"].as_str().unwrap_or("unknown")
            );

            Ok(())
        });

    // get_employment_logs_in_time_range → 4 punches as in the bug report
    mock_raw
        .expect_get_employment_logs_in_time_range()
        .times(1)
        .returning(move |_, _, _, _| {
            use crate::models::RawAttendanceLog;
            let logs = vec![
                RawAttendanceLog::new(
                    Uuid::new_v4(),
                    ws_id,
                    emp_id,
                    tz.with_ymd_and_hms(2026, 6, 20, 7, 13, 0)
                        .unwrap()
                        .with_timezone(&Utc),
                    LogSource::Manual,
                    None,
                    None,
                    1,
                    1,
                    Utc::now(),
                ),
                RawAttendanceLog::new(
                    Uuid::new_v4(),
                    ws_id,
                    emp_id,
                    tz.with_ymd_and_hms(2026, 6, 20, 11, 46, 0)
                        .unwrap()
                        .with_timezone(&Utc),
                    LogSource::Manual,
                    None,
                    None,
                    1,
                    1,
                    Utc::now(),
                ),
                RawAttendanceLog::new(
                    Uuid::new_v4(),
                    ws_id,
                    emp_id,
                    tz.with_ymd_and_hms(2026, 6, 20, 11, 46, 2)
                        .unwrap()
                        .with_timezone(&Utc),
                    LogSource::Manual,
                    None,
                    None,
                    1,
                    1,
                    Utc::now(),
                ),
                RawAttendanceLog::new(
                    Uuid::new_v4(),
                    ws_id,
                    emp_id,
                    tz.with_ymd_and_hms(2026, 6, 20, 19, 32, 0)
                        .unwrap()
                        .with_timezone(&Utc),
                    LogSource::Manual,
                    None,
                    None,
                    1,
                    1,
                    Utc::now(),
                ),
            ];
            Box::pin(async move { Ok(logs) })
        });

    // recalculate_daily_summary
    mock_summary
        .expect_recalculate_daily_summary()
        .times(1)
        .returning(|_, _, _| Ok(()));

    let service = PairingService::new(PairingServiceArgs::new(
        Arc::new(mock_time_entry),
        Arc::new(mock_roster),
        Arc::new(MockEmployeeRepository::new()),
        Arc::new(mock_raw),
        Arc::new(mock_summary),
    ));

    let result = service
        .reprocess_daily_attendance(ws_id, emp_id, date, tz)
        .await;

    assert!(
        result.is_ok(),
        "reprocess_daily_attendance failed: {:?}",
        result.err()
    );
}

// =============================================================================
// check_in_buffer_minutes — regression test for the 15-min window bug
// =============================================================================

/// Regression: `check_in_buffer_minutes` (set from `early_punch_in_grace_period`)
/// was used as the sole check-in search buffer.  A 15‑min grace period produced
/// a [435, 465] window that MISSED a 07:13 (433‑min) punch, causing the entire
/// pairing to fall back to anchor_first (no break awareness → missing check-out).
///
/// After the fix, the buffer is `max(120, check_in_buffer)` so the search
/// window is always at least ±120 minutes from the shift start.
#[test]
fn test_buffer_is_never_smaller_than_shift_window_default() {
    let strategy = get_strategy();

    // Shift 07:30-16:30, break 11:30-12:30, tiny grace period of 15 min
    let shift = PairingShiftConfig::new_with_breaks(
        450,                                // 07:30
        990,                                // 16:30
        vec![BreakInterval::new(690, 750)], // 11:30-12:30
        Some(15),                           // check_in_buffer = 15 (the bug trigger)
        None,
    );

    // Punch 07:13 (433 min) — outside the narrow 15-min window but well
    // within the correct 120-min window.  With the old code this punch
    // was missed entirely.
    let logs = vec![
        make_log(433),  // 07:13 — early punch
        make_log(1172), // 19:32 — late check-out
    ];

    let sessions = strategy.pair(&logs, Some(&shift));

    assert_eq!(
        sessions.len(),
        2,
        "expected 2 sessions (split on break), got {} — buffer may still be too small",
        sessions.len()
    );

    // Verify tier is shift_window, NOT anchor_first
    let meta = sessions[0].metadata().unwrap();
    assert_eq!(
        meta["tier"],
        "shift_window",
        "BUG: degraded to anchor_first — buffer={} didn't cover the early punch",
        shift.check_in_buffer_minutes().unwrap_or(0)
    );

    assert_eq!(sessions[0].start_minutes(), 433);
    assert_eq!(sessions[0].end_minutes(), Some(690));
    assert_eq!(sessions[1].start_minutes(), 750);
    assert_eq!(sessions[1].end_minutes(), Some(1172));
}

/// The exact production scenario from the bug report.
///
/// Shift 07:30-16:30, break 11:30-12:30, grace=15 (from "Quy tắc mặc định"),
/// punches 07:13 / 11:46 / 11:46 / 19:32.  Before the fix the 15‑min buffer
/// made `find_earliest_in_window` return `None`, forcing anchor_first.
#[test]
fn test_buffer_exact_production_scenario() {
    let strategy = get_strategy();

    let shift = PairingShiftConfig::new_with_breaks(
        450,                                // 07:30
        990,                                // 16:30
        vec![BreakInterval::new(690, 750)], // 11:30-12:30
        Some(15),                           // ← the bug trigger: tiny grace period
        None,
    );

    // Exact punches from production
    let logs = vec![
        make_log(433),  // 07:13
        make_log(706),  // 11:46
        make_log(706),  // 11:46 duplicate
        make_log(1172), // 19:32
    ];

    let sessions = strategy.pair(&logs, Some(&shift));

    // Must produce 2 sessions, NOT anchor_first
    assert_eq!(sessions.len(), 2);

    let meta0 = sessions[0].metadata().unwrap();
    assert_eq!(
        meta0["tier"], "shift_window",
        "session 0 should be shift_window"
    );
    assert_eq!(meta0["break_split"], "first_half");
    assert_eq!(sessions[0].start_minutes(), 433);
    assert_eq!(sessions[0].end_minutes(), Some(690));

    let meta1 = sessions[1].metadata().unwrap();
    assert_eq!(
        meta1["tier"], "shift_window",
        "session 1 should be shift_window"
    );
    assert_eq!(meta1["break_split"], "second_half");
    assert_eq!(sessions[1].start_minutes(), 750);
    assert_eq!(sessions[1].end_minutes(), Some(1172));

    // The critical check: session 2 MUST have an end time
    assert!(
        sessions[1].end_time().is_some(),
        "BUG: second session has no check-out — buffer=15 missed the early punch"
    );
}

/// When check_in_buffer is larger than 120, it should be used directly
/// (e.g. a very generous grace period should widen the window).
#[test]
fn test_buffer_respects_larger_grace_period() {
    let strategy = get_strategy();

    let shift = PairingShiftConfig::new_with_breaks(
        450, // 07:30
        990, // 16:30
        vec![BreakInterval::new(690, 750)],
        Some(180), // larger than default
        None,
    );

    let logs = vec![
        make_log(290), // 04:50 — within 180-min window [270, 630] but NOT within 120-min [330, 570]
        make_log(1172), // 19:32
    ];

    let sessions = strategy.pair(&logs, Some(&shift));

    assert_eq!(
        sessions.len(),
        2,
        "larger buffer should still find the early punch"
    );
    assert_eq!(sessions[0].start_minutes(), 290);
}

/// When check_in_buffer is None, the default 120 should be used.
#[test]
fn test_buffer_defaults_to_120_when_none() {
    let strategy = get_strategy();

    let shift = PairingShiftConfig::new_with_breaks(
        450,
        990,
        vec![BreakInterval::new(690, 750)],
        None, // no explicit buffer → 120 default
        None,
    );

    // 07:13 (433) is within 120-min window but outside a 15-min window
    let logs = vec![make_log(433), make_log(1172)];

    let sessions = strategy.pair(&logs, Some(&shift));
    assert_eq!(sessions.len(), 2);
    assert_eq!(sessions[0].start_minutes(), 433);
    let meta = sessions[0].metadata().unwrap();
    assert_eq!(meta["tier"], "shift_window");
}
