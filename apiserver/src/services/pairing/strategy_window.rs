use crate::services::pairing::strategy::{ShiftAwareStrategy, ALGORITHM_NAME};
use crate::services::pairing::types::{
    BreakInterval, EntryType, LogEvent, PairedSession, PairingNote, PairingShiftConfig,
};

const SHIFT_WINDOW_BUFFER: i32 = 120;

/// Minutes in one calendar day. A break whose `end_minutes()` reaches or
/// exceeds this value would place the post-break segment on the *next*
/// calendar day (D+1) relative to the shift date.
const MINUTES_PER_DAY: i32 = 1440;

pub(crate) fn pair_by_shift_window(
    logs: &[LogEvent],
    shift: &PairingShiftConfig,
) -> Vec<PairedSession> {
    // Buffer is always at least SHIFT_WINDOW_BUFFER (120 min).
    // check_in_buffer_minutes (aka early_punch_in_grace_period) is a policy
    // threshold for "late" status, NOT a search range — using it as the sole
    // buffer contracts the window too aggressively and misses early punches.
    let buffer = std::cmp::max(
        SHIFT_WINDOW_BUFFER,
        shift.check_in_buffer_minutes().unwrap_or(0),
    );

    // Find earliest log in check-in window
    let first_in = find_earliest_in_window(logs, shift.start_minutes(), buffer);

    // If we can't find logs in shift windows, fall back to anchor-first
    let Some(first_in_log) = first_in else {
        tracing::info!(
            log_minutes = ?logs.iter().map(LogEvent::minutes).collect::<Vec<_>>(),
            shift_start = shift.start_minutes(),
            buffer = buffer,
            check_in_window = format!("[{}, {}]", shift.start_minutes().saturating_sub(buffer), shift.start_minutes().saturating_add(buffer)),
            "pair_by_shift_window: no log in check-in window, falling back to anchor_first"
        );
        return ShiftAwareStrategy::pair_anchor_first(logs, shift.max_session_gap_minutes());
    };

    let start = first_in_log.minutes();

    // Last Out: take the last log of the day (after check-in)
    let last_out = find_last_out(logs, start, first_in_log.id());

    let end = last_out.map(LogEvent::minutes);
    let start_note = ShiftAwareStrategy::extract_note(first_in_log);
    let end_note = last_out.and_then(ShiftAwareStrategy::extract_note);
    let combined_note = ShiftAwareStrategy::combine_notes(start_note, end_note);
    let entry_type = ShiftAwareStrategy::extract_entry_type(first_in_log);
    let is_break_overtime = ShiftAwareStrategy::extract_is_break_overtime(first_in_log);

    if let Some(last_out_log) = last_out {
        let end_val = last_out_log.minutes();
        let mut active_breaks = Vec::new();
        for b in shift.breaks() {
            // A break is a valid split point only when the segment that starts
            // *after* it still lands on the shift's calendar day (D). A break
            // whose end crosses midnight (end >= MINUTES_PER_DAY) would place the
            // post-break segment on day D+1, where `build_time_entries` discards
            // it via its `date_naive() != self.date` guard — silently dropping the
            // employee's real check-out (the reported night-shift bug).
            //
            // In that case we intentionally do NOT split: the session stays
            // continuous (check-in → check-out) and the attendance calculator
            // still deducts the break / accrues break-OT from the shift config
            // (see `calculate_buckets`), so the computed hours are unchanged.
            // Splitting is only meaningful when a real punch materializes the
            // break; a synthetic config boundary crossing midnight must not.
            if start < b.start_minutes()
                && end_val > b.end_minutes()
                && b.end_minutes() < MINUTES_PER_DAY
            {
                active_breaks.push(b.clone());
            }
        }
        active_breaks.sort_by_key(BreakInterval::start_minutes);

        if !active_breaks.is_empty() {
            return super::strategy_split::split_by_breaks(
                &super::strategy_split::SplitByBreaksInput {
                    logs,
                    start,
                    end_val,
                    active_breaks: &active_breaks,
                    first_in_log,
                    last_out_log,
                    shift,
                    entry_type: &entry_type,
                    combined_note: combined_note.as_ref(),
                    is_break_overtime,
                },
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
        .find(|l| l.minutes() >= check_in_window_start && l.minutes() <= check_in_window_end)
}

fn find_last_out(logs: &[LogEvent], start_minutes: i32, start_id: uuid::Uuid) -> Option<&LogEvent> {
    logs.iter()
        .rev()
        .find(|l| l.id() != start_id && l.minutes() > start_minutes)
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
    let single_id = [first_in_log.id()];
    let fallback_ids = logs
        .iter()
        .find(|l| l.id() == first_in_log.id())
        .map(LogEvent::all_ids)
        .unwrap_or(&single_id);
    let log_ids = ShiftAwareStrategy::collect_log_ids(logs, start, end, fallback_ids);

    PairedSession::new(
        start,
        end,
        first_in_log.punch_time(),
        last_out.map(LogEvent::punch_time),
        entry_type,
        final_note,
        is_break_overtime,
        Some(serde_json::json!({
            "algorithm": ALGORITHM_NAME,
            "tier": "shift_window",
            "log_ids": log_ids,
            "shift_start": shift.start_minutes(),
            "shift_end": shift.end_minutes()
        })),
    )
}
