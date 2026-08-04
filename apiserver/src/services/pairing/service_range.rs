use crate::diesel_pool::TenantAuthToken;
use crate::errors::AppError;
use crate::models::{RawAttendanceLog, TimeEntry};
use chrono::{Duration, NaiveDate, Utc};
use chrono_tz::Tz;
use diesel_async::{AsyncConnection, AsyncPgConnection};
use tracing::info;
use uuid::Uuid;

use crate::services::pairing::context_helpers;
use crate::services::pairing::service::PairingService;

pub(crate) async fn reprocess_attendance_range(
    service: &PairingService,
    workspace_id: Uuid,
    employment_id: Uuid,
    start_date: NaiveDate,
    end_date: NaiveDate,
    tz: Tz,
) -> Result<(), AppError> {
    info!(
        workspace_id = %workspace_id,
        employment_id = %employment_id,
        start_date = %start_date,
        end_date = %end_date,
        "Reprocessing range attendance"
    );

    let auth_token = TenantAuthToken::system_override(workspace_id);

    let emp_sched = service
        .fetch_schedules_for_range(&auth_token, employment_id, start_date, end_date)
        .await?;

    let (raw_logs, adjacent_entries) = service
        .fetch_logs_and_entries_for_range(
            &auth_token,
            workspace_id,
            employment_id,
            start_date,
            end_date,
            tz,
        )
        .await?;

    let new_entries = process_range_days(
        start_date,
        end_date,
        &emp_sched,
        tz,
        workspace_id,
        employment_id,
        &raw_logs,
        &adjacent_entries,
    )?;

    // Entries replacement, range summary recalculation, and the overnight tail
    // (next-day reprocessing + propagation) all run inside ONE transaction so a
    // mid-pipeline failure rolls back the entire write set atomically.
    if let Some(pools) = &service.diesel_pools {
        let mut tx_conn = pools.tenant_transaction_conn(&auth_token).await?;
        tx_conn
            .transaction(|conn| {
                Box::pin(async move {
                    save_range_entries_tx(
                        service,
                        conn,
                        &auth_token,
                        workspace_id,
                        employment_id,
                        start_date,
                        end_date,
                        tz,
                        new_entries,
                    )
                    .await?;
                    handle_overnight_tail(
                        service,
                        Some(conn),
                        workspace_id,
                        employment_id,
                        end_date,
                        &emp_sched,
                        tz,
                    )
                    .await
                })
            })
            .await
    } else {
        save_range_entries(
            service,
            &auth_token,
            workspace_id,
            employment_id,
            start_date,
            end_date,
            tz,
            new_entries,
        )
        .await?;

        handle_overnight_tail(
            service,
            None,
            workspace_id,
            employment_id,
            end_date,
            &emp_sched,
            tz,
        )
        .await
    }
}

#[allow(clippy::too_many_arguments)]
fn process_range_days(
    curr_date: NaiveDate,
    end_date: NaiveDate,
    emp_sched: &crate::models::EmployeeSchedule,
    tz: Tz,
    workspace_id: Uuid,
    employment_id: Uuid,
    raw_logs: &[RawAttendanceLog],
    adjacent_entries: &[TimeEntry],
) -> Result<Vec<TimeEntry>, AppError> {
    let mut new_entries_accumulated = Vec::new();
    let mut curr = curr_date;

    let schedule_map: std::collections::HashMap<NaiveDate, &crate::models::EffectiveSchedule> =
        emp_sched
            .schedules()
            .iter()
            .map(|s| (s.date(), s))
            .collect();

    while curr <= end_date {
        if let Some(sched) = schedule_map.get(&curr) {
            let day_new_entries = process_single_day_in_range(
                curr,
                tz,
                workspace_id,
                employment_id,
                sched,
                raw_logs,
                adjacent_entries,
                &new_entries_accumulated,
            )?;
            new_entries_accumulated.extend(day_new_entries);
        }
        curr = curr.succ_opt().unwrap_or(curr);
    }
    Ok(new_entries_accumulated)
}

#[allow(clippy::too_many_arguments)]
fn process_single_day_in_range(
    curr_date: NaiveDate,
    tz: Tz,
    workspace_id: Uuid,
    employment_id: Uuid,
    sched: &crate::models::schedule::EffectiveSchedule,
    raw_logs: &[RawAttendanceLog],
    adjacent_entries: &[TimeEntry],
    new_entries_accumulated: &[TimeEntry],
) -> Result<Vec<TimeEntry>, AppError> {
    let context = crate::services::pairing::types::PairingContext::new(curr_date, tz);
    let shift_config = context.build_shift_config_from_schedule(sched);

    tracing::info!(
        employment_id = %employment_id,
        date = %curr_date,
        has_shift_config = shift_config.is_some(),
        shift_start = ?sched.start_time(),
        shift_end = ?sched.end_time(),
        break_start = ?sched.break_start(),
        break_end = ?sched.break_end(),
        is_break_auto_deduct = ?sched.is_break_auto_deduct(),
        "process_single_day_in_range: shift_config built"
    );

    // Calculate fetch window for this specific day
    let (window_start_utc, window_end_utc) =
        context.calculate_fetch_window(shift_config.as_ref())?;

    // Exclude logs consumed by adjacent days (excluding current date)
    let mut consumed_log_ids =
        context_helpers::extract_consumed_ids(adjacent_entries, curr_date, tz);

    // Also exclude logs already consumed by previously processed days in this batch
    context_helpers::extend_consumed_from_entries(&mut consumed_log_ids, new_entries_accumulated);

    let filtered_raw_logs: Vec<RawAttendanceLog> = raw_logs
        .iter()
        .filter(|log| log.punch_time() >= window_start_utc && log.punch_time() <= window_end_utc)
        .filter(|log| !consumed_log_ids.contains(&log.id()))
        .cloned()
        .collect();

    let filtered_log_events = context.build_log_events(&filtered_raw_logs)?;

    if filtered_log_events.is_empty() {
        return Ok(Vec::new());
    }

    let strategy = crate::services::pairing::strategy::get_strategy();
    let sessions = strategy.pair(&filtered_log_events, shift_config.as_ref());

    let traces_val = serde_json::json!(sched.traces());
    context.build_time_entries(
        workspace_id,
        employment_id,
        sched.shift_id(),
        sessions,
        &traces_val,
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) async fn save_range_entries(
    service: &PairingService,
    auth_token: &TenantAuthToken,
    _workspace_id: Uuid,
    employment_id: Uuid,
    start_date: NaiveDate,
    end_date: NaiveDate,
    tz: Tz,
    new_entries: Vec<TimeEntry>,
) -> Result<(), AppError> {
    if let Some(pools) = &service.diesel_pools {
        let mut tx_conn = pools.tenant_transaction_conn(auth_token).await?;
        tx_conn
            .transaction(|conn| {
                Box::pin(async move {
                    save_range_entries_tx(
                        service,
                        conn,
                        auth_token,
                        auth_token.workspace_id(),
                        employment_id,
                        start_date,
                        end_date,
                        tz,
                        new_entries,
                    )
                    .await
                })
            })
            .await
    } else {
        let (range_start_utc, range_end_utc) = range_utc_bounds(tz, start_date, end_date)?;

        service
            .time_entry_repo
            .replace_time_entries(
                auth_token,
                employment_id,
                range_start_utc,
                range_end_utc,
                new_entries,
            )
            .await?;

        service
            .recalculate_range_summary_with_retry(auth_token, employment_id, start_date, end_date)
            .await?;

        Ok(())
    }
}

/// [`save_range_entries`] variant that runs both writes on a caller-owned
/// transaction connection. If the summary recalculation fails after the entries
/// were replaced, the caller's transaction rollback undoes both.
#[allow(clippy::too_many_arguments)]
pub(crate) async fn save_range_entries_tx(
    service: &PairingService,
    conn: &mut AsyncPgConnection,
    auth_token: &TenantAuthToken,
    workspace_id: Uuid,
    employment_id: Uuid,
    start_date: NaiveDate,
    end_date: NaiveDate,
    tz: Tz,
    new_entries: Vec<TimeEntry>,
) -> Result<(), AppError> {
    let (range_start_utc, range_end_utc) = range_utc_bounds(tz, start_date, end_date)?;

    service
        .time_entry_repo
        .replace_time_entries_tx(
            conn,
            workspace_id,
            employment_id,
            range_start_utc,
            range_end_utc,
            new_entries,
        )
        .await?;

    service
        .recalculate_range_summary_with_retry_tx(
            conn,
            auth_token,
            employment_id,
            start_date,
            end_date,
        )
        .await
}

fn range_utc_bounds(
    tz: Tz,
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> Result<(chrono::DateTime<Utc>, chrono::DateTime<Utc>), AppError> {
    let range_start_local = crate::utils::tz::local_to_tz(&tz, start_date, 0, 0, 0)?;
    let range_end_local =
        crate::utils::tz::local_to_tz(&tz, end_date + Duration::days(1), 0, 0, 0)?;
    let range_start_utc = range_start_local.with_timezone(&Utc);
    let range_end_utc = range_end_local.with_timezone(&Utc);
    Ok((range_start_utc, range_end_utc))
}

// `as_deref_mut` reborrows the `Option<&mut _>` for two sequential calls on the
// same connection; the type is unchanged so the lint is a false positive here.
#[allow(clippy::needless_option_as_deref)]
async fn handle_overnight_tail(
    service: &PairingService,
    mut conn: Option<&mut AsyncPgConnection>,
    workspace_id: Uuid,
    employment_id: Uuid,
    end_date: NaiveDate,
    emp_sched: &crate::models::EmployeeSchedule,
    tz: Tz,
) -> Result<(), AppError> {
    let schedule_map: std::collections::HashMap<NaiveDate, &crate::models::EffectiveSchedule> =
        emp_sched
            .schedules()
            .iter()
            .map(|s| (s.date(), s))
            .collect();

    let last_day_is_overnight = if let Some(sched) = schedule_map.get(&end_date) {
        if let (Some(st), Some(et)) = (sched.start_time(), sched.end_time()) {
            et < st
        } else {
            false
        }
    } else {
        false
    };

    if last_day_is_overnight {
        let next_date = end_date + Duration::days(1);
        info!(
            workspace_id = %workspace_id,
            employment_id = %employment_id,
            next_date = %next_date,
            "Last day of range is overnight, triggering next day reprocessing for self-healing"
        );
        let next_shift_config = super::service_single::reprocess_single_day_with_conn(
            service,
            conn.as_deref_mut(),
            workspace_id,
            employment_id,
            next_date,
            tz,
            None,
        )
        .await?;
        super::service_propagation::propagate_overnight_reprocess_with_conn(
            service,
            conn.as_deref_mut(),
            workspace_id,
            employment_id,
            next_date,
            tz,
            next_shift_config,
        )
        .await?;
    }

    Ok(())
}
