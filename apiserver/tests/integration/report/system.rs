//! Report system logic tests (type safety checks)

use apiserver::queue::{
    DateRange, Job, JobPayload, JobPriority, ReportConfig, ReportPayload, ReportType, TaskType,
};
use chrono::NaiveDate;
use uuid::Uuid;

#[tokio::test]
async fn test_job_creation_pipeline_type_safety() {
    let job_id = Uuid::new_v4();
    let ws_id = Uuid::new_v4();

    let job = Job {
        job_id,
        parent_job_id: Uuid::new_v4(),
        batch_index: 0,
        task_type: TaskType::AttendanceReport,
        workspace_id: ws_id,
        payload: JobPayload::AttendanceReport(ReportPayload {
            employee_ids: vec![],
            date_range: DateRange {
                start: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                end: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            },
            config: ReportConfig {
                report_type: ReportType::Summary,
                timezone: "UTC".to_string(),
                rounding_policy: apiserver::models::RoundingPolicy::None,
                work_unit_minutes: 60,
            },
        }),
        created_at: chrono::Utc::now(),
        retry_count: 0,
        max_retries: 3,
        priority: JobPriority::Normal,
        trace_context: None,
    };

    // Verify struct instantiated correctly (type safety check)
    assert_eq!(job.job_id, job_id);
    assert_eq!(job.workspace_id, ws_id);
    assert_eq!(job.retry_count, 0);
    assert_eq!(job.max_retries, 3);
}
