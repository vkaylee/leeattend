use crate::services::pairing::strategy::{ShiftAwareStrategy, ALGORITHM_NAME};
use crate::services::pairing::types::{
    BreakInterval, EntryType, LogEvent, PairedSession, PairingNote, PairingShiftConfig,
};

const SHIFT_WINDOW_BUFFER: i32 = 120;

pub(crate) fn pair_by_shift_window(
    logs: &[LogEvent],
    shift: &PairingShiftConfig,
) -> Vec<PairedSession> {
    let buffer = shift.check_in_buffer_minutes.unwrap_or(SHIFT_WINDOW_BUFFER);

    // Find earliest log in check-in window
    let first_in = find_earliest_in_window(logs, shift.start_minutes, buffer);

    // If we can't find logs in shift windows, fall back to anchor-first
    let Some(first_in_log) = first_in else {
        return ShiftAwareStrategy::pair_anchor_first(logs, shift.max_session_gap_minutes);
    };

    let start = first_in_log.minutes;

    // Last Out: take the last log of the day (after check-in)
    let last_out = find_last_out(logs, start, first_in_log.id);

    let end = last_out.map(|l| l.minutes);
    let start_note = ShiftAwareStrategy::extract_note(first_in_log);
    let end_note = last_out.and_then(ShiftAwareStrategy::extract_note);
    let combined_note = ShiftAwareStrategy::combine_notes(start_note, end_note);
    let entry_type = ShiftAwareStrategy::extract_entry_type(first_in_log);
    let is_break_overtime = ShiftAwareStrategy::extract_is_break_overtime(first_in_log);

    if let Some(last_out_log) = last_out {
        let end_val = last_out_log.minutes;
        let mut active_breaks = Vec::new();
        for b in shift.breaks() {
            if start < b.start_minutes() && end_val > b.end_minutes() {
                active_breaks.push(b.clone());
            }
        }
        active_breaks.sort_by_key(BreakInterval::start_minutes);

        if !active_breaks.is_empty() {
            return super::strategy_split::split_by_breaks(
                logs,
                start,
                end_val,
                &active_breaks,
                first_in_log,
                last_out_log,
                shift,
                &entry_type,
                combined_note.as_ref(),
                is_break_overtime,
            );
        }
    }

    vec![build_single_session(
        logs,
        start,
        end,
        first_in_log,
        last_out,
        shift,
        entry_type,
        combined_note,
        is_break_overtime,
    )]
}

fn find_earliest_in_window(
    logs: &[LogEvent],
    start_minutes: i32,
    buffer: i32,
) -> Option<&LogEvent> {
    let check_in_window_start = start_minutes.saturating_sub(buffer);
    let check_in_window_end = start_minutes.saturating_add(buffer);
    logs.iter()
        .find(|l| l.minutes >= check_in_window_start && l.minutes <= check_in_window_end)
}

fn find_last_out(logs: &[LogEvent], start_minutes: i32, start_id: uuid::Uuid) -> Option<&LogEvent> {
    logs.iter()
        .rev()
        .find(|l| l.id != start_id && l.minutes > start_minutes)
}

fn build_single_session(
    logs: &[LogEvent],
    start: i32,
    end: Option<i32>,
    first_in_log: &LogEvent,
    last_out: Option<&LogEvent>,
    shift: &PairingShiftConfig,
    entry_type: EntryType,
    combined_note: Option<PairingNote>,
    is_break_overtime: bool,
) -> PairedSession {
    let final_note = ShiftAwareStrategy::finalize_note(combined_note, end.is_none());
    let single_id = [first_in_log.id];
    let fallback_ids = logs
        .iter()
        .find(|l| l.id == first_in_log.id)
        .map(|l| l.all_ids.as_slice())
        .unwrap_or(&single_id);
    let log_ids = ShiftAwareStrategy::collect_log_ids(logs, start, end, fallback_ids);

    PairedSession::new(
        start,
        end,
        first_in_log.punch_time,
        last_out.map(|l| l.punch_time),
        entry_type,
        final_note,
        is_break_overtime,
        Some(serde_json::json!({
            "algorithm": ALGORITHM_NAME,
            "tier": "shift_window",
            "log_ids": log_ids,
            "shift_start": shift.start_minutes,
            "shift_end": shift.end_minutes
        })),
    )
}
