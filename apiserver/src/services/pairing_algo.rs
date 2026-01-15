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
}

/// Represents a standardized attendance pattern to match against
struct AttendancePattern {
    #[allow(dead_code)]
    name: &'static str,
    slots: Vec<PatternSlot>,
    /// Penalty for using this pattern
    base_penalty: u32, 
}

struct PatternSlot {
    target: i32, // Minutes from midnight
    kind: SlotKind,
    mandatory: bool,
}

#[derive(PartialEq, Debug, Clone, Copy)]
enum SlotKind {
    In,
    Out,
}

#[derive(Clone, Copy, Debug)]
enum MatchAction {
    Match(i32), // Log value
    SkipSlot,
    SkipLog,
}

pub fn pair_logs_min_cost(
    logs: &[i32], // Minutes from midnight
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
    if let (Some(bs), Some(be)) = (shift.break_start_minutes, shift.break_end_minutes) {
        patterns.push(AttendancePattern {
            name: "Full Day",
            slots: vec![
                PatternSlot { target: shift.start_minutes, kind: SlotKind::In, mandatory: true },
                PatternSlot { target: bs, kind: SlotKind::Out, mandatory: false },
                PatternSlot { target: be, kind: SlotKind::In, mandatory: false },
                PatternSlot { target: shift.end_minutes, kind: SlotKind::Out, mandatory: true },
            ],
            base_penalty: 0,
        });

        // P2: Morning Only
        patterns.push(AttendancePattern {
            name: "Morning Only",
            slots: vec![
                PatternSlot { target: shift.start_minutes, kind: SlotKind::In, mandatory: true },
                PatternSlot { target: bs, kind: SlotKind::Out, mandatory: true },
            ],
            base_penalty: 10,
        });

        // P3: Afternoon Only
        patterns.push(AttendancePattern {
            name: "Afternoon Only",
            slots: vec![
                PatternSlot { target: be, kind: SlotKind::In, mandatory: true },
                PatternSlot { target: shift.end_minutes, kind: SlotKind::Out, mandatory: true },
            ],
            base_penalty: 10,
        });
    }

    // P4: Continuous
    patterns.push(AttendancePattern {
        name: "Continuous",
        slots: vec![
            PatternSlot { target: shift.start_minutes, kind: SlotKind::In, mandatory: true },
            PatternSlot { target: shift.end_minutes, kind: SlotKind::Out, mandatory: true },
        ],
        base_penalty: if shift.break_start_minutes.is_some() { 60 } else { 0 }, 
    });

    patterns
}

fn evaluate_pattern(logs: &[i32], pattern: &AttendancePattern) -> (u32, Vec<PairedSession>) {
    let (cost, actions) = match_recursive(logs, &pattern.slots, 0, 0);
    let total_score = cost.saturating_add(pattern.base_penalty);
    let sessions = build_sessions(actions, &pattern.slots);
    (total_score, sessions)
}

fn match_recursive(
    logs: &[i32],
    slots: &[PatternSlot],
    log_idx: usize,
    slot_idx: usize,
) -> (u32, Vec<MatchAction>) {
    const PENALTY_UNMATCHED_LOG: u32 = 120; 
    const PENALTY_MANDATORY_SLOT: u32 = 120;
    const PENALTY_OPTIONAL_SLOT: u32 = 60;

    if slot_idx >= slots.len() {
        let remaining_logs = (logs.len() - log_idx) as u32;
        let cost = remaining_logs.saturating_mul(PENALTY_UNMATCHED_LOG);
        let actions = vec![MatchAction::SkipLog; logs.len() - log_idx];
        return (cost, actions);
    }

    if log_idx >= logs.len() {
        let mut cost = 0;
        let mut actions = Vec::new();
        for slot in slots.iter().skip(slot_idx) {
            cost += if slot.mandatory { PENALTY_MANDATORY_SLOT } else { PENALTY_OPTIONAL_SLOT };
            actions.push(MatchAction::SkipSlot);
        }
        return (cost, actions);
    }

    // A. MATCH
    let diff = (logs[log_idx] - slots[slot_idx].target).unsigned_abs();
    let (cost_a, mut acts_a) = match_recursive(logs, slots, log_idx + 1, slot_idx + 1);
    let total_cost_a = diff.saturating_add(cost_a);

    // B. SKIP SLOT
    let slot_penalty = if slots[slot_idx].mandatory { PENALTY_MANDATORY_SLOT } else { PENALTY_OPTIONAL_SLOT };
    let (cost_b, mut acts_b) = match_recursive(logs, slots, log_idx, slot_idx + 1);
    let total_cost_b = slot_penalty.saturating_add(cost_b);

    // C. SKIP LOG
    let (cost_c, mut acts_c) = match_recursive(logs, slots, log_idx + 1, slot_idx);
    let total_cost_c = PENALTY_UNMATCHED_LOG.saturating_add(cost_c);

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
    let mut current_start: Option<i32> = None;

    let slot_actions: Vec<&MatchAction> = actions.iter()
        .filter(|a| !matches!(a, MatchAction::SkipLog))
        .collect();

    for (i, action) in slot_actions.iter().enumerate() {
        if i >= slots.len() { break; }
        
        let val = match action {
            MatchAction::Match(v) => Some(*v),
            MatchAction::SkipSlot => None,
            _ => None,
        };

        match slots[i].kind {
            SlotKind::In => {
                current_start = val;
            }
            SlotKind::Out => {
                if let Some(start) = current_start {
                    if let Some(end) = val {
                        sessions.push(PairedSession {
                            start_minutes: start,
                            end_minutes: Some(end),
                            entry_type: "work".to_string(),
                            note: None,
                        });
                    } else {
                        // Missing Out
                        sessions.push(PairedSession {
                            start_minutes: start,
                            end_minutes: None,
                            entry_type: "work".to_string(),
                            note: Some("Missing Punch Out".to_string()),
                        });
                    }
                    current_start = None;
                } else if let Some(end) = val {
                    // Orphaned Out
                    sessions.push(PairedSession {
                        start_minutes: end,
                        end_minutes: Some(end),
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

    fn standard_shift() -> PairingShiftConfig {
        PairingShiftConfig {
            start_minutes: 8 * 60, // 480
            end_minutes: 17 * 60, // 1020
            break_start_minutes: Some(12 * 60), // 720
            break_end_minutes: Some(13 * 60), // 780
        }
    }

    #[test]
    fn test_perfect_day() {
        let shift = standard_shift();
        let logs = vec![475, 725, 775, 1025]; // 7:55, 12:05, 12:55, 17:05
        
        let sessions = pair_logs_min_cost(&logs, &shift);
        
        assert_eq!(sessions.len(), 2);
        assert_eq!(sessions[0].start_minutes, 475);
        assert_eq!(sessions[0].end_minutes, Some(725));
        assert_eq!(sessions[1].start_minutes, 775);
        assert_eq!(sessions[1].end_minutes, Some(1025));
    }

    #[test]
    fn test_overnight_shift() {
        // 22:00 (1320) to 06:00 (1440 + 360 = 1800)
        let shift = PairingShiftConfig {
            start_minutes: 1320,
            end_minutes: 1800,
            break_start_minutes: None,
            break_end_minutes: None,
        };

        // Punch in 21:55 (1315), Punch out 06:05 (1805)
        let logs = vec![1315, 1805];

        let sessions = pair_logs_min_cost(&logs, &shift);

        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].start_minutes, 1315);
        assert_eq!(sessions[0].end_minutes, Some(1805));
    }
}