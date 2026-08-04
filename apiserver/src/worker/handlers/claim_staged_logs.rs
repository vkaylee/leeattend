//! Claim Staged Logs Handler
//!
//! Claims orphan attendance staging logs for a newly created or imported employee.
//! Moves matching logs from `attendance_staging_logs` into `raw_attendance_logs`
//! (atomically via repo transaction) and triggers pairing/reprocessing for affected dates.

use tracing::{error, info};

use crate::queue::Job;

pub struct ClaimStagedLogsHandler;

impl ClaimStagedLogsHandler {
    pub async fn handle(job: &Job, deps: &crate::worker::WorkerDeps) -> Result<(), String> {
        let payload = job
            .claim_staged_logs_payload()
            .ok_or("Invalid payload for ClaimStagedLogs job")?;

        // 1. Fetch punch times BEFORE claiming (for pairing reprocessing)
        let staged_logs = deps
            .staging_repo
            .get_pending_staged_logs(payload.workspace_id, &payload.employee_id_number)
            .await
            .map_err(|e| format!("Failed to fetch staged logs: {e}"))?;

        // Resolve workspace timezone for correct local-date grouping
        let tz = deps
            .workspace_repo
            .get_workspace_by_id(payload.workspace_id)
            .await
            .map_err(|e| format!("Failed to fetch workspace: {e}"))?
            .map(|ws| crate::utils::tz::parse_workspace_tz(ws.timezone()))
            .unwrap_or(chrono_tz::UTC);

        let dates_to_reprocess: std::collections::HashSet<chrono::NaiveDate> = staged_logs
            .iter()
            .map(|log| log.punch_time.with_timezone(&tz).date_naive())
            .collect();

        // 2. Atomic claim: INSERT raw logs + UPDATE staging status in one transaction
        let claimed = deps
            .staging_repo
            .claim_staged_logs(
                payload.workspace_id,
                &payload.employee_id_number,
                payload.employee_id,
            )
            .await
            .map_err(|e| e.to_string())?;

        info!(
            employee_id_number = %payload.employee_id_number,
            claimed_count = claimed,
            "ClaimStagedLogs job completed"
        );

        // 3. Trigger pairing reprocessing for affected dates (best-effort, non-fatal)
        if claimed > 0 {
            let pairing_svc = deps.pairing_service();

            for date in dates_to_reprocess {
                if let Err(e) = pairing_svc
                    .reprocess_daily_attendance(payload.workspace_id, payload.employee_id, date, tz)
                    .await
                {
                    error!(
                        employee_id = %payload.employee_id,
                        date = %date,
                        error = %e,
                        "Pairing reprocessing failed after claim"
                    );
                }
            }
        }

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
        use crate::repositories::roster::MockRosterRepository;
        use crate::repositories::staging_log_repository::MockStagingLogRepository;
        use crate::repositories::staging_log_repository::StagingLog;
        use crate::worker::test_utils::MockWorkerDepsBuilder;
        use chrono::Utc;

        let ws_id = Uuid::new_v4();
        let emp_id = Uuid::new_v4();

        let mut staging_repo = MockStagingLogRepository::new();

        // Mock get_pending_staged_logs — returns one staged log
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

        // Mock claim_staged_logs — atomic repo call returns 1 claimed
        staging_repo
            .expect_claim_staged_logs()
            .with(
                mockall::predicate::eq(ws_id),
                mockall::predicate::eq("EMP001"),
                mockall::predicate::eq(emp_id),
            )
            .returning(|_, _, _| Ok(1));

        // Pairing service deps (for reprocessing after claim)
        let mut roster_repo = MockRosterRepository::new();
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

        let mut raw_log_repo =
            crate::repositories::raw_attendance_log_repository::MockRawAttendanceLogRepository::new(
            );
        raw_log_repo
            .expect_get_employment_logs_in_time_range()
            .returning(|_, _, _, _| Box::pin(async { Ok(vec![]) }));

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
            .returning(|_, _, _, _, _, _, _, _, _, _, _, _, _| Ok(()));

        let mut builder = MockWorkerDepsBuilder::new();
        builder
            .workspace_repo
            .expect_get_workspace_by_id()
            .returning(move |_| {
                Box::pin(async move {
                    let ws = crate::models::Workspace::new(NewWorkspaceParams {
            id: ws_id,
            name: "Test Workspace".to_string(),
            date_format: "YYYY-MM-DD".to_string(),
            time_format: "HH:mm".to_string(),
            timezone: "UTC".to_string(),
            theme_primary_color: "#000000".to_string(),
            default_language: "en".to_string(),
            logo_url: None,
            tagline: None,
            holiday_overnight_strategy: "split".to_string(),
            default_holiday_ot_multiplier: rust_decimal::Decimal::new(15, 1),
            created_at: Utc::now(),
            custom_db_url_encrypted: None,
        });
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
