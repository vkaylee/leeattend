use crate::diesel_pool::TenantAuthToken;
use crate::errors::AppError;
use crate::models::{RawAttendanceLog, TimeEntry};
use chrono::{Duration, NaiveDate};
use chrono_tz::Tz;
use tracing::info;
use uuid::Uuid;

use crate::services::pairing::context_helpers::ConsumedLogTracker;
use crate::services::pairing::service::PairingService;
use crate::services::pairing::types::PairingShiftConfig;

pub(crate) async fn propagate_overnight_reprocess(
    service: &PairingService,
    workspace_id: Uuid,
    employment_id: Uuid,
    start_date: NaiveDate,
    tz: Tz,
    initial_shift_config: Option<PairingShiftConfig>,
) -> Result<(), AppError> {
    let Some(mut current_config) = initial_shift_config else {
        return Ok(());
    };

    if current_config.end_minutes() <= 1440 {
        return Ok(());
    }

    let lookahead_start = start_date + Duration::days(1);
    let lookahead_end = start_date + Duration::days(7);

    info!(
        workspace_id = %workspace_id,
        employment_id = %employment_id,
        start_date = %lookahead_start,
        end_date = %lookahead_end,
        "Propagating overnight shifts, batch fetching lookahead range"
    );

    let auth_token = TenantAuthToken::system_override(workspace_id);

    // 1. Batch fetch schedules
    let emp_sched = match service
        .fetch_schedules_for_range(&auth_token, employment_id, lookahead_start, lookahead_end)
        .await
    {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!(error = %e, "No schedules found during propagation lookahead, skipping propagation");
            return Ok(());
        }
    };

    // 2. Batch fetch logs and adjacent entries
    let (raw_logs, adjacent_entries) = service
        .fetch_logs_and_entries_for_range(
            &auth_token,
            workspace_id,
            employment_id,
            lookahead_start,
            lookahead_end,
            tz,
        )
        .await?;

    // 3. Process day-by-day in memory
    let mut new_entries_accumulated: Vec<TimeEntry> = Vec::new();
    let mut resolved_end_date = start_date;
    let mut current_date = lookahead_start;

    let schedule_map: std::collections::HashMap<
        NaiveDate,
        &crate::models::schedule::EffectiveSchedule,
    > = emp_sched
        .schedules()
        .iter()
        .map(|s| (s.date(), s))
        .collect();

    while current_date <= lookahead_end {
        if current_config.end_minutes() <= 1440 {
            break;
        }

        if let Some(sched) = schedule_map.get(&current_date) {
            let (day_entries, next_shift_config) = process_propagation_day(
                current_date,
                tz,
                workspace_id,
                employment_id,
                sched,
                &raw_logs,
                &adjacent_entries,
                &new_entries_accumulated,
            )?;

            new_entries_accumulated.extend(day_entries);
            resolved_end_date = current_date;

            if let Some(config) = next_shift_config {
                current_config = config;
            } else {
                break;
            }
        } else {
            break;
        }

        current_date += Duration::days(1);
    }

    // 4. If we actually reprocessed any dates, save the entries and update summaries in bulk
    if resolved_end_date > start_date {
        super::service_range::save_range_entries(
            service,
            &auth_token,
            workspace_id,
            employment_id,
            lookahead_start,
            resolved_end_date,
            tz,
            new_entries_accumulated,
        )
        .await?;
    }

    Ok(())
}

fn process_propagation_day(
    current_date: NaiveDate,
    tz: Tz,
    workspace_id: Uuid,
    employment_id: Uuid,
    sched: &crate::models::schedule::EffectiveSchedule,
    raw_logs: &[RawAttendanceLog],
    adjacent_entries: &[TimeEntry],
    new_entries_accumulated: &[TimeEntry],
) -> Result<(Vec<TimeEntry>, Option<PairingShiftConfig>), AppError> {
    let context = crate::services::pairing::types::PairingContext::new(current_date, tz);
    let next_shift_config = context.build_shift_config_from_schedule(sched);

    let (window_start_utc, window_end_utc) =
        context.calculate_fetch_window(next_shift_config.as_ref())?;

    let mut consumed_log_ids =
        ConsumedLogTracker::extract_consumed_ids(adjacent_entries, current_date, tz);

    for entry in new_entries_accumulated {
        if let Some(metadata) = entry.generation_metadata() {
            if let Ok(paired_meta) = serde_json::from_value::<
                crate::services::pairing_algo::PairingMetadata,
            >(metadata.clone())
            {
                for id in paired_meta.log_ids() {
                    consumed_log_ids.insert(*id);
                }
            }
        }
    }

    let filtered_raw_logs: Vec<RawAttendanceLog> = raw_logs
        .iter()
        .filter(|log| log.punch_time() >= window_start_utc && log.punch_time() <= window_end_utc)
        .filter(|log| !consumed_log_ids.contains(&log.id()))
        .cloned()
        .collect();

    let filtered_log_events = context.build_log_events(&filtered_raw_logs)?;

    let day_entries = if filtered_log_events.is_empty() {
        Vec::new()
    } else {
        let strategy = crate::services::pairing::strategy::get_strategy();
        let sessions = strategy.pair(&filtered_log_events, next_shift_config.as_ref());

        let traces_val = serde_json::json!(sched.traces());
        context.build_time_entries(
            workspace_id,
            employment_id,
            sched.shift_id(),
            sessions,
            &traces_val,
        )?
    };

    Ok((day_entries, next_shift_config))
}
