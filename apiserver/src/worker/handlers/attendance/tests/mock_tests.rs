use super::super::*;
use crate::models::DailyAttendanceReport;
use crate::models::{NewEffectiveScheduleParams, NewTimeEntryParams};
use crate::services::attendance_utils::AttendanceStatus;

#[tokio::test]
async fn test_attendance_handler_with_mocks() {
    use crate::repositories::cache_repository::MockCacheRepository;
    use crate::repositories::publisher_repository::MockPublisherRepository;
    use crate::repositories::queue_repository::MockQueueRepository;
    use crate::repositories::shift_repository::MockShiftRepository;
    use crate::repositories::workspace_repository::MockWorkspaceRepository;
    use crate::worker::test_utils::MockWorkerDepsBuilder;
    use std::collections::HashMap;
    use std::sync::Arc;

    let mut queue = MockQueueRepository::new();
    let mut publisher = MockPublisherRepository::new();
    let mut cache_repo = MockCacheRepository::new();

    // Setup expectations
    queue.expect_store_result().returning(|_, _, _| Ok(()));
    queue.expect_get_job_metadata().returning(|_| {
        Ok(Some(crate::queue::JobMetadata {
            job_id: Uuid::new_v4(),
            requester_id: Uuid::new_v4(),
            total_batches: 1,
            completed_batches: 0,
            failed_batches: 0,
            status: crate::queue::JobStatus::Pending,
            workspace_id: Uuid::new_v4(),
            report_type: Some(crate::queue::ReportType::Summary),
            date_range: None,
            created_at: chrono::Utc::now(),
            completed_at: None,
            error_message: None,
            task_type: None,
            payload: None,
        }))
    });

    publisher.expect_publish_chunk().returning(|_, _, _| Ok(()));

    cache_repo.expect_get_json().returning(|_| {
        Ok(Some(
            r#"{"id":"00000000-0000-0000-0000-000000000000","timezone":"UTC"}"#.to_string(),
        ))
    });

    let workspace_repo = Arc::new(MockWorkspaceRepository::new());
    let shift_repo = Arc::new(MockShiftRepository::new());
    let config_repo = Arc::new(
        crate::repositories::workspace_config_repository::MockWorkspaceConfigRepository::new(),
    );
    let cache_manager = Arc::new(crate::cache::CacheManager::new(
        Arc::new(cache_repo),
        workspace_repo.clone(),
        shift_repo.clone(),
        config_repo,
    ));

    // Use Builder to auto-fill 20 mock repositories
    let mut builder = MockWorkerDepsBuilder::new();

    builder
        .employer_repo
        .expect_get_employers()
        .returning(|_, _, _| Ok((vec![], 0)));

    builder
        .roster_repo
        .expect_get_workspace_schedules()
        .returning(|_, _, _, _, _, _, _| Box::pin(async move { Ok((vec![], 0)) }));
    builder
        .time_entry_repo
        .expect_get_workspace_time_entries_batch_by_employments()
        .returning(|_, _, _, _| Ok(HashMap::new()));
    builder
        .leave_request_repo
        .expect_get_batch_leave_days()
        .returning(|_, _, _, _| Box::pin(async move { Ok(HashMap::new()) }));
    builder
        .employee_repo
        .expect_get_working_conditions_batch_by_employments()
        .returning(|_, _, _, _| Ok(Default::default()));
    builder
        .policy_repo
        .expect_get_employee_effective_policies_batch()
        .returning(|_, _, _| Ok(HashMap::new()));

    let deps = builder.build();

    let job = crate::queue::Job {
        job_id: Uuid::new_v4(),
        parent_job_id: Uuid::new_v4(),
        batch_index: 0,
        task_type: crate::queue::TaskType::AttendanceReport,
        workspace_id: Uuid::new_v4(),
        payload: crate::queue::JobPayload::AttendanceReport(crate::queue::ReportPayload {
            employee_ids: vec![],
            date_range: crate::queue::DateRange {
                start: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                end: NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
            },
            config: crate::queue::ReportConfig {
                report_type: crate::queue::ReportType::Summary,
                timezone: "UTC".to_string(),
                work_unit_minutes: 30,
                rounding_policy: crate::models::RoundingPolicy::Floor,
            },
        }),
        priority: crate::queue::JobPriority::Normal,
        created_at: chrono::Utc::now(),
        retry_count: 0,
        max_retries: 3,
        trace_context: None,
    };

    let queue_arc: Arc<dyn crate::repositories::QueueRepository> = Arc::new(queue);
    let pub_arc: Arc<dyn crate::repositories::PublisherRepository> = Arc::new(publisher);

    let result =
        AttendanceHandler::handle(&queue_arc, &pub_arc, &cache_manager, &job, None, &deps).await;

    assert!(result.is_ok(), "Handler should succeed with mocks");
}

#[tokio::test]
async fn test_attendance_handler_calculate_detail_late_early() {
    use crate::repositories::cache_repository::MockCacheRepository;
    use crate::repositories::publisher_repository::MockPublisherRepository;
    use crate::repositories::queue_repository::MockQueueRepository;
    use crate::repositories::shift_repository::MockShiftRepository;
    use crate::repositories::workspace_repository::MockWorkspaceRepository;
    use crate::worker::test_utils::MockWorkerDepsBuilder;
    use chrono::{NaiveDate, NaiveTime, TimeZone};
    use std::collections::HashMap;
    use std::sync::Arc;

    let mut queue = MockQueueRepository::new();
    let mut publisher = MockPublisherRepository::new();
    let mut cache_repo = MockCacheRepository::new();

    let emp_id = Uuid::new_v4();
    let ws_id = Uuid::new_v4();
    let emp_id_for_policy = emp_id;

    queue
        .expect_store_result()
        .returning(move |_, _, result_json| {
            let reports: Vec<DailyAttendanceReport> = serde_json::from_str(result_json).unwrap();
            assert_eq!(reports.len(), 1);
            let report = &reports[0];
            assert_eq!(report.status, AttendanceStatus::Late);
            // 8h shift (480 mins) - 15m late - 15m early = 450 mins. 450 / 30 = 15 units.
            assert_eq!(report.std_work_units, 15);
            assert_eq!(report.ot_work_units, 0);
            Ok(())
        });

    queue.expect_get_job_metadata().returning(|_| {
        Ok(Some(crate::queue::JobMetadata {
            job_id: Uuid::new_v4(),
            requester_id: Uuid::new_v4(),
            total_batches: 1,
            completed_batches: 0,
            failed_batches: 0,
            status: crate::queue::JobStatus::Pending,
            workspace_id: Uuid::new_v4(),
            report_type: Some(crate::queue::ReportType::Detail),
            date_range: None,
            created_at: chrono::Utc::now(),
            completed_at: None,
            error_message: None,
            task_type: None,
            payload: None,
        }))
    });

    publisher.expect_publish_chunk().returning(|_, _, _| Ok(()));
    cache_repo.expect_get_json().returning(|_| {
        Ok(Some(
            r#"{"id":"00000000-0000-0000-0000-000000000000","timezone":"UTC"}"#.to_string(),
        ))
    });

    let workspace_repo = Arc::new(MockWorkspaceRepository::new());
    let shift_repo = Arc::new(MockShiftRepository::new());
    let config_repo = Arc::new(
        crate::repositories::workspace_config_repository::MockWorkspaceConfigRepository::new(),
    );
    let cache_manager = Arc::new(crate::cache::CacheManager::new(
        Arc::new(cache_repo),
        workspace_repo.clone(),
        shift_repo.clone(),
        config_repo,
    ));

    let mut builder = MockWorkerDepsBuilder::new();

    builder
        .employer_repo
        .expect_get_employers()
        .returning(|_, _, _| Ok((vec![], 0)));

    let sched_date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
    let schedule = crate::models::EmployeeSchedule::new(
        emp_id,
        Uuid::new_v4(),
        "EMP01".to_string(),
        "Test User".to_string(),
        vec![crate::models::EffectiveSchedule::new(NewEffectiveScheduleParams {
            date: sched_date,
            shift_id: Some(Uuid::new_v4()),
            source: "fixed".to_string(),
            shift_name: Some("Day Shift".to_string()),
            shift_symbol: Some("HC".to_string()),
            start_time: Some(NaiveTime::from_hms_opt(8, 0, 0).unwrap()),
            end_time: Some(NaiveTime::from_hms_opt(17, 0, 0).unwrap()),
            break_duration_minutes: Some(60),
            break_start: Some(NaiveTime::from_hms_opt(12, 0, 0).unwrap()),
            break_end: Some(NaiveTime::from_hms_opt(13, 0, 0).unwrap()),
            is_break_auto_deduct: Some(true),
            early_punch_in_grace_period_minutes: Some(0),
            allow_early_ot: Some(false),
            break_is_overtime: Some(false),
            max_early_ot_minutes: None,
            traces: vec![],
            is_holiday: false,
            is_holiday_paid: false,
            holiday_name: None,
        })],
    );

    builder
        .roster_repo
        .expect_get_workspace_schedules()
        .returning(move |_, _, _, _, _, _, _| {
            let sched = schedule.clone();
            Box::pin(async move { Ok((vec![sched], 1)) })
        });

    let punch_in = chrono::Utc.with_ymd_and_hms(2026, 1, 15, 8, 15, 0).unwrap();
    let punch_out = chrono::Utc
        .with_ymd_and_hms(2026, 1, 15, 16, 45, 0)
        .unwrap();
    let entry = TimeEntry::new(NewTimeEntryParams {
        id: Uuid::new_v4(),
        employment_id: emp_id,
        workspace_id: ws_id,
        shift_id: None,
        segment_id: None,
        punch_in_time: punch_in,
        punch_out_time: Some(punch_out),
        duration_minutes: Some(450),
        is_break_overtime: false,
        entry_type: crate::services::pairing::types::EntryType::Work,
        note: None,
        generation_metadata: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    });

    builder
        .time_entry_repo
        .expect_get_workspace_time_entries_batch_by_employments()
        .returning(move |_, _, _, _| {
            let mut map = HashMap::new();
            map.insert(emp_id, vec![entry.clone()]);
            Ok(map)
        });

    builder
        .leave_request_repo
        .expect_get_batch_leave_days()
        .returning(|_, _, _, _| Box::pin(async move { Ok(HashMap::new()) }));
    builder
        .employee_repo
        .expect_get_working_conditions_batch_by_employments()
        .returning(|_, _, _, _| Ok(Default::default()));
    builder
        .policy_repo
        .expect_get_employee_effective_policies_batch()
        .returning(move |_, _, _| {
            let mut map = HashMap::new();
            map.insert(
                emp_id_for_policy,
                crate::models::AttendancePolicyVersion {
                    id: Uuid::new_v4(),
                    policy_id: Uuid::new_v4(),
                    rules: crate::models::AttendanceRules {
                        work_unit_minutes: 30,
                        rounding_policy: crate::models::RoundingPolicy::Floor,
                        early_arrival_strategy: crate::models::EarlyArrivalStrategy::Block,
                        grace_period_minutes: Some(10),
                    },
                    valid_from: chrono::Utc::now(),
                    valid_to: None,
                    created_by: None,
                    created_at: chrono::Utc::now(),
                },
            );
            Ok(map)
        });

    let deps = builder.build();

    let job = crate::queue::Job {
        job_id: Uuid::new_v4(),
        parent_job_id: Uuid::new_v4(),
        batch_index: 0,
        task_type: crate::queue::TaskType::AttendanceReport,
        workspace_id: ws_id,
        payload: crate::queue::JobPayload::AttendanceReport(crate::queue::ReportPayload {
            employee_ids: vec![emp_id],
            date_range: crate::queue::DateRange {
                start: NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
                end: NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
            },
            config: crate::queue::ReportConfig {
                report_type: crate::queue::ReportType::Detail,
                timezone: "UTC".to_string(),
                work_unit_minutes: 30,
                rounding_policy: crate::models::RoundingPolicy::Floor,
            },
        }),
        priority: crate::queue::JobPriority::Normal,
        created_at: chrono::Utc::now(),
        retry_count: 0,
        max_retries: 3,
        trace_context: None,
    };

    let queue_arc: Arc<dyn crate::repositories::QueueRepository> = Arc::new(queue);
    let pub_arc: Arc<dyn crate::repositories::PublisherRepository> = Arc::new(publisher);

    let result =
        AttendanceHandler::handle(&queue_arc, &pub_arc, &cache_manager, &job, None, &deps).await;

    assert!(result.is_ok());
}
