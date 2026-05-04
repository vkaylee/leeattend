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
    pub entry_type: String,
    pub note: Option<String>,
    pub is_break_overtime: bool,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LogEvent {
    pub id: uuid::Uuid,
    pub minutes: i32,
    pub raw_data: Option<serde_json::Value>,
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
/// 1. If a shift config is provided → "First In near shift start, Last Out of day"
/// 2. Fallback → anchor-first (first log = IN, last within 16h = OUT)
pub fn get_strategy() -> ShiftAwareStrategy {
    ShiftAwareStrategy
}

pub struct ShiftAwareStrategy;

impl ShiftAwareStrategy {
    pub fn algorithm_name(&self) -> &'static str {
        ALGORITHM_NAME
    }

    pub fn pair(
        &self,
        logs: &[LogEvent],
        config: Option<&PairingShiftConfig>,
    ) -> Vec<PairedSession> {
        if logs.is_empty() {
            return vec![];
        }

        // Tier 1: If shift config available, use shift-window-based pairing
        if let Some(shift) = config {
            return Self::pair_by_shift_window(logs, shift);
        }

        // Tier 2: Fallback to anchor-first (original behavior)
        Self::pair_anchor_first(logs)
    }

    /// Tier 1: "First In near shift start, Last Out of day" (industry standard).
    /// Earliest log near shift start = IN, last log of the day = OUT.
    /// Check-in uses a buffer window to avoid picking up logs from previous day.
    /// Check-out has no buffer — captures any amount of overtime.
    fn pair_by_shift_window(logs: &[LogEvent], shift: &PairingShiftConfig) -> Vec<PairedSession> {
        let check_in_window_start = shift.start_minutes - SHIFT_WINDOW_BUFFER;
        let check_in_window_end = shift.start_minutes + SHIFT_WINDOW_BUFFER;

        // Find earliest log in check-in window
        let first_in = logs
            .iter()
            .find(|l| l.minutes >= check_in_window_start && l.minutes <= check_in_window_end);

        // If we can't find logs in shift windows, fall back to anchor-first
        let (start, start_id) = match first_in {
            Some(log) => (log.minutes, log.id),
            None => return Self::pair_anchor_first(logs),
        };

        // Last Out: take the last log of the day (after check-in)
        let last_out = logs
            .iter()
            .rev()
            .find(|l| l.id != start_id && l.minutes > start);

        let (end, end_id, end_note) = match last_out {
            Some(log) => {
                let note = log
                    .raw_data
                    .as_ref()
                    .and_then(|r| r.get("note"))
                    .and_then(|n| n.as_str())
                    .map(|s| s.to_string());
                (Some(log.minutes), Some(log.id), note)
            }
            _ => (None, None, None),
        };

        let mut log_ids = vec![start_id];
        if let Some(eid) = end_id {
            log_ids.push(eid);
        }

        let start_note = first_in
            .and_then(|l| l.raw_data.as_ref())
            .and_then(|r| r.get("note"))
            .and_then(|n| n.as_str())
            .map(|s| s.to_string());

        let combined_note = match (start_note, end_note) {
            (Some(n1), Some(n2)) => Some(format!("{n1}\n{n2}")),
            (Some(n1), None) => Some(n1),
            (None, Some(n2)) => Some(n2),
            (None, None) => None,
        };

        let entry_type = first_in
            .and_then(|l| l.raw_data.as_ref())
            .and_then(|r| r.get("entry_type"))
            .and_then(|e| e.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "work".to_string());

        let final_note = if end.is_none() {
            match combined_note {
                Some(n) => Some(format!("{n}\nMissing Punch Out")),
                None => Some("Missing Punch Out".to_string()),
            }
        } else {
            combined_note
        };

        let is_break_overtime = first_in
            .and_then(|l| l.raw_data.as_ref())
            .and_then(|r| r.get("is_break_overtime"))
            .and_then(|e| e.as_bool())
            .unwrap_or(false);

        vec![PairedSession {
            start_minutes: start,
            end_minutes: end,
            entry_type,
            note: final_note,
            is_break_overtime,
            metadata: Some(serde_json::json!({
                "algorithm": ALGORITHM_NAME,
                "tier": "shift_window",
                "log_ids": log_ids,
                "shift_start": shift.start_minutes,
                "shift_end": shift.end_minutes
            })),
        }]
    }

    /// Tier 2: Anchor-first fallback (original behavior).
    /// First log = IN, furthest log within 16h = OUT.
    fn pair_anchor_first(logs: &[LogEvent]) -> Vec<PairedSession> {
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
        let start_note = first
            .raw_data
            .as_ref()
            .and_then(|r| r.get("note"))
            .and_then(|n| n.as_str())
            .map(|s| s.to_string());

        let (end, end_note) = if last_idx > 0 {
            let log = &logs[last_idx];
            let note = log
                .raw_data
                .as_ref()
                .and_then(|r| r.get("note"))
                .and_then(|n| n.as_str())
                .map(|s| s.to_string());
            (Some(log.minutes), note)
        } else {
            (None, None)
        };

        let combined_note = match (start_note, end_note) {
            (Some(n1), Some(n2)) => Some(format!("{n1}\n{n2}")),
            (Some(n1), None) => Some(n1),
            (None, Some(n2)) => Some(n2),
            (None, None) => None,
        };

        let entry_type = first
            .raw_data
            .as_ref()
            .and_then(|r| r.get("entry_type"))
            .and_then(|e| e.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "work".to_string());

        let final_note = if end.is_none() {
            match combined_note {
                Some(n) => Some(format!("{n}\nMissing Punch Out")),
                None => Some("Missing Punch Out".to_string()),
            }
        } else {
            combined_note
        };

        let mut log_ids = vec![first.id];
        if last_idx > 0 {
            log_ids.push(logs[last_idx].id);
        }

        let is_break_overtime = first
            .raw_data
            .as_ref()
            .and_then(|r| r.get("is_break_overtime"))
            .and_then(|e| e.as_bool())
            .unwrap_or(false);

        vec![PairedSession {
            start_minutes: start,
            end_minutes: end,
            entry_type,
            note: final_note,
            is_break_overtime,
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

    fn make_log(minutes: i32) -> LogEvent {
        LogEvent {
            id: Uuid::new_v4(),
            minutes,
            raw_data: None,
        }
    }

    fn standard_shift() -> PairingShiftConfig {
        PairingShiftConfig {
            start_minutes: 480,             // 08:00
            end_minutes: 1020,              // 17:00
            break_start_minutes: Some(720), // 12:00
            break_end_minutes: Some(780),   // 13:00
        }
    }

    // --- Tier 1: Shift Window tests ---

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
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].start_minutes, 475);
        assert_eq!(sessions[0].end_minutes, Some(1025));

        let meta = sessions[0].metadata.as_ref().unwrap();
        assert_eq!(meta["tier"], "shift_window");
    }

    #[test]
    fn test_shift_window_long_overtime() {
        let strategy = get_strategy();
        let shift = standard_shift(); // 08:00-17:00
                                      // Employee works until 23:00 (6h overtime)
        let logs = vec![
            make_log(475),  // 07:55 — check-in
            make_log(1380), // 23:00 — check-out (6h after shift end)
        ];

        let sessions = strategy.pair(&logs, Some(&shift));
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].start_minutes, 475);
        assert_eq!(sessions[0].end_minutes, Some(1380)); // 23:00 captured!

        let meta = sessions[0].metadata.as_ref().unwrap();
        assert_eq!(meta["tier"], "shift_window");
    }

    #[test]
    fn test_shift_window_logs_outside_window_fallback() {
        let strategy = get_strategy();
        let shift = standard_shift();
        // All logs far from shift windows — should fallback to anchor_first
        let logs = vec![
            make_log(200), // 03:20 — way before shift
            make_log(300), // 05:00 — still before window
        ];

        let sessions = strategy.pair(&logs, Some(&shift));
        assert_eq!(sessions.len(), 1);

        let meta = sessions[0].metadata.as_ref().unwrap();
        assert_eq!(meta["tier"], "anchor_first"); // Fell back
    }

    // --- Tier 2: Anchor First tests ---

    #[test]
    fn test_anchor_first_perfect_day() {
        let strategy = get_strategy();
        let logs = vec![make_log(480), make_log(1020)];

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
        let logs = vec![make_log(480)];

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
        let logs = vec![make_log(480), make_log(1500)];

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

    // --- Overnight shift windows ---

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
            make_log(1310), // 21:50 — within check-in window
            make_log(1810), // 06:10 next day — within check-out window
        ];

        let sessions = strategy.pair(&logs, Some(&shift));
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].start_minutes, 1310);
        assert_eq!(sessions[0].end_minutes, Some(1810));

        let meta = sessions[0].metadata.as_ref().unwrap();
        assert_eq!(meta["tier"], "shift_window");
    }

    // --- Single log edge cases ---

    #[test]
    fn test_single_log_no_config() {
        let strategy = get_strategy();
        let logs = vec![make_log(480)];

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
        let logs = vec![make_log(480)];

        let sessions = strategy.pair(&logs, Some(&shift));
        assert_eq!(sessions.len(), 1);
        assert!(sessions[0].end_minutes.is_none());
    }

    // --- Anchor-first boundary (exactly at 16h gap) ---

    #[test]
    fn test_anchor_first_exactly_at_16h_gap() {
        let strategy = get_strategy();
        let logs = vec![
            make_log(480),
            make_log(1440), // exactly MAX_SESSION_GAP_MINUTES away
        ];

        let sessions = strategy.pair(&logs, None);
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].end_minutes, Some(1440));
    }

    #[test]
    fn test_anchor_first_one_minute_over_16h_gap() {
        let strategy = get_strategy();
        let logs = vec![
            make_log(480),
            make_log(1441), // 961 min gap > MAX_SESSION_GAP_MINUTES (960)
        ];

        let sessions = strategy.pair(&logs, None);
        assert_eq!(sessions.len(), 1);
        assert!(sessions[0].end_minutes.is_none());
    }

    // --- Multiple logs in check-in window picks earliest ---

    #[test]
    fn test_shift_window_earliest_checkin() {
        let strategy = get_strategy();
        let shift = standard_shift();
        let logs = vec![
            make_log(470),  // 07:50
            make_log(485),  // 08:05 — also in window
            make_log(1020), // 17:00 — check-out
        ];

        let sessions = strategy.pair(&logs, Some(&shift));
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].start_minutes, 470); // earliest in window
        assert_eq!(sessions[0].end_minutes, Some(1020));
    }

    // --- Algorithm name ---

    #[test]
    fn test_algorithm_name() {
        let strategy = get_strategy();
        assert_eq!(strategy.algorithm_name(), "shift_aware");
    }

    // --- PairedSession serialization ---

    #[test]
    fn test_paired_session_serde_roundtrip() {
        let session = PairedSession {
            start_minutes: 480,
            end_minutes: Some(1020),
            entry_type: "work".to_string(),
            note: None,
            is_break_overtime: false,
            metadata: Some(serde_json::json!({"tier": "test"})),
        };
        let json = serde_json::to_string(&session).unwrap();
        let parsed: PairedSession = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, session);
    }

    // --- Edge case: Overnight shift + overtime ---

    #[test]
    fn test_overnight_shift_with_overtime() {
        let strategy = get_strategy();
        let shift = PairingShiftConfig {
            start_minutes: 1320, // 22:00
            end_minutes: 1800,   // 06:00 next day
            break_start_minutes: None,
            break_end_minutes: None,
        };

        // Employee works overtime until 08:00 (2h OT after 06:00 shift end)
        let logs = vec![
            make_log(1310), // 21:50 — check-in
            make_log(1920), // 08:00 next day — check-out (2h OT)
        ];

        let sessions = strategy.pair(&logs, Some(&shift));
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].start_minutes, 1310);
        assert_eq!(sessions[0].end_minutes, Some(1920)); // 08:00 captured!

        let meta = sessions[0].metadata.as_ref().unwrap();
        assert_eq!(meta["tier"], "shift_window");
    }

    // --- Edge case: Single log in check-in window (forgot to clock out) ---

    #[test]
    fn test_single_log_in_checkin_window_missing_checkout() {
        let strategy = get_strategy();
        let shift = standard_shift(); // 08:00-17:00
        let logs = vec![
            make_log(478), // 07:58 — within check-in window, no other logs
        ];

        let sessions = strategy.pair(&logs, Some(&shift));
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].start_minutes, 478);
        assert!(sessions[0].end_minutes.is_none());
        assert_eq!(sessions[0].note.as_deref(), Some("Missing Punch Out"));

        let meta = sessions[0].metadata.as_ref().unwrap();
        assert_eq!(meta["tier"], "shift_window");
    }

    // --- Edge case: Rapid multiple punches (like employee 1541) ---

    #[test]
    fn test_rapid_multiple_punches() {
        let strategy = get_strategy();
        let shift = PairingShiftConfig {
            start_minutes: 450,             // 07:30
            end_minutes: 990,               // 16:30
            break_start_minutes: Some(690), // 11:30
            break_end_minutes: Some(750),   // 12:30
        };

        // Simulating employee 1541: multiple rapid punches at start, then checkout at 17:00
        let logs = vec![
            make_log(446),  // 07:26 — first log, within check-in window
            make_log(455),  // 07:35
            make_log(461),  // 07:41
            make_log(464),  // 07:44
            make_log(466),  // 07:46
            make_log(1020), // 17:00 — last log of day
        ];

        let sessions = strategy.pair(&logs, Some(&shift));
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].start_minutes, 446); // earliest in window
        assert_eq!(sessions[0].end_minutes, Some(1020)); // last log
    }

    // --- Edge case: Log far before shift window (should fallback) ---

    #[test]
    fn test_log_before_shift_window_falls_back() {
        let strategy = get_strategy();
        let shift = standard_shift(); // 08:00-17:00, window starts at 06:00
                                      // All logs at 03:00-04:00 — way before check-in window (480-120=360=06:00)
        let logs = vec![
            make_log(180), // 03:00
            make_log(240), // 04:00
        ];

        let sessions = strategy.pair(&logs, Some(&shift));
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].start_minutes, 180);
        assert_eq!(sessions[0].end_minutes, Some(240));

        let meta = sessions[0].metadata.as_ref().unwrap();
        assert_eq!(meta["tier"], "anchor_first"); // Fell back because no logs in check-in window
    }
}
