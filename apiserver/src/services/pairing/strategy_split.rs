use crate::services::pairing::strategy::{ShiftAwareStrategy, ALGORITHM_NAME};
use crate::services::pairing::types::{
    BreakInterval, EntryType, LogEvent, PairedSession, PairingNote, PairingShiftConfig,
};

/// Split a work session into segments around active break intervals.
#[allow(clippy::too_many_arguments)]
pub(crate) fn split_by_breaks(
    logs: &[LogEvent],
    start: i32,
    end_val: i32,
    active_breaks: &[BreakInterval],
    first_in_log: &LogEvent,
    last_out_log: &LogEvent,
    shift: &PairingShiftConfig,
    entry_type: &EntryType,
    combined_note: Option<&PairingNote>,
    is_break_overtime: bool,
) -> Vec<PairedSession> {
    let mut sessions = Vec::new();
    let k = active_breaks.len();
    for j in 0..=k {
        let seg_start = if j == 0 {
            start
        } else {
            active_breaks[j - 1].end_minutes()
        };
        let seg_end = if j == k {
            end_val
        } else {
            active_breaks[j].start_minutes()
        };

        let log_ids = if j == 0 {
            ShiftAwareStrategy::collect_log_ids(
                logs,
                seg_start,
                Some(seg_end),
                first_in_log.all_ids(),
            )
        } else if j == k {
            ShiftAwareStrategy::collect_log_ids(
                logs,
                seg_start,
                Some(seg_end),
                last_out_log.all_ids(),
            )
        } else {
            ShiftAwareStrategy::collect_log_ids(logs, seg_start, Some(seg_end), &[])
        };

        let start_time = if j == k {
            let diff = i64::from(end_val) - i64::from(seg_start);
            last_out_log.punch_time() - chrono::Duration::minutes(diff)
        } else {
            let diff = i64::from(seg_start) - i64::from(start);
            first_in_log.punch_time() + chrono::Duration::minutes(diff)
        };

        let end_time = if j == k {
            last_out_log.punch_time()
        } else {
            let diff = i64::from(seg_end) - i64::from(start);
            first_in_log.punch_time() + chrono::Duration::minutes(diff)
        };

        let split_name = if j == 0 {
            "first_half"
        } else if j == k {
            "second_half"
        } else {
            "middle_segment"
        };

        sessions.push(PairedSession::new(
            seg_start,
            Some(seg_end),
            start_time,
            Some(end_time),
            entry_type.clone(),
            if j == 0 { combined_note.cloned() } else { None },
            is_break_overtime,
            Some(serde_json::json!({
                "algorithm": ALGORITHM_NAME,
                "tier": "shift_window",
                "log_ids": log_ids,
                "shift_start": shift.start_minutes(),
                "shift_end": shift.end_minutes(),
                "break_split": split_name
            })),
        ));
    }
    sessions
}
