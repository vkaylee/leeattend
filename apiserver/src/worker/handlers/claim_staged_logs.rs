//! Claim Staged Logs Handler
//!
//! Claims orphan attendance staging logs for a newly created or imported employee.
//! Moves matching logs from `attendance_staging_logs` into `raw_attendance_logs`
//! and triggers pairing/reprocessing for affected dates.

use tracing::info;

use crate::queue::Job;

pub struct ClaimStagedLogsHandler;

impl ClaimStagedLogsHandler {
    pub async fn handle(
        job: &Job,
        deps: &crate::worker::WorkerDeps,
    ) -> Result<(), String> {
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
            .await?;

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
}
