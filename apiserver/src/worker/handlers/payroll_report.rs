use tracing::warn;

use crate::queue::Job;

pub struct PayrollReportHandler;

impl PayrollReportHandler {
    #[allow(clippy::unused_async)]
    pub async fn handle(job: &Job, _deps: &crate::worker::WorkerDeps) -> Result<(), String> {
        let _payload = job
            .report_payload()
            .ok_or_else(|| "Missing payroll report payload".to_string())?;

        warn!(
            job_id = %job.job_id,
            workspace_id = %job.workspace_id,
            "PayrollReport task received but not yet implemented (No-Op)"
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::queue::{DateRange, JobPayload, ReportConfig, ReportPayload, ReportType, TaskType};
    use chrono::NaiveDate;
    use uuid::Uuid;

    #[test]
    fn test_payload_extraction() {
        let job = Job::new(
            Uuid::new_v4(),
            0,
            TaskType::PayrollReport,
            Uuid::new_v4(),
            JobPayload::PayrollReport(ReportPayload {
                employee_ids: vec![Uuid::new_v4()],
                date_range: DateRange {
                    start: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                    end: NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
                },
                config: ReportConfig {
                    report_type: ReportType::Summary,
                    timezone: "UTC".to_string(),
                    rounding_policy: "floor".to_string(),
                    work_unit_minutes: 30,
                },
            }),
        );

        let payload = job.report_payload().expect("should extract");
        assert_eq!(payload.employee_ids.len(), 1);
    }

    #[test]
    fn test_wrong_payload_returns_none() {
        let job = Job::new(
            Uuid::new_v4(),
            0,
            TaskType::NightlyHousekeeping,
            Uuid::new_v4(),
            JobPayload::NightlyHousekeeping(crate::queue::HousekeepingPayload {
                reference_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            }),
        );

        assert!(job.report_payload().is_none());
    }

    #[tokio::test]
    async fn test_payroll_report_handler_success() {
        let deps = crate::worker::test_utils::MockWorkerDepsBuilder::new().build();

        let job = Job::new(
            Uuid::new_v4(),
            0,
            TaskType::PayrollReport,
            Uuid::new_v4(),
            JobPayload::PayrollReport(ReportPayload {
                employee_ids: vec![Uuid::new_v4()],
                date_range: DateRange {
                    start: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                    end: NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
                },
                config: ReportConfig {
                    report_type: ReportType::Summary,
                    timezone: "UTC".to_string(),
                    rounding_policy: "floor".to_string(),
                    work_unit_minutes: 30,
                },
            }),
        );

        let result = PayrollReportHandler::handle(&job, &deps).await;
        assert!(result.is_ok());
    }
}
