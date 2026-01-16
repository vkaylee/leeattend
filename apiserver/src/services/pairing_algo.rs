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

#[derive(Debug, Clone, PartialEq)]
pub struct LogEvent {
    pub minutes: i32,
    pub punch_type: Option<i32>, // 0: Check-In, 1: Check-Out, 2: Break-In, 3: Break-Out, 4: Overtime-In, 5: Overtime-Out
}

pub trait PairingStrategy: Send + Sync {
    fn pair(&self, logs: &[LogEvent], config: Option<&PairingShiftConfig>) -> Vec<PairedSession>;
}

pub struct DpPatternStrategy;
pub struct AnchorFirstStrategy;
pub struct SequentialStrategy;
pub struct FsmStrategy;

/// Factory function to get the requested strategy
pub fn get_strategy(algorithm: &str) -> Box<dyn PairingStrategy> {
    match algorithm {
        "anchor_first" => Box::new(AnchorFirstStrategy),
        "sequential" => Box::new(SequentialStrategy),
        "fsm" => Box::new(FsmStrategy),
        _ => Box::new(DpPatternStrategy),
    }
}

// --- DpPatternStrategy Implementation ---

impl PairingStrategy for DpPatternStrategy {
    fn pair(&self, logs: &[LogEvent], config: Option<&PairingShiftConfig>) -> Vec<PairedSession> {
        if logs.is_empty() {
            return vec![];
        }

        let minutes: Vec<i32> = logs.iter().map(|l| l.minutes).collect();

        let Some(shift) = config else {
            // Fallback if no shift config provided for DP strategy
            return AnchorFirstStrategy.pair(logs, None);
        };

        // 1. Generate Patterns
        let patterns = generate_patterns(shift);

        // 2. Evaluate all patterns
        let mut best_score = u32::MAX;
        let mut best_sessions = Vec::new();

        for pattern in patterns {
            let (score, sessions) = evaluate_pattern(&minutes, &pattern);
            if score < best_score {
                best_score = score;
                best_sessions = sessions;
            }
        }

        best_sessions
    }
}

// --- AnchorFirstStrategy Implementation ---

impl PairingStrategy for AnchorFirstStrategy {
    fn pair(&self, logs: &[LogEvent], _config: Option<&PairingShiftConfig>) -> Vec<PairedSession> {
        if logs.is_empty() {
            return vec![];
        }

        let start = logs[0].minutes;
        let end = if logs.len() > 1 {
            Some(logs[logs.len() - 1].minutes)
        } else {
            None
        };

        vec![PairedSession {
            start_minutes: start,
            end_minutes: end,
            entry_type: "work".to_string(),
            note: if end.is_none() { Some("Missing Punch Out".to_string()) } else { None },
        }]
    }
}

// --- SequentialStrategy Implementation ---

impl PairingStrategy for SequentialStrategy {
    fn pair(&self, logs: &[LogEvent], _config: Option<&PairingShiftConfig>) -> Vec<PairedSession> {
        let mut sessions = Vec::new();
        let mut chunks = logs.chunks_exact(2);
        
        for chunk in chunks.by_ref() {
            sessions.push(PairedSession {
                start_minutes: chunk[0].minutes,
                end_minutes: Some(chunk[1].minutes),
                entry_type: "work".to_string(),
                note: None,
            });
        }
        
        if let Some(last) = chunks.remainder().first() {
             sessions.push(PairedSession {
                start_minutes: last.minutes,
                end_minutes: None,
                entry_type: "work".to_string(),
                note: Some("Missing Punch Out".to_string()),
            });
        }
        
        sessions
    }
}

// --- FsmStrategy Implementation ---

impl PairingStrategy for FsmStrategy {
    fn pair(&self, logs: &[LogEvent], _config: Option<&PairingShiftConfig>) -> Vec<PairedSession> {
        if logs.is_empty() {
            return vec![];
        }

        let mut sessions = Vec::new();
        let mut current_start: Option<i32> = None;

        for event in logs {
            match event.punch_type {
                Some(0) | Some(2) | Some(4) => { // In types
                    // If already have a start, close it as missing out
                    if let Some(start) = current_start {
                        sessions.push(PairedSession {
                            start_minutes: start,
                            end_minutes: None,
                            entry_type: "work".to_string(),
                            note: Some("Missing Punch Out".to_string()),
                        });
                    }
                    current_start = Some(event.minutes);
                }
                Some(1) | Some(3) | Some(5) => { // Out types
                    if let Some(start) = current_start {
                        sessions.push(PairedSession {
                            start_minutes: start,
                            end_minutes: Some(event.minutes),
                            entry_type: "work".to_string(),
                            note: None,
                        });
                        current_start = None;
                    } else {
                        // Orphaned Out
                        sessions.push(PairedSession {
                            start_minutes: event.minutes,
                            end_minutes: Some(event.minutes),
                            entry_type: "work".to_string(),
                            note: Some("Missing Punch In".to_string()),
                        });
                    }
                }
                _ => {
                    // Fallback to toggle behavior if no metadata
                    if let Some(start) = current_start {
                        sessions.push(PairedSession {
                            start_minutes: start,
                            end_minutes: Some(event.minutes),
                            entry_type: "work".to_string(),
                            note: None,
                        });
                        current_start = None;
                    } else {
                        current_start = Some(event.minutes);
                    }
                }
            }
        }

        if let Some(start) = current_start {
            sessions.push(PairedSession {
                start_minutes: start,
                end_minutes: None,
                entry_type: "work".to_string(),
                note: Some("Missing Punch Out".to_string()),
            });
        }

        sessions
    }
}

// --- Private Helper Logic for DP (Internal to this module) ---

struct AttendancePattern {
    #[allow(dead_code)]
    name: &'static str,
    slots: Vec<PatternSlot>,
    base_penalty: u32, 
}

struct PatternSlot {
    target: i32,
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
    Match(i32),
    SkipSlot,
    SkipLog,
}

fn generate_patterns(shift: &PairingShiftConfig) -> Vec<AttendancePattern> {
    let mut patterns = Vec::new();

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

        patterns.push(AttendancePattern {
            name: "Morning Only",
            slots: vec![
                PatternSlot { target: shift.start_minutes, kind: SlotKind::In, mandatory: true },
                PatternSlot { target: bs, kind: SlotKind::Out, mandatory: true },
            ],
            base_penalty: 10,
        });

        patterns.push(AttendancePattern {
            name: "Afternoon Only",
            slots: vec![
                PatternSlot { target: be, kind: SlotKind::In, mandatory: true },
                PatternSlot { target: shift.end_minutes, kind: SlotKind::Out, mandatory: true },
            ],
            base_penalty: 10,
        });
    }

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
    const PENALTY_UNMATCHED_LOG: u32 = 300; 
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

    let diff = (logs[log_idx] - slots[slot_idx].target).unsigned_abs();
    let (cost_a, mut acts_a) = match_recursive(logs, slots, log_idx + 1, slot_idx + 1);
    let total_cost_a = diff.saturating_add(cost_a);

    let slot_penalty = if slots[slot_idx].mandatory { PENALTY_MANDATORY_SLOT } else { PENALTY_OPTIONAL_SLOT };
    let (cost_b, mut acts_b) = match_recursive(logs, slots, log_idx, slot_idx + 1);
    let total_cost_b = slot_penalty.saturating_add(cost_b);

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
                        sessions.push(PairedSession {
                            start_minutes: start,
                            end_minutes: None,
                            entry_type: "work".to_string(),
                            note: Some("Missing Punch Out".to_string()),
                        });
                    }
                    current_start = None;
                } else if let Some(end) = val {
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
    fn test_dp_perfect_day() {
        let strategy = DpPatternStrategy;
        let shift = standard_shift();
        let logs = vec![
            LogEvent { minutes: 475, punch_type: None },
            LogEvent { minutes: 725, punch_type: None },
            LogEvent { minutes: 775, punch_type: None },
            LogEvent { minutes: 1025, punch_type: None },
        ]; 
        
        let sessions = strategy.pair(&logs, Some(&shift));
        
        assert_eq!(sessions.len(), 2);
        assert_eq!(sessions[0].start_minutes, 475);
        assert_eq!(sessions[0].end_minutes, Some(725));
        assert_eq!(sessions[1].start_minutes, 775);
        assert_eq!(sessions[1].end_minutes, Some(1025));
    }

    #[test]
    fn test_anchor_perfect_day() {
        let strategy = AnchorFirstStrategy;
        let logs = vec![
            LogEvent { minutes: 475, punch_type: None },
            LogEvent { minutes: 725, punch_type: None },
            LogEvent { minutes: 775, punch_type: None },
            LogEvent { minutes: 1025, punch_type: None },
        ]; 
        
        let sessions = strategy.pair(&logs, None);
        
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].start_minutes, 475);
        assert_eq!(sessions[0].end_minutes, Some(1025));
    }
}
