use super::*;
use crate::diesel_pool::TenantAuthToken;
use crate::errors::AppError;
use crate::models::enums::LogSource;
use crate::models::{RawAttendanceLog, Shift, TimeEntry};
use crate::services::pairing::service_helpers::retry_with_log;
use chrono::{NaiveDate, NaiveTime, TimeZone, Utc};
use std::sync::Arc;
use uuid::Uuid;

use crate::repositories::employee::MockEmployeeRepository;
use crate::repositories::raw_attendance_log_repository::MockRawAttendanceLogRepository;
use crate::repositories::roster::MockRosterRepository;
use crate::repositories::shift_repository::MockShiftRepository;
use crate::repositories::time_entry_repository::MockTimeEntryRepository;
use crate::services::attendance_summary_service::MockSummaryRecalculator;

// --- Helper Functions for Strategy Tests ---

fn make_log(minutes: i32) -> LogEvent {
    let id = Uuid::new_v4();
    let punch_time = Utc.with_ymd_and_hms(2026, 6, 20, 0, 0, 0).unwrap()
        + chrono::Duration::minutes(minutes as i64);
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
    let log_ids: Vec<String> = meta["log_ids"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect();
    assert_eq!(log_ids.len(), 3);
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

    let shift = Shift::new(
        Uuid::new_v4(),
        Uuid::new_v4(),
        "Hành chính".to_string(),
        None,
        NaiveTime::from_hms_opt(7, 30, 0).unwrap(),
        NaiveTime::from_hms_opt(16, 30, 0).unwrap(),
        60,
        None,
        None,
        false,
        false,
        0,
        false,
        None,
        true,
        None,
        None,
        None,
        Utc::now(),
        Utc::now(),
    );

    let shift_config_utc = context_utc.build_shift_config(Some(&shift));
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

    let shift_config_hcm = context_hcm.build_shift_config(Some(&shift));
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

    let shift = Shift::new(
        Uuid::new_v4(),
        Uuid::new_v4(),
        "Night".to_string(),
        None,
        NaiveTime::from_hms_opt(22, 0, 0).unwrap(),
        NaiveTime::from_hms_opt(6, 0, 0).unwrap(),
        60,
        Some(NaiveTime::from_hms_opt(2, 0, 0).unwrap()),
        Some(NaiveTime::from_hms_opt(3, 0, 0).unwrap()),
        false,
        false,
        0,
        false,
        None,
        true,
        None,
        None,
        None,
        Utc::now(),
        Utc::now(),
    );

    let config = context.build_shift_config(Some(&shift)).unwrap();
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
        Arc::new(MockShiftRepository::new()),
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
        Arc::new(MockShiftRepository::new()),
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
        Arc::new(MockShiftRepository::new()),
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
        Arc::new(MockShiftRepository::new()),
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
        .returning(move |_, _, s_date, e_date, _| {
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
                Ok(vec![crate::models::EmployeeSchedule::new(
                    sched_emp_id,
                    sched_emp_id,
                    "EMP001".to_string(),
                    "Test Employee".to_string(),
                    schedules,
                )])
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
        Arc::new(MockShiftRepository::new()),
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
