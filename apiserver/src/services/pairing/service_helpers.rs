use crate::diesel_pool::TenantAuthToken;
use crate::errors::AppError;
use crate::models::{RawAttendanceLog, TimeEntry};
use chrono::{DateTime, Duration, NaiveDate, Utc};
use chrono_tz::Tz;
use uuid::Uuid;

use crate::services::pairing::service::PairingService;

use super::types::ADJACENT_ENTRY_SEARCH_DAYS;

const LOG_SEARCH_PADDING_HOURS: i64 = 12;

pub(crate) async fn fetch_schedules_for_range(
    service: &PairingService,
    auth_token: &TenantAuthToken,
    employment_id: Uuid,
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> Result<crate::models::EmployeeSchedule, AppError> {
    let schedules = service
        .roster_repo
        .get_workspace_schedules(
            None,
            auth_token,
            start_date,
            end_date,
            Some(&vec![employment_id]),
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

pub(crate) async fn recalculate_summary_with_retry(
    service: &PairingService,
    workspace_id: Uuid,
    employment_id: Uuid,
    date: NaiveDate,
) -> Result<(), AppError> {
    let summary_service = &service.summary_service;
    retry_with_log(
        "recalculate_daily_summary",
        workspace_id,
        employment_id,
        &format!("date={date}"),
        3,
        100,
        || summary_service.recalculate_daily_summary(workspace_id, employment_id, date),
    )
    .await
}

pub(crate) async fn recalculate_range_summary_with_retry(
    service: &PairingService,
    workspace_id: Uuid,
    employment_id: Uuid,
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> Result<(), AppError> {
    let summary_service = &service.summary_service;
    retry_with_log(
        "recalculate_attendance_range",
        workspace_id,
        employment_id,
        &format!("range={start_date}..{end_date}"),
        3,
        100,
        || {
            summary_service.recalculate_attendance_range(
                workspace_id,
                employment_id,
                start_date,
                end_date,
            )
        },
    )
    .await
}

/// Execute an async operation with retries and structured logging.
pub(crate) async fn retry_with_log<F, Fut>(
    operation_name: &str,
    workspace_id: Uuid,
    employment_id: Uuid,
    context: &str,
    max_attempts: u32,
    delay_ms: u64,
    f: F,
) -> Result<(), AppError>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<(), AppError>>,
{
    let mut attempts = 0;
    loop {
        attempts += 1;
        match f().await {
            Ok(()) => return Ok(()),
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
