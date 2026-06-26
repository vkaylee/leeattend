//! Attendance Log Service integration tests

use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use crate::common::helpers;
use apiserver::models::LogSource;
use apiserver::services::attendance_log_service::AttendanceLogService;
use chrono::{TimeZone, Utc};

#[tokio::test]
async fn test_process_log_success_with_timezone() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.diesel_pools.clone();
    let factory = TestFactory::new(pool.clone());
    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&test_db.url());

    let workspace_id = factory
        .workspace()
        .with_timezone("Asia/Ho_Chi_Minh")
        .create()
        .await;
    let _employee_id = factory
        .employee()
        .with_workspace(workspace_id)
        .with_employee_id_number("EMP_VN_001")
        .create()
        .await;

    // 2. Process a naive timestamp (8 AM VN should be 1 AM UTC)
    let punch_time_raw = "2026-01-23 08:00:00";
    let repo = std::sync::Arc::new(apiserver::repositories::DieselTimeEntryRepository::new(
        diesel_pools.clone(),
    ));
    let summary_service = std::sync::Arc::new(apiserver::services::AttendanceSummaryService::new(
        apiserver::services::attendance_summary_service::AttendanceSummaryServiceArgs {
            time_entry_repo: repo.clone(),
            roster_repo: std::sync::Arc::new(apiserver::repositories::DieselRosterRepository::new(
                diesel_pools.clone(),
            )),
            policy_repo: std::sync::Arc::new(
                apiserver::repositories::DieselAttendancePolicyRepository::new(
                    diesel_pools.clone(),
                ),
            ),
            workspace_repo: std::sync::Arc::new(
                apiserver::repositories::DieselWorkspaceRepository::new(diesel_pools.clone()),
            ),
            employee_repo: std::sync::Arc::new(
                apiserver::repositories::DieselEmployeeRepository::new(diesel_pools.clone()),
            ),
            holiday_repo: std::sync::Arc::new(
                apiserver::repositories::DieselHolidayRepository::new(diesel_pools.clone()),
            ),
            employment_repo: std::sync::Arc::new(
                apiserver::repositories::DieselEmploymentRepository::new(diesel_pools.clone()),
            ),
            leave_request_repo: std::sync::Arc::new(
                apiserver::repositories::DieselLeaveRequestRepository::new(diesel_pools.clone()),
            ),
            daily_summary_repo: std::sync::Arc::new(
                apiserver::repositories::DieselAttendanceDailySummaryRepository::new(
                    diesel_pools.clone(),
                ),
            ),
        },
    ));
    let pairing = std::sync::Arc::new(apiserver::services::pairing::PairingService::new(
        apiserver::services::pairing::PairingServiceArgs::new(
            repo.clone(),
            std::sync::Arc::new(apiserver::repositories::DieselRosterRepository::new(
                diesel_pools.clone(),
            )),
            std::sync::Arc::new(apiserver::repositories::DieselEmployeeRepository::new(
                diesel_pools.clone(),
            )),
            std::sync::Arc::new(
                apiserver::repositories::DieselRawAttendanceLogRepository::new(
                    diesel_pools.clone(),
                ),
            ),
            summary_service as std::sync::Arc<dyn apiserver::services::SummaryRecalculator>,
        ),
    ));
    let emp_repo: std::sync::Arc<dyn apiserver::repositories::EmployeeRepository> =
        std::sync::Arc::new(apiserver::repositories::DieselEmployeeRepository::new(
            diesel_pools.clone(),
        ));
    let rl_repo: std::sync::Arc<
        dyn apiserver::repositories::raw_attendance_log_repository::RawAttendanceLogRepository,
    > = std::sync::Arc::new(
        apiserver::repositories::DieselRawAttendanceLogRepository::new(diesel_pools.clone()),
    );
    let st_repo: std::sync::Arc<
        dyn apiserver::repositories::staging_log_repository::StagingLogRepository,
    > = std::sync::Arc::new(apiserver::repositories::DieselStagingLogRepository::new(
        diesel_pools.clone(),
    ));
    let er_repo: std::sync::Arc<
        dyn apiserver::repositories::attendance_log_error_repository::AttendanceLogErrorRepository,
    > = std::sync::Arc::new(
        apiserver::repositories::DieselAttendanceLogErrorRepository::new(diesel_pools.clone()),
    );
    let (service, _) = AttendanceLogService::new(
        pairing,
        apiserver::services::attendance_log_service::AttendanceLogServiceConfig {
            buffer_capacity: 100,
            batch_size: 10,
            flush_interval_ms: 1000,
        },
        apiserver::services::attendance_log_service::AttendanceLogServiceDeps {
            employee_repo: emp_repo,
            raw_log_repo: rl_repo,
            staging_repo: st_repo,
            error_repo: er_repo,
        },
    );
    let result = service
        .process_log(
            apiserver::services::attendance_log_service::ProcessLogArgs {
                workspace_id,
                employee_id_number: "EMP_VN_001",
                punch_time_raw,
                source: LogSource::Device,
                device_id: Some("DEV_01".to_string()),
                raw_data: None,
                timezone: "Asia/Ho_Chi_Minh",
                date_fmt: Some("YYYY-MM-DD"),
                time_fmt: Some("24h"),
            },
        )
        .await;

    if let Err(e) = &result {
        panic!("result was Err: {:?}", e);
    }
    assert!(result.unwrap());

    // Verify in DB
    let logs =
        helpers::get_paginated_raw_logs(&diesel_pools, workspace_id, None, None, None, 10, 0)
            .await
            .unwrap();
    assert_eq!(logs.len(), 1);
    // 08:00:00+07:00 -> 01:00:00 UTC
    assert_eq!(
        logs[0].punch_time,
        Utc.with_ymd_and_hms(2026, 1, 23, 1, 0, 0).unwrap()
    );
}

#[tokio::test]
async fn test_same_second_punch_with_different_status_allowed() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.diesel_pools.clone();
    let factory = TestFactory::new(pool.clone());
    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&test_db.url());

    let workspace_id = factory.workspace().create().await;
    let _employee_id = factory
        .employee()
        .with_workspace(workspace_id)
        .with_employee_id_number("EMP001")
        .create()
        .await;

    let punch_time_raw = "2026-01-23 08:00:00";

    let repo = std::sync::Arc::new(apiserver::repositories::DieselTimeEntryRepository::new(
        diesel_pools.clone(),
    ));
    let summary_service = std::sync::Arc::new(apiserver::services::AttendanceSummaryService::new(
        apiserver::services::attendance_summary_service::AttendanceSummaryServiceArgs {
            time_entry_repo: repo.clone(),
            roster_repo: std::sync::Arc::new(apiserver::repositories::DieselRosterRepository::new(
                diesel_pools.clone(),
            )),
            policy_repo: std::sync::Arc::new(
                apiserver::repositories::DieselAttendancePolicyRepository::new(
                    diesel_pools.clone(),
                ),
            ),
            workspace_repo: std::sync::Arc::new(
                apiserver::repositories::DieselWorkspaceRepository::new(diesel_pools.clone()),
            ),
            employee_repo: std::sync::Arc::new(
                apiserver::repositories::DieselEmployeeRepository::new(diesel_pools.clone()),
            ),
            holiday_repo: std::sync::Arc::new(
                apiserver::repositories::DieselHolidayRepository::new(diesel_pools.clone()),
            ),
            employment_repo: std::sync::Arc::new(
                apiserver::repositories::DieselEmploymentRepository::new(diesel_pools.clone()),
            ),
            leave_request_repo: std::sync::Arc::new(
                apiserver::repositories::DieselLeaveRequestRepository::new(diesel_pools.clone()),
            ),
            daily_summary_repo: std::sync::Arc::new(
                apiserver::repositories::DieselAttendanceDailySummaryRepository::new(
                    diesel_pools.clone(),
                ),
            ),
        },
    ));
    let pairing = std::sync::Arc::new(apiserver::services::pairing::PairingService::new(
        apiserver::services::pairing::PairingServiceArgs::new(
            repo.clone(),
            std::sync::Arc::new(apiserver::repositories::DieselRosterRepository::new(
                diesel_pools.clone(),
            )),
            std::sync::Arc::new(apiserver::repositories::DieselEmployeeRepository::new(
                diesel_pools.clone(),
            )),
            std::sync::Arc::new(
                apiserver::repositories::DieselRawAttendanceLogRepository::new(
                    diesel_pools.clone(),
                ),
            ),
            summary_service as std::sync::Arc<dyn apiserver::services::SummaryRecalculator>,
        ),
    ));
    let emp_repo: std::sync::Arc<dyn apiserver::repositories::EmployeeRepository> =
        std::sync::Arc::new(apiserver::repositories::DieselEmployeeRepository::new(
            diesel_pools.clone(),
        ));
    let rl_repo: std::sync::Arc<
        dyn apiserver::repositories::raw_attendance_log_repository::RawAttendanceLogRepository,
    > = std::sync::Arc::new(
        apiserver::repositories::DieselRawAttendanceLogRepository::new(diesel_pools.clone()),
    );
    let st_repo: std::sync::Arc<
        dyn apiserver::repositories::staging_log_repository::StagingLogRepository,
    > = std::sync::Arc::new(apiserver::repositories::DieselStagingLogRepository::new(
        diesel_pools.clone(),
    ));
    let er_repo: std::sync::Arc<
        dyn apiserver::repositories::attendance_log_error_repository::AttendanceLogErrorRepository,
    > = std::sync::Arc::new(
        apiserver::repositories::DieselAttendanceLogErrorRepository::new(diesel_pools.clone()),
    );
    let (service, _) = AttendanceLogService::new(
        pairing,
        apiserver::services::attendance_log_service::AttendanceLogServiceConfig {
            buffer_capacity: 100,
            batch_size: 10,
            flush_interval_ms: 1000,
        },
        apiserver::services::attendance_log_service::AttendanceLogServiceDeps {
            employee_repo: emp_repo,
            raw_log_repo: rl_repo,
            staging_repo: st_repo,
            error_repo: er_repo,
        },
    );

    // 1. First punch (check-in)
    let res1 = service
        .process_log(
            apiserver::services::attendance_log_service::ProcessLogArgs {
                workspace_id,
                employee_id_number: "EMP001",
                punch_time_raw,
                source: LogSource::Device,
                device_id: Some("DEV_01".to_string()),
                raw_data: Some(serde_json::json!({"punch": 0, "status": 1})),
                timezone: "UTC",
                date_fmt: Some("YYYY-MM-DD"),
                time_fmt: Some("24h"),
            },
        )
        .await;
    assert!(res1.unwrap(), "First log should be saved");

    // 2. Second punch at EXACTLY same time but different status (check-out)
    let res2 = service
        .process_log(
            apiserver::services::attendance_log_service::ProcessLogArgs {
                workspace_id,
                employee_id_number: "EMP001",
                punch_time_raw,
                source: LogSource::Device,
                device_id: Some("DEV_01".to_string()),
                raw_data: Some(serde_json::json!({"punch": 1, "status": 1})),
                timezone: "UTC",
                date_fmt: Some("YYYY-MM-DD"),
                time_fmt: Some("24h"),
            },
        )
        .await;
    assert!(
        res2.unwrap(),
        "Second log at same second with different punch should be saved"
    );

    // 3. Exact duplicate (same time, punch, status) - should be skipped
    let res3 = service
        .process_log(
            apiserver::services::attendance_log_service::ProcessLogArgs {
                workspace_id,
                employee_id_number: "EMP001",
                punch_time_raw,
                source: LogSource::Device,
                device_id: Some("DEV_01".to_string()),
                raw_data: Some(serde_json::json!({"punch": 0, "status": 1})),
                timezone: "UTC",
                date_fmt: Some("YYYY-MM-DD"),
                time_fmt: Some("24h"),
            },
        )
        .await;
    assert!(
        !res3.unwrap(),
        "Exact duplicate should be skipped (Ok(false))"
    );

    // 4. Verify both unique logs are in DB
    let logs =
        helpers::get_paginated_raw_logs(&diesel_pools, workspace_id, None, None, None, 10, 0)
            .await
            .unwrap();
    assert_eq!(
        logs.len(),
        2,
        "DB should contain exactly 2 unique logs for the same second"
    );
}
