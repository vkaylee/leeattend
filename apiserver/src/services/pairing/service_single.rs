use crate::diesel_pool::TenantAuthToken;
use crate::errors::AppError;
use crate::models::{RawAttendanceLog, TimeEntry};
use crate::services::pairing::context_helpers;
use crate::services::pairing::service::PairingService;
use crate::services::pairing::types::PairingShiftConfig;
use chrono::{DateTime, NaiveDate, Utc};
use chrono_tz::Tz;
use diesel_async::{AsyncConnection, AsyncPgConnection};
use tracing::info;
use uuid::Uuid;

pub(crate) async fn reprocess_single_day(
    service: &PairingService,
    workspace_id: Uuid,
    employment_id: Uuid,
    date: NaiveDate,
    tz: Tz,
    prefetched_adjacent: Option<&[TimeEntry]>,
) -> Result<Option<PairingShiftConfig>, AppError> {
    reprocess_single_day_with_conn(
        service,
        None,
        workspace_id,
        employment_id,
        date,
        tz,
        prefetched_adjacent,
    )
    .await
}

/// [`reprocess_single_day`] variant that runs its write phase on a caller-owned
/// transaction connection (`Some`) or acquires its own transaction (`None`
/// with pools available), keeping multi-write pipelines atomic.
pub(crate) async fn reprocess_single_day_with_conn(
    service: &PairingService,
    conn: Option<&mut AsyncPgConnection>,
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
    let next_day_utc = crate::utils::tz::start_of_next_day_utc(&tz, date)?;

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
        context_helpers::extract_consumed_ids(adjacent, date, tz)
    } else {
        super::service_helpers::fetch_consumed_log_ids(
            &*service.time_entry_repo,
            &auth_token,
            employment_id,
            date,
            tz,
            start_of_day_utc,
            next_day_utc,
        )
        .await?
    };

    let filtered_raw_logs: Vec<RawAttendanceLog> = logs
        .into_iter()
        .filter(|log| !consumed_log_ids.contains(&log.id()))
        .collect();

    let filtered_log_events = context.build_log_events(&filtered_raw_logs)?;

    if filtered_log_events.is_empty() {
        clear_and_recalculate(
            service,
            conn,
            &auth_token,
            workspace_id,
            employment_id,
            date,
            start_of_day_utc,
            next_day_utc,
        )
        .await?;

        return Ok(shift_config);
    }

    // 4. Run Algorithm
    let strategy = crate::services::pairing::strategy::get_strategy();
    let sessions = strategy.pair(&filtered_log_events, shift_config.as_ref());

    // 5. Update Database and Recalculate
    let new_entries =
        context.build_time_entries(workspace_id, employment_id, shift_id_opt, sessions, &traces)?;

    if let Some(c) = conn {
        save_paired_entries_and_update_summary_tx(
            service,
            c,
            &auth_token,
            workspace_id,
            employment_id,
            date,
            start_of_day_utc,
            next_day_utc,
            new_entries,
        )
        .await?;
    } else {
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
    }

    Ok(shift_config)
}

/// Clear the day's auto entries and recalculate its summary.
///
/// When no caller-owned connection is provided but pools are available, the two
/// writes run inside their own transaction so they commit (or roll back) as one
/// unit. In tests (mock repos, no pools) it falls back to the legacy calls.
#[allow(clippy::too_many_arguments)]
async fn clear_and_recalculate(
    service: &PairingService,
    conn: Option<&mut AsyncPgConnection>,
    auth_token: &TenantAuthToken,
    workspace_id: Uuid,
    employment_id: Uuid,
    date: NaiveDate,
    start_of_day_utc: DateTime<Utc>,
    next_day_utc: DateTime<Utc>,
) -> Result<(), AppError> {
    if let Some(c) = conn {
        service
            .clear_auto_entries_tx(
                c,
                workspace_id,
                employment_id,
                start_of_day_utc,
                next_day_utc,
            )
            .await?;
        service
            .recalculate_summary_with_retry_tx(c, auth_token, employment_id, date)
            .await
    } else if let Some(pools) = &service.diesel_pools {
        let mut tx_conn = pools.tenant_transaction_conn(auth_token).await?;
        tx_conn
            .transaction(|c| {
                Box::pin(async move {
                    service
                        .clear_auto_entries_tx(
                            c,
                            workspace_id,
                            employment_id,
                            start_of_day_utc,
                            next_day_utc,
                        )
                        .await?;
                    service
                        .recalculate_summary_with_retry_tx(c, auth_token, employment_id, date)
                        .await
                })
            })
            .await
    } else {
        service
            .clear_auto_entries(auth_token, employment_id, start_of_day_utc, next_day_utc)
            .await?;
        service
            .recalculate_summary_with_retry(auth_token, employment_id, date)
            .await
    }
}

pub(crate) async fn fetch_effective_shift_config(
    service: &PairingService,
    auth_token: &TenantAuthToken,
    employment_id: Uuid,
    date: NaiveDate,
    context: &crate::services::pairing::types::PairingContext,
) -> Result<(Option<Uuid>, Option<PairingShiftConfig>, serde_json::Value), AppError> {
    let (shift_id_opt, traces, roster_data) = service
        .roster_repo
        .get_effective_shift(auth_token, employment_id, date)
        .await?;

    let shift_config = if let Some(shift_id) = shift_id_opt {
        let from_intervals = roster_data
            .shift_intervals
            .get(&shift_id)
            .and_then(|intervals| intervals.first())
            .and_then(|interval| {
                context.build_shift_config(
                    interval.start_time,
                    interval.end_time,
                    interval.break_start,
                    interval.break_end,
                    interval.early_punch_in_grace_period_minutes,
                )
            });

        // When intervals are available, use them directly.
        // When they are missing (e.g. legacy shifts created before the interval
        // migration), fall back to fetching the resolved EffectiveSchedule so
        // pairing retains break awareness instead of degrading to anchor_first.
        if from_intervals.is_some() {
            from_intervals
        } else {
            let (schedules, _) = service
                .roster_repo
                .get_workspace_schedules(
                    None,
                    auth_token,
                    date,
                    date,
                    Some(&vec![employment_id]),
                    None,
                    None,
                )
                .await?;
            schedules
                .first()
                .and_then(|es| es.schedules().first())
                .and_then(|sched| context.build_shift_config_from_schedule(sched))
        }
    } else {
        None
    };

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
    if let Some(pools) = &service.diesel_pools {
        let mut tx_conn = pools.tenant_transaction_conn(auth_token).await?;
        tx_conn
            .transaction(|conn| {
                Box::pin(async move {
                    save_paired_entries_and_update_summary_tx(
                        service,
                        conn,
                        auth_token,
                        auth_token.workspace_id(),
                        employment_id,
                        date,
                        start_of_day_utc,
                        next_day_utc,
                        new_entries,
                    )
                    .await
                })
            })
            .await
    } else {
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
            .recalculate_summary_with_retry(auth_token, employment_id, date)
            .await?;

        Ok(())
    }
}

/// [`save_paired_entries_and_update_summary`] variant that runs both writes on a
/// caller-owned transaction connection. If the summary recalculation fails after
/// the entries were replaced, the caller's transaction rollback undoes both.
#[allow(clippy::too_many_arguments)]
pub(crate) async fn save_paired_entries_and_update_summary_tx(
    service: &PairingService,
    conn: &mut AsyncPgConnection,
    auth_token: &TenantAuthToken,
    workspace_id: Uuid,
    employment_id: Uuid,
    date: NaiveDate,
    start_of_day_utc: DateTime<Utc>,
    next_day_utc: DateTime<Utc>,
    new_entries: Vec<TimeEntry>,
) -> Result<(), AppError> {
    service
        .time_entry_repo
        .replace_time_entries_tx(
            conn,
            workspace_id,
            employment_id,
            start_of_day_utc,
            next_day_utc,
            new_entries,
        )
        .await?;

    service
        .recalculate_summary_with_retry_tx(conn, auth_token, employment_id, date)
        .await
}
