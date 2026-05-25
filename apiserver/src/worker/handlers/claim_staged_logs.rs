//! Claim Staged Logs Handler
//!
//! Claims orphan attendance staging logs for a newly created or imported employee.
//! Moves matching logs from `attendance_staging_logs` into `raw_attendance_logs`
//! and triggers pairing/reprocessing for affected dates.

use tracing::info;

use crate::queue::Job;

pub struct ClaimStagedLogsHandler;

impl ClaimStagedLogsHandler {
    pub async fn handle(job: &Job, deps: &crate::worker::WorkerDeps) -> Result<(), String> {
        let payload = job
            .claim_staged_logs_payload()
            .ok_or("Invalid payload for ClaimStagedLogs job")?;

        let (log_svc, _handle) = deps.log_service();

        let claimed = log_svc
            .claim_staged_logs(
                payload.workspace_id,
                payload.employee_id,
                &payload.employee_id_number,
            )
            .await
            .map_err(|e| e.to_string())?;

        info!(
            employee_id_number = %payload.employee_id_number,
            claimed_count = claimed,
            "ClaimStagedLogs job completed"
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::queue::{ClaimStagedLogsPayload, JobPayload, TaskType};
    use uuid::Uuid;

    #[test]
    fn test_payload_extraction() {
        let ws_id = Uuid::new_v4();
        let emp_id = Uuid::new_v4();
        let job = Job::new(
            Uuid::new_v4(),
            0,
            TaskType::ClaimStagedLogs,
            ws_id,
            JobPayload::ClaimStagedLogs(ClaimStagedLogsPayload {
                workspace_id: ws_id,
                employee_id: emp_id,
                employee_id_number: "EMP001".to_string(),
            }),
        );

        let payload = job.claim_staged_logs_payload().expect("should extract");
        assert_eq!(payload.workspace_id, ws_id);
        assert_eq!(payload.employee_id, emp_id);
        assert_eq!(payload.employee_id_number, "EMP001");
    }

    #[test]
    fn test_wrong_payload_returns_none() {
        let job = Job::new(
            Uuid::new_v4(),
            0,
            TaskType::LogIngestion,
            Uuid::new_v4(),
            JobPayload::LogIngestion(crate::queue::LogIngestionPayload {
                workspace_id: Uuid::new_v4(),
                employee_id_number: "EMP001".to_string(),
                punch_time_raw: "2026-01-01 08:00:00".to_string(),
                device_id: None,
                raw_data: None,
                timezone: "UTC".to_string(),
                date_fmt: "YYYY-MM-DD".to_string(),
                time_fmt: "24h".to_string(),
            }),
        );

        assert!(job.claim_staged_logs_payload().is_none());
    }

    #[tokio::test]
    async fn test_claim_staged_logs_handler_success() {
        use crate::repositories::raw_attendance_log_repository::MockRawAttendanceLogRepository;
        use crate::repositories::roster::MockRosterRepository;
        use crate::repositories::staging_log_repository::MockStagingLogRepository;
        use crate::repositories::staging_log_repository::StagingLog;
        use crate::worker::test_utils::MockWorkerDepsBuilder;
        use chrono::Utc;

        let ws_id = Uuid::new_v4();
        let emp_id = Uuid::new_v4();

        let mut staging_repo = MockStagingLogRepository::new();
        staging_repo
            .expect_get_pending_staged_logs()
            .with(
                mockall::predicate::eq(ws_id),
                mockall::predicate::eq("EMP001"),
            )
            .returning(move |_, _| {
                Ok(vec![StagingLog {
                    id: Uuid::new_v4(),
                    workspace_id: ws_id,
                    device_id: Some("D1".to_string()),
                    raw_user_id: "EMP001".to_string(),
                    punch_time: Utc::now(),
                    raw_data: None,
                    status: "pending".to_string(),
                    fingerprint: "hash".to_string(),
                    created_at: Utc::now(),
                }])
            });

        staging_repo
            .expect_update_staging_status()
            .returning(|_, _, _| Ok(true));

        let mut raw_log_repo = MockRawAttendanceLogRepository::new();
        raw_log_repo
            .expect_create_raw_log()
            .returning(|_, _| Box::pin(async { Ok(true) }));
        raw_log_repo
            .expect_get_employee_logs_in_time_range()
            .returning(|_, _, _, _| Box::pin(async { Ok(vec![]) }));

        let mut roster_repo = MockRosterRepository::new();
        roster_repo
            .expect_get_effective_shift()
            .returning(|_, _, _| Box::pin(async { Ok((None, vec![])) }));
        roster_repo
            .expect_get_workspace_schedules()
            .returning(|_, _, _, _, _| Box::pin(async { Ok(vec![]) }));

        let mut time_entry_repo =
            crate::repositories::time_entry_repository::MockTimeEntryRepository::new();
        time_entry_repo
            .expect_replace_time_entries()
            .returning(|_, _, _, _, _| Ok(()));

        let mut daily_summary_repo = crate::repositories::attendance_daily_summary_repository::MockAttendanceDailySummaryRepository::new();
        daily_summary_repo
            .expect_upsert_calculated_summary()
            .returning(|_, _, _, _, _, _, _, _| Ok(()));

        let mut builder = MockWorkerDepsBuilder::new();
        builder
            .workspace_repo
            .expect_get_workspace_by_id()
            .returning(move |_| {
                Box::pin(async move {
                    let ws = crate::models::Workspace {
                        id: ws_id,
                        name: "Test Workspace".to_string(),
                        date_format: "YYYY-MM-DD".to_string(),
                        time_format: "HH:mm".to_string(),
                        timezone: "UTC".to_string(),
                        theme_primary_color: "#000000".to_string(),
                        default_language: "en".to_string(),
                        logo_url: None,
                        tagline: None,
                        custom_db_url_encrypted: None,
                        holiday_overnight_strategy: "split".to_string(),
                        default_holiday_ot_multiplier: rust_decimal::Decimal::new(15, 1),
                        created_at: Utc::now(),
                    };
                    Ok(Some(ws))
                })
            });
        builder.staging_repo = staging_repo;
        builder.raw_log_repo = raw_log_repo;
        builder.roster_repo = roster_repo;
        builder.time_entry_repo = time_entry_repo;
        builder.daily_summary_repo = daily_summary_repo;

        let deps = builder.build();

        let job = Job::new(
            Uuid::new_v4(),
            0,
            TaskType::ClaimStagedLogs,
            ws_id,
            JobPayload::ClaimStagedLogs(ClaimStagedLogsPayload {
                workspace_id: ws_id,
                employee_id: emp_id,
                employee_id_number: "EMP001".to_string(),
            }),
        );

        let result = ClaimStagedLogsHandler::handle(&job, &deps).await;

        assert!(result.is_ok(), "Handler failed: {:?}", result.err());
    }
}
