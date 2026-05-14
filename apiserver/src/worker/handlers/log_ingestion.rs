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
    pub async fn handle(
        diesel: &crate::diesel_pool::DieselPools,
        job: &Job,
        deps: &crate::worker::WorkerDeps,
    ) -> Result<(), String> {
        let payload = job
            .log_ingestion_payload()
            .ok_or("Invalid payload for LogIngestion job")?;

        let (log_svc, _handle) = deps.log_service(diesel);

        log_svc
            .process_log(crate::services::attendance_log_service::ProcessLogArgs {
                workspace_id: payload.workspace_id,
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
                    rounding_policy: "floor".to_string(),
                    work_unit_minutes: 30,
                },
            }),
        );

        assert!(job.log_ingestion_payload().is_none());
    }
}
