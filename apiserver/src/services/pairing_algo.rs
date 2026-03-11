use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq)]
pub struct PairingShiftConfig {
    // All times in minutes from midnight (00:00) of the shift start day.
    // Values can be > 1440 (24h) for overnight shifts.
    pub start_minutes: i32,
    pub end_minutes: i32,
    pub break_start_minutes: Option<i32>,
    pub break_end_minutes: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PairedSession {
    // Minutes from midnight of the analysis date.
    // Can be > 1440 (representing next day).
    pub start_minutes: i32,
    pub end_minutes: Option<i32>,
    pub entry_type: String, // "work"
    pub note: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LogEvent {
    pub id: uuid::Uuid,
    pub minutes: i32,
    pub punch_type: Option<i32>, // 0: Check-In, 1: Check-Out, 2: Break-In, 3: Break-Out, 4: Overtime-In, 5: Overtime-Out
}

const ALGORITHM_NAME: &str = "shift_aware";

/// Maximum gap (in minutes) between first and last log to consider them a single session.
/// 16 hours = 960 minutes.
const MAX_SESSION_GAP_MINUTES: i32 = 960;

/// Buffer around shift start/end times (in minutes) for matching logs to shift windows.
const SHIFT_WINDOW_BUFFER: i32 = 120;

/// Returns the single shift-aware pairing strategy.
///
/// This strategy auto-adapts based on available context:
/// 1. If logs have explicit punch_type (device supports IN/OUT) → use directly
/// 2. If a shift config is provided → pair using shift time windows (industry standard)
/// 3. Fallback → anchor-first (first log = IN, last within 16h = OUT)
pub fn get_strategy() -> ShiftAwareStrategy {
    ShiftAwareStrategy
}

pub struct ShiftAwareStrategy;

impl ShiftAwareStrategy {
    pub fn algorithm_name(&self) -> &'static str {
        ALGORITHM_NAME
    }

    pub fn pair(&self, logs: &[LogEvent], config: Option<&PairingShiftConfig>) -> Vec<PairedSession> {
        if logs.is_empty() {
            return vec![];
        }

        // Tier 1: If logs have explicit punch_type, use punch-type-based pairing
        if self.has_valid_punch_types(logs) {
            return self.pair_by_punch_type(logs);
        }

        // Tier 2: If shift config available, use shift-window-based pairing
        if let Some(shift) = config {
            return self.pair_by_shift_window(logs, shift);
        }

        // Tier 3: Fallback to anchor-first (original behavior)
        self.pair_anchor_first(logs)
    }

    /// Check if logs have meaningful punch_type data (at least one IN and optionally OUT).
    fn has_valid_punch_types(&self, logs: &[LogEvent]) -> bool {
        let has_check_in = logs.iter().any(|l| l.punch_type == Some(0));
        let has_check_out = logs.iter().any(|l| l.punch_type == Some(1));
        // Need at least a check-in; having both IN and OUT is ideal
        has_check_in && has_check_out
    }

    /// Tier 1: Pair using explicit punch_type from device.
    /// Finds the earliest Check-In (type=0) and latest Check-Out (type=1).
    fn pair_by_punch_type(&self, logs: &[LogEvent]) -> Vec<PairedSession> {
        let first_in = logs.iter().find(|l| l.punch_type == Some(0));
        let last_out = logs.iter().rev().find(|l| l.punch_type == Some(1));

        let (start, start_id) = match first_in {
            Some(log) => (log.minutes, log.id),
            None => return self.pair_anchor_first(logs), // fallback
        };

        let (end, end_id) = match last_out {
            Some(log) if log.minutes > start => (Some(log.minutes), Some(log.id)),
            _ => (None, None),
        };

        let mut log_ids = vec![start_id];
        if let Some(eid) = end_id {
            log_ids.push(eid);
        }

        vec![PairedSession {
            start_minutes: start,
            end_minutes: end,
            entry_type: "work".to_string(),
            note: if end.is_none() {
                Some("Missing Punch Out".to_string())
            } else {
                None
            },
            metadata: Some(serde_json::json!({
                "algorithm": ALGORITHM_NAME,
                "tier": "punch_type",
                "log_ids": log_ids
            })),
        }]
    }

    /// Tier 2: Pair using shift time windows (industry standard, like ZKBioTime).
    /// Earliest log near shift start = IN, latest log near shift end = OUT.
    fn pair_by_shift_window(&self, logs: &[LogEvent], shift: &PairingShiftConfig) -> Vec<PairedSession> {
        let check_in_window_start = shift.start_minutes - SHIFT_WINDOW_BUFFER;
        let check_in_window_end = shift.start_minutes + SHIFT_WINDOW_BUFFER;
        let check_out_window_start = shift.end_minutes - SHIFT_WINDOW_BUFFER;
        let check_out_window_end = shift.end_minutes + SHIFT_WINDOW_BUFFER;

        // Find earliest log in check-in window
        let first_in = logs.iter().find(|l| {
            l.minutes >= check_in_window_start && l.minutes <= check_in_window_end
        });

        // Find latest log in check-out window
        let last_out = logs.iter().rev().find(|l| {
            l.minutes >= check_out_window_start && l.minutes <= check_out_window_end
        });

        // If we can't find logs in shift windows, fall back to anchor-first
        let (start, start_id) = match first_in {
            Some(log) => (log.minutes, log.id),
            None => return self.pair_anchor_first(logs),
        };

        let (end, end_id) = match last_out {
            Some(log) if log.id != start_id => (Some(log.minutes), Some(log.id)),
            _ => (None, None),
        };

        let mut log_ids = vec![start_id];
        if let Some(eid) = end_id {
            log_ids.push(eid);
        }

        vec![PairedSession {
            start_minutes: start,
            end_minutes: end,
            entry_type: "work".to_string(),
            note: if end.is_none() {
                Some("Missing Punch Out".to_string())
            } else {
                None
            },
            metadata: Some(serde_json::json!({
                "algorithm": ALGORITHM_NAME,
                "tier": "shift_window",
                "log_ids": log_ids,
                "shift_start": shift.start_minutes,
                "shift_end": shift.end_minutes
            })),
        }]
    }

    /// Tier 3: Anchor-first fallback (original behavior).
    /// First log = IN, furthest log within 16h = OUT.
    fn pair_anchor_first(&self, logs: &[LogEvent]) -> Vec<PairedSession> {
        let first = &logs[0];
        let mut last_idx = 0;

        for (i, log) in logs.iter().enumerate().skip(1) {
            if log.minutes - first.minutes <= MAX_SESSION_GAP_MINUTES {
                last_idx = i;
            } else {
                break;
            }
        }

        let start = first.minutes;
        let end = if last_idx > 0 {
            Some(logs[last_idx].minutes)
        } else {
            None
        };

        let mut log_ids = vec![first.id];
        if last_idx > 0 {
            log_ids.push(logs[last_idx].id);
        }

        vec![PairedSession {
            start_minutes: start,
            end_minutes: end,
            entry_type: "work".to_string(),
            note: if end.is_none() {
                Some("Missing Punch Out".to_string())
            } else {
                None
            },
            metadata: Some(serde_json::json!({
                "algorithm": ALGORITHM_NAME,
                "tier": "anchor_first",
                "log_ids": log_ids
            })),
        }]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn make_log(minutes: i32, punch_type: Option<i32>) -> LogEvent {
        LogEvent {
            id: Uuid::new_v4(),
            minutes,
            punch_type,
        }
    }

    fn standard_shift() -> PairingShiftConfig {
        PairingShiftConfig {
            start_minutes: 480,  // 08:00
            end_minutes: 1020,   // 17:00
            break_start_minutes: Some(720),  // 12:00
            break_end_minutes: Some(780),    // 13:00
        }
    }

    // --- Tier 1: Punch Type tests ---

    #[test]
    fn test_punch_type_perfect_day() {
        let strategy = get_strategy();
        let logs = vec![
            make_log(480, Some(0)),  // 08:00 Check-In
            make_log(720, Some(2)),  // 12:00 Break-In
            make_log(780, Some(3)),  // 13:00 Break-Out
            make_log(1020, Some(1)), // 17:00 Check-Out
        ];

        let sessions = strategy.pair(&logs, None);
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].start_minutes, 480);
        assert_eq!(sessions[0].end_minutes, Some(1020));
        assert!(sessions[0].note.is_none());

        let meta = sessions[0].metadata.as_ref().unwrap();
        assert_eq!(meta["tier"], "punch_type");
    }

    #[test]
    fn test_punch_type_missing_checkout() {
        let strategy = get_strategy();
        // Only Check-In, no Check-Out — should fallback to anchor_first
        let logs = vec![
            make_log(480, Some(0)),
        ];

        let sessions = strategy.pair(&logs, None);
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].start_minutes, 480);
        assert!(sessions[0].end_minutes.is_none());

        let meta = sessions[0].metadata.as_ref().unwrap();
        assert_eq!(meta["tier"], "anchor_first"); // Fell back
    }

    // --- Tier 2: Shift Window tests ---

    #[test]
    fn test_shift_window_normal_day() {
        let strategy = get_strategy();
        let shift = standard_shift();
        // Logs without punch_type, but within shift windows
        let logs = vec![
            make_log(475, None),  // 07:55 — within check-in window (480 ± 120)
            make_log(725, None),  // 12:05 — break area
            make_log(775, None),  // 12:55 — break area
            make_log(1025, None), // 17:05 — within check-out window (1020 ± 120)
        ];

        let sessions = strategy.pair(&logs, Some(&shift));
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].start_minutes, 475);
        assert_eq!(sessions[0].end_minutes, Some(1025));

        let meta = sessions[0].metadata.as_ref().unwrap();
        assert_eq!(meta["tier"], "shift_window");
    }

    #[test]
    fn test_shift_window_logs_outside_window_fallback() {
        let strategy = get_strategy();
        let shift = standard_shift();
        // All logs far from shift windows — should fallback to anchor_first
        let logs = vec![
            make_log(200, None),  // 03:20 — way before shift
            make_log(300, None),  // 05:00 — still before window
        ];

        let sessions = strategy.pair(&logs, Some(&shift));
        assert_eq!(sessions.len(), 1);

        let meta = sessions[0].metadata.as_ref().unwrap();
        assert_eq!(meta["tier"], "anchor_first"); // Fell back
    }

    // --- Tier 3: Anchor First tests ---

    #[test]
    fn test_anchor_first_perfect_day() {
        let strategy = get_strategy();
        let logs = vec![
            make_log(480, None),
            make_log(1020, None),
        ];

        let sessions = strategy.pair(&logs, None);
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].start_minutes, 480);
        assert_eq!(sessions[0].end_minutes, Some(1020));

        let meta = sessions[0].metadata.as_ref().unwrap();
        assert_eq!(meta["tier"], "anchor_first");
    }

    #[test]
    fn test_anchor_first_missing_out() {
        let strategy = get_strategy();
        let logs = vec![make_log(480, None)];

        let sessions = strategy.pair(&logs, None);
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].start_minutes, 480);
        assert!(sessions[0].end_minutes.is_none());
        assert_eq!(sessions[0].note.as_deref(), Some("Missing Punch Out"));
    }

    #[test]
    fn test_anchor_first_16h_limit() {
        let strategy = get_strategy();
        // Log 2: 01:00 next day (1500) -> 17h gap (1020 mins). Should NOT pair.
        let logs = vec![
            make_log(480, None),
            make_log(1500, None),
        ];

        let sessions = strategy.pair(&logs, None);
        assert_eq!(sessions.len(), 1);
        assert!(sessions[0].end_minutes.is_none());
    }

    #[test]
    fn test_empty_logs() {
        let strategy = get_strategy();
        let sessions = strategy.pair(&[], None);
        assert!(sessions.is_empty());
    }

    // --- Priority tests ---

    #[test]
    fn test_punch_type_takes_priority_over_shift() {
        let strategy = get_strategy();
        let shift = standard_shift();
        // Logs WITH punch_type AND shift config — punch_type should win
        let logs = vec![
            make_log(490, Some(0)),  // Check-In slightly late
            make_log(1010, Some(1)), // Check-Out slightly early
        ];

        let sessions = strategy.pair(&logs, Some(&shift));
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].start_minutes, 490);
        assert_eq!(sessions[0].end_minutes, Some(1010));

        let meta = sessions[0].metadata.as_ref().unwrap();
        assert_eq!(meta["tier"], "punch_type"); // Punch type wins
    }

    // =====================================================================
    // Overnight shift windows
    // =====================================================================

    #[test]
    fn test_shift_window_overnight_shift() {
        let strategy = get_strategy();
        let shift = PairingShiftConfig {
            start_minutes: 1320, // 22:00
            end_minutes: 1800,   // 06:00 next day (30h from midnight)
            break_start_minutes: None,
            break_end_minutes: None,
        };

        let logs = vec![
            make_log(1310, None), // 21:50 — within check-in window
            make_log(1810, None), // 06:10 next day — within check-out window
        ];

        let sessions = strategy.pair(&logs, Some(&shift));
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].start_minutes, 1310);
        assert_eq!(sessions[0].end_minutes, Some(1810));

        let meta = sessions[0].metadata.as_ref().unwrap();
        assert_eq!(meta["tier"], "shift_window");
    }

    // =====================================================================
    // Single log edge cases
    // =====================================================================

    #[test]
    fn test_single_log_no_config() {
        let strategy = get_strategy();
        let logs = vec![make_log(480, None)];

        let sessions = strategy.pair(&logs, None);
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].start_minutes, 480);
        assert!(sessions[0].end_minutes.is_none());
        assert_eq!(sessions[0].note.as_deref(), Some("Missing Punch Out"));
    }

    #[test]
    fn test_single_log_with_shift_config() {
        let strategy = get_strategy();
        let shift = standard_shift();
        let logs = vec![make_log(480, None)];

        let sessions = strategy.pair(&logs, Some(&shift));
        assert_eq!(sessions.len(), 1);
        // Only one log in check-in window, no check-out → Missing Punch Out
        assert!(sessions[0].end_minutes.is_none());
    }

    // =====================================================================
    // Anchor-first boundary (exactly at 16h gap)
    // =====================================================================

    #[test]
    fn test_anchor_first_exactly_at_16h_gap() {
        let strategy = get_strategy();
        // First log at 480 (08:00), second at 1440 (24:00) = 960 min gap (=16h)
        let logs = vec![
            make_log(480, None),
            make_log(1440, None), // exactly MAX_SESSION_GAP_MINUTES away
        ];

        let sessions = strategy.pair(&logs, None);
        assert_eq!(sessions.len(), 1);
        // 960 min gap = MAX, should still pair
        assert_eq!(sessions[0].end_minutes, Some(1440));
    }

    #[test]
    fn test_anchor_first_one_minute_over_16h_gap() {
        let strategy = get_strategy();
        let logs = vec![
            make_log(480, None),
            make_log(1441, None), // 961 min gap > MAX_SESSION_GAP_MINUTES (960)
        ];

        let sessions = strategy.pair(&logs, None);
        assert_eq!(sessions.len(), 1);
        // Over 16h gap → should NOT pair
        assert!(sessions[0].end_minutes.is_none());
    }

    // =====================================================================
    // Multiple logs in check-in window picks earliest
    // =====================================================================

    #[test]
    fn test_shift_window_earliest_checkin() {
        let strategy = get_strategy();
        let shift = standard_shift();
        // Multiple logs near shift start — should pick the earliest
        let logs = vec![
            make_log(470, None),  // 07:50
            make_log(485, None),  // 08:05 — also in window
            make_log(1020, None), // 17:00 — check-out
        ];

        let sessions = strategy.pair(&logs, Some(&shift));
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].start_minutes, 470); // earliest in window
        assert_eq!(sessions[0].end_minutes, Some(1020));
    }

    // =====================================================================
    // Punch type: checkout before checkin is invalid
    // =====================================================================

    #[test]
    fn test_punch_type_checkout_before_checkin() {
        let strategy = get_strategy();
        let logs = vec![
            make_log(480, Some(0)),   // Check-In at 08:00
            make_log(400, Some(1)),   // Check-Out at 06:40 (before check-in!)
        ];

        let sessions = strategy.pair(&logs, None);
        assert_eq!(sessions.len(), 1);
        // Check-out time < start → should be treated as no valid checkout
        assert!(sessions[0].end_minutes.is_none());
    }

    // =====================================================================
    // Algorithm name
    // =====================================================================

    #[test]
    fn test_algorithm_name() {
        let strategy = get_strategy();
        assert_eq!(strategy.algorithm_name(), "shift_aware");
    }

    // =====================================================================
    // PairedSession serialization
    // =====================================================================

    #[test]
    fn test_paired_session_serde_roundtrip() {
        let session = PairedSession {
            start_minutes: 480,
            end_minutes: Some(1020),
            entry_type: "work".to_string(),
            note: None,
            metadata: Some(serde_json::json!({"tier": "test"})),
        };
        let json = serde_json::to_string(&session).unwrap();
        let parsed: PairedSession = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, session);
    }
}

