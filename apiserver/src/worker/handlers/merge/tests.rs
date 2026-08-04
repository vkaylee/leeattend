use super::*;
use crate::diesel_pool::TenantAuthToken;
use crate::models::{AttendanceSummary, DailyAttendanceReport, NewWorkspaceParams};
use chrono::{NaiveDate, Utc};
use uuid::Uuid;

fn en_msgs() -> ExcelReportMessages {
    ExcelReportMessages::for_lang("en")
}

#[tokio::test]
async fn test_merge_summary_empty_batches() {
    use crate::worker::test_utils::MockWorkerDepsBuilder;
    let deps = MockWorkerDepsBuilder::new().build();
    let auth_token = TenantAuthToken::system_override(Uuid::new_v4());

    let batch_results: Vec<(String, String)> = vec![
        ("batch_00000".to_string(), "[]".to_string()),
        ("batch_00001".to_string(), "[]".to_string()),
    ];

    let result = MergeHandler::merge_summary_results(
        &batch_results,
        "Test Workspace",
        (
            NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
        ),
        Uuid::new_v4(),
        &en_msgs(),
        &deps,
        &auth_token,
    )
    .await;

    assert!(result.is_ok());
    let (data, name, count) = result.unwrap();
    assert!(data.len() > 100); // Valid Excel file
    assert!(name.contains("attendance_summary_"));
    assert_eq!(count, 0);
}

#[tokio::test]
async fn test_merge_summary_with_data() {
    use crate::worker::test_utils::MockWorkerDepsBuilder;
    let deps = MockWorkerDepsBuilder::new().build();
    let auth_token = TenantAuthToken::system_override(Uuid::new_v4());

    let summaries1 = vec![AttendanceSummary {
        employment_id: Uuid::new_v4(),
        employee_id: Uuid::new_v4(),
        employee_name: "Charlie Brown".to_string(),
        employee_id_number: "EMP003".to_string(),
        total_std_units: 160,
        total_ot_units: 0,
        total_leave_days: 2,
        unit_type: crate::models::enums::TimekeepingUnit::DAY,
        unit_coefficient_hours: 8.0,
        computed_std_hours: 160.0,
        computed_ot_hours: 0.0,
        employer_id: None,
        employer_name: None,
    }];
    let summaries2 = vec![AttendanceSummary {
        employment_id: Uuid::new_v4(),
        employee_id: Uuid::new_v4(),
        employee_name: "Alice Smith".to_string(),
        employee_id_number: "EMP001".to_string(),
        total_std_units: 152,
        total_ot_units: 0,
        total_leave_days: 4,
        unit_type: crate::models::enums::TimekeepingUnit::DAY,
        unit_coefficient_hours: 8.0,
        computed_std_hours: 152.0,
        computed_ot_hours: 0.0,
        employer_id: None,
        employer_name: None,
    }];

    let batch_results: Vec<(String, String)> = vec![
        (
            "batch_00000".to_string(),
            serde_json::to_string(&summaries1).unwrap(),
        ),
        (
            "batch_00001".to_string(),
            serde_json::to_string(&summaries2).unwrap(),
        ),
    ];

    let result = MergeHandler::merge_summary_results(
        &batch_results,
        "Test Workspace",
        (
            NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
        ),
        Uuid::new_v4(),
        &en_msgs(),
        &deps,
        &auth_token,
    )
    .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_merge_detail_sorting() {
    use crate::worker::test_utils::MockWorkerDepsBuilder;
    let deps = MockWorkerDepsBuilder::new().build();
    let auth_token = TenantAuthToken::system_override(Uuid::new_v4());

    let report1 = DailyAttendanceReport {
        date: NaiveDate::from_ymd_opt(2026, 1, 2).unwrap(),
        employment_id: Uuid::new_v4(),
        employee_id: Uuid::new_v4(),
        employee_name: "Bob Jones".to_string(),
        employee_id_number: "EMP002".to_string(),
        shift_symbol: None,
        shift_name: None,
        scheduled_start: None,
        scheduled_end: None,
        grace_period_minutes: None,
        break_duration_minutes: None,
        break_start: None,
        break_end: None,
        is_break_auto_deduct: None,
        actual_punch_in: None,
        actual_punch_out: None,
        std_work_units: 8,
        ot_work_units: 0,
        ot_claim_status: None,
        unit_type: crate::models::enums::TimekeepingUnit::DAY,
        unit_coefficient_hours: 8.0,
        computed_std_hours: 8.0,
        computed_ot_hours: 0.0,
        status: crate::services::attendance_utils::AttendanceStatus::Present,
        is_holiday: false,
        is_unscheduled: false,
        holiday_name: None,
        generation_metadata: None,
        is_leave_paid: None,
        employer_name: None,
        timezone: None,
        segment_id: None,
        shift_id: None,
        interval_id: None,
        interval_name: None,
        segment_type: None,
        sort_order: None,
        segment_count: None,
        break_split: None,
    };
    let report2 = DailyAttendanceReport {
        date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        employment_id: Uuid::new_v4(),
        employee_id: Uuid::new_v4(),
        employee_name: "Alice Smith".to_string(),
        employee_id_number: "EMP001".to_string(),
        shift_symbol: None,
        shift_name: None,
        scheduled_start: None,
        scheduled_end: None,
        grace_period_minutes: None,
        break_duration_minutes: None,
        break_start: None,
        break_end: None,
        is_break_auto_deduct: None,
        actual_punch_in: None,
        actual_punch_out: None,
        std_work_units: 8,
        ot_work_units: 0,
        ot_claim_status: None,
        unit_type: crate::models::enums::TimekeepingUnit::DAY,
        unit_coefficient_hours: 8.0,
        computed_std_hours: 8.0,
        computed_ot_hours: 0.0,
        status: crate::services::attendance_utils::AttendanceStatus::Present,
        is_holiday: false,
        is_unscheduled: false,
        holiday_name: None,
        generation_metadata: None,
        is_leave_paid: None,
        employer_name: None,
        timezone: None,
        segment_id: None,
        shift_id: None,
        interval_id: None,
        interval_name: None,
        segment_type: None,
        sort_order: None,
        segment_count: None,
        break_split: None,
    };

    let batch_results: Vec<(String, String)> = vec![
        (
            "batch_00000".to_string(),
            serde_json::to_string(&vec![report1]).unwrap(),
        ),
        (
            "batch_00001".to_string(),
            serde_json::to_string(&vec![report2]).unwrap(),
        ),
    ];

    let result = MergeHandler::merge_detail_results(
        &batch_results,
        "Test Workspace",
        (
            NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
        ),
        Uuid::new_v4(),
        "UTC",
        30,
        &en_msgs(),
        &deps,
        &auth_token,
    )
    .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_invalid_json_handling() {
    use crate::worker::test_utils::MockWorkerDepsBuilder;
    let deps = MockWorkerDepsBuilder::new().build();
    let auth_token = TenantAuthToken::system_override(Uuid::new_v4());

    let batch_results: Vec<(String, String)> =
        vec![("batch_00000".to_string(), "invalid json".to_string())];

    let result = MergeHandler::merge_summary_results(
        &batch_results,
        "Test Workspace",
        (
            NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
        ),
        Uuid::new_v4(),
        &en_msgs(),
        &deps,
        &auth_token,
    )
    .await;

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Parse error"));
}

/// Optimization #9: total_records must equal the actual number of deserialized records,
/// not a fragile string match count.
#[tokio::test]
async fn test_merge_summary_returns_accurate_total_records() {
    use crate::worker::test_utils::MockWorkerDepsBuilder;
    let deps = MockWorkerDepsBuilder::new().build();
    let auth_token = TenantAuthToken::system_override(Uuid::new_v4());

    let s1 = vec![
        AttendanceSummary {
            employment_id: Uuid::new_v4(),
            employee_id: Uuid::new_v4(),
            employee_name: "A".to_string(),
            employee_id_number: "ID-A".to_string(),
            total_std_units: 160,
            total_ot_units: 0,
            total_leave_days: 0,
            unit_type: crate::models::enums::TimekeepingUnit::DAY,
            unit_coefficient_hours: 8.0,
            computed_std_hours: 160.0,
            computed_ot_hours: 0.0,
            employer_id: None,
            employer_name: None,
        },
        AttendanceSummary {
            employment_id: Uuid::new_v4(),
            employee_id: Uuid::new_v4(),
            employee_name: "B".to_string(),
            employee_id_number: "ID-B".to_string(),
            total_std_units: 120,
            total_ot_units: 0,
            total_leave_days: 3,
            unit_type: crate::models::enums::TimekeepingUnit::DAY,
            unit_coefficient_hours: 8.0,
            computed_std_hours: 120.0,
            computed_ot_hours: 0.0,
            employer_id: None,
            employer_name: None,
        },
    ];
    let s2 = vec![AttendanceSummary {
        employment_id: Uuid::new_v4(),
        employee_id: Uuid::new_v4(),
        employee_name: "C".to_string(),
        employee_id_number: "ID-C".to_string(),
        total_std_units: 80,
        total_ot_units: 0,
        total_leave_days: 1,
        unit_type: crate::models::enums::TimekeepingUnit::DAY,
        unit_coefficient_hours: 8.0,
        computed_std_hours: 80.0,
        computed_ot_hours: 0.0,
        employer_id: None,
        employer_name: None,
    }];

    let batch_results = vec![
        (
            "batch_00000".to_string(),
            serde_json::to_string(&s1).unwrap(),
        ),
        (
            "batch_00001".to_string(),
            serde_json::to_string(&s2).unwrap(),
        ),
    ];

    let (_, _, total_records) = MergeHandler::merge_summary_results(
        &batch_results,
        "W",
        (
            NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
        ),
        Uuid::new_v4(),
        &en_msgs(),
        &deps,
        &auth_token,
    )
    .await
    .unwrap();

    assert_eq!(total_records, 3, "Should be 2 + 1 = 3 total records");
}

/// Verify detail merge also returns accurate total_records.
#[tokio::test]
async fn test_merge_detail_returns_accurate_total_records() {
    use crate::worker::test_utils::MockWorkerDepsBuilder;
    let deps = MockWorkerDepsBuilder::new().build();
    let auth_token = TenantAuthToken::system_override(Uuid::new_v4());

    let make_report = |name: &str, day: u32| DailyAttendanceReport {
        date: NaiveDate::from_ymd_opt(2026, 1, day).unwrap(),
        employment_id: Uuid::new_v4(),
        employee_id: Uuid::new_v4(),
        employee_name: name.to_string(),
        employee_id_number: "EMP001".to_string(),
        shift_symbol: None,
        shift_name: None,
        scheduled_start: None,
        scheduled_end: None,
        grace_period_minutes: None,
        break_duration_minutes: None,
        break_start: None,
        break_end: None,
        is_break_auto_deduct: None,
        actual_punch_in: None,
        actual_punch_out: None,
        std_work_units: 8,
        ot_work_units: 0,
        ot_claim_status: None,
        unit_type: crate::models::enums::TimekeepingUnit::DAY,
        unit_coefficient_hours: 8.0,
        computed_std_hours: 8.0,
        computed_ot_hours: 0.0,
        status: crate::services::attendance_utils::AttendanceStatus::Present,
        is_holiday: false,
        is_unscheduled: false,
        holiday_name: None,
        generation_metadata: None,
        is_leave_paid: None,
        employer_name: None,
        timezone: None,
        segment_id: None,
        shift_id: None,
        interval_id: None,
        interval_name: None,
        segment_type: None,
        sort_order: None,
        segment_count: None,
        break_split: None,
    };

    let b1 = vec![
        make_report("Alice", 1),
        make_report("Alice", 2),
        make_report("Bob", 1),
    ];
    let b2 = vec![make_report("Charlie", 1)];

    let batch_results = vec![
        (
            "batch_00000".to_string(),
            serde_json::to_string(&b1).unwrap(),
        ),
        (
            "batch_00001".to_string(),
            serde_json::to_string(&b2).unwrap(),
        ),
    ];

    let (_, _, total_records) = MergeHandler::merge_detail_results(
        &batch_results,
        "W",
        (
            NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
        ),
        Uuid::new_v4(),
        "UTC",
        30,
        &en_msgs(),
        &deps,
        &auth_token,
    )
    .await
    .unwrap();

    assert_eq!(total_records, 4, "Should be 3 + 1 = 4 total records");
}

#[test]
fn test_attendance_summary_serialization_preserves_employee_id_number() {
    let summary = AttendanceSummary {
        employment_id: Uuid::new_v4(),
        employee_id: Uuid::new_v4(),
        employee_name: "Alice".to_string(),
        employee_id_number: "EMP-001".to_string(),
        total_std_units: 160,
        total_ot_units: 0,
        total_leave_days: 2,
        unit_type: crate::models::enums::TimekeepingUnit::DAY,
        unit_coefficient_hours: 8.0,
        computed_std_hours: 160.0,
        computed_ot_hours: 0.0,
        employer_id: None,
        employer_name: None,
    };
    let json = serde_json::to_string(&summary).unwrap();
    assert!(json.contains("EMP-001"));
    let deserialized: AttendanceSummary = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.employee_id_number, "EMP-001");
}

#[test]
fn test_daily_attendance_report_serialization_preserves_employee_id_number() {
    let report = DailyAttendanceReport {
        date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        employment_id: Uuid::new_v4(),
        employee_id: Uuid::new_v4(),
        employee_name: "Bob".to_string(),
        employee_id_number: "EMP-002".to_string(),
        shift_symbol: None,
        shift_name: None,
        scheduled_start: None,
        scheduled_end: None,
        grace_period_minutes: None,
        break_duration_minutes: None,
        break_start: None,
        break_end: None,
        is_break_auto_deduct: None,
        actual_punch_in: None,
        actual_punch_out: None,
        std_work_units: 8,
        ot_work_units: 0,
        ot_claim_status: None,
        unit_type: crate::models::enums::TimekeepingUnit::DAY,
        unit_coefficient_hours: 8.0,
        computed_std_hours: 8.0,
        computed_ot_hours: 0.0,
        status: crate::services::attendance_utils::AttendanceStatus::Present,
        is_holiday: false,
        is_unscheduled: false,
        holiday_name: None,
        generation_metadata: None,
        is_leave_paid: None,
        employer_name: None,
        timezone: None,
        segment_id: None,
        shift_id: None,
        interval_id: None,
        interval_name: None,
        segment_type: None,
        sort_order: None,
        segment_count: None,
        break_split: None,
    };
    let json = serde_json::to_string(&report).unwrap();
    assert!(json.contains("EMP-002"));
    let deserialized: DailyAttendanceReport = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.employee_id_number, "EMP-002");
}

#[tokio::test]
async fn test_merge_handler_handle_inner_with_mocks() {
    use crate::queue::{
        DateRange, Job, JobMetadata, JobPayload, JobPriority, JobStatus, MergePayload,
        ReportConfig, ReportType, TaskType,
    };
    use crate::repositories::publisher_repository::MockPublisherRepository;
    use crate::repositories::queue_repository::MockQueueRepository;
    use crate::worker::test_utils::MockWorkerDepsBuilder;
    use std::sync::Arc;

    let ws_id = Uuid::new_v4();
    let parent_job_id = Uuid::new_v4();

    let mut queue = MockQueueRepository::new();
    queue.expect_get_job_metadata().returning(move |_| {
        Ok(Some(JobMetadata {
            job_id: parent_job_id,
            requester_id: Uuid::new_v4(),
            total_batches: 2,
            completed_batches: 2,
            failed_batches: 0,
            status: JobStatus::Completed,
            workspace_id: ws_id,
            report_type: Some(ReportType::Summary),
            date_range: Some(DateRange {
                start: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                end: NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
            }),
            created_at: Utc::now(),
            completed_at: Some(Utc::now()),
            error_message: None,
            task_type: None,
            payload: None,
        }))
    });

    queue.expect_get_all_results().returning(move |_| {
        let summary = AttendanceSummary {
            employment_id: Uuid::new_v4(),
            employee_id: Uuid::new_v4(),
            employee_name: "Mocked Emp".to_string(),
            employee_id_number: "MOCK-001".to_string(),
            total_std_units: 8,
            total_ot_units: 0,
            total_leave_days: 0,
            unit_type: crate::models::enums::TimekeepingUnit::DAY,
            unit_coefficient_hours: 8.0,
            computed_std_hours: 8.0,
            computed_ot_hours: 0.0,
            employer_id: None,
            employer_name: None,
        };
        let json = serde_json::to_string(&vec![summary]).unwrap();
        Ok(vec![
            ("batch_0".to_string(), json.clone()),
            ("batch_1".to_string(), json),
        ])
    });

    queue.expect_set_result_data().returning(|_, _, _| Ok(()));
    queue.expect_update_job_status().returning(|_, _| Ok(()));
    queue.expect_delete_results().returning(|_| Ok(()));

    let mut publisher = MockPublisherRepository::new();
    publisher.expect_publish_done().returning(|_, _, _| Ok(()));

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

    let deps = builder.build();

    let job = Job {
        job_id: Uuid::new_v4(),
        parent_job_id,
        batch_index: 0,
        task_type: TaskType::MergeResults,
        workspace_id: ws_id,
        payload: JobPayload::MergeResults(MergePayload {
            date_range: DateRange {
                start: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                end: NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
            },
            config: ReportConfig {
                report_type: ReportType::Summary,
                timezone: "UTC".to_string(),
                work_unit_minutes: 30,
                rounding_policy: crate::models::RoundingPolicy::Floor,
            },
        }),
        priority: JobPriority::Normal,
        created_at: Utc::now(),
        retry_count: 0,
        max_retries: 3,
        trace_context: None,
    };

    let queue_arc: Arc<dyn crate::repositories::QueueRepository> = Arc::new(queue);
    let pub_arc: Arc<dyn crate::repositories::PublisherRepository> = Arc::new(publisher);

    let result = MergeHandler::handle_inner(&queue_arc, &pub_arc, &job, &deps).await;

    assert!(
        result.is_ok(),
        "MergeHandler::handle_inner failed: {:?}",
        result.err()
    );
}

#[tokio::test]
async fn test_merge_handler_handle_inner_detail_with_mocks() {
    use crate::queue::{
        DateRange, Job, JobMetadata, JobPayload, JobPriority, JobStatus, MergePayload,
        ReportConfig, ReportType, TaskType,
    };
    use crate::repositories::publisher_repository::MockPublisherRepository;
    use crate::repositories::queue_repository::MockQueueRepository;
    use crate::worker::test_utils::MockWorkerDepsBuilder;
    use std::sync::Arc;

    let ws_id = Uuid::new_v4();
    let parent_job_id = Uuid::new_v4();

    let mut queue = MockQueueRepository::new();
    queue.expect_get_job_metadata().returning(move |_| {
        Ok(Some(JobMetadata {
            job_id: parent_job_id,
            requester_id: Uuid::new_v4(),
            total_batches: 2,
            completed_batches: 2,
            failed_batches: 0,
            status: JobStatus::Completed,
            workspace_id: ws_id,
            report_type: Some(ReportType::Detail),
            date_range: Some(DateRange {
                start: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                end: NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
            }),
            created_at: Utc::now(),
            completed_at: Some(Utc::now()),
            error_message: None,
            task_type: None,
            payload: None,
        }))
    });

    queue.expect_get_all_results().returning(move |_| {
        let report = DailyAttendanceReport {
            date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            employment_id: Uuid::new_v4(),
            employee_id: Uuid::new_v4(),
            employee_name: "Mocked Emp".to_string(),
            employee_id_number: "MOCK-001".to_string(),
            shift_symbol: Some("HC".to_string()),
            shift_name: Some("Day Shift".to_string()),
            scheduled_start: None,
            scheduled_end: None,
            grace_period_minutes: None,
            break_duration_minutes: None,
            break_start: None,
            break_end: None,
            is_break_auto_deduct: None,
            actual_punch_in: None,
            actual_punch_out: None,
            std_work_units: 8,
            ot_work_units: 0,
            ot_claim_status: None,
            unit_type: crate::models::enums::TimekeepingUnit::DAY,
            unit_coefficient_hours: 8.0,
            computed_std_hours: 8.0,
            computed_ot_hours: 0.0,
            status: crate::services::attendance_utils::AttendanceStatus::Present,
            is_holiday: false,
            is_unscheduled: false,
            holiday_name: None,
            generation_metadata: None,
            is_leave_paid: None,
            employer_name: None,
            timezone: None,
            segment_id: None,
            shift_id: None,
            interval_id: None,
            interval_name: None,
            segment_type: None,
            sort_order: None,
            segment_count: None,
            break_split: None,
        };
        let json = serde_json::to_string(&vec![report]).unwrap();
        Ok(vec![
            ("batch_0".to_string(), json.clone()),
            ("batch_1".to_string(), json),
        ])
    });

    queue.expect_set_result_data().returning(|_, _, _| Ok(()));
    queue.expect_update_job_status().returning(|_, _| Ok(()));
    queue.expect_delete_results().returning(|_| Ok(()));

    let mut publisher = MockPublisherRepository::new();
    publisher.expect_publish_done().returning(|_, _, _| Ok(()));

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

    let deps = builder.build();

    let job = Job {
        job_id: Uuid::new_v4(),
        parent_job_id,
        batch_index: 0,
        task_type: TaskType::MergeResults,
        workspace_id: ws_id,
        payload: JobPayload::MergeResults(MergePayload {
            date_range: DateRange {
                start: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                end: NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
            },
            config: ReportConfig {
                report_type: ReportType::Detail,
                timezone: "UTC".to_string(),
                work_unit_minutes: 30,
                rounding_policy: crate::models::RoundingPolicy::Floor,
            },
        }),
        priority: JobPriority::Normal,
        created_at: Utc::now(),
        retry_count: 0,
        max_retries: 3,
        trace_context: None,
    };

    let queue_arc: Arc<dyn crate::repositories::QueueRepository> = Arc::new(queue);
    let pub_arc: Arc<dyn crate::repositories::PublisherRepository> = Arc::new(publisher);

    let result = MergeHandler::handle_inner(&queue_arc, &pub_arc, &job, &deps).await;

    assert!(
        result.is_ok(),
        "MergeHandler::handle_inner detail failed: {:?}",
        result.err()
    );
}
