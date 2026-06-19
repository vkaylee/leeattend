use serde::{Deserialize, Serialize};

/// Type-safe entry classification for paired sessions.
///
/// Replaces raw `String` to prevent typos and invalid values at compile time.
/// Serializes to `"work"` / `"break"` via serde for JSON backward compatibility.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum EntryType {
    #[default]
    Work,
    Break,
}

impl std::fmt::Display for EntryType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Work => write!(f, "work"),
            Self::Break => write!(f, "break"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PairingShiftConfig {
    // All times in minutes from midnight (00:00) of the shift start day.
    // Values can be > 1440 (24h) for overnight shifts.
    start_minutes: i32,
    end_minutes: i32,
    // TODO(TD-BREAK): These fields are populated but not yet consumed by the
    // pairing algorithm. They will be needed when break session splitting is
    // implemented (related: TD-07 multi-session support).
    #[allow(dead_code)]
    break_start_minutes: Option<i32>,
    #[allow(dead_code)]
    break_end_minutes: Option<i32>,
    check_in_buffer_minutes: Option<i32>,
    max_session_gap_minutes: Option<i32>,
}

impl PairingShiftConfig {
    pub fn new(
        start_minutes: i32,
        end_minutes: i32,
        break_start_minutes: Option<i32>,
        break_end_minutes: Option<i32>,
        check_in_buffer_minutes: Option<i32>,
        max_session_gap_minutes: Option<i32>,
    ) -> Self {
        Self {
            start_minutes,
            end_minutes,
            break_start_minutes,
            break_end_minutes,
            check_in_buffer_minutes,
            max_session_gap_minutes,
        }
    }

    pub fn start_minutes(&self) -> i32 {
        self.start_minutes
    }

    pub fn end_minutes(&self) -> i32 {
        self.end_minutes
    }

    pub fn break_start_minutes(&self) -> Option<i32> {
        self.break_start_minutes
    }

    pub fn break_end_minutes(&self) -> Option<i32> {
        self.break_end_minutes
    }

    pub fn check_in_buffer_minutes(&self) -> Option<i32> {
        self.check_in_buffer_minutes
    }

    pub fn max_session_gap_minutes(&self) -> Option<i32> {
        self.max_session_gap_minutes
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PairedSession {
    // Minutes from midnight of the analysis date.
    // Can be > 1440 (representing next day).
    start_minutes: i32,
    end_minutes: Option<i32>,
    entry_type: EntryType,
    note: Option<String>,
    is_break_overtime: bool,
    metadata: Option<serde_json::Value>,
}

impl PairedSession {
    pub fn new(
        start_minutes: i32,
        end_minutes: Option<i32>,
        entry_type: EntryType,
        note: Option<String>,
        is_break_overtime: bool,
        metadata: Option<serde_json::Value>,
    ) -> Self {
        Self {
            start_minutes,
            end_minutes,
            entry_type,
            note,
            is_break_overtime,
            metadata,
        }
    }

    pub fn start_minutes(&self) -> i32 {
        self.start_minutes
    }

    pub fn end_minutes(&self) -> Option<i32> {
        self.end_minutes
    }

    pub fn entry_type(&self) -> &EntryType {
        &self.entry_type
    }

    pub fn note(&self) -> Option<&str> {
        self.note.as_deref()
    }

    pub fn is_break_overtime(&self) -> bool {
        self.is_break_overtime
    }

    pub fn metadata(&self) -> Option<&serde_json::Value> {
        self.metadata.as_ref()
    }

    pub fn into_metadata(self) -> Option<serde_json::Value> {
        self.metadata
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogEvent {
    id: uuid::Uuid,
    minutes: i32,
    raw_data: Option<serde_json::Value>,
    all_ids: Vec<uuid::Uuid>,
}

impl LogEvent {
    pub fn new(
        id: uuid::Uuid,
        minutes: i32,
        raw_data: Option<serde_json::Value>,
        all_ids: Vec<uuid::Uuid>,
    ) -> Self {
        Self {
            id,
            minutes,
            raw_data,
            all_ids,
        }
    }

    pub fn id(&self) -> uuid::Uuid {
        self.id
    }

    pub fn minutes(&self) -> i32 {
        self.minutes
    }

    pub fn raw_data(&self) -> Option<&serde_json::Value> {
        self.raw_data.as_ref()
    }

    pub fn all_ids(&self) -> &[uuid::Uuid] {
        &self.all_ids
    }

    pub fn push_id(&mut self, id: uuid::Uuid) {
        self.all_ids.push(id);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairingMetadata {
    log_ids: Vec<uuid::Uuid>,
}

impl PairingMetadata {
    pub fn new(log_ids: Vec<uuid::Uuid>) -> Self {
        Self { log_ids }
    }

    pub fn log_ids(&self) -> &[uuid::Uuid] {
        &self.log_ids
    }
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
///
/// # Known Limitations
///
/// ## TD-07: Single Session Per Day
/// Currently, this algorithm only supports pairing a maximum of one session per day
/// (either within the shift window or via the fallback strategy). It does not support
/// multiple independent work sessions (e.g. split shifts, multi-session days).
///
/// **Impact:** Employees with split shifts (e.g. 06:00-10:00 + 14:00-18:00) will have
/// their entire day collapsed into a single session from first-in to last-out.
///
/// **Required for resolution:**
/// - Business requirements for split shift pairing rules
/// - Algorithm to detect session gaps vs break gaps
/// - Break deduction logic using `break_start/end_minutes` (TD-BREAK)
pub fn get_strategy() -> ShiftAwareStrategy {
    ShiftAwareStrategy
}

pub struct ShiftAwareStrategy;

impl ShiftAwareStrategy {
    pub fn algorithm_name(&self) -> &'static str {
        ALGORITHM_NAME
    }

    pub fn pair(&self, logs: &[LogEvent], configs: &[PairingShiftConfig]) -> Vec<PairedSession> {
        if logs.is_empty() {
            return vec![];
        }

        // Tier 1: If shift config available, use shift-window-based pairing
        if !configs.is_empty() {
            let mut all_sessions = Vec::new();
            for shift in configs {
                let sessions = Self::pair_by_shift_window(logs, shift);
                all_sessions.extend(sessions);
            }
            return all_sessions;
        }

        // Tier 2: Fallback to anchor-first (original behavior)
        Self::pair_anchor_first(logs, None)
    }

    fn find_earliest_in_window(
        logs: &[LogEvent],
        start_minutes: i32,
        buffer: i32,
    ) -> Option<&LogEvent> {
        let check_in_window_start = start_minutes - buffer;
        let check_in_window_end = start_minutes + buffer;
        logs.iter()
            .find(|l| l.minutes >= check_in_window_start && l.minutes <= check_in_window_end)
    }

    fn find_last_out(
        logs: &[LogEvent],
        start_minutes: i32,
        start_id: uuid::Uuid,
    ) -> Option<&LogEvent> {
        logs.iter()
            .rev()
            .find(|l| l.id != start_id && l.minutes > start_minutes)
    }

    fn combine_notes(start_note: Option<String>, end_note: Option<String>) -> Option<String> {
        match (start_note, end_note) {
            (Some(n1), Some(n2)) => Some(format!("{n1}\n{n2}")),
            (Some(n1), None) => Some(n1),
            (None, Some(n2)) => Some(n2),
            (None, None) => None,
        }
    }

    fn find_last_idx_within_gap(logs: &[LogEvent], start_minutes: i32, max_gap: i32) -> usize {
        let mut last_idx = 0;
        for (i, log) in logs.iter().enumerate().skip(1) {
            if log.minutes - start_minutes <= max_gap {
                last_idx = i;
            } else {
                break;
            }
        }
        last_idx
    }

    // --- Shared helpers for TD-P2 DRY extraction ---

    /// Extract note from a log event's raw_data.
    fn extract_note(log: &LogEvent) -> Option<String> {
        log.raw_data
            .as_ref()
            .and_then(|r| r.get("note"))
            .and_then(|n| n.as_str())
            .map(std::string::ToString::to_string)
    }

    /// Extract entry type from a log event's raw_data, defaulting to Work.
    fn extract_entry_type(log: &LogEvent) -> EntryType {
        log.raw_data
            .as_ref()
            .and_then(|r| r.get("entry_type"))
            .and_then(|e| e.as_str())
            .and_then(|s| serde_json::from_value(serde_json::Value::String(s.to_string())).ok())
            .unwrap_or_default()
    }

    /// Extract is_break_overtime from a log event's raw_data.
    fn extract_is_break_overtime(log: &LogEvent) -> bool {
        log.raw_data
            .as_ref()
            .and_then(|r| r.get("is_break_overtime"))
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false)
    }

    /// Append "Missing Punch Out" note if no end time.
    fn finalize_note(note: Option<String>, missing_out: bool) -> Option<String> {
        if missing_out {
            match note {
                Some(n) => Some(format!("{n}\nMissing Punch Out")),
                None => Some("Missing Punch Out".to_string()),
            }
        } else {
            note
        }
    }

    /// Collect all log IDs between start and end minutes.
    fn collect_log_ids(
        logs: &[LogEvent],
        start: i32,
        end: Option<i32>,
        fallback_ids: &[uuid::Uuid],
    ) -> Vec<uuid::Uuid> {
        if let Some(e_val) = end {
            logs.iter()
                .filter(|l| l.minutes >= start && l.minutes <= e_val)
                .flat_map(|l| l.all_ids.clone())
                .collect()
        } else {
            fallback_ids.to_vec()
        }
    }

    /// Tier 1: "First In near shift start, Last Out of day" (industry standard).
    /// Earliest log near shift start = IN, last log of the day = OUT.
    /// Check-in uses a buffer window to avoid picking up logs from previous day.
    /// Check-out has no buffer — captures any amount of overtime.
    fn pair_by_shift_window(logs: &[LogEvent], shift: &PairingShiftConfig) -> Vec<PairedSession> {
        let buffer = shift.check_in_buffer_minutes.unwrap_or(SHIFT_WINDOW_BUFFER);

        // Find earliest log in check-in window
        let first_in = Self::find_earliest_in_window(logs, shift.start_minutes, buffer);

        // If we can't find logs in shift windows, fall back to anchor-first
        let Some(first_in_log) = first_in else {
            return Self::pair_anchor_first(logs, shift.max_session_gap_minutes);
        };

        let start = first_in_log.minutes;

        // Last Out: take the last log of the day (after check-in)
        let last_out = Self::find_last_out(logs, start, first_in_log.id);

        let end = last_out.map(|l| l.minutes);
        let start_note = Self::extract_note(first_in_log);
        let end_note = last_out.and_then(Self::extract_note);
        let combined_note = Self::combine_notes(start_note, end_note);
        let entry_type = Self::extract_entry_type(first_in_log);
        let is_break_overtime = Self::extract_is_break_overtime(first_in_log);
        let final_note = Self::finalize_note(combined_note, end.is_none());
        let single_id = [first_in_log.id];
        let fallback_ids = logs
            .iter()
            .find(|l| l.id == first_in_log.id)
            .map(|l| l.all_ids.as_slice())
            .unwrap_or(&single_id);
        let log_ids = Self::collect_log_ids(logs, start, end, fallback_ids);

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
    fn pair_anchor_first(logs: &[LogEvent], max_gap: Option<i32>) -> Vec<PairedSession> {
        let max_gap_val = max_gap.unwrap_or(MAX_SESSION_GAP_MINUTES);
        let first = &logs[0];
        let last_idx = Self::find_last_idx_within_gap(logs, first.minutes, max_gap_val);

        let start = first.minutes;
        let start_note = Self::extract_note(first);

        let (end, end_note) = if last_idx > 0 {
            let log = &logs[last_idx];
            (Some(log.minutes), Self::extract_note(log))
        } else {
            (None, None)
        };

        let combined_note = Self::combine_notes(start_note, end_note);
        let entry_type = Self::extract_entry_type(first);
        let is_break_overtime = Self::extract_is_break_overtime(first);
        let final_note = Self::finalize_note(combined_note, end.is_none());
        let log_ids = Self::collect_log_ids(logs, start, end, &first.all_ids);

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
        let id = Uuid::new_v4();
        LogEvent {
            id,
            minutes,
            raw_data: None,
            all_ids: vec![id],
        }
    }

    fn standard_shift() -> PairingShiftConfig {
        PairingShiftConfig {
            start_minutes: 480,             // 08:00
            end_minutes: 1020,              // 17:00
            break_start_minutes: Some(720), // 12:00
            break_end_minutes: Some(780),   // 13:00
            check_in_buffer_minutes: None,
            max_session_gap_minutes: None,
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

        let sessions = strategy.pair(&logs, &[shift]);
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

        let sessions = strategy.pair(&logs, &[shift]);
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

        let sessions = strategy.pair(&logs, &[shift]);
        assert_eq!(sessions.len(), 1);

        let meta = sessions[0].metadata.as_ref().unwrap();
        assert_eq!(meta["tier"], "anchor_first"); // Fell back
    }

    // --- Tier 2: Anchor First tests ---

    #[test]
    fn test_anchor_first_perfect_day() {
        let strategy = get_strategy();
        let logs = vec![make_log(480), make_log(1020)];

        let sessions = strategy.pair(&logs, &[]);
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

        let sessions = strategy.pair(&logs, &[]);
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

        let sessions = strategy.pair(&logs, &[]);
        assert_eq!(sessions.len(), 1);
        assert!(sessions[0].end_minutes.is_none());
    }

    #[test]
    fn test_empty_logs() {
        let strategy = get_strategy();
        let sessions = strategy.pair(&[], &[]);
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
            check_in_buffer_minutes: None,
            max_session_gap_minutes: None,
        };

        let logs = vec![
            make_log(1310), // 21:50 — within check-in window
            make_log(1810), // 06:10 next day — within check-out window
        ];

        let sessions = strategy.pair(&logs, &[shift]);
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

        let sessions = strategy.pair(&logs, &[]);
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

        let sessions = strategy.pair(&logs, &[shift]);
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

        let sessions = strategy.pair(&logs, &[]);
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

        let sessions = strategy.pair(&logs, &[]);
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

        let sessions = strategy.pair(&logs, &[shift]);
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
            entry_type: EntryType::Work,
            note: None,
            is_break_overtime: false,
            metadata: Some(serde_json::json!({"tier": "test"})),
        };
        let json = serde_json::to_string(&session).unwrap();
        let parsed: PairedSession = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, session);
        // Verify backward-compatible JSON serialization
        assert!(json.contains("\"work\""));
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
            check_in_buffer_minutes: None,
            max_session_gap_minutes: None,
        };

        // Employee works overtime until 08:00 (2h OT after 06:00 shift end)
        let logs = vec![
            make_log(1310), // 21:50 — check-in
            make_log(1920), // 08:00 next day — check-out (2h OT)
        ];

        let sessions = strategy.pair(&logs, &[shift]);
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

        let sessions = strategy.pair(&logs, &[shift]);
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
            check_in_buffer_minutes: None,
            max_session_gap_minutes: None,
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

        let sessions = strategy.pair(&logs, &[shift]);
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

        let sessions = strategy.pair(&logs, &[shift]);
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].start_minutes, 180);
        assert_eq!(sessions[0].end_minutes, Some(240));

        let meta = sessions[0].metadata.as_ref().unwrap();
        assert_eq!(meta["tier"], "anchor_first"); // Fell back because no logs in check-in window
    }

    #[test]
    fn test_overnight_shift_multiple_checkout_punches() {
        let strategy = get_strategy();
        let shift = PairingShiftConfig {
            start_minutes: 1080, // 18:00
            end_minutes: 1560,   // 02:00 next day
            break_start_minutes: None,
            break_end_minutes: None,
            check_in_buffer_minutes: None,
            max_session_gap_minutes: None,
        };

        let logs = vec![make_log(1085), make_log(1825), make_log(1826)];

        let sessions = strategy.pair(&logs, &[shift]);
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].start_minutes, 1085);
        assert_eq!(sessions[0].end_minutes, Some(1826)); // should be 1826

        let meta = sessions[0].metadata.as_ref().unwrap();
        let log_ids: Vec<String> = meta["log_ids"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap().to_string())
            .collect();
        assert_eq!(log_ids.len(), 3);
        assert!(log_ids.contains(&logs[0].id.to_string()));
        assert!(log_ids.contains(&logs[1].id.to_string()));
        assert!(log_ids.contains(&logs[2].id.to_string()));
    }
}
