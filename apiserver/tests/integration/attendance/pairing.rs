//! Comprehensive Pairing Strategy integration tests
//!
//! Tests the ShiftAwareStrategy across all 3 tiers:
//!   Tier 1: punch_type-based pairing (device reports IN/OUT)
//!   Tier 2: shift_window-based pairing (shift schedule context)
//!   Tier 3: anchor_first fallback (no context)

use apiserver::services::pairing_algo::{
    self, LogEvent, PairedSession, PairingShiftConfig, ShiftAwareStrategy,
};
use uuid::Uuid;

// ═══════════════════════════════════════════════════════════════
//  Helpers
// ═══════════════════════════════════════════════════════════════

fn strategy() -> ShiftAwareStrategy {
    pairing_algo::get_strategy()
}

fn log(minutes: i32, punch_type: Option<i32>) -> LogEvent {
    LogEvent {
        id: Uuid::new_v4(),
        minutes,
        punch_type,
    }
}

fn day_shift() -> PairingShiftConfig {
    PairingShiftConfig {
        start_minutes: 480,  // 08:00
        end_minutes: 1020,   // 17:00
        break_start_minutes: Some(720),
        break_end_minutes: Some(780),
    }
}

fn overnight_shift() -> PairingShiftConfig {
    PairingShiftConfig {
        start_minutes: 1320, // 22:00
        end_minutes: 1800,   // 06:00 next day (30h from midnight = 1800)
        break_start_minutes: Some(1560), // 02:00
        break_end_minutes: Some(1590),   // 02:30
    }
}

fn early_morning_shift() -> PairingShiftConfig {
    PairingShiftConfig {
        start_minutes: 360, // 06:00
        end_minutes: 840,   // 14:00
        break_start_minutes: None,
        break_end_minutes: None,
    }
}

fn tier(session: &PairedSession) -> String {
    session
        .metadata
        .as_ref()
        .and_then(|m| m["tier"].as_str())
        .unwrap_or("unknown")
        .to_string()
}

fn algorithm(session: &PairedSession) -> String {
    session
        .metadata
        .as_ref()
        .and_then(|m| m["algorithm"].as_str())
        .unwrap_or("unknown")
        .to_string()
}

// ═══════════════════════════════════════════════════════════════
//  Tier 1: Punch Type
// ═══════════════════════════════════════════════════════════════

#[test]
fn tier1_perfect_day_with_breaks() {
    let logs = vec![
        log(480, Some(0)),  // 08:00 Check-In
        log(720, Some(2)),  // 12:00 Break-In
        log(780, Some(3)),  // 13:00 Break-Out
        log(1020, Some(1)), // 17:00 Check-Out
    ];

    let sessions = strategy().pair(&logs, None);
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].start_minutes, 480);
    assert_eq!(sessions[0].end_minutes, Some(1020));
    assert!(sessions[0].note.is_none());
    assert_eq!(tier(&sessions[0]), "punch_type");
    assert_eq!(algorithm(&sessions[0]), "shift_aware");
}

#[test]
fn tier1_multiple_checkins_takes_first() {
    // Employee badges in multiple times (gate + office door)
    let logs = vec![
        log(475, Some(0)),  // Gate badge 07:55
        log(478, Some(0)),  // Office badge 07:58
        log(1020, Some(1)), // Check-Out 17:00
    ];

    let sessions = strategy().pair(&logs, None);
    assert_eq!(sessions[0].start_minutes, 475); // Earliest IN
    assert_eq!(sessions[0].end_minutes, Some(1020));
    assert_eq!(tier(&sessions[0]), "punch_type");
}

#[test]
fn tier1_multiple_checkouts_takes_last() {
    let logs = vec![
        log(480, Some(0)),  // Check-In
        log(1015, Some(1)), // First out (office door)
        log(1020, Some(1)), // Last out (gate badge)
    ];

    let sessions = strategy().pair(&logs, None);
    assert_eq!(sessions[0].end_minutes, Some(1020)); // Latest OUT
}

#[test]
fn tier1_checkout_before_checkin_no_pair() {
    // Edge case: checkout time <= checkin time
    let logs = vec![
        log(1020, Some(0)), // Check-In at 17:00
        log(480, Some(1)),  // "Check-Out" at 08:00 (before IN)
    ];

    let sessions = strategy().pair(&logs, None);
    // end should be None because check-out (480) < check-in (1020)
    assert!(sessions[0].end_minutes.is_none());
}

#[test]
fn tier1_only_checkin_no_checkout_falls_back() {
    // Device only sent Check-In, no Check-Out → falls back to anchor_first
    let logs = vec![log(480, Some(0))];

    let sessions = strategy().pair(&logs, None);
    assert_eq!(sessions[0].start_minutes, 480);
    assert!(sessions[0].end_minutes.is_none());
    assert_eq!(tier(&sessions[0]), "anchor_first"); // Fell back
    assert_eq!(sessions[0].note.as_deref(), Some("Missing Punch Out"));
}

#[test]
fn tier1_priority_over_shift_config() {
    // Both punch_type AND shift config provided → punch_type wins
    let shift = day_shift();
    let logs = vec![
        log(490, Some(0)),  // Late IN
        log(1010, Some(1)), // Early OUT
    ];

    let sessions = strategy().pair(&logs, Some(&shift));
    assert_eq!(sessions[0].start_minutes, 490);
    assert_eq!(sessions[0].end_minutes, Some(1010));
    assert_eq!(tier(&sessions[0]), "punch_type"); // Punch type wins over shift
}

// ═══════════════════════════════════════════════════════════════
//  Tier 2: Shift Window
// ═══════════════════════════════════════════════════════════════

#[test]
fn tier2_normal_day_within_window() {
    let shift = day_shift();
    let logs = vec![
        log(475, None),  // 07:55 — in check-in window
        log(725, None),  // Mid-day
        log(1025, None), // 17:05 — in check-out window
    ];

    let sessions = strategy().pair(&logs, Some(&shift));
    assert_eq!(sessions[0].start_minutes, 475);
    assert_eq!(sessions[0].end_minutes, Some(1025));
    assert_eq!(tier(&sessions[0]), "shift_window");
}

#[test]
fn tier2_early_arrival() {
    let shift = day_shift();
    // Employee arrives 90 min early — still within 120 min buffer
    let logs = vec![
        log(390, None), // 06:30 (480 - 90 = 390, within buffer)
        log(1020, None),
    ];

    let sessions = strategy().pair(&logs, Some(&shift));
    assert_eq!(sessions[0].start_minutes, 390);
    assert_eq!(tier(&sessions[0]), "shift_window");
}

#[test]
fn tier2_late_departure() {
    let shift = day_shift();
    // Employee leaves 60 min after shift end — within buffer
    let logs = vec![
        log(480, None),
        log(1080, None), // 18:00 (1020 + 60, within buffer)
    ];

    let sessions = strategy().pair(&logs, Some(&shift));
    assert_eq!(sessions[0].end_minutes, Some(1080));
    assert_eq!(tier(&sessions[0]), "shift_window");
}

#[test]
fn tier2_boundary_exact_buffer_edge() {
    let shift = day_shift();
    // Exactly at buffer edge (120 min)
    let logs = vec![
        log(360, None), // 06:00 = 480 - 120 (exact edge)
        log(1140, None), // 19:00 = 1020 + 120 (exact edge)
    ];

    let sessions = strategy().pair(&logs, Some(&shift));
    assert_eq!(sessions[0].start_minutes, 360);
    assert_eq!(sessions[0].end_minutes, Some(1140));
    assert_eq!(tier(&sessions[0]), "shift_window");
}

#[test]
fn tier2_outside_buffer_falls_to_anchor() {
    let shift = day_shift();
    // All logs way outside shift windows
    let logs = vec![
        log(200, None), // 03:20 — far from 08:00 ± 2h
        log(300, None), // 05:00 — still outside
    ];

    let sessions = strategy().pair(&logs, Some(&shift));
    assert_eq!(tier(&sessions[0]), "anchor_first"); // Fell back
}

#[test]
fn tier2_only_checkin_in_window() {
    let shift = day_shift();
    // Check-in log in window, but no log near check-out window
    let logs = vec![
        log(480, None), // In check-in window
        log(600, None), // 10:00 — not in check-out window (900-1140)
    ];

    let sessions = strategy().pair(&logs, Some(&shift));
    assert_eq!(sessions[0].start_minutes, 480);
    // No log in check-out window → end is None (missing punch out)
    assert!(sessions[0].end_minutes.is_none());
    assert_eq!(tier(&sessions[0]), "shift_window");
}

#[test]
fn tier2_overnight_shift() {
    let shift = overnight_shift();
    let logs = vec![
        log(1315, None), // 21:55 — near shift start 22:00
        log(1805, None), // 06:05 — near shift end 06:00 next day
    ];

    let sessions = strategy().pair(&logs, Some(&shift));
    assert_eq!(sessions[0].start_minutes, 1315);
    assert_eq!(sessions[0].end_minutes, Some(1805));
    assert_eq!(tier(&sessions[0]), "shift_window");
}

#[test]
fn tier2_early_morning_shift() {
    let shift = early_morning_shift();
    let logs = vec![
        log(355, None), // 05:55 — near shift start 06:00
        log(845, None), // 14:05 — near shift end 14:00
    ];

    let sessions = strategy().pair(&logs, Some(&shift));
    assert_eq!(sessions[0].start_minutes, 355);
    assert_eq!(sessions[0].end_minutes, Some(845));
    assert_eq!(tier(&sessions[0]), "shift_window");
}

// ═══════════════════════════════════════════════════════════════
//  Tier 3: Anchor First (fallback)
// ═══════════════════════════════════════════════════════════════

#[test]
fn tier3_basic_pair() {
    let logs = vec![
        log(480, None),
        log(1020, None),
    ];

    let sessions = strategy().pair(&logs, None);
    assert_eq!(sessions[0].start_minutes, 480);
    assert_eq!(sessions[0].end_minutes, Some(1020));
    assert_eq!(tier(&sessions[0]), "anchor_first");
}

#[test]
fn tier3_single_log_missing_out() {
    let sessions = strategy().pair(&[log(480, None)], None);
    assert_eq!(sessions.len(), 1);
    assert!(sessions[0].end_minutes.is_none());
    assert_eq!(sessions[0].note.as_deref(), Some("Missing Punch Out"));
}

#[test]
fn tier3_16h_limit_within() {
    // 15h gap (900 min) — should pair (under 960 min limit)
    let logs = vec![
        log(60, None),  // 01:00
        log(960, None), // 16:00 (900 min gap)
    ];

    let sessions = strategy().pair(&logs, None);
    assert_eq!(sessions[0].end_minutes, Some(960));
}

#[test]
fn tier3_16h_limit_exactly_at() {
    // Exactly 16h gap (960 min) — should pair (≤ limit)
    let logs = vec![
        log(0, None),
        log(960, None),
    ];

    let sessions = strategy().pair(&logs, None);
    assert_eq!(sessions[0].end_minutes, Some(960));
}

#[test]
fn tier3_16h_limit_exceeded() {
    // 17h gap (1020 min) — should NOT pair
    let logs = vec![
        log(0, None),
        log(1020, None),
    ];

    let sessions = strategy().pair(&logs, None);
    assert!(sessions[0].end_minutes.is_none());
}

#[test]
fn tier3_many_logs_picks_last_within_limit() {
    let logs = vec![
        log(480, None), // 08:00
        log(600, None), // 10:00
        log(720, None), // 12:00
        log(900, None), // 15:00
        log(1020, None), // 17:00 — 540 min gap, within limit
    ];

    let sessions = strategy().pair(&logs, None);
    assert_eq!(sessions[0].start_minutes, 480);
    assert_eq!(sessions[0].end_minutes, Some(1020));
}

// ═══════════════════════════════════════════════════════════════
//  Edge Cases
// ═══════════════════════════════════════════════════════════════

#[test]
fn empty_logs_returns_empty() {
    let sessions = strategy().pair(&[], None);
    assert!(sessions.is_empty());
}

#[test]
fn empty_logs_with_shift_returns_empty() {
    let sessions = strategy().pair(&[], Some(&day_shift()));
    assert!(sessions.is_empty());
}

#[test]
fn all_punch_type_none_uses_shift_or_anchor() {
    // No punch_type on any log + shift provided → tier 2
    let shift = day_shift();
    let logs = vec![
        log(480, None),
        log(1020, None),
    ];
    let sessions = strategy().pair(&logs, Some(&shift));
    assert_eq!(tier(&sessions[0]), "shift_window");
}

#[test]
fn mixed_punch_types_without_both_in_out_uses_shift() {
    // Has punch_type but not both IN(0) and OUT(1) → falls through to shift/anchor
    let shift = day_shift();
    let logs = vec![
        log(480, Some(2)), // Break-In only
        log(780, Some(3)), // Break-Out only
    ];
    // has_valid_punch_types requires both type=0 AND type=1
    let sessions = strategy().pair(&logs, Some(&shift));
    // Falls to shift_window since logs are in shift windows
    assert_eq!(tier(&sessions[0]), "shift_window");
}

#[test]
fn metadata_always_has_algorithm_and_tier() {
    // All tiers should set algorithm and tier in metadata
    let s = strategy();

    // Tier 1
    let t1 = s.pair(&[log(480, Some(0)), log(1020, Some(1))], None);
    assert_eq!(algorithm(&t1[0]), "shift_aware");
    assert_eq!(tier(&t1[0]), "punch_type");

    // Tier 2
    let t2 = s.pair(&[log(480, None), log(1020, None)], Some(&day_shift()));
    assert_eq!(algorithm(&t2[0]), "shift_aware");
    assert_eq!(tier(&t2[0]), "shift_window");

    // Tier 3
    let t3 = s.pair(&[log(480, None), log(1020, None)], None);
    assert_eq!(algorithm(&t3[0]), "shift_aware");
    assert_eq!(tier(&t3[0]), "anchor_first");
}

#[test]
fn metadata_log_ids_are_present() {
    let logs = vec![
        log(480, Some(0)),
        log(1020, Some(1)),
    ];
    let sessions = strategy().pair(&logs, None);
    let meta = sessions[0].metadata.as_ref().unwrap();
    let log_ids = meta["log_ids"].as_array().unwrap();
    assert_eq!(log_ids.len(), 2);
    // IDs should match the input logs
    assert_eq!(log_ids[0].as_str().unwrap(), logs[0].id.to_string());
    assert_eq!(log_ids[1].as_str().unwrap(), logs[1].id.to_string());
}

#[test]
fn algorithm_name_is_shift_aware() {
    assert_eq!(strategy().algorithm_name(), "shift_aware");
}

#[test]
fn entry_type_always_work() {
    // All tiers should set entry_type = "work"
    let s = strategy();
    let t1 = s.pair(&[log(480, Some(0)), log(1020, Some(1))], None);
    let t2 = s.pair(&[log(480, None), log(1020, None)], Some(&day_shift()));
    let t3 = s.pair(&[log(480, None), log(1020, None)], None);

    assert_eq!(t1[0].entry_type, "work");
    assert_eq!(t2[0].entry_type, "work");
    assert_eq!(t3[0].entry_type, "work");
}
