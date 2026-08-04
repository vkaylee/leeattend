use crate::services::pairing::strategy::{ShiftAwareStrategy, ALGORITHM_NAME};
use crate::services::pairing::types::{
    BreakInterval, EntryType, LogEvent, NewPairedSessionParams, PairedSession, PairingNote,
    PairingShiftConfig,
};

/// Input struct for break-splitting logic, replacing 10 positional parameters.
pub(crate) struct SplitByBreaksInput<'a> {
    pub(crate) logs: &'a [LogEvent],
    pub(crate) start: i32,
    pub(crate) end_val: i32,
    pub(crate) active_breaks: &'a [BreakInterval],
    pub(crate) first_in_log: &'a LogEvent,
    pub(crate) last_out_log: &'a LogEvent,
    pub(crate) shift: &'a PairingShiftConfig,
    pub(crate) entry_type: &'a EntryType,
    pub(crate) combined_note: Option<&'a PairingNote>,
    pub(crate) is_break_overtime: bool,
}

/// Split a work session into segments around active break intervals.
pub(crate) fn split_by_breaks(input: &SplitByBreaksInput<'_>) -> Vec<PairedSession> {
    let mut sessions = Vec::new();
    let k = input.active_breaks.len();
    for j in 0..=k {
        let seg_start = if j == 0 {
            input.start
        } else {
            input.active_breaks[j - 1].end_minutes()
        };
        let seg_end = if j == k {
            input.end_val
        } else {
            input.active_breaks[j].start_minutes()
        };

        let log_ids = if j == 0 {
            ShiftAwareStrategy::collect_log_ids(
                input.logs,
                seg_start,
                Some(seg_end),
                input.first_in_log.all_ids(),
            )
        } else if j == k {
            ShiftAwareStrategy::collect_log_ids(
                input.logs,
                seg_start,
                Some(seg_end),
                input.last_out_log.all_ids(),
            )
        } else {
            ShiftAwareStrategy::collect_log_ids(input.logs, seg_start, Some(seg_end), &[])
        };

        let start_time = {
            let diff = i64::from(seg_start) - i64::from(input.start);
            input.first_in_log.punch_time() + chrono::Duration::minutes(diff)
        };

        let end_time = if j == k {
            input.last_out_log.punch_time()
        } else {
            let diff = i64::from(seg_end) - i64::from(input.start);
            input.first_in_log.punch_time() + chrono::Duration::minutes(diff)
        };

        let split_name = if j == 0 {
            "first_half"
        } else if j == k {
            "second_half"
        } else {
            "middle_segment"
        };

        sessions.push(PairedSession::new(NewPairedSessionParams {
            start_minutes: seg_start,
            end_minutes: Some(seg_end),
            start_time: start_time,
            end_time: Some(end_time),
            entry_type: *input.entry_type,
            note: if j == 0 { input.combined_note.cloned() } else { None },
            is_break_overtime: input.is_break_overtime,
            metadata: Some(serde_json::json!({ "algorithm": ALGORITHM_NAME, "tier": "shift_window", "log_ids": log_ids, "shift_start": input.shift.start_minutes(), "shift_end": input.shift.end_minutes(), "break_split": split_name })),
        }));
    }
    sessions
}
