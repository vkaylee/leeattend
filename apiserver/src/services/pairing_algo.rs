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

pub trait PairingStrategy: Send + Sync {
    fn pair(&self, logs: &[LogEvent], config: Option<&PairingShiftConfig>) -> Vec<PairedSession>;
    fn algorithm_name(&self) -> &'static str;
}

pub struct AnchorFirstStrategy;

/// Factory function to get the requested strategy.
/// Currently defaults to AnchorFirstStrategy as other algorithms are deprecated.
pub fn get_strategy(_algorithm: &str) -> Box<dyn PairingStrategy> {
    Box::new(AnchorFirstStrategy)
}

// --- AnchorFirstStrategy Implementation ---

impl PairingStrategy for AnchorFirstStrategy {
    fn algorithm_name(&self) -> &'static str {
        "anchor_first"
    }

    fn pair(&self, logs: &[LogEvent], _config: Option<&PairingShiftConfig>) -> Vec<PairedSession> {
        if logs.is_empty() {
            return vec![];
        }

        let first = &logs[0];
        let mut last_idx = 0;

        // Find the furthest log that is within 12 hours (720 minutes) of the first log
        for (i, log) in logs.iter().enumerate().skip(1) {
            if log.minutes - first.minutes <= 720 {
                last_idx = i;
            } else {
                // Once we hit a log too far away, we stop for AnchorFirst
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
                "algorithm": self.algorithm_name(),
                "log_ids": log_ids
            })),
        }]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_anchor_perfect_day() {
        let strategy = AnchorFirstStrategy;
        let logs = vec![
            LogEvent {
                id: Uuid::new_v4(),
                minutes: 480, // 08:00
                punch_type: None,
            },
            LogEvent {
                id: Uuid::new_v4(),
                minutes: 1020, // 17:00
                punch_type: None,
            },
        ];

        let sessions = strategy.pair(&logs, None);

        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].start_minutes, 480);
        assert_eq!(sessions[0].end_minutes, Some(1020));
    }

    #[test]
    fn test_anchor_missing_out() {
        let strategy = AnchorFirstStrategy;
        let logs = vec![
            LogEvent {
                id: Uuid::new_v4(),
                minutes: 480,
                punch_type: None,
            },
        ];

        let sessions = strategy.pair(&logs, None);

        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].start_minutes, 480);
        assert!(sessions[0].end_minutes.is_none());
    }

    #[test]
    fn test_anchor_12h_limit() {
        let strategy = AnchorFirstStrategy;
        
        // Log 1: 08:00 (480)
        // Log 2: 21:00 (1260) -> 13h gap (780 mins). Should NOT pair.
        let logs = vec![
            LogEvent { id: Uuid::new_v4(), minutes: 480, punch_type: None },
            LogEvent { id: Uuid::new_v4(), minutes: 1260, punch_type: None },
        ];
        
        let sessions = strategy.pair(&logs, None);
        assert_eq!(sessions.len(), 1);
        assert!(sessions[0].end_minutes.is_none());
    }
}