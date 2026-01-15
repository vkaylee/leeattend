use chrono::NaiveTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq)]
pub struct PairingShiftConfig {
    pub start_time: NaiveTime,
    pub end_time: NaiveTime,
    pub break_start: Option<NaiveTime>,
    pub break_end: Option<NaiveTime>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PairedSession {
    pub start: NaiveTime,
    pub end: Option<NaiveTime>,
    pub entry_type: String, // "work"
    pub note: Option<String>,
}

/// Represents a standardized attendance pattern to match against
struct AttendancePattern {
    #[allow(dead_code)]
    name: &'static str,
    slots: Vec<PatternSlot>,
    /// Penalty for using this pattern (e.g., standard = 0, no-break = 50)
    base_penalty: u32, 
}

struct PatternSlot {
    target: NaiveTime,
    kind: SlotKind, // Start or End
    mandatory: bool,
}

#[derive(PartialEq, Debug, Clone, Copy)]
enum SlotKind {
    In,
    Out,
}

#[derive(Clone, Copy, Debug)]
enum MatchAction {
    Match(NaiveTime), // Log maps to Slot
    SkipSlot,         // Slot is empty
    SkipLog,          // Log is noise (ignored)
}

pub fn pair_logs_min_cost(
    logs: &[NaiveTime],
    shift: &PairingShiftConfig,
) -> Vec<PairedSession> {
    if logs.is_empty() {
        return vec![];
    }

    // 1. Generate Patterns
    let patterns = generate_patterns(shift);

    // 2. Evaluate all patterns
    let mut best_score = u32::MAX;
    let mut best_sessions = Vec::new();

    for pattern in patterns {
        let (score, sessions) = evaluate_pattern(logs, &pattern);
        if score < best_score {
            best_score = score;
            best_sessions = sessions;
        }
    }

    best_sessions
}

fn generate_patterns(shift: &PairingShiftConfig) -> Vec<AttendancePattern> {
    let mut patterns = Vec::new();

    // P1: Standard Full Day (In -> BreakOut -> BreakIn -> Out)
    if let (Some(bs), Some(be)) = (shift.break_start, shift.break_end) {
        patterns.push(AttendancePattern {
            name: "Full Day",
            slots: vec![
                PatternSlot { target: shift.start_time, kind: SlotKind::In, mandatory: true },
                PatternSlot { target: bs, kind: SlotKind::Out, mandatory: false },
                PatternSlot { target: be, kind: SlotKind::In, mandatory: false },
                PatternSlot { target: shift.end_time, kind: SlotKind::Out, mandatory: true },
            ],
            base_penalty: 0,
        });

        // P2: Morning Only
        patterns.push(AttendancePattern {
            name: "Morning Only",
            slots: vec![
                PatternSlot { target: shift.start_time, kind: SlotKind::In, mandatory: true },
                PatternSlot { target: bs, kind: SlotKind::Out, mandatory: true },
            ],
            base_penalty: 10, // Slight penalty to prefer full day if ambiguous
        });

        // P3: Afternoon Only
        patterns.push(AttendancePattern {
            name: "Afternoon Only",
            slots: vec![
                PatternSlot { target: be, kind: SlotKind::In, mandatory: true },
                PatternSlot { target: shift.end_time, kind: SlotKind::Out, mandatory: true },
            ],
            base_penalty: 10,
        });
    }

    // P4: Continuous (No Break) - Works for shifts without breaks or "Working through lunch"
    patterns.push(AttendancePattern {
        name: "Continuous",
        slots: vec![
            PatternSlot { target: shift.start_time, kind: SlotKind::In, mandatory: true },
            PatternSlot { target: shift.end_time, kind: SlotKind::Out, mandatory: true },
        ],
        // If shift has break, this is "Through Lunch" (higher penalty). 
        // If shift has no break, this is standard (0 penalty).
        base_penalty: if shift.break_start.is_some() { 60 } else { 0 }, 
    });

    patterns
}

fn evaluate_pattern(logs: &[NaiveTime], pattern: &AttendancePattern) -> (u32, Vec<PairedSession>) {
    let (cost, actions) = match_recursive(logs, &pattern.slots, 0, 0);
    let total_score = cost.saturating_add(pattern.base_penalty);
    let sessions = build_sessions(actions, &pattern.slots);
    (total_score, sessions)
}

fn match_recursive(
    logs: &[NaiveTime],
    slots: &[PatternSlot],
    log_idx: usize,
    slot_idx: usize,
) -> (u32, Vec<MatchAction>) {
    // Constants
    const PENALTY_UNMATCHED_LOG: u32 = 120; // 2 hours
    const PENALTY_MANDATORY_SLOT: u32 = 120; // 2 hours
    const PENALTY_OPTIONAL_SLOT: u32 = 60; // 1 hour

    // Base Case 1: No more slots
    if slot_idx >= slots.len() {
        let remaining_logs = (logs.len() - log_idx) as u32;
        let cost = remaining_logs.saturating_mul(PENALTY_UNMATCHED_LOG);
        // We append SkipLog actions just for completeness (though usually unused in session build)
        let actions = vec![MatchAction::SkipLog; logs.len() - log_idx];
        return (cost, actions);
    }

    // Base Case 2: No more logs
    if log_idx >= logs.len() {
        let mut cost = 0;
        let mut actions = Vec::new();
        for i in slot_idx..slots.len() {
            cost += if slots[i].mandatory { PENALTY_MANDATORY_SLOT } else { PENALTY_OPTIONAL_SLOT };
            actions.push(MatchAction::SkipSlot);
        }
        return (cost, actions);
    }

    // Recursive Steps
    
    // A. MATCH: Use current log for current slot
    let diff = (logs[log_idx] - slots[slot_idx].target).num_minutes().abs() as u32;
    let (cost_a, mut acts_a) = match_recursive(logs, slots, log_idx + 1, slot_idx + 1);
    let total_cost_a = diff.saturating_add(cost_a);

    // B. SKIP SLOT: Leave current slot empty
    let slot_penalty = if slots[slot_idx].mandatory { PENALTY_MANDATORY_SLOT } else { PENALTY_OPTIONAL_SLOT };
    let (cost_b, mut acts_b) = match_recursive(logs, slots, log_idx, slot_idx + 1);
    let total_cost_b = slot_penalty.saturating_add(cost_b);

    // C. SKIP LOG: Treat current log as noise
    let (cost_c, mut acts_c) = match_recursive(logs, slots, log_idx + 1, slot_idx);
    let total_cost_c = PENALTY_UNMATCHED_LOG.saturating_add(cost_c);

    // Find Minimum Cost
    if total_cost_a <= total_cost_b && total_cost_a <= total_cost_c {
        let mut res = vec![MatchAction::Match(logs[log_idx])];
        res.append(&mut acts_a);
        (total_cost_a, res)
    } else if total_cost_b <= total_cost_c {
        let mut res = vec![MatchAction::SkipSlot];
        res.append(&mut acts_b);
        (total_cost_b, res)
    } else {
        let mut res = vec![MatchAction::SkipLog];
        res.append(&mut acts_c);
        (total_cost_c, res)
    }
}

fn build_sessions(actions: Vec<MatchAction>, slots: &[PatternSlot]) -> Vec<PairedSession> {
    let mut sessions = Vec::new();
    let mut current_start: Option<NaiveTime> = None;

    // Filter out SkipLog actions as they don't align with slots
    let slot_actions: Vec<&MatchAction> = actions.iter()
        .filter(|a| !matches!(a, MatchAction::SkipLog))
        .collect();

    for (i, action) in slot_actions.iter().enumerate() {
        if i >= slots.len() { break; }
        
        let time = match action {
            MatchAction::Match(t) => Some(*t),
            MatchAction::SkipSlot => None,
            _ => None, // Should not happen due to filter
        };

        match slots[i].kind {
            SlotKind::In => {
                if let Some(t) = time {
                    current_start = Some(t);
                } else {
                    current_start = None;
                }
            }
            SlotKind::Out => {
                if let Some(start) = current_start {
                    if let Some(end) = time {
                        sessions.push(PairedSession {
                            start,
                            end: Some(end),
                            entry_type: "work".to_string(),
                            note: None,
                        });
                    } else {
                        // Missing Punch Out
                        sessions.push(PairedSession {
                            start,
                            end: None,
                            entry_type: "work".to_string(),
                            note: Some("Missing Punch Out".to_string()),
                        });
                    }
                    current_start = None; // Reset
                } else if let Some(end) = time {
                    // Orphaned OUT (No Start).
                    sessions.push(PairedSession {
                        start: end,
                        end: Some(end),
                        entry_type: "work".to_string(),
                        note: Some("Missing Punch In".to_string()),
                    });
                }
            }
        }
    }
    sessions
}
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveTime;

    fn t(hour: u32, minute: u32) -> NaiveTime {
        NaiveTime::from_hms_opt(hour, minute, 0).unwrap()
    }

    fn standard_shift() -> PairingShiftConfig {
        PairingShiftConfig {
            start_time: t(8, 0),
            end_time: t(17, 0),
            break_start: Some(t(12, 0)),
            break_end: Some(t(13, 0)),
        }
    }

    #[test]
    fn test_perfect_day() {
        let shift = standard_shift();
        let logs = vec![t(7, 55), t(12, 5), t(12, 55), t(17, 5)];
        
        let sessions = pair_logs_min_cost(&logs, &shift);
        
        assert_eq!(sessions.len(), 2);
        assert_eq!(sessions[0].start, t(7, 55));
        assert_eq!(sessions[0].end, Some(t(12, 5)));
        assert_eq!(sessions[1].start, t(12, 55));
        assert_eq!(sessions[1].end, Some(t(17, 5)));
    }

    #[test]
    fn test_half_day_morning() {
        let shift = standard_shift();
        let logs = vec![t(8, 0), t(12, 0)];
        
        let sessions = pair_logs_min_cost(&logs, &shift);
        
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].start, t(8, 0));
        assert_eq!(sessions[0].end, Some(t(12, 0)));
    }

    #[test]
    fn test_half_day_afternoon() {
        let shift = standard_shift();
        let logs = vec![t(13, 0), t(17, 0)];
        
        let sessions = pair_logs_min_cost(&logs, &shift);
        
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].start, t(13, 0));
        assert_eq!(sessions[0].end, Some(t(17, 0)));
    }

    #[test]
    fn test_through_lunch() {
        let shift = standard_shift();
        let logs = vec![t(8, 0), t(17, 0)];
        
        let sessions = pair_logs_min_cost(&logs, &shift);
        
        // Should match "Continuous" pattern (Cost ~60 penalty vs FullDay cost > 200)
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].start, t(8, 0));
        assert_eq!(sessions[0].end, Some(t(17, 0)));
    }

    #[test]
    fn test_missing_out() {
        let shift = standard_shift();
        let logs = vec![t(8, 0)];
        
        let sessions = pair_logs_min_cost(&logs, &shift);
        
        // Should have one open session or session with Missing Out note
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].start, t(8, 0));
        assert!(sessions[0].end.is_none() || sessions[0].note.as_ref().unwrap().contains("Missing Punch Out"));
    }

    #[test]
    fn test_noisy_logs() {
        let shift = standard_shift();
        // 08:00 (In), 10:00 (Random Noise), 12:00 (Out)
        let logs = vec![t(8, 0), t(10, 0), t(12, 0)];
        
        let sessions = pair_logs_min_cost(&logs, &shift);
        
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].start, t(8, 0));
        assert_eq!(sessions[0].end, Some(t(12, 0)));
    }
}
