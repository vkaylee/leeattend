//! Comprehensive Pairing Strategy integration tests
//!
//! Tests the `ShiftAwareStrategy` across both tiers:
//!   Tier 1: shift_window-based pairing (shift schedule context)
//!   Tier 2: `anchor_first` fallback (no context)

use apiserver::services::pairing_algo::{
    self, EntryType, LogEvent, PairedSession, PairingNote, PairingShiftConfig, ShiftAwareStrategy,
};
use uuid::Uuid;

// ═══════════════════════════════════════════════════════════════
//  Helpers
// ═══════════════════════════════════════════════════════════════

fn strategy() -> ShiftAwareStrategy {
    pairing_algo::get_strategy()
}

fn log(minutes: i32) -> LogEvent {
    let id = Uuid::new_v4();
    LogEvent::new(id, minutes, None, vec![id])
}

fn day_shift() -> PairingShiftConfig {
    PairingShiftConfig::new(
        480,  // 08:00
        1020, // 17:00
        Some(720),
        Some(780),
        None,
        None,
    )
}

fn overnight_shift() -> PairingShiftConfig {
    PairingShiftConfig::new(
        1320,       // 22:00
        1800,       // 06:00 next day (30h from midnight = 1800)
        Some(1560), // 02:00
        Some(1590), // 02:30
        None,
        None,
    )
}

fn early_morning_shift() -> PairingShiftConfig {
    PairingShiftConfig::new(
        360, // 06:00
        840, // 14:00
        None, None, None, None,
    )
}

fn tier(session: &PairedSession) -> String {
    session
        .metadata()
        .as_ref()
        .and_then(|m| m["tier"].as_str())
        .unwrap_or("unknown")
        .to_string()
}

fn algorithm(session: &PairedSession) -> String {
    session
        .metadata()
        .as_ref()
        .and_then(|m| m["algorithm"].as_str())
        .unwrap_or("unknown")
        .to_string()
}

// ═══════════════════════════════════════════════════════════════
//  Tier 2: Shift Window
// ═══════════════════════════════════════════════════════════════

#[test]
fn tier2_normal_day_within_window() {
    let shift = day_shift();
    let logs = vec![
        log(475),  // 07:55 — in check-in window
        log(725),  // Mid-day
        log(1025), // 17:05 — in check-out window
    ];

    let sessions = strategy().pair(&logs, &[shift]);
    assert_eq!(sessions[0].start_minutes(), 475);
    assert_eq!(sessions[0].end_minutes(), Some(1025));
    assert_eq!(tier(&sessions[0]), "shift_window");
}

#[test]
fn tier2_early_arrival() {
    let shift = day_shift();
    // Employee arrives 90 min early — still within 120 min buffer
    let logs = vec![
        log(390), // 06:30 (480 - 90 = 390, within buffer)
        log(1020),
    ];

    let sessions = strategy().pair(&logs, &[shift]);
    assert_eq!(sessions[0].start_minutes(), 390);
    assert_eq!(tier(&sessions[0]), "shift_window");
}

#[test]
fn tier2_late_departure() {
    let shift = day_shift();
    // Employee leaves 60 min after shift end — within buffer
    let logs = vec![
        log(480),
        log(1080), // 18:00 (1020 + 60, within buffer)
    ];

    let sessions = strategy().pair(&logs, &[shift]);
    assert_eq!(sessions[0].end_minutes(), Some(1080));
    assert_eq!(tier(&sessions[0]), "shift_window");
}

#[test]
fn tier2_boundary_exact_buffer_edge() {
    let shift = day_shift();
    // Exactly at buffer edge (120 min)
    let logs = vec![
        log(360),  // 06:00 = 480 - 120 (exact edge)
        log(1140), // 19:00 = 1020 + 120 (exact edge)
    ];

    let sessions = strategy().pair(&logs, &[shift]);
    assert_eq!(sessions[0].start_minutes(), 360);
    assert_eq!(sessions[0].end_minutes(), Some(1140));
    assert_eq!(tier(&sessions[0]), "shift_window");
}

#[test]
fn tier2_outside_buffer_falls_to_anchor() {
    let shift = day_shift();
    // All logs way outside shift windows
    let logs = vec![
        log(200), // 03:20 — far from 08:00 ± 2h
        log(300), // 05:00 — still outside
    ];

    let sessions = strategy().pair(&logs, &[shift]);
    assert_eq!(tier(&sessions[0]), "anchor_first"); // Fell back
}

#[test]
fn tier2_only_checkin_in_window() {
    let shift = day_shift();
    // Check-in log in window, second log is NOT near shift end
    // but the algorithm uses "Last Out of day" (no checkout window),
    // so 600 (10:00) is the last log and becomes the checkout.
    let logs = vec![
        log(480), // In check-in window
        log(600), // 10:00 — last log of day
    ];

    let sessions = strategy().pair(&logs, &[shift]);
    assert_eq!(sessions[0].start_minutes(), 480);
    // "Last Out of day" design: any log after check-in counts as checkout
    assert_eq!(sessions[0].end_minutes(), Some(600));
    assert_eq!(tier(&sessions[0]), "shift_window");
}

#[test]
fn tier2_overnight_shift() {
    let shift = overnight_shift();
    let logs = vec![
        log(1315), // 21:55 — near shift start 22:00
        log(1805), // 06:05 — near shift end 06:00 next day
    ];

    let sessions = strategy().pair(&logs, &[shift]);
    assert_eq!(sessions[0].start_minutes(), 1315);
    assert_eq!(sessions[0].end_minutes(), Some(1805));
    assert_eq!(tier(&sessions[0]), "shift_window");
}

#[test]
fn tier2_early_morning_shift() {
    let shift = early_morning_shift();
    let logs = vec![
        log(355), // 05:55 — near shift start 06:00
        log(845), // 14:05 — near shift end 14:00
    ];

    let sessions = strategy().pair(&logs, &[shift]);
    assert_eq!(sessions[0].start_minutes(), 355);
    assert_eq!(sessions[0].end_minutes(), Some(845));
    assert_eq!(tier(&sessions[0]), "shift_window");
}

// ═══════════════════════════════════════════════════════════════
//  Tier 3: Anchor First (fallback)
// ═══════════════════════════════════════════════════════════════

#[test]
fn tier3_basic_pair() {
    let logs = vec![log(480), log(1020)];

    let sessions = strategy().pair(&logs, &[]);
    assert_eq!(sessions[0].start_minutes(), 480);
    assert_eq!(sessions[0].end_minutes(), Some(1020));
    assert_eq!(tier(&sessions[0]), "anchor_first");
}

#[test]
fn tier3_single_log_missing_out() {
    let sessions = strategy().pair(&[log(480)], &[]);
    assert_eq!(sessions.len(), 1);
    assert!(sessions[0].end_minutes().is_none());
    assert_eq!(sessions[0].note(), Some(&PairingNote::MissingPunchOut));
}

#[test]
fn tier3_16h_limit_within() {
    // 15h gap (900 min) — should pair (under 960 min limit)
    let logs = vec![
        log(60),  // 01:00
        log(960), // 16:00 (900 min gap)
    ];

    let sessions = strategy().pair(&logs, &[]);
    assert_eq!(sessions[0].end_minutes(), Some(960));
}

#[test]
fn tier3_16h_limit_exactly_at() {
    // Exactly 16h gap (960 min) — should pair (≤ limit)
    let logs = vec![log(0), log(960)];

    let sessions = strategy().pair(&logs, &[]);
    assert_eq!(sessions[0].end_minutes(), Some(960));
}

#[test]
fn tier3_16h_limit_exceeded() {
    // 17h gap (1020 min) — should NOT pair
    let logs = vec![log(0), log(1020)];

    let sessions = strategy().pair(&logs, &[]);
    assert_eq!(sessions[0].end_minutes().is_none(), true);
}

#[test]
fn tier3_many_logs_picks_last_within_limit() {
    let logs = vec![
        log(480),  // 08:00
        log(600),  // 10:00
        log(720),  // 12:00
        log(900),  // 15:00
        log(1020), // 17:00 — 540 min gap, within limit
    ];

    let sessions = strategy().pair(&logs, &[]);
    assert_eq!(sessions[0].start_minutes(), 480);
    assert_eq!(sessions[0].end_minutes(), Some(1020));
}

// ═══════════════════════════════════════════════════════════════
//  Edge Cases
// ═══════════════════════════════════════════════════════════════

#[test]
fn empty_logs_returns_empty() {
    let sessions = strategy().pair(&[], &[]);
    assert!(sessions.is_empty());
}

#[test]
fn empty_logs_with_shift_returns_empty() {
    let sessions = strategy().pair(&[], &[day_shift()]);
    assert!(sessions.is_empty());
}

#[test]
fn metadata_always_has_algorithm_and_tier() {
    // All tiers should set algorithm and tier in metadata
    let s = strategy();

    // Tier 2
    let t2 = s.pair(&[log(480), log(1020)], &[day_shift()]);
    assert_eq!(algorithm(&t2[0]), "shift_aware");
    assert_eq!(tier(&t2[0]), "shift_window");

    // Tier 3
    let t3 = s.pair(&[log(480), log(1020)], &[]);
    assert_eq!(algorithm(&t3[0]), "shift_aware");
    assert_eq!(tier(&t3[0]), "anchor_first");
}

#[test]
fn metadata_log_ids_are_present() {
    let logs = vec![log(480), log(1020)];
    let sessions = strategy().pair(&logs, &[]);
    let meta = sessions[0].metadata().unwrap();
    let log_ids = meta["log_ids"].as_array().unwrap();
    assert_eq!(log_ids.len(), 2);
    // IDs should match the input logs
    assert_eq!(log_ids[0].as_str().unwrap(), logs[0].id().to_string());
    assert_eq!(log_ids[1].as_str().unwrap(), logs[1].id().to_string());
}

#[test]
fn algorithm_name_is_shift_aware() {
    assert_eq!(strategy().algorithm_name(), "shift_aware");
}

#[test]
fn entry_type_always_work() {
    // All tiers should set entry_type = "work"
    let s = strategy();
    let t2 = s.pair(&[log(480), log(1020)], &[day_shift()]);
    let t3 = s.pair(&[log(480), log(1020)], &[]);

    assert_eq!(*t2[0].entry_type(), EntryType::Work);
    assert_eq!(*t3[0].entry_type(), EntryType::Work);
}
