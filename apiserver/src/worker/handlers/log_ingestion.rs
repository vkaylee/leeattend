//! Log Ingestion Handler
//!
//! Processes attendance log entries from the job queue.
//! Each job contains a single log entry with workspace config,
//! which is routed through the existing `AttendanceLogService::process_log`.

use crate::models::LogSource;
use crate::queue::Job;

pub struct LogIngestionHandler;

impl LogIngestionHandler {
    /// Process a single attendance log ingestion job.
    ///
    /// Delegates to `AttendanceLogService::process_log()` which handles:
    /// - Employee lookup
    /// - Timestamp parsing with timezone
    /// - DB insert (with ON CONFLICT dedup)
    /// - Pairing logic
    /// - Unknown user staging
    /// - Error logging
    pub async fn handle(job: &Job, deps: &crate::worker::WorkerDeps) -> Result<(), String> {
        let payload = job
            .log_ingestion_payload()
            .ok_or("Invalid payload for LogIngestion job")?;

        let log_svc = deps.log_service();

        let auth_token = crate::diesel_pool::TenantAuthToken::system_override(payload.workspace_id);
        log_svc
            .process_log(crate::services::attendance_log_service::ProcessLogArgs {
                workspace_id: payload.workspace_id,
                auth_token: &auth_token,
                employee_id_number: &payload.employee_id_number,
                punch_time_raw: &payload.punch_time_raw,
                source: LogSource::Device,
                device_id: payload.device_id.clone(),
                raw_data: payload.raw_data.clone(),
                timezone: &payload.timezone,
                date_fmt: Some(&payload.date_fmt),
                time_fmt: Some(&payload.time_fmt),
            })
            .await
            .map(|_| ())
            .map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::queue::{JobPayload, JobPriority, LogIngestionPayload, TaskType};
    use uuid::Uuid;

    #[test]
    fn test_payload_extraction() {
        let job = Job::with_priority(
            Uuid::new_v4(),
            0,
            TaskType::LogIngestion,
            Uuid::new_v4(),
            JobPayload::LogIngestion(LogIngestionPayload {
                workspace_id: Uuid::new_v4(),
                employee_id_number: "EMP001".to_string(),
                punch_time_raw: "2026-01-26 09:00:00".to_string(),
                device_id: Some("D1".to_string()),
                raw_data: None,
                timezone: "Asia/Ho_Chi_Minh".to_string(),
                date_fmt: "DD/MM/YYYY".to_string(),
                time_fmt: "24h".to_string(),
            }),
            JobPriority::Low,
        );

        let payload = job.log_ingestion_payload().expect("should extract");
        assert_eq!(payload.employee_id_number, "EMP001");
        assert_eq!(payload.timezone, "Asia/Ho_Chi_Minh");
        assert_eq!(job.priority, JobPriority::Low);
    }

    #[test]
    fn test_wrong_payload_returns_none() {
        let job = Job::new(
            Uuid::new_v4(),
            0,
            TaskType::AttendanceReport,
            Uuid::new_v4(),
            JobPayload::AttendanceReport(crate::queue::ReportPayload {
                employee_ids: vec![],
                date_range: crate::queue::DateRange {
                    start: chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                    end: chrono::NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
                },
                config: crate::queue::ReportConfig {
                    report_type: crate::queue::ReportType::Summary,
                    timezone: "UTC".to_string(),
                    rounding_policy: crate::models::RoundingPolicy::Floor,
                    work_unit_minutes: 30,
                },
            }),
        );

        assert!(job.log_ingestion_payload().is_none());
    }

    #[tokio::test]
    async fn test_log_ingestion_handler_with_mocks() {
        use crate::models::Employee;
use crate::models::{NewEmploymentParams, NewWorkspaceParams};
        use crate::worker::test_utils::MockWorkerDepsBuilder;

        let ws_id = Uuid::new_v4();
        let emp_id = Uuid::new_v4();

        let employment_id = Uuid::new_v4();

        // Mock Employee repo to return an active employee
        let mut emp_repo = crate::repositories::employee::MockEmployeeRepository::new();
        emp_repo
            .expect_find_by_number()
            .withf(move |ws, id| {
                *ws == crate::diesel_pool::TenantAuthToken::system_override(ws_id) && id == "1"
            })
            .returning(move |_, _| {
                let emp = Employee::new(
                    emp_id,
                    ws_id,
                    None,
                    "Test User".to_string(),
                    "EMP001".to_string(),
                    vec![],
                    chrono::Utc::now(),
                    chrono::Utc::now(),
                    None,
                )
                .with_employment_info(Some(employment_id), None, None);
                Ok(Some(emp))
            });

        emp_repo
            .expect_get_employee_timezone_fallback()
            .returning(|_, _| Ok(chrono_tz::UTC));
        emp_repo
            .expect_get_employee_timezone_at_date()
            .returning(|_, _, _| Ok(chrono_tz::UTC));

        // Mock Raw log repo to succeed on insert
        let mut raw_log_repo =
            crate::repositories::raw_attendance_log_repository::MockRawAttendanceLogRepository::new(
            );
        raw_log_repo
            .expect_has_duplicate_raw_log()
            .returning(|_, _, _, _| Box::pin(async { Ok(false) }));
        raw_log_repo
            .expect_create_raw_log()
            .returning(|_, _| Box::pin(async { Ok(true) }));
        raw_log_repo
            .expect_get_employment_logs_in_time_range()
            .returning(|_, _, _, _| Box::pin(async { Ok(vec![]) }));

        // Mock Staging repo just in case
        let staging_repo =
            crate::repositories::staging_log_repository::MockStagingLogRepository::new();
        let error_repo = crate::repositories::attendance_log_error_repository::MockAttendanceLogErrorRepository::new();

        let mut workspace_repo =
            crate::repositories::workspace_repository::MockWorkspaceRepository::new();
        workspace_repo
            .expect_get_workspace_by_id()
            .with(mockall::predicate::eq(ws_id))
            .returning(move |_| {
                use rust_decimal::Decimal;
                Box::pin(async move {
                    Ok(Some(crate::models::Workspace::new(NewWorkspaceParams {
            id: ws_id,
            name: "Test Workspace".to_string(),
            date_format: "YYYY-MM-DD".to_string(),
            time_format: "HH:mm".to_string(),
            timezone: "UTC".to_string(),
            theme_primary_color: "#000000".to_string(),
            default_language: "en".to_string(),
            logo_url: None,
            tagline: None,
            holiday_overnight_strategy: "NextDay".to_string(),
            default_holiday_ot_multiplier: Decimal::new(100, 2),
            created_at: chrono::Utc::now(),
            custom_db_url_encrypted: None,
        })))
                })
            });

        let mut roster_repo = crate::repositories::roster::MockRosterRepository::new();
        roster_repo
            .expect_get_effective_shift()
            .returning(|_, _, _| {
                Box::pin(async {
                    Ok((
                        None,
                        vec![],
                        std::sync::Arc::new(crate::cache::RosterData::default()),
                    ))
                })
            });
        roster_repo
            .expect_get_workspace_schedules()
            .returning(|_, _, _, _, _, _, _| Box::pin(async { Ok((vec![], 0)) }));

        let mut time_entry_repo =
            crate::repositories::time_entry_repository::MockTimeEntryRepository::new();
        time_entry_repo
            .expect_replace_time_entries()
            .returning(|_, _, _, _, _| Ok(()));
        time_entry_repo
            .expect_get_employment_time_entries()
            .returning(|_, _, _, _| Ok(vec![]));

        let mut daily_summary_repo = crate::repositories::attendance_daily_summary_repository::MockAttendanceDailySummaryRepository::new();
        daily_summary_repo
            .expect_upsert_calculated_summary()
            .returning(|_| Ok(()));

        // Mock Employment repo to return the employment for the punch date
        let mut employment_repo =
            crate::repositories::employment_repository::MockEmploymentRepository::new();
        employment_repo
            .expect_resolve_employment_on_date()
            .returning(move |_, _, _| {
                Ok(crate::models::employment::EmploymentResolution::Resolved(
                    crate::models::employment::Employment::new(NewEmploymentParams {
                        id: employment_id,
                        workspace_id: ws_id,
                        employee_id: emp_id,
                        employer_id: uuid::Uuid::new_v4(),
                        primary_location_id: None,
                        department_id: None,
                        status: "active".to_string(),
                        start_date: chrono::NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
                        end_date: None,
                        job_title: None,
                        employment_type: "full_time".to_string(),
                        manager_employment_id: None,
                        created_at: chrono::Utc::now(),
                        updated_at: chrono::Utc::now(),
                        termination_reason: None,
                        notes: None,
                    }),
                ))
            });
        employment_repo
            .expect_get_employments_for_employees()
            .returning(|_, _| Ok(vec![]));
        employment_repo
            .expect_get_employment_by_id()
            .returning(move |_, id| {
                Ok(Some(crate::models::employment::Employment::new(NewEmploymentParams {
                    id: id,
                    workspace_id: ws_id,
                    employee_id: emp_id,
                    employer_id: uuid::Uuid::new_v4(),
                    primary_location_id: None,
                    department_id: None,
                    status: "active".to_string(),
                    start_date: chrono::NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
                    end_date: None,
                    job_title: None,
                    employment_type: "full_time".to_string(),
                    manager_employment_id: None,
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                    termination_reason: None,
                    notes: None,
                })))
            });

        let mut builder = MockWorkerDepsBuilder::new();
        builder.employee_repo = emp_repo;
        builder.employment_repo = employment_repo;
        builder.raw_log_repo = raw_log_repo;
        builder.staging_repo = staging_repo;
        builder.error_repo = error_repo;
        builder.workspace_repo = workspace_repo;
        builder.roster_repo = roster_repo;
        builder.time_entry_repo = time_entry_repo;
        builder.daily_summary_repo = daily_summary_repo;
        let deps = builder.build();

        let job = Job::with_priority(
            Uuid::new_v4(),
            0,
            TaskType::LogIngestion,
            ws_id,
            JobPayload::LogIngestion(crate::queue::LogIngestionPayload {
                workspace_id: ws_id,
                employee_id_number: "EMP001".to_string(),
                punch_time_raw: "2026-01-26 09:00:00".to_string(),
                device_id: Some("D1".to_string()),
                raw_data: None,
                timezone: "Asia/Ho_Chi_Minh".to_string(),
                date_fmt: "YYYY-MM-DD".to_string(),
                time_fmt: "24h".to_string(),
            }),
            JobPriority::High,
        );

        let result = LogIngestionHandler::handle(&job, &deps).await;

        assert!(result.is_ok(), "Handler failed: {:?}", result.err());
    }
}
