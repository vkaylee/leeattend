use super::*;

#[test]
fn test_job_serialization() {
    let job = Job::new(
        Uuid::new_v4(),
        0,
        TaskType::AttendanceReport,
        Uuid::new_v4(),
        JobPayload::AttendanceReport(ReportPayload {
            employee_ids: vec![Uuid::new_v4(), Uuid::new_v4()],
            date_range: DateRange {
                start: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                end: NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
            },
            config: ReportConfig {
                report_type: ReportType::Summary,
                timezone: "Asia/Ho_Chi_Minh".to_string(),
                rounding_policy: crate::models::RoundingPolicy::Floor,
                work_unit_minutes: 30,
            },
        }),
    );

    let json = serde_json::to_string(&job).unwrap();
    let deserialized: Job = serde_json::from_str(&json).unwrap();

    assert_eq!(job.job_id, deserialized.job_id);
    assert_eq!(job.batch_index, deserialized.batch_index);
    assert_eq!(job.task_type, deserialized.task_type);
}

#[test]
fn test_job_metadata_progress() {
    let mut meta = JobMetadata::new(
        Uuid::new_v4(),
        10,
        Uuid::new_v4(),
        Uuid::new_v4(),
        None,
        Some(ReportType::Summary),
        None,
    );

    assert!(!meta.is_complete());
    assert!((meta.progress_percent() - 0.0).abs() < f32::EPSILON);

    meta.completed_batches = 5;
    assert!((meta.progress_percent() - 50.0).abs() < f32::EPSILON);

    meta.completed_batches = 8;
    meta.failed_batches = 2;
    assert!(meta.is_complete());
    assert!((meta.progress_percent() - 100.0).abs() < f32::EPSILON);
}

#[test]
fn test_retry_logic() {
    let mut job = Job::new(
        Uuid::new_v4(),
        0,
        TaskType::AttendanceReport,
        Uuid::new_v4(),
        JobPayload::AttendanceReport(ReportPayload {
            employee_ids: vec![],
            date_range: DateRange {
                start: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                end: NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
            },
            config: ReportConfig {
                report_type: ReportType::Summary,
                timezone: "UTC".to_string(),
                rounding_policy: crate::models::RoundingPolicy::Floor,
                work_unit_minutes: 30,
            },
        }),
    );

    assert!(!job.increment_retry()); // 1 <= 3
    assert!(!job.increment_retry()); // 2 <= 3
    assert!(!job.increment_retry()); // 3 <= 3
    assert!(job.increment_retry()); // 4 > 3, exhausted
}

#[test]
fn test_payload_helpers() {
    let report_job = Job::new(
        Uuid::new_v4(),
        0,
        TaskType::AttendanceReport,
        Uuid::new_v4(),
        JobPayload::AttendanceReport(ReportPayload {
            employee_ids: vec![Uuid::new_v4()],
            date_range: DateRange {
                start: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                end: NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
            },
            config: ReportConfig {
                report_type: ReportType::Summary,
                timezone: "UTC".to_string(),
                rounding_policy: crate::models::RoundingPolicy::Floor,
                work_unit_minutes: 30,
            },
        }),
    );

    assert!(report_job.report_payload().is_some());
    assert!(report_job.merge_payload().is_none());

    let merge_job = Job::new(
        Uuid::new_v4(),
        0,
        TaskType::MergeResults,
        Uuid::new_v4(),
        JobPayload::MergeResults(MergePayload {
            date_range: DateRange {
                start: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                end: NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
            },
            config: ReportConfig {
                report_type: ReportType::Summary,
                timezone: "UTC".to_string(),
                rounding_policy: crate::models::RoundingPolicy::Floor,
                work_unit_minutes: 30,
            },
        }),
    );

    assert!(merge_job.merge_payload().is_some());
    assert!(merge_job.report_payload().is_none());
}

// ====== NEW TESTS: Serde Compatibility & Payload Validation ======

/// Verify the tagged-enum serde format produces expected JSON structure.
/// This is critical for Redis interoperability — changing the format
/// would break in-flight jobs during a rolling deploy.
#[test]
fn test_job_payload_serde_tagged_format() {
    let payload = JobPayload::AttendanceReport(ReportPayload {
        employee_ids: vec![],
        date_range: DateRange {
            start: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            end: NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
        },
        config: ReportConfig {
            report_type: ReportType::Summary,
            timezone: "UTC".to_string(),
            rounding_policy: crate::models::RoundingPolicy::Floor,
            work_unit_minutes: 30,
        },
    });

    let json = serde_json::to_value(&payload).unwrap();
    // Tagged enum: {"type": "AttendanceReport", "data": {...}}
    assert_eq!(json["type"], "AttendanceReport");
    assert!(json["data"].is_object());
    assert!(json["data"]["employee_ids"].is_array());
    assert!(json["data"]["config"]["report_type"].is_string());
}

/// MergeResults payload uses separate MergePayload (no employee_ids).
#[test]
fn test_merge_payload_serde_format() {
    let payload = JobPayload::MergeResults(MergePayload {
        date_range: DateRange {
            start: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            end: NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
        },
        config: ReportConfig {
            report_type: ReportType::Detail,
            timezone: "Asia/Ho_Chi_Minh".to_string(),
            rounding_policy: crate::models::RoundingPolicy::Ceil,
            work_unit_minutes: 15,
        },
    });

    let json = serde_json::to_value(&payload).unwrap();
    assert_eq!(json["type"], "MergeResults");
    // MergePayload has no employee_ids field
    assert!(json["data"]["employee_ids"].is_null());
    assert_eq!(json["data"]["config"]["report_type"], "detail");
}

/// PayrollReport shares ReportPayload with AttendanceReport.
/// Both should be extractable via report_payload().
#[test]
fn test_payroll_report_payload_extraction() {
    let emp_id = Uuid::new_v4();
    let payroll_job = Job::new(
        Uuid::new_v4(),
        0,
        TaskType::PayrollReport,
        Uuid::new_v4(),
        JobPayload::PayrollReport(ReportPayload {
            employee_ids: vec![emp_id],
            date_range: DateRange {
                start: NaiveDate::from_ymd_opt(2026, 2, 1).unwrap(),
                end: NaiveDate::from_ymd_opt(2026, 2, 28).unwrap(),
            },
            config: ReportConfig {
                report_type: ReportType::Summary,
                timezone: "UTC".to_string(),
                rounding_policy: crate::models::RoundingPolicy::Floor,
                work_unit_minutes: 30,
            },
        }),
    );

    // PayrollReport and AttendanceReport both return via report_payload()
    let payload = payroll_job.report_payload().expect("should extract");
    assert_eq!(payload.employee_ids.len(), 1);
    assert_eq!(payload.employee_ids[0], emp_id);

    // Should NOT be a merge payload
    assert!(payroll_job.merge_payload().is_none());
}

/// Serde roundtrip must preserve all Job fields including nested payload.
#[test]
fn test_full_job_serde_roundtrip() {
    let parent_id = Uuid::new_v4();
    let ws_id = Uuid::new_v4();
    let emp_ids: Vec<Uuid> = (0..5).map(|_| Uuid::new_v4()).collect();

    let job = Job::new(
        parent_id,
        3,
        TaskType::AttendanceReport,
        ws_id,
        JobPayload::AttendanceReport(ReportPayload {
            employee_ids: emp_ids.clone(),
            date_range: DateRange {
                start: NaiveDate::from_ymd_opt(2026, 3, 1).unwrap(),
                end: NaiveDate::from_ymd_opt(2026, 3, 31).unwrap(),
            },
            config: ReportConfig {
                report_type: ReportType::Detail,
                timezone: "Asia/Tokyo".to_string(),
                rounding_policy: crate::models::RoundingPolicy::Round,
                work_unit_minutes: 15,
            },
        }),
    );

    let json = serde_json::to_string(&job).unwrap();
    let roundtripped: Job = serde_json::from_str(&json).unwrap();

    // Top-level fields
    assert_eq!(roundtripped.job_id, job.job_id);
    assert_eq!(roundtripped.parent_job_id, parent_id);
    assert_eq!(roundtripped.batch_index, 3);
    assert_eq!(roundtripped.task_type, TaskType::AttendanceReport);
    assert_eq!(roundtripped.workspace_id, ws_id);
    assert_eq!(roundtripped.retry_count, 0);
    assert_eq!(roundtripped.max_retries, 3);

    // Nested payload
    let payload = roundtripped.report_payload().expect("should be report");
    assert_eq!(payload.employee_ids, emp_ids);
    assert_eq!(
        payload.date_range.start,
        NaiveDate::from_ymd_opt(2026, 3, 1).unwrap()
    );
    assert_eq!(payload.config.timezone, "Asia/Tokyo");
    assert_eq!(payload.config.report_type, ReportType::Detail);
    assert_eq!(payload.config.work_unit_minutes, 15);
}

/// Job::new() should set sensible defaults.
#[test]
fn test_job_new_defaults() {
    let job = Job::new(
        Uuid::new_v4(),
        0,
        TaskType::AttendanceReport,
        Uuid::new_v4(),
        JobPayload::AttendanceReport(ReportPayload {
            employee_ids: vec![],
            date_range: DateRange {
                start: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                end: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            },
            config: ReportConfig {
                report_type: ReportType::Summary,
                timezone: "UTC".to_string(),
                rounding_policy: crate::models::RoundingPolicy::Floor,
                work_unit_minutes: 30,
            },
        }),
    );

    assert_eq!(job.retry_count, 0);
    assert_eq!(job.max_retries, 3);
    assert_eq!(job.batch_index, 0);
    // job_id should be unique
    assert_ne!(job.job_id, Uuid::nil());
}

/// JobMetadata::with_date_range should store the date range.
#[test]
fn test_metadata_with_date_range() {
    let dr = DateRange {
        start: NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
        end: NaiveDate::from_ymd_opt(2026, 6, 30).unwrap(),
    };
    let meta = JobMetadata::with_date_range(
        Uuid::new_v4(),
        5,
        Uuid::new_v4(),
        Uuid::new_v4(),
        None,
        Some(ReportType::Detail),
        dr,
        None,
    );

    assert!(meta.date_range.is_some());
    let stored_dr = meta.date_range.unwrap();
    assert_eq!(
        stored_dr.start,
        NaiveDate::from_ymd_opt(2026, 6, 1).unwrap()
    );
    assert_eq!(stored_dr.end, NaiveDate::from_ymd_opt(2026, 6, 30).unwrap());
}

/// Zero total_batches should report 100% progress (division by zero guard).
#[test]
fn test_metadata_zero_batches_progress() {
    let meta = JobMetadata::new(
        Uuid::new_v4(),
        0, // No batches
        Uuid::new_v4(),
        Uuid::new_v4(),
        None,
        Some(ReportType::Summary),
        None,
    );

    assert!((meta.progress_percent() - 100.0).abs() < f32::EPSILON);
    assert!(meta.is_complete()); // 0 + 0 >= 0
}

/// TaskType serde uses snake_case.
#[test]
fn test_task_type_serde_snake_case() {
    let json = serde_json::to_string(&TaskType::AttendanceReport).unwrap();
    assert_eq!(json, "\"attendance_report\"");

    let json = serde_json::to_string(&TaskType::MergeResults).unwrap();
    assert_eq!(json, "\"merge_results\"");

    let json = serde_json::to_string(&TaskType::PayrollReport).unwrap();
    assert_eq!(json, "\"payroll_report\"");

    // Roundtrip
    let decoded: TaskType = serde_json::from_str("\"attendance_report\"").unwrap();
    assert_eq!(decoded, TaskType::AttendanceReport);
}

/// JobStatus serde uses snake_case.
#[test]
fn test_job_status_serde() {
    assert_eq!(
        serde_json::to_string(&JobStatus::Pending).unwrap(),
        "\"pending\""
    );
    assert_eq!(
        serde_json::to_string(&JobStatus::Processing).unwrap(),
        "\"processing\""
    );
    assert_eq!(
        serde_json::to_string(&JobStatus::Completed).unwrap(),
        "\"completed\""
    );
    assert_eq!(
        serde_json::to_string(&JobStatus::Failed).unwrap(),
        "\"failed\""
    );
}

// ====== PRIORITY + LOG INGESTION TESTS ======

/// JobPriority serde uses snake_case.
#[test]
fn test_job_priority_serde() {
    assert_eq!(
        serde_json::to_string(&JobPriority::High).unwrap(),
        "\"high\""
    );
    assert_eq!(
        serde_json::to_string(&JobPriority::Normal).unwrap(),
        "\"normal\""
    );
    assert_eq!(serde_json::to_string(&JobPriority::Low).unwrap(), "\"low\"");

    // Roundtrip
    let decoded: JobPriority = serde_json::from_str("\"low\"").unwrap();
    assert_eq!(decoded, JobPriority::Low);
}

/// Default priority is Normal.
#[test]
fn test_job_priority_default() {
    let priority = JobPriority::default();
    assert_eq!(priority, JobPriority::Normal);
}

/// Job::new defaults to Normal priority.
#[test]
fn test_job_new_defaults_to_normal_priority() {
    let job = Job::new(
        Uuid::new_v4(),
        0,
        TaskType::AttendanceReport,
        Uuid::new_v4(),
        JobPayload::AttendanceReport(ReportPayload {
            employee_ids: vec![],
            date_range: DateRange {
                start: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                end: NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
            },
            config: ReportConfig {
                report_type: ReportType::Summary,
                timezone: "UTC".to_string(),
                rounding_policy: crate::models::RoundingPolicy::Floor,
                work_unit_minutes: 30,
            },
        }),
    );

    assert_eq!(job.priority, JobPriority::Normal);
}

/// Job::with_priority sets the correct priority.
#[test]
fn test_job_with_priority() {
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

    assert_eq!(job.priority, JobPriority::Low);
    assert_eq!(job.task_type, TaskType::LogIngestion);
}

/// LogIngestionPayload serde roundtrip.
#[test]
fn test_log_ingestion_payload_serde_roundtrip() {
    let ws_id = Uuid::new_v4();
    let payload = LogIngestionPayload {
        workspace_id: ws_id,
        employee_id_number: "EMP042".to_string(),
        punch_time_raw: "26/01/2026 09:15:00".to_string(),
        device_id: Some("ZK-MAIN".to_string()),
        raw_data: Some(serde_json::json!({"punch": 0, "status": 1})),
        timezone: "Asia/Ho_Chi_Minh".to_string(),
        date_fmt: "DD/MM/YYYY".to_string(),
        time_fmt: "24h".to_string(),
    };

    let json = serde_json::to_string(&payload).unwrap();
    let decoded: LogIngestionPayload = serde_json::from_str(&json).unwrap();

    assert_eq!(decoded.workspace_id, ws_id);
    assert_eq!(decoded.employee_id_number, "EMP042");
    assert_eq!(decoded.device_id, Some("ZK-MAIN".to_string()));
    assert_eq!(decoded.raw_data.unwrap()["punch"], 0);
}

/// LogIngestion tagged enum serde format.
#[test]
fn test_log_ingestion_payload_tagged_format() {
    let payload = JobPayload::LogIngestion(LogIngestionPayload {
        workspace_id: Uuid::new_v4(),
        employee_id_number: "EMP001".to_string(),
        punch_time_raw: "2026-01-26 09:00:00".to_string(),
        device_id: None,
        raw_data: None,
        timezone: "UTC".to_string(),
        date_fmt: "YYYY-MM-DD".to_string(),
        time_fmt: "24h".to_string(),
    });

    let json = serde_json::to_value(&payload).unwrap();
    assert_eq!(json["type"], "LogIngestion");
    assert!(json["data"].is_object());
    assert_eq!(json["data"]["employee_id_number"], "EMP001");
}

/// LogIngestion task type serde.
#[test]
fn test_log_ingestion_task_type_serde() {
    let json = serde_json::to_string(&TaskType::LogIngestion).unwrap();
    assert_eq!(json, "\"log_ingestion\"");

    let decoded: TaskType = serde_json::from_str("\"log_ingestion\"").unwrap();
    assert_eq!(decoded, TaskType::LogIngestion);
}

/// Full Job with LogIngestion + Low priority roundtrip.
#[test]
fn test_log_ingestion_job_full_roundtrip() {
    let parent_id = Uuid::new_v4();
    let ws_id = Uuid::new_v4();

    let job = Job::with_priority(
        parent_id,
        0,
        TaskType::LogIngestion,
        ws_id,
        JobPayload::LogIngestion(LogIngestionPayload {
            workspace_id: ws_id,
            employee_id_number: "EMP999".to_string(),
            punch_time_raw: "2026-03-04 13:00:00".to_string(),
            device_id: Some("DEV-1".to_string()),
            raw_data: Some(serde_json::json!({"punch": 0, "status": 1, "verify": 1})),
            timezone: "Asia/Ho_Chi_Minh".to_string(),
            date_fmt: "YYYY-MM-DD".to_string(),
            time_fmt: "24h".to_string(),
        }),
        JobPriority::Low,
    );

    let json = serde_json::to_string(&job).unwrap();
    assert!(json.contains("\"log_ingestion\""));
    assert!(json.contains("\"low\""));

    let decoded: Job = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded.job_id, job.job_id);
    assert_eq!(decoded.task_type, TaskType::LogIngestion);
    assert_eq!(decoded.priority, JobPriority::Low);

    let payload = decoded.log_ingestion_payload().expect("should extract");
    assert_eq!(payload.employee_id_number, "EMP999");
    assert_eq!(payload.workspace_id, ws_id);
}

/// Backward compatibility: Job JSON without "priority" field should
/// deserialize with default Normal priority.
#[test]
fn test_job_backward_compatible_no_priority_field() {
    // Simulate a Job serialized BEFORE the priority field was added
    let json = serde_json::json!({
        "job_id": Uuid::new_v4(),
        "parent_job_id": Uuid::new_v4(),
        "batch_index": 0,
        "task_type": "attendance_report",
        "workspace_id": Uuid::new_v4(),
        "payload": {
            "type": "AttendanceReport",
            "data": {
                "employee_ids": [],
                "date_range": {"start": "2026-01-01", "end": "2026-01-31"},
                "config": {
                    "report_type": "summary",
                    "timezone": "UTC",
                    "rounding_policy": "floor",
                    "work_unit_minutes": 30
                }
            }
        },
        "created_at": "2026-01-01T00:00:00Z",
        "retry_count": 0,
        "max_retries": 3
    });

    let job: Job = serde_json::from_value(json).unwrap();
    // Missing "priority" field should default to Normal
    assert_eq!(job.priority, JobPriority::Normal);
}

/// log_ingestion_payload() returns None for non-LogIngestion jobs.
#[test]
fn test_log_ingestion_payload_helper_returns_none_for_other_types() {
    let job = Job::new(
        Uuid::new_v4(),
        0,
        TaskType::AttendanceReport,
        Uuid::new_v4(),
        JobPayload::AttendanceReport(ReportPayload {
            employee_ids: vec![],
            date_range: DateRange {
                start: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                end: NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
            },
            config: ReportConfig {
                report_type: ReportType::Summary,
                timezone: "UTC".to_string(),
                rounding_policy: crate::models::RoundingPolicy::Floor,
                work_unit_minutes: 30,
            },
        }),
    );

    assert!(job.log_ingestion_payload().is_none());
    assert!(job.report_payload().is_some());
}

// ====== LEAVE IMPORT TESTS ======

/// LeaveImportPayload serde roundtrip.
#[test]
fn test_leave_import_payload_serde_roundtrip() {
    let user_id = Uuid::new_v4();
    let import_job_id = Uuid::new_v4();
    let payload = LeaveImportPayload {
        file_path: "/storage/imports/leave_2026-03.xlsx".to_string(),
        user_id,
        auto_approve: true,
        import_job_id,
    };

    let json = serde_json::to_string(&payload).unwrap();
    let decoded: LeaveImportPayload = serde_json::from_str(&json).unwrap();

    assert_eq!(decoded.file_path, "/storage/imports/leave_2026-03.xlsx");
    assert_eq!(decoded.user_id, user_id);
    assert!(decoded.auto_approve);
    assert_eq!(decoded.import_job_id, import_job_id);
}

/// LeaveImport tagged enum serde format.
#[test]
fn test_leave_import_payload_tagged_format() {
    let payload = JobPayload::LeaveImport(LeaveImportPayload {
        file_path: "/tmp/test.xlsx".to_string(),
        user_id: Uuid::new_v4(),
        auto_approve: false,
        import_job_id: Uuid::new_v4(),
    });

    let json = serde_json::to_value(&payload).unwrap();
    assert_eq!(json["type"], "LeaveImport");
    assert!(json["data"].is_object());
    assert_eq!(json["data"]["file_path"], "/tmp/test.xlsx");
    assert_eq!(json["data"]["auto_approve"], false);
}

/// LeaveImport task type serde.
#[test]
fn test_leave_import_task_type_serde() {
    let json = serde_json::to_string(&TaskType::LeaveImport).unwrap();
    assert_eq!(json, "\"leave_import\"");

    let decoded: TaskType = serde_json::from_str("\"leave_import\"").unwrap();
    assert_eq!(decoded, TaskType::LeaveImport);
}

/// Full Job with LeaveImport payload roundtrip.
#[test]
fn test_leave_import_job_full_roundtrip() {
    let parent_id = Uuid::new_v4();
    let ws_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let import_job_id = Uuid::new_v4();

    let job = Job::new(
        parent_id,
        0,
        TaskType::LeaveImport,
        ws_id,
        JobPayload::LeaveImport(LeaveImportPayload {
            file_path: "/storage/imports/leave.xlsx".to_string(),
            user_id,
            auto_approve: true,
            import_job_id,
        }),
    );

    let json = serde_json::to_string(&job).unwrap();
    assert!(json.contains("\"leave_import\""));

    let decoded: Job = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded.job_id, job.job_id);
    assert_eq!(decoded.task_type, TaskType::LeaveImport);

    let payload = decoded.leave_import_payload().expect("should extract");
    assert_eq!(payload.file_path, "/storage/imports/leave.xlsx");
    assert_eq!(payload.user_id, user_id);
    assert!(payload.auto_approve);
    assert_eq!(payload.import_job_id, import_job_id);
}

/// leave_import_payload() returns None for non-LeaveImport jobs.
#[test]
fn test_leave_import_payload_helper_returns_none_for_other_types() {
    let job = Job::new(
        Uuid::new_v4(),
        0,
        TaskType::AttendanceReport,
        Uuid::new_v4(),
        JobPayload::AttendanceReport(ReportPayload {
            employee_ids: vec![],
            date_range: DateRange {
                start: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                end: NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
            },
            config: ReportConfig {
                report_type: ReportType::Summary,
                timezone: "UTC".to_string(),
                rounding_policy: crate::models::RoundingPolicy::Floor,
                work_unit_minutes: 30,
            },
        }),
    );

    assert!(job.leave_import_payload().is_none());
}

// ====== EMPLOYEE IMPORT TESTS ======

/// EmployeeImportPayload serde roundtrip.
#[test]
fn test_employee_import_payload_serde_roundtrip() {
    let user_id = Uuid::new_v4();
    let import_job_id = Uuid::new_v4();
    let payload = EmployeeImportPayload {
        import_job_id,
        user_id,
    };

    let json = serde_json::to_string(&payload).unwrap();
    let decoded: EmployeeImportPayload = serde_json::from_str(&json).unwrap();

    assert_eq!(decoded.user_id, user_id);
    assert_eq!(decoded.import_job_id, import_job_id);
}

/// EmployeeImport tagged enum serde format.
#[test]
fn test_employee_import_payload_tagged_format() {
    let payload = JobPayload::EmployeeImport(EmployeeImportPayload {
        user_id: Uuid::new_v4(),
        import_job_id: Uuid::new_v4(),
    });

    let json = serde_json::to_value(&payload).unwrap();
    assert_eq!(json["type"], "EmployeeImport");
    assert!(json["data"].is_object());
}

/// EmployeeImport task type serde.
#[test]
fn test_employee_import_task_type_serde() {
    let json = serde_json::to_string(&TaskType::EmployeeImport).unwrap();
    assert_eq!(json, "\"employee_import\"");

    let decoded: TaskType = serde_json::from_str("\"employee_import\"").unwrap();
    assert_eq!(decoded, TaskType::EmployeeImport);
}

/// Full Job with EmployeeImport payload roundtrip.
#[test]
fn test_employee_import_job_full_roundtrip() {
    let parent_id = Uuid::new_v4();
    let ws_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let import_job_id = Uuid::new_v4();

    let job = Job::new(
        parent_id,
        0,
        TaskType::EmployeeImport,
        ws_id,
        JobPayload::EmployeeImport(EmployeeImportPayload {
            import_job_id,
            user_id,
        }),
    );

    let json = serde_json::to_string(&job).unwrap();
    assert!(json.contains("\"employee_import\""));

    let decoded: Job = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded.job_id, job.job_id);
    assert_eq!(decoded.task_type, TaskType::EmployeeImport);

    let payload = decoded.employee_import_payload().expect("should extract");
    assert_eq!(payload.user_id, user_id);
    assert_eq!(payload.import_job_id, import_job_id);
}

/// employee_import_payload() returns None for non-EmployeeImport jobs.
#[test]
fn test_employee_import_payload_helper_returns_none_for_other_types() {
    let job = Job::new(
        Uuid::new_v4(),
        0,
        TaskType::AttendanceReport,
        Uuid::new_v4(),
        JobPayload::AttendanceReport(ReportPayload {
            employee_ids: vec![],
            date_range: DateRange {
                start: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                end: NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
            },
            config: ReportConfig {
                report_type: ReportType::Summary,
                timezone: "UTC".to_string(),
                rounding_policy: crate::models::RoundingPolicy::Floor,
                work_unit_minutes: 30,
            },
        }),
    );
    assert!(job.employee_import_payload().is_none());
}
