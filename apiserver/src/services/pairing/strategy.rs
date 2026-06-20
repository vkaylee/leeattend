use crate::services::pairing::types::{
    EntryType, LogEvent, PairedSession, PairingNote, PairingShiftConfig,
};
use tracing::info;

pub const ALGORITHM_NAME: &str = "shift_aware";
const MAX_SESSION_GAP_MINUTES: i32 = 960;

pub struct ShiftAwareStrategy;

pub fn get_strategy() -> ShiftAwareStrategy {
    ShiftAwareStrategy
}

impl ShiftAwareStrategy {
    pub fn algorithm_name(&self) -> &'static str {
        ALGORITHM_NAME
    }

    #[tracing::instrument(skip_all, fields(log_count = logs.len(), has_config = config.is_some()))]
    pub fn pair(
        &self,
        logs: &[LogEvent],
        config: Option<&PairingShiftConfig>,
    ) -> Vec<PairedSession> {
        if logs.is_empty() {
            return vec![];
        }

        let mut sorted_logs = logs.to_vec();
        sorted_logs.sort_by_key(|l| l.minutes());

        // Tier 1: If shift config available, use shift-window-based pairing
        if let Some(shift) = config {
            let sessions = Self::pair_by_shift_window(&sorted_logs, shift);
            info!(
                tier = "shift_window",
                session_count = sessions.len(),
                "Pairing complete"
            );
            return sessions;
        }

        // Tier 2: Fallback to anchor-first (original behavior)
        let sessions = Self::pair_anchor_first(&sorted_logs, None);
        info!(
            tier = "anchor_first",
            session_count = sessions.len(),
            "Pairing complete"
        );
        sessions
    }

    pub(crate) fn pair_by_shift_window(
        logs: &[LogEvent],
        shift: &PairingShiftConfig,
    ) -> Vec<PairedSession> {
        super::strategy_window::pair_by_shift_window(logs, shift)
    }

    /// Tier 2: Anchor-first fallback.
    /// Pairs logs sequentially to allow multi-session per day support, respecting the max gap limit.
    pub(crate) fn pair_anchor_first(logs: &[LogEvent], max_gap: Option<i32>) -> Vec<PairedSession> {
        let max_gap_val = max_gap.unwrap_or(MAX_SESSION_GAP_MINUTES);
        let mut sessions = Vec::new();
        let mut i = 0;
        while i < logs.len() {
            let first = &logs[i];
            let start_time = first.punch_time();
            let start_note = Self::extract_note(first);
            let entry_type = Self::extract_entry_type(first);
            let is_break_overtime = Self::extract_is_break_overtime(first);

            if i + 1 < logs.len() {
                let second = &logs[i + 1];
                if second
                    .minutes()
                    .checked_sub(first.minutes())
                    .is_some_and(|diff| diff <= max_gap_val)
                {
                    let end_time = Some(second.punch_time());
                    let end_note = Self::extract_note(second);
                    let combined_note = Self::combine_notes(start_note, end_note);
                    let log_ids = Self::collect_log_ids(
                        logs,
                        first.minutes(),
                        Some(second.minutes()),
                        first.all_ids(),
                    );

                    sessions.push(PairedSession::new(
                        first.minutes(),
                        Some(second.minutes()),
                        start_time,
                        end_time,
                        entry_type,
                        combined_note,
                        is_break_overtime,
                        Some(serde_json::json!({
                            "algorithm": ALGORITHM_NAME,
                            "tier": "anchor_first",
                            "log_ids": log_ids
                        })),
                    ));
                    i += 2;
                } else {
                    let final_note = Self::finalize_note(start_note, true);
                    let log_ids = first.all_ids().to_vec();

                    sessions.push(PairedSession::new(
                        first.minutes(),
                        None,
                        start_time,
                        None,
                        entry_type,
                        final_note,
                        is_break_overtime,
                        Some(serde_json::json!({
                            "algorithm": ALGORITHM_NAME,
                            "tier": "anchor_first",
                            "log_ids": log_ids
                        })),
                    ));
                    i += 1;
                }
            } else {
                let final_note = Self::finalize_note(start_note, true);
                let log_ids = first.all_ids().to_vec();

                sessions.push(PairedSession::new(
                    first.minutes(),
                    None,
                    start_time,
                    None,
                    entry_type,
                    final_note,
                    is_break_overtime,
                    Some(serde_json::json!({
                        "algorithm": ALGORITHM_NAME,
                        "tier": "anchor_first",
                        "log_ids": log_ids
                    })),
                ));
                i += 1;
            }
        }
        sessions
    }

    pub(crate) fn combine_notes(
        start_note: Option<PairingNote>,
        end_note: Option<PairingNote>,
    ) -> Option<PairingNote> {
        match (start_note, end_note) {
            (Some(PairingNote::Custom(n1)), Some(PairingNote::Custom(n2))) => {
                Some(PairingNote::Custom(format!("{n1}\n{n2}")))
            }
            (Some(n1), None | Some(_)) => Some(n1),
            (None, Some(n2)) => Some(n2),
            (None, None) => None,
        }
    }

    pub(crate) fn extract_note(log: &LogEvent) -> Option<PairingNote> {
        log.raw_data()
            .as_ref()
            .and_then(|r| r.get("note"))
            .and_then(|n| n.as_str())
            .map(|s| PairingNote::Custom(s.to_string()))
    }

    pub(crate) fn extract_entry_type(log: &LogEvent) -> EntryType {
        log.raw_data()
            .as_ref()
            .and_then(|r| r.get("entry_type"))
            .and_then(|e| e.as_str())
            .and_then(|s| serde_json::from_value(serde_json::Value::String(s.to_string())).ok())
            .unwrap_or_default()
    }

    pub(crate) fn extract_is_break_overtime(log: &LogEvent) -> bool {
        log.raw_data()
            .as_ref()
            .and_then(|r| r.get("is_break_overtime"))
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false)
    }

    pub(crate) fn finalize_note(
        note: Option<PairingNote>,
        missing_out: bool,
    ) -> Option<PairingNote> {
        if missing_out {
            match note {
                Some(PairingNote::Custom(n)) => {
                    Some(PairingNote::Custom(format!("{n}\nmissing_punch_out")))
                }
                None => Some(PairingNote::MissingPunchOut),
                Some(other) => Some(other),
            }
        } else {
            note
        }
    }

    pub(crate) fn collect_log_ids(
        logs: &[LogEvent],
        start: i32,
        end: Option<i32>,
        fallback_ids: &[uuid::Uuid],
    ) -> Vec<uuid::Uuid> {
        if let Some(e_val) = end {
            logs.iter()
                .filter(|l| l.minutes() >= start && l.minutes() <= e_val)
                .flat_map(|l| l.all_ids().to_vec())
                .collect()
        } else {
            fallback_ids.to_vec()
        }
    }
}
