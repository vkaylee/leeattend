use crate::diesel_pool::TenantAuthToken;
use crate::errors::AppError;
use crate::models::{RawAttendanceLog, TimeEntry};
use chrono::{DateTime, Duration, NaiveDate, Utc};
use chrono_tz::Tz;
use diesel_async::AsyncPgConnection;
use uuid::Uuid;

use crate::repositories::TimeEntryRepository;
use crate::services::pairing::service::PairingService;

use super::types::ADJACENT_ENTRY_SEARCH_DAYS;

const LOG_SEARCH_PADDING_HOURS: i64 = 12;

/// Fetch IDs of raw logs already consumed by adjacent days' time entries.
/// Extracted from the former `ConsumedLogTracker` struct to keep DB I/O
/// in the service layer (per coding-standards §7 — N+1 Prevention).
pub(crate) async fn fetch_consumed_log_ids(
    time_entry_repo: &dyn TimeEntryRepository,
    auth_token: &TenantAuthToken,
    employment_id: Uuid,
    date: NaiveDate,
    tz: Tz,
    start_of_day_utc: DateTime<Utc>,
    next_day_utc: DateTime<Utc>,
) -> Result<std::collections::HashSet<Uuid>, AppError> {
    let search_start = start_of_day_utc - Duration::days(ADJACENT_ENTRY_SEARCH_DAYS);
    let search_end = next_day_utc + Duration::days(ADJACENT_ENTRY_SEARCH_DAYS);
    let adjacent_entries = time_entry_repo
        .get_employment_time_entries(auth_token, employment_id, search_start, search_end)
        .await?;

    Ok(super::context_helpers::extract_consumed_ids(
        &adjacent_entries,
        date,
        tz,
    ))
}

pub(crate) async fn fetch_schedules_for_range(
    service: &PairingService,
    auth_token: &TenantAuthToken,
    employment_id: Uuid,
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> Result<crate::models::EmployeeSchedule, AppError> {
    let (schedules, _total) = service
        .roster_repo
        .get_workspace_schedules(
            None,
            auth_token,
            start_date,
            end_date,
            Some(&vec![employment_id]),
            None,
            None,
        )
        .await?;

    schedules.into_iter().next().ok_or_else(|| {
        AppError::Internal(format!(
            "No schedule found for employee {employment_id} in range {start_date} to {end_date}"
        ))
    })
}

pub(crate) async fn fetch_logs_and_entries_for_range(
    service: &PairingService,
    auth_token: &TenantAuthToken,
    workspace_id: Uuid,
    employment_id: Uuid,
    start_date: NaiveDate,
    end_date: NaiveDate,
    tz: Tz,
) -> Result<(Vec<RawAttendanceLog>, Vec<TimeEntry>), AppError> {
    let range_start_local = crate::utils::tz::local_to_tz(&tz, start_date, 0, 0, 0)?;
    let range_end_local =
        crate::utils::tz::local_to_tz(&tz, end_date + Duration::days(1), 0, 0, 0)?;

    let start_search_utc =
        (range_start_local - Duration::hours(LOG_SEARCH_PADDING_HOURS)).with_timezone(&Utc);
    let end_search_utc =
        (range_end_local + Duration::hours(LOG_SEARCH_PADDING_HOURS)).with_timezone(&Utc);

    let raw_logs = service
        .raw_log_repo
        .get_employment_logs_in_time_range(
            workspace_id,
            employment_id,
            start_search_utc,
            end_search_utc,
        )
        .await?;

    let search_start_utc =
        (range_start_local - Duration::days(ADJACENT_ENTRY_SEARCH_DAYS)).with_timezone(&Utc);
    let search_end_utc =
        (range_end_local + Duration::days(ADJACENT_ENTRY_SEARCH_DAYS)).with_timezone(&Utc);

    let adjacent_entries = service
        .time_entry_repo
        .get_employment_time_entries(auth_token, employment_id, search_start_utc, search_end_utc)
        .await?;

    Ok((raw_logs, adjacent_entries))
}

pub(crate) async fn clear_auto_entries(
    service: &PairingService,
    auth_token: &TenantAuthToken,
    employment_id: Uuid,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
) -> Result<(), AppError> {
    service
        .time_entry_repo
        .replace_time_entries(auth_token, employment_id, start, end, Vec::new())
        .await
}

/// Clear auto-generated entries inside an existing transaction.
pub(crate) async fn clear_auto_entries_tx(
    service: &PairingService,
    conn: &mut AsyncPgConnection,
    workspace_id: Uuid,
    employment_id: Uuid,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
) -> Result<(), AppError> {
    service
        .time_entry_repo
        .replace_time_entries_tx(conn, workspace_id, employment_id, start, end, Vec::new())
        .await
}

/// Recalculate the daily summary, retrying only the side-effect-free
/// compute phase so that destructive writes (cancel leave, upsert summary)
/// are executed at most once.
///
/// Without this split, a retry after `cancel_leave_request` succeeded but
/// `upsert_summary` failed would cancel the same leave again, causing a
/// double refund of the leave balance.
pub(crate) async fn recalculate_summary_with_retry(
    service: &PairingService,
    auth_token: &crate::diesel_pool::TenantAuthToken,
    employment_id: Uuid,
    date: NaiveDate,
) -> Result<(), AppError> {
    let summary_service = &service.summary_service;

    // Phase 1: Compute (retry-safe — reads + pure computation, no destructive writes)
    let plan = retry_with_log(
        "compute_daily_recalculation",
        auth_token.workspace_id(),
        employment_id,
        &format!("date={date}"),
        3,
        100,
        || summary_service.compute_daily_recalculation_plan(auth_token, employment_id, date),
    )
    .await?;

    // Phase 2: Persist once — MUST NOT retry destructive writes
    summary_service
        .execute_daily_recalculation_plan(auth_token, &plan)
        .await
}

/// Recalculate the daily summary inside an existing transaction, retrying the
/// recalculation on the same connection.
///
/// Each retry attempt runs in a nested transaction (SAVEPOINT), so a failed
/// attempt rolls back its partial writes without aborting the outer
/// transaction that holds the replaced time entries.
pub(crate) async fn recalculate_summary_with_retry_tx(
    service: &PairingService,
    conn: &mut AsyncPgConnection,
    auth_token: &crate::diesel_pool::TenantAuthToken,
    employment_id: Uuid,
    date: NaiveDate,
) -> Result<(), AppError> {
    let summary_service = &service.summary_service;
    let workspace_id = auth_token.workspace_id();
    let context = format!("date={date}");
    let mut attempts = 0;
    loop {
        attempts += 1;
        use diesel_async::AsyncConnection;
        let result = conn
            .transaction(|conn| {
                Box::pin(async move {
                    summary_service
                        .recalculate_daily_summary_tx(conn, auth_token, employment_id, date)
                        .await
                })
            })
            .await;
        match result {
            Ok(()) => return Ok(()),
            Err(e) => {
                if attempts >= 3 {
                    tracing::error!(
                        workspace_id = %workspace_id,
                        employment_id = %employment_id,
                        context = %context,
                        error = %e,
                        "Failed recalculate_daily_summary after {} attempts",
                        attempts,
                    );
                    return Err(e);
                }
                tracing::warn!(
                    workspace_id = %workspace_id,
                    employment_id = %employment_id,
                    context = %context,
                    error = %e,
                    attempt = attempts,
                    "Temporary failure in recalculate_daily_summary, retrying inside transaction...",
                );
                let base_delay = 100 * 2u64.pow(attempts - 1);
                let jitter = rand::random::<u64>() % 50;
                tokio::time::sleep(std::time::Duration::from_millis(base_delay + jitter)).await;
            }
        }
    }
}

pub(crate) async fn recalculate_range_summary_with_retry(
    service: &PairingService,
    auth_token: &crate::diesel_pool::TenantAuthToken,
    employment_id: Uuid,
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> Result<(), AppError> {
    let summary_service = &service.summary_service;
    retry_with_log(
        "recalculate_attendance_range",
        auth_token.workspace_id(),
        employment_id,
        &format!("range={start_date}..{end_date}"),
        3,
        100,
        || {
            summary_service.recalculate_attendance_range(
                auth_token,
                employment_id,
                start_date,
                end_date,
            )
        },
    )
    .await
}

/// Recalculate a range of daily summaries inside an existing transaction,
/// retrying the recalculation on the same connection via SAVEPOINTs.
pub(crate) async fn recalculate_range_summary_with_retry_tx(
    service: &PairingService,
    conn: &mut AsyncPgConnection,
    auth_token: &crate::diesel_pool::TenantAuthToken,
    employment_id: Uuid,
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> Result<(), AppError> {
    let summary_service = &service.summary_service;
    let workspace_id = auth_token.workspace_id();
    let context = format!("range={start_date}..{end_date}");
    let mut attempts = 0;
    loop {
        attempts += 1;
        use diesel_async::AsyncConnection;
        let result = conn
            .transaction(|conn| {
                Box::pin(async move {
                    summary_service
                        .recalculate_attendance_range_tx(
                            conn,
                            auth_token,
                            employment_id,
                            start_date,
                            end_date,
                        )
                        .await
                })
            })
            .await;
        match result {
            Ok(()) => return Ok(()),
            Err(e) => {
                if attempts >= 3 {
                    tracing::error!(
                        workspace_id = %workspace_id,
                        employment_id = %employment_id,
                        context = %context,
                        error = %e,
                        "Failed recalculate_attendance_range after {} attempts",
                        attempts,
                    );
                    return Err(e);
                }
                tracing::warn!(
                    workspace_id = %workspace_id,
                    employment_id = %employment_id,
                    context = %context,
                    error = %e,
                    attempt = attempts,
                    "Temporary failure in recalculate_attendance_range, retrying inside transaction...",
                );
                let base_delay = 100 * 2u64.pow(attempts - 1);
                let jitter = rand::random::<u64>() % 50;
                tokio::time::sleep(std::time::Duration::from_millis(base_delay + jitter)).await;
            }
        }
    }
}

/// Execute an async operation with retries and structured logging.
pub(crate) async fn retry_with_log<T, F, Fut>(
    operation_name: &str,
    workspace_id: Uuid,
    employment_id: Uuid,
    context: &str,
    max_attempts: u32,
    delay_ms: u64,
    f: F,
) -> Result<T, AppError>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, AppError>>,
{
    let mut attempts = 0;
    loop {
        attempts += 1;
        match f().await {
            Ok(val) => return Ok(val),
            Err(e) => {
                if attempts >= max_attempts {
                    tracing::error!(
                        workspace_id = %workspace_id,
                        employment_id = %employment_id,
                        context = %context,
                        error = %e,
                        "Failed {} after {} attempts",
                        operation_name,
                        max_attempts,
                    );
                    return Err(e);
                }
                tracing::warn!(
                    workspace_id = %workspace_id,
                    employment_id = %employment_id,
                    context = %context,
                    error = %e,
                    attempt = attempts,
                    "Temporary failure in {}, retrying...",
                    operation_name,
                );
                let base_delay = delay_ms * 2u64.pow(attempts - 1);
                let jitter = rand::random::<u64>() % 50;
                let delay = base_delay + jitter;
                tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
            }
        }
    }
}
