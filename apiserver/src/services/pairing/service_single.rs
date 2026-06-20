use crate::diesel_pool::TenantAuthToken;
use crate::errors::AppError;
use crate::models::{RawAttendanceLog, TimeEntry};
use chrono::{DateTime, Duration, NaiveDate, Utc};
use chrono_tz::Tz;
use tracing::info;
use uuid::Uuid;

use crate::services::pairing::context_helpers::ConsumedLogTracker;
use crate::services::pairing::service::PairingService;
use crate::services::pairing::types::PairingShiftConfig;

pub(crate) async fn reprocess_single_day(
    service: &PairingService,
    workspace_id: Uuid,
    employment_id: Uuid,
    date: NaiveDate,
    tz: Tz,
    prefetched_adjacent: Option<&[TimeEntry]>,
) -> Result<Option<PairingShiftConfig>, AppError> {
    info!(
        workspace_id = %workspace_id,
        employment_id = %employment_id,
        date = %date,
        "Reprocessing single day"
    );

    let auth_token = TenantAuthToken::system_override(workspace_id);
    let context = crate::services::pairing::types::PairingContext::new(date, tz);

    // 1. Fetch effective schedule and config
    let (shift_id_opt, shift_config, traces) =
        fetch_effective_shift_config(service, &auth_token, employment_id, date, &context).await?;

    // 2. Fetch raw logs
    let start_of_day_local = crate::utils::tz::local_to_tz(&tz, date, 0, 0, 0)?;
    let start_of_day_utc = start_of_day_local.with_timezone(&Utc);
    let next_day_utc = (start_of_day_local + Duration::days(1)).with_timezone(&Utc);

    let (window_start_utc, window_end_utc) =
        context.calculate_fetch_window(shift_config.as_ref())?;

    let logs = service
        .raw_log_repo
        .get_employment_logs_in_time_range(
            workspace_id,
            employment_id,
            window_start_utc,
            window_end_utc,
        )
        .await?;

    // 3. Exclude logs already consumed by adjacent days
    let consumed_log_ids = if let Some(adjacent) = prefetched_adjacent {
        ConsumedLogTracker::extract_consumed_ids(adjacent, date, tz)
    } else {
        let tracker = ConsumedLogTracker::new(
            &*service.time_entry_repo,
            &auth_token,
            employment_id,
            date,
            tz,
        );
        tracker
            .get_consumed_log_ids(start_of_day_utc, next_day_utc)
            .await?
    };

    let filtered_raw_logs: Vec<RawAttendanceLog> = logs
        .into_iter()
        .filter(|log| !consumed_log_ids.contains(&log.id()))
        .collect();

    let filtered_log_events = context.build_log_events(&filtered_raw_logs)?;

    if filtered_log_events.is_empty() {
        service
            .clear_auto_entries(&auth_token, employment_id, start_of_day_utc, next_day_utc)
            .await?;

        service
            .recalculate_summary_with_retry(workspace_id, employment_id, date)
            .await?;

        return Ok(shift_config);
    }

    // 4. Run Algorithm
    let strategy = crate::services::pairing::strategy::get_strategy();
    let sessions = strategy.pair(&filtered_log_events, shift_config.as_ref());

    // 5. Update Database and Recalculate
    let new_entries =
        context.build_time_entries(workspace_id, employment_id, shift_id_opt, sessions, &traces)?;

    save_paired_entries_and_update_summary(
        service,
        &auth_token,
        employment_id,
        date,
        start_of_day_utc,
        next_day_utc,
        new_entries,
    )
    .await?;

    Ok(shift_config)
}

pub(crate) async fn fetch_effective_shift_config(
    service: &PairingService,
    auth_token: &TenantAuthToken,
    employment_id: Uuid,
    date: NaiveDate,
    context: &crate::services::pairing::types::PairingContext,
) -> Result<(Option<Uuid>, Option<PairingShiftConfig>, serde_json::Value), AppError> {
    let (shift_id_opt, traces) = service
        .roster_repo
        .get_effective_shift(auth_token, employment_id, date)
        .await?;

    let shift = if let Some(shift_id) = shift_id_opt {
        service
            .shift_repo
            .get_shift_by_id(auth_token, shift_id)
            .await?
    } else {
        None
    };

    let shift_config = context.build_shift_config(shift.as_ref());
    Ok((shift_id_opt, shift_config, serde_json::json!(traces)))
}

pub(crate) async fn save_paired_entries_and_update_summary(
    service: &PairingService,
    auth_token: &TenantAuthToken,
    employment_id: Uuid,
    date: NaiveDate,
    start_of_day_utc: DateTime<Utc>,
    next_day_utc: DateTime<Utc>,
    new_entries: Vec<TimeEntry>,
) -> Result<(), AppError> {
    service
        .time_entry_repo
        .replace_time_entries(
            auth_token,
            employment_id,
            start_of_day_utc,
            next_day_utc,
            new_entries,
        )
        .await?;

    service
        .recalculate_summary_with_retry(auth_token.workspace_id(), employment_id, date)
        .await?;

    Ok(())
}
