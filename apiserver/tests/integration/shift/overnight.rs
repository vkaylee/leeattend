//! Overnight shift pairing tests

use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::models::LogSource;
use chrono::{Duration, NaiveDate, NaiveTime, TimeZone, Utc};
use chrono_tz::Tz;
use uuid::Uuid;

#[tokio::test]
async fn test_overnight_shift_pairing() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.diesel_pools.clone();
    let factory = TestFactory::new(pool.clone());
    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&test_db.url());

    let user_id = factory.user().create().await;
    let ws_id = factory
        .workspace()
        .with_timezone("Asia/Ho_Chi_Minh")
        .create_with_owner(user_id)
        .await;
    let employee_id = factory.employee().with_workspace(ws_id).create().await;
    let employment_id: Uuid = crate::test_query_scalar_uuid!(
        &pool.api,
        "SELECT id FROM employments WHERE employee_id = $1 LIMIT 1",
        diesel::sql_types::Uuid,
        employee_id
    );

    // 1. Overnight Shift: 22:00 -> 06:00
    let shift_id = factory
        .shift()
        .with_workspace(ws_id)
        .with_name("Night Shift")
        .with_times(
            NaiveTime::from_hms_opt(22, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(6, 0, 0).unwrap(),
        )
        .create()
        .await;

    let shift_date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();

    // Assign shift
    let shift_repo = apiserver::repositories::DieselShiftRepository::new(diesel_pools.clone());
    use apiserver::repositories::shift_repository::ShiftRepository;
    shift_repo
        .assign_employment_shift(
            &apiserver::diesel_pool::TenantAuthToken::system_override(ws_id),
            employment_id,
            shift_id,
            shift_date,
            None,
        )
        .await
        .unwrap();

    let roster_repo = apiserver::repositories::DieselRosterRepository::new(diesel_pools.clone());

    // 2. Insert Logs
    let tz: Tz = "Asia/Ho_Chi_Minh".parse().unwrap();
    let punch_in_utc = tz
        .from_local_datetime(&shift_date.and_hms_opt(21, 55, 0).unwrap())
        .single()
        .unwrap()
        .with_timezone(&Utc);
    let punch_out_utc = tz
        .from_local_datetime(
            &(shift_date + Duration::days(1))
                .and_hms_opt(6, 5, 0)
                .unwrap(),
        )
        .single()
        .unwrap()
        .with_timezone(&Utc);

    let raw_repo =
        apiserver::repositories::DieselRawAttendanceLogRepository::new(diesel_pools.clone());
    use apiserver::repositories::raw_attendance_log_repository::RawAttendanceLogRepository;

    let raw_log_in = apiserver::models::RawAttendanceLog::new(
        Uuid::new_v4(),
        ws_id,
        employment_id,
        punch_in_utc,
        LogSource::Device,
        None,
        None,
        -1,
        -1,
        Utc::now(),
    );
    raw_repo.create_raw_log(ws_id, &raw_log_in).await.unwrap();

    let raw_log_out = apiserver::models::RawAttendanceLog::new(
        Uuid::new_v4(),
        ws_id,
        employment_id,
        punch_out_utc,
        LogSource::Device,
        None,
        None,
        -1,
        -1,
        Utc::now(),
    );
    raw_repo.create_raw_log(ws_id, &raw_log_out).await.unwrap();

    // 3. Process Pairing
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
            employment_repo: std::sync::Arc::new(
                apiserver::repositories::DieselEmploymentRepository::new(diesel_pools.clone()),
            ),
            holiday_repo: std::sync::Arc::new(
                apiserver::repositories::DieselHolidayRepository::new(diesel_pools.clone()),
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

    apiserver::services::pairing_service::PairingService::new(
        apiserver::services::pairing_service::PairingServiceArgs::new(
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
            std::sync::Arc::new(apiserver::repositories::DieselShiftRepository::new(
                diesel_pools.clone(),
            )),
            summary_service as std::sync::Arc<dyn apiserver::services::SummaryRecalculator>,
        ),
    )
    .reprocess_daily_attendance(ws_id, employment_id, shift_date, tz)
    .await
    .unwrap();

    // 4. Verify
    use apiserver::repositories::time_entry_repository::TimeEntryRepository;
    let entries = repo
        .get_employment_time_entries(
            &apiserver::diesel_pool::TenantAuthToken::system_override(ws_id),
            employment_id,
            shift_date.and_hms_opt(0, 0, 0).unwrap().and_utc(),
            (shift_date + Duration::days(1))
                .and_hms_opt(23, 59, 59)
                .unwrap()
                .and_utc(),
        )
        .await
        .unwrap();

    assert_eq!(entries.len(), 1);
    let entry = &entries[0];

    assert_eq!(entry.punch_in_time(), punch_in_utc);
    assert_eq!(entry.punch_out_time(), Some(punch_out_utc));
    assert_eq!(entry.duration_minutes(), Some(490));
}

#[tokio::test]
async fn test_overnight_shift_multiple_checkout_punches_subminute_deduplication() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.diesel_pools.clone();
    let factory = TestFactory::new(pool.clone());
    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&test_db.url());

    let user_id = factory.user().create().await;
    let ws_id = factory
        .workspace()
        .with_timezone("Asia/Ho_Chi_Minh")
        .create_with_owner(user_id)
        .await;
    let employee_id = factory.employee().with_workspace(ws_id).create().await;
    let employment_id: Uuid = crate::test_query_scalar_uuid!(
        &pool.api,
        "SELECT id FROM employments WHERE employee_id = $1 LIMIT 1",
        diesel::sql_types::Uuid,
        employee_id
    );

    // 1. Overnight Shift: 18:00 -> 02:00 next day
    let shift_id = factory
        .shift()
        .with_workspace(ws_id)
        .with_name("Overnight Shift 18-02")
        .with_times(
            NaiveTime::from_hms_opt(18, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(2, 0, 0).unwrap(),
        )
        .create()
        .await;

    let shift_date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();

    // Assign shift
    let shift_repo = apiserver::repositories::DieselShiftRepository::new(diesel_pools.clone());
    use apiserver::repositories::shift_repository::ShiftRepository;
    shift_repo
        .assign_employment_shift(
            &apiserver::diesel_pool::TenantAuthToken::system_override(ws_id),
            employment_id,
            shift_id,
            shift_date,
            None,
        )
        .await
        .unwrap();

    let roster_repo = apiserver::repositories::DieselRosterRepository::new(diesel_pools.clone());

    // 2. Insert Logs
    let tz: Tz = "Asia/Ho_Chi_Minh".parse().unwrap();
    let punch_in_utc = tz
        .from_local_datetime(&shift_date.and_hms_opt(18, 0, 0).unwrap())
        .single()
        .unwrap()
        .with_timezone(&Utc);

    // First checkout log at 06:25:01
    let punch_out1_utc = tz
        .from_local_datetime(
            &(shift_date + Duration::days(1))
                .and_hms_opt(6, 25, 1)
                .unwrap(),
        )
        .single()
        .unwrap()
        .with_timezone(&Utc);

    // Second checkout log at 06:25:02 (same minute)
    let punch_out2_utc = tz
        .from_local_datetime(
            &(shift_date + Duration::days(1))
                .and_hms_opt(6, 25, 2)
                .unwrap(),
        )
        .single()
        .unwrap()
        .with_timezone(&Utc);

    // Expected punch out in time entry is at 06:25:00 (rounded to minute)
    let expected_punch_out_utc = tz
        .from_local_datetime(
            &(shift_date + Duration::days(1))
                .and_hms_opt(6, 25, 0)
                .unwrap(),
        )
        .single()
        .unwrap()
        .with_timezone(&Utc);

    let raw_repo =
        apiserver::repositories::DieselRawAttendanceLogRepository::new(diesel_pools.clone());
    use apiserver::repositories::raw_attendance_log_repository::RawAttendanceLogRepository;

    let log_in = apiserver::models::RawAttendanceLog::new(
        Uuid::new_v4(),
        ws_id,
        employment_id,
        punch_in_utc,
        LogSource::Device,
        None,
        None,
        -1,
        -1,
        Utc::now(),
    );
    raw_repo.create_raw_log(ws_id, &log_in).await.unwrap();

    let log_out1 = apiserver::models::RawAttendanceLog::new(
        Uuid::new_v4(),
        ws_id,
        employment_id,
        punch_out1_utc,
        LogSource::Device,
        None,
        None,
        -1,
        -1,
        Utc::now(),
    );
    raw_repo.create_raw_log(ws_id, &log_out1).await.unwrap();

    let log_out2 = apiserver::models::RawAttendanceLog::new(
        Uuid::new_v4(),
        ws_id,
        employment_id,
        punch_out2_utc,
        LogSource::Device,
        None,
        None,
        -1,
        -1,
        Utc::now(),
    );
    raw_repo.create_raw_log(ws_id, &log_out2).await.unwrap();

    // 3. Process Pairing for Day 1
    let time_entry_repo = std::sync::Arc::new(
        apiserver::repositories::DieselTimeEntryRepository::new(diesel_pools.clone()),
    );
    let summary_service = std::sync::Arc::new(apiserver::services::AttendanceSummaryService::new(
        apiserver::services::attendance_summary_service::AttendanceSummaryServiceArgs {
            time_entry_repo: time_entry_repo.clone(),
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
            employment_repo: std::sync::Arc::new(
                apiserver::repositories::DieselEmploymentRepository::new(diesel_pools.clone()),
            ),
            holiday_repo: std::sync::Arc::new(
                apiserver::repositories::DieselHolidayRepository::new(diesel_pools.clone()),
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

    let service = apiserver::services::pairing_service::PairingService::new(
        apiserver::services::pairing_service::PairingServiceArgs::new(
            time_entry_repo.clone(),
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
            std::sync::Arc::new(apiserver::repositories::DieselShiftRepository::new(
                diesel_pools.clone(),
            )),
            summary_service as std::sync::Arc<dyn apiserver::services::SummaryRecalculator>,
        ),
    );

    service
        .reprocess_daily_attendance(ws_id, employment_id, shift_date, tz)
        .await
        .unwrap();

    // 4. Verify Day 1 Time Entry contains both raw log IDs in metadata
    use apiserver::repositories::time_entry_repository::TimeEntryRepository;
    let day1_entries = time_entry_repo
        .get_employment_time_entries(
            &apiserver::diesel_pool::TenantAuthToken::system_override(ws_id),
            employment_id,
            shift_date.and_hms_opt(0, 0, 0).unwrap().and_utc(),
            shift_date.and_hms_opt(23, 59, 59).unwrap().and_utc(),
        )
        .await
        .unwrap();

    assert_eq!(day1_entries.len(), 1);
    let entry = &day1_entries[0];
    assert_eq!(entry.punch_in_time(), punch_in_utc);
    // Should choose the latest punch in the same minute as checkout time (06:25:00 after minute precision conversion)
    assert_eq!(entry.punch_out_time(), Some(expected_punch_out_utc));

    let meta = entry.generation_metadata().unwrap();
    let log_ids = meta["log_ids"].as_array().unwrap();
    // Verify that both punch_out1_utc and punch_out2_utc are registered as consumed log IDs
    let log_ids_strs: Vec<&str> = log_ids.iter().map(|v| v.as_str().unwrap()).collect();
    assert!(log_ids_strs.contains(&log_in.id().to_string().as_str()));
    assert!(log_ids_strs.contains(&log_out1.id().to_string().as_str()));
    assert!(log_ids_strs.contains(&log_out2.id().to_string().as_str()));

    // 5. Reprocess Day 2 (Day 2 has no shift / is off day)
    let next_date = shift_date + Duration::days(1);
    service
        .reprocess_daily_attendance(ws_id, employment_id, next_date, tz)
        .await
        .unwrap();

    // Verify Day 2 has ZERO time entries because the 06:25:02 log was consumed by Day 1
    let day2_entries = time_entry_repo
        .get_employment_time_entries(
            &apiserver::diesel_pool::TenantAuthToken::system_override(ws_id),
            employment_id,
            next_date.and_hms_opt(0, 0, 0).unwrap().and_utc(),
            next_date.and_hms_opt(23, 59, 59).unwrap().and_utc(),
        )
        .await
        .unwrap();

    assert_eq!(day2_entries.len(), 0);
}

#[tokio::test]
async fn test_overnight_shift_multiple_checkout_punches_ingested_sequentially() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.diesel_pools.clone();
    let factory = TestFactory::new(pool.clone());
    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&test_db.url());

    let user_id = factory.user().create().await;
    let ws_id = factory
        .workspace()
        .with_timezone("Asia/Ho_Chi_Minh")
        .create_with_owner(user_id)
        .await;
    let employee_id = factory.employee().with_workspace(ws_id).create().await;
    let employment_id: Uuid = crate::test_query_scalar_uuid!(
        &pool.api,
        "SELECT id FROM employments WHERE employee_id = $1 LIMIT 1",
        diesel::sql_types::Uuid,
        employee_id
    );

    // 1. Overnight Shift: 18:00 -> 02:00 next day
    let shift_id = factory
        .shift()
        .with_workspace(ws_id)
        .with_name("Overnight Shift 18-02")
        .with_times(
            NaiveTime::from_hms_opt(18, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(2, 0, 0).unwrap(),
        )
        .create()
        .await;

    let shift_date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();

    // Assign shift
    let shift_repo = apiserver::repositories::DieselShiftRepository::new(diesel_pools.clone());
    use apiserver::repositories::shift_repository::ShiftRepository;
    shift_repo
        .assign_employment_shift(
            &apiserver::diesel_pool::TenantAuthToken::system_override(ws_id),
            employment_id,
            shift_id,
            shift_date,
            None,
        )
        .await
        .unwrap();

    // 2. Setup Service
    let time_entry_repo = std::sync::Arc::new(
        apiserver::repositories::DieselTimeEntryRepository::new(diesel_pools.clone()),
    );
    let summary_service = std::sync::Arc::new(apiserver::services::AttendanceSummaryService::new(
        apiserver::services::attendance_summary_service::AttendanceSummaryServiceArgs {
            time_entry_repo: time_entry_repo.clone(),
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
            employment_repo: std::sync::Arc::new(
                apiserver::repositories::DieselEmploymentRepository::new(diesel_pools.clone()),
            ),
            holiday_repo: std::sync::Arc::new(
                apiserver::repositories::DieselHolidayRepository::new(diesel_pools.clone()),
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

    let service = apiserver::services::pairing_service::PairingService::new(
        apiserver::services::pairing_service::PairingServiceArgs::new(
            time_entry_repo.clone(),
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
            std::sync::Arc::new(apiserver::repositories::DieselShiftRepository::new(
                diesel_pools.clone(),
            )),
            summary_service as std::sync::Arc<dyn apiserver::services::SummaryRecalculator>,
        ),
    );

    // 3. Process logs sequentially
    let tz: Tz = "Asia/Ho_Chi_Minh".parse().unwrap();
    let punch_in_utc = tz
        .from_local_datetime(&shift_date.and_hms_opt(18, 0, 0).unwrap())
        .single()
        .unwrap()
        .with_timezone(&Utc);

    let log_in = apiserver::models::RawAttendanceLog::new(
        Uuid::new_v4(),
        ws_id,
        employment_id,
        punch_in_utc,
        LogSource::Device,
        None,
        None,
        -1,
        -1,
        Utc::now(),
    );
    let raw_repo =
        apiserver::repositories::DieselRawAttendanceLogRepository::new(diesel_pools.clone());
    use apiserver::repositories::raw_attendance_log_repository::RawAttendanceLogRepository;
    raw_repo.create_raw_log(ws_id, &log_in).await.unwrap();

    // Process first log (punch-in)
    service.process_new_raw_log(&log_in).await.unwrap();

    // First checkout log at 06:25:01
    let punch_out1_utc = tz
        .from_local_datetime(
            &(shift_date + Duration::days(1))
                .and_hms_opt(6, 25, 1)
                .unwrap(),
        )
        .single()
        .unwrap()
        .with_timezone(&Utc);

    let log_out1 = apiserver::models::RawAttendanceLog::new(
        Uuid::new_v4(),
        ws_id,
        employment_id,
        punch_out1_utc,
        LogSource::Device,
        None,
        None,
        -1,
        -1,
        Utc::now(),
    );
    raw_repo.create_raw_log(ws_id, &log_out1).await.unwrap();

    // Process second log (first punch-out)
    service.process_new_raw_log(&log_out1).await.unwrap();

    // Second checkout log at 06:25:02 (same minute)
    let punch_out2_utc = tz
        .from_local_datetime(
            &(shift_date + Duration::days(1))
                .and_hms_opt(6, 25, 2)
                .unwrap(),
        )
        .single()
        .unwrap()
        .with_timezone(&Utc);

    let log_out2 = apiserver::models::RawAttendanceLog::new(
        Uuid::new_v4(),
        ws_id,
        employment_id,
        punch_out2_utc,
        LogSource::Device,
        None,
        None,
        -1,
        -1,
        Utc::now(),
    );
    raw_repo.create_raw_log(ws_id, &log_out2).await.unwrap();

    // Process third log (second punch-out)
    service.process_new_raw_log(&log_out2).await.unwrap();

    // 4. Verify Day 1 Time Entry contains both raw log IDs in metadata
    use apiserver::repositories::time_entry_repository::TimeEntryRepository;
    let day1_entries = time_entry_repo
        .get_employment_time_entries(
            &apiserver::diesel_pool::TenantAuthToken::system_override(ws_id),
            employment_id,
            shift_date.and_hms_opt(0, 0, 0).unwrap().and_utc(),
            shift_date.and_hms_opt(23, 59, 59).unwrap().and_utc(),
        )
        .await
        .unwrap();

    assert_eq!(day1_entries.len(), 1);
    let entry = &day1_entries[0];
    assert_eq!(entry.punch_in_time(), punch_in_utc);

    let expected_punch_out_utc = tz
        .from_local_datetime(
            &(shift_date + Duration::days(1))
                .and_hms_opt(6, 25, 0)
                .unwrap(),
        )
        .single()
        .unwrap()
        .with_timezone(&Utc);
    assert_eq!(entry.punch_out_time(), Some(expected_punch_out_utc));

    // Verify Day 2 (Day 2 has no shift / is off day)
    let next_date = shift_date + Duration::days(1);
    let day2_entries = time_entry_repo
        .get_employment_time_entries(
            &apiserver::diesel_pool::TenantAuthToken::system_override(ws_id),
            employment_id,
            next_date.and_hms_opt(0, 0, 0).unwrap().and_utc(),
            next_date.and_hms_opt(23, 59, 59).unwrap().and_utc(),
        )
        .await
        .unwrap();

    // Verify Day 2 has ZERO time entries because the 06:25 punches were claimed by Day 1 overnight shift
    // and thus ignored during Day 2 reprocessing.
    assert_eq!(day2_entries.len(), 0);
}

#[tokio::test]
async fn test_overnight_recalculate_day1_reclaims_punches() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.diesel_pools.clone();
    let factory = TestFactory::new(pool.clone());
    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&test_db.url());

    let user_id = factory.user().create().await;
    let ws_id = factory
        .workspace()
        .with_timezone("Asia/Ho_Chi_Minh")
        .create_with_owner(user_id)
        .await;
    let employee_id = factory.employee().with_workspace(ws_id).create().await;
    let employment_id: Uuid = crate::test_query_scalar_uuid!(
        &pool.api,
        "SELECT id FROM employments WHERE employee_id = $1 LIMIT 1",
        diesel::sql_types::Uuid,
        employee_id
    );

    // 1. Overnight Shift on Day 1 (18:00 -> 02:00 next day)
    let shift_id = factory
        .shift()
        .with_workspace(ws_id)
        .with_name("Overnight Shift 18-02")
        .with_times(
            NaiveTime::from_hms_opt(18, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(2, 0, 0).unwrap(),
        )
        .create()
        .await;

    let shift_date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();

    // Assign shift
    let shift_repo = apiserver::repositories::DieselShiftRepository::new(diesel_pools.clone());
    use apiserver::repositories::shift_repository::ShiftRepository;
    shift_repo
        .assign_employment_shift(
            &apiserver::diesel_pool::TenantAuthToken::system_override(ws_id),
            employment_id,
            shift_id,
            shift_date,
            None,
        )
        .await
        .unwrap();

    // Setup Service
    let time_entry_repo = std::sync::Arc::new(
        apiserver::repositories::DieselTimeEntryRepository::new(diesel_pools.clone()),
    );
    let summary_service = std::sync::Arc::new(apiserver::services::AttendanceSummaryService::new(
        apiserver::services::attendance_summary_service::AttendanceSummaryServiceArgs {
            time_entry_repo: time_entry_repo.clone(),
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
            employment_repo: std::sync::Arc::new(
                apiserver::repositories::DieselEmploymentRepository::new(diesel_pools.clone()),
            ),
            holiday_repo: std::sync::Arc::new(
                apiserver::repositories::DieselHolidayRepository::new(diesel_pools.clone()),
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

    let service = apiserver::services::pairing_service::PairingService::new(
        apiserver::services::pairing_service::PairingServiceArgs::new(
            time_entry_repo.clone(),
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
            std::sync::Arc::new(apiserver::repositories::DieselShiftRepository::new(
                diesel_pools.clone(),
            )),
            summary_service as std::sync::Arc<dyn apiserver::services::SummaryRecalculator>,
        ),
    );

    let tz: Tz = "Asia/Ho_Chi_Minh".parse().unwrap();
    let punch_in_utc = tz
        .from_local_datetime(&shift_date.and_hms_opt(18, 0, 0).unwrap())
        .single()
        .unwrap()
        .with_timezone(&Utc);

    let punch_out_utc = tz
        .from_local_datetime(
            &(shift_date + Duration::days(1))
                .and_hms_opt(6, 25, 0)
                .unwrap(),
        )
        .single()
        .unwrap()
        .with_timezone(&Utc);

    let log_in = apiserver::models::RawAttendanceLog::new(
        Uuid::new_v4(),
        ws_id,
        employment_id,
        punch_in_utc,
        LogSource::Device,
        None,
        None,
        -1,
        -1,
        Utc::now(),
    );
    let log_out = apiserver::models::RawAttendanceLog::new(
        Uuid::new_v4(),
        ws_id,
        employment_id,
        punch_out_utc,
        LogSource::Device,
        None,
        None,
        -1,
        -1,
        Utc::now(),
    );
    let raw_repo =
        apiserver::repositories::DieselRawAttendanceLogRepository::new(diesel_pools.clone());
    use apiserver::repositories::raw_attendance_log_repository::RawAttendanceLogRepository;
    raw_repo.create_raw_log(ws_id, &log_in).await.unwrap();
    raw_repo.create_raw_log(ws_id, &log_out).await.unwrap();

    // Manually simulate the bug state first:
    // Day 2 has claimed 06:25 as a punch-in (because Day 2 was reprocessed in isolation).
    let day2_date = shift_date + Duration::days(1);

    // We construct a fake time entry for Day 2 that claims log_out
    let fake_day2_entry = apiserver::models::TimeEntry::new(
        Uuid::new_v4(),
        employment_id,
        ws_id,
        None,
        punch_out_utc,
        None,
        None,
        false,
        "work".to_string(),
        None,
        Some(serde_json::json!({
            "log_ids": vec![log_out.id().to_string()]
        })),
        Utc::now(),
        Utc::now(),
    );

    // Save the fake Day 2 entry directly to database
    use apiserver::repositories::time_entry_repository::TimeEntryRepository;
    let auth_token = apiserver::diesel_pool::TenantAuthToken::system_override(ws_id);
    time_entry_repo
        .replace_time_entries(
            &auth_token,
            employment_id,
            day2_date.and_hms_opt(0, 0, 0).unwrap().and_utc(),
            day2_date.and_hms_opt(23, 59, 59).unwrap().and_utc(),
            vec![fake_day2_entry],
        )
        .await
        .unwrap();

    // Now reprocess Day 1
    service
        .reprocess_daily_attendance(ws_id, employment_id, shift_date, tz)
        .await
        .unwrap();

    // Reprocess Day 2 next to let it self-heal and clear the stolen punch
    service
        .reprocess_daily_attendance(ws_id, employment_id, day2_date, tz)
        .await
        .unwrap();

    // Day 1 should have successfully claimed the 06:25 punch-out, despite Day 2 having it previously
    let day1_entries = time_entry_repo
        .get_employment_time_entries(
            &auth_token,
            employment_id,
            shift_date.and_hms_opt(0, 0, 0).unwrap().and_utc(),
            shift_date.and_hms_opt(23, 59, 59).unwrap().and_utc(),
        )
        .await
        .unwrap();

    assert_eq!(day1_entries.len(), 1);
    assert_eq!(day1_entries[0].punch_out_time(), Some(punch_out_utc));

    // Day 2 should have zero entries
    let day2_entries = time_entry_repo
        .get_employment_time_entries(
            &auth_token,
            employment_id,
            day2_date.and_hms_opt(0, 0, 0).unwrap().and_utc(),
            day2_date.and_hms_opt(23, 59, 59).unwrap().and_utc(),
        )
        .await
        .unwrap();
    assert_eq!(day2_entries.len(), 0);
}

#[tokio::test]
async fn test_overnight_recalculate_range_reclaims_punches() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.diesel_pools.clone();
    let factory = TestFactory::new(pool.clone());
    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&test_db.url());

    let user_id = factory.user().create().await;
    let ws_id = factory
        .workspace()
        .with_timezone("Asia/Ho_Chi_Minh")
        .create_with_owner(user_id)
        .await;
    let employee_id = factory.employee().with_workspace(ws_id).create().await;
    let employment_id: Uuid = crate::test_query_scalar_uuid!(
        &pool.api,
        "SELECT id FROM employments WHERE employee_id = $1 LIMIT 1",
        diesel::sql_types::Uuid,
        employee_id
    );

    // 1. Overnight Shift on Day 1 (18:00 -> 02:00 next day)
    let shift_id = factory
        .shift()
        .with_workspace(ws_id)
        .with_name("Overnight Shift 18-02")
        .with_times(
            NaiveTime::from_hms_opt(18, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(2, 0, 0).unwrap(),
        )
        .create()
        .await;

    let shift_date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();

    // Assign shift
    let shift_repo = apiserver::repositories::DieselShiftRepository::new(diesel_pools.clone());
    use apiserver::repositories::shift_repository::ShiftRepository;
    shift_repo
        .assign_employment_shift(
            &apiserver::diesel_pool::TenantAuthToken::system_override(ws_id),
            employment_id,
            shift_id,
            shift_date,
            None,
        )
        .await
        .unwrap();

    // Setup Service
    let time_entry_repo = std::sync::Arc::new(
        apiserver::repositories::DieselTimeEntryRepository::new(diesel_pools.clone()),
    );
    let summary_service = std::sync::Arc::new(apiserver::services::AttendanceSummaryService::new(
        apiserver::services::attendance_summary_service::AttendanceSummaryServiceArgs {
            time_entry_repo: time_entry_repo.clone(),
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
            employment_repo: std::sync::Arc::new(
                apiserver::repositories::DieselEmploymentRepository::new(diesel_pools.clone()),
            ),
            holiday_repo: std::sync::Arc::new(
                apiserver::repositories::DieselHolidayRepository::new(diesel_pools.clone()),
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

    let service = apiserver::services::pairing_service::PairingService::new(
        apiserver::services::pairing_service::PairingServiceArgs::new(
            time_entry_repo.clone(),
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
            std::sync::Arc::new(apiserver::repositories::DieselShiftRepository::new(
                diesel_pools.clone(),
            )),
            summary_service as std::sync::Arc<dyn apiserver::services::SummaryRecalculator>,
        ),
    );

    let tz: Tz = "Asia/Ho_Chi_Minh".parse().unwrap();
    let punch_in_utc = tz
        .from_local_datetime(&shift_date.and_hms_opt(18, 0, 0).unwrap())
        .single()
        .unwrap()
        .with_timezone(&Utc);

    let punch_out_utc = tz
        .from_local_datetime(
            &(shift_date + Duration::days(1))
                .and_hms_opt(6, 25, 0)
                .unwrap(),
        )
        .single()
        .unwrap()
        .with_timezone(&Utc);

    let log_in = apiserver::models::RawAttendanceLog::new(
        Uuid::new_v4(),
        ws_id,
        employment_id,
        punch_in_utc,
        LogSource::Device,
        None,
        None,
        -1,
        -1,
        Utc::now(),
    );
    let log_out = apiserver::models::RawAttendanceLog::new(
        Uuid::new_v4(),
        ws_id,
        employment_id,
        punch_out_utc,
        LogSource::Device,
        None,
        None,
        -1,
        -1,
        Utc::now(),
    );
    let raw_repo =
        apiserver::repositories::DieselRawAttendanceLogRepository::new(diesel_pools.clone());
    use apiserver::repositories::raw_attendance_log_repository::RawAttendanceLogRepository;
    raw_repo.create_raw_log(ws_id, &log_in).await.unwrap();
    raw_repo.create_raw_log(ws_id, &log_out).await.unwrap();

    // Manually simulate the bug state first:
    // Day 2 has claimed 06:25 as a punch-in (because Day 2 was reprocessed in isolation).
    let day2_date = shift_date + Duration::days(1);

    // We construct a fake time entry for Day 2 that claims log_out
    let fake_day2_entry = apiserver::models::TimeEntry::new(
        Uuid::new_v4(),
        employment_id,
        ws_id,
        None,
        punch_out_utc,
        None,
        None,
        false,
        "work".to_string(),
        None,
        Some(serde_json::json!({
            "log_ids": vec![log_out.id().to_string()]
        })),
        Utc::now(),
        Utc::now(),
    );

    // Save the fake Day 2 entry directly to database
    use apiserver::repositories::time_entry_repository::TimeEntryRepository;
    let auth_token = apiserver::diesel_pool::TenantAuthToken::system_override(ws_id);
    time_entry_repo
        .replace_time_entries(
            &auth_token,
            employment_id,
            day2_date.and_hms_opt(0, 0, 0).unwrap().and_utc(),
            day2_date.and_hms_opt(23, 59, 59).unwrap().and_utc(),
            vec![fake_day2_entry],
        )
        .await
        .unwrap();

    // Now reprocess Day 1 and Day 2 as a range
    service
        .reprocess_attendance_range(ws_id, employment_id, shift_date, day2_date, tz)
        .await
        .unwrap();

    // Day 1 should have successfully claimed the 06:25 punch-out, despite Day 2 having it previously
    let day1_entries = time_entry_repo
        .get_employment_time_entries(
            &auth_token,
            employment_id,
            shift_date.and_hms_opt(0, 0, 0).unwrap().and_utc(),
            shift_date.and_hms_opt(23, 59, 59).unwrap().and_utc(),
        )
        .await
        .unwrap();

    assert_eq!(day1_entries.len(), 1);
    assert_eq!(day1_entries[0].punch_out_time(), Some(punch_out_utc));

    // Day 2 should have zero entries because it was cleared during range reprocessing self-healing
    let day2_entries = time_entry_repo
        .get_employment_time_entries(
            &auth_token,
            employment_id,
            day2_date.and_hms_opt(0, 0, 0).unwrap().and_utc(),
            day2_date.and_hms_opt(23, 59, 59).unwrap().and_utc(),
        )
        .await
        .unwrap();
    assert_eq!(day2_entries.len(), 0);
}

#[tokio::test]
async fn test_overnight_recalculate_day1_only_automatically_heals_day2() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.diesel_pools.clone();
    let factory = TestFactory::new(pool.clone());
    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&test_db.url());

    let user_id = factory.user().create().await;
    let ws_id = factory
        .workspace()
        .with_timezone("Asia/Ho_Chi_Minh")
        .create_with_owner(user_id)
        .await;
    let employee_id = factory.employee().with_workspace(ws_id).create().await;
    let employment_id: Uuid = crate::test_query_scalar_uuid!(
        &pool.api,
        "SELECT id FROM employments WHERE employee_id = $1 LIMIT 1",
        diesel::sql_types::Uuid,
        employee_id
    );

    // 1. Overnight Shift on Day 1 (18:00 -> 02:00 next day)
    let shift_id = factory
        .shift()
        .with_workspace(ws_id)
        .with_name("Overnight Shift 18-02")
        .with_times(
            NaiveTime::from_hms_opt(18, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(2, 0, 0).unwrap(),
        )
        .create()
        .await;

    let shift_date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();

    // Assign shift
    let shift_repo = apiserver::repositories::DieselShiftRepository::new(diesel_pools.clone());
    use apiserver::repositories::shift_repository::ShiftRepository;
    shift_repo
        .assign_employment_shift(
            &apiserver::diesel_pool::TenantAuthToken::system_override(ws_id),
            employment_id,
            shift_id,
            shift_date,
            None,
        )
        .await
        .unwrap();

    // Setup Service
    let time_entry_repo = std::sync::Arc::new(
        apiserver::repositories::DieselTimeEntryRepository::new(diesel_pools.clone()),
    );
    let summary_service = std::sync::Arc::new(apiserver::services::AttendanceSummaryService::new(
        apiserver::services::attendance_summary_service::AttendanceSummaryServiceArgs {
            time_entry_repo: time_entry_repo.clone(),
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
            employment_repo: std::sync::Arc::new(
                apiserver::repositories::DieselEmploymentRepository::new(diesel_pools.clone()),
            ),
            holiday_repo: std::sync::Arc::new(
                apiserver::repositories::DieselHolidayRepository::new(diesel_pools.clone()),
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

    let service = apiserver::services::pairing_service::PairingService::new(
        apiserver::services::pairing_service::PairingServiceArgs::new(
            time_entry_repo.clone(),
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
            std::sync::Arc::new(apiserver::repositories::DieselShiftRepository::new(
                diesel_pools.clone(),
            )),
            summary_service as std::sync::Arc<dyn apiserver::services::SummaryRecalculator>,
        ),
    );

    let tz: Tz = "Asia/Ho_Chi_Minh".parse().unwrap();
    let punch_in_utc = tz
        .from_local_datetime(&shift_date.and_hms_opt(18, 0, 0).unwrap())
        .single()
        .unwrap()
        .with_timezone(&Utc);

    let punch_out_utc = tz
        .from_local_datetime(
            &(shift_date + Duration::days(1))
                .and_hms_opt(6, 25, 0)
                .unwrap(),
        )
        .single()
        .unwrap()
        .with_timezone(&Utc);

    let log_in = apiserver::models::RawAttendanceLog::new(
        Uuid::new_v4(),
        ws_id,
        employment_id,
        punch_in_utc,
        LogSource::Device,
        None,
        None,
        -1,
        -1,
        Utc::now(),
    );
    let log_out = apiserver::models::RawAttendanceLog::new(
        Uuid::new_v4(),
        ws_id,
        employment_id,
        punch_out_utc,
        LogSource::Device,
        None,
        None,
        -1,
        -1,
        Utc::now(),
    );
    let raw_repo =
        apiserver::repositories::DieselRawAttendanceLogRepository::new(diesel_pools.clone());
    use apiserver::repositories::raw_attendance_log_repository::RawAttendanceLogRepository;
    raw_repo.create_raw_log(ws_id, &log_in).await.unwrap();
    raw_repo.create_raw_log(ws_id, &log_out).await.unwrap();

    // Manually simulate the bug state first:
    // Day 2 has claimed 06:25 as a punch-in.
    let day2_date = shift_date + Duration::days(1);
    let fake_day2_entry = apiserver::models::TimeEntry::new(
        Uuid::new_v4(),
        employment_id,
        ws_id,
        None,
        punch_out_utc,
        None,
        None,
        false,
        "work".to_string(),
        None,
        Some(serde_json::json!({
            "log_ids": vec![log_out.id().to_string()]
        })),
        Utc::now(),
        Utc::now(),
    );

    // Save the fake Day 2 entry directly to database
    use apiserver::repositories::time_entry_repository::TimeEntryRepository;
    let auth_token = apiserver::diesel_pool::TenantAuthToken::system_override(ws_id);
    time_entry_repo
        .replace_time_entries(
            &auth_token,
            employment_id,
            day2_date.and_hms_opt(0, 0, 0).unwrap().and_utc(),
            day2_date.and_hms_opt(23, 59, 59).unwrap().and_utc(),
            vec![fake_day2_entry],
        )
        .await
        .unwrap();

    // Now reprocess Day 1 ONLY.
    // It should automatically trigger reprocessing of Day 2 because Day 1 has an overnight shift.
    service
        .reprocess_daily_attendance(ws_id, employment_id, shift_date, tz)
        .await
        .unwrap();

    // Day 1 should have successfully claimed the 06:25 punch-out, despite Day 2 having it previously
    let day1_entries = time_entry_repo
        .get_employment_time_entries(
            &auth_token,
            employment_id,
            shift_date.and_hms_opt(0, 0, 0).unwrap().and_utc(),
            shift_date.and_hms_opt(23, 59, 59).unwrap().and_utc(),
        )
        .await
        .unwrap();

    assert_eq!(day1_entries.len(), 1);
    assert_eq!(day1_entries[0].punch_out_time(), Some(punch_out_utc));

    // Day 2 should have zero entries because it was automatically cleared during Day 1's daily reprocess next-day trigger
    let day2_entries = time_entry_repo
        .get_employment_time_entries(
            &auth_token,
            employment_id,
            day2_date.and_hms_opt(0, 0, 0).unwrap().and_utc(),
            day2_date.and_hms_opt(23, 59, 59).unwrap().and_utc(),
        )
        .await
        .unwrap();
    assert_eq!(day2_entries.len(), 0);
}

#[tokio::test]
async fn test_overnight_recalculate_day1_next_day_has_shift_with_own_punches() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.diesel_pools.clone();
    let factory = TestFactory::new(pool.clone());
    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&test_db.url());

    let user_id = factory.user().create().await;
    let ws_id = factory
        .workspace()
        .with_timezone("Asia/Ho_Chi_Minh")
        .create_with_owner(user_id)
        .await;
    let employee_id = factory.employee().with_workspace(ws_id).create().await;
    let employment_id: Uuid = crate::test_query_scalar_uuid!(
        &pool.api,
        "SELECT id FROM employments WHERE employee_id = $1 LIMIT 1",
        diesel::sql_types::Uuid,
        employee_id
    );

    // 1. Overnight Shift on Day 1 (18:00 -> 02:00 next day)
    let shift_id_1 = factory
        .shift()
        .with_workspace(ws_id)
        .with_name("Overnight Shift 18-02")
        .with_times(
            NaiveTime::from_hms_opt(18, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(2, 0, 0).unwrap(),
        )
        .create()
        .await;

    // 2. Day Shift on Day 2 (08:00 -> 17:00)
    let shift_id_2 = factory
        .shift()
        .with_workspace(ws_id)
        .with_name("Day Shift 08-17")
        .with_times(
            NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
        )
        .create()
        .await;

    let shift_date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
    let day2_date = shift_date + Duration::days(1);

    // Assign shifts
    let shift_repo = apiserver::repositories::DieselShiftRepository::new(diesel_pools.clone());
    use apiserver::repositories::shift_repository::ShiftRepository;
    shift_repo
        .assign_employment_shift(
            &apiserver::diesel_pool::TenantAuthToken::system_override(ws_id),
            employment_id,
            shift_id_1,
            shift_date,
            None,
        )
        .await
        .unwrap();
    shift_repo
        .assign_employment_shift(
            &apiserver::diesel_pool::TenantAuthToken::system_override(ws_id),
            employment_id,
            shift_id_2,
            day2_date,
            None,
        )
        .await
        .unwrap();

    // Setup Service
    let time_entry_repo = std::sync::Arc::new(
        apiserver::repositories::DieselTimeEntryRepository::new(diesel_pools.clone()),
    );
    let summary_service = std::sync::Arc::new(apiserver::services::AttendanceSummaryService::new(
        apiserver::services::attendance_summary_service::AttendanceSummaryServiceArgs {
            time_entry_repo: time_entry_repo.clone(),
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
            employment_repo: std::sync::Arc::new(
                apiserver::repositories::DieselEmploymentRepository::new(diesel_pools.clone()),
            ),
            holiday_repo: std::sync::Arc::new(
                apiserver::repositories::DieselHolidayRepository::new(diesel_pools.clone()),
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

    let service = apiserver::services::pairing_service::PairingService::new(
        apiserver::services::pairing_service::PairingServiceArgs::new(
            time_entry_repo.clone(),
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
            std::sync::Arc::new(apiserver::repositories::DieselShiftRepository::new(
                diesel_pools.clone(),
            )),
            summary_service as std::sync::Arc<dyn apiserver::services::SummaryRecalculator>,
        ),
    );

    let tz: Tz = "Asia/Ho_Chi_Minh".parse().unwrap();
    let punch_in_1_utc = tz
        .from_local_datetime(&shift_date.and_hms_opt(18, 0, 0).unwrap())
        .single()
        .unwrap()
        .with_timezone(&Utc);

    let punch_out_1_utc = tz
        .from_local_datetime(&day2_date.and_hms_opt(6, 25, 0).unwrap())
        .single()
        .unwrap()
        .with_timezone(&Utc);

    let punch_in_2_utc = tz
        .from_local_datetime(&day2_date.and_hms_opt(8, 5, 0).unwrap())
        .single()
        .unwrap()
        .with_timezone(&Utc);

    let punch_out_2_utc = tz
        .from_local_datetime(&day2_date.and_hms_opt(17, 5, 0).unwrap())
        .single()
        .unwrap()
        .with_timezone(&Utc);

    let log_in_1 = apiserver::models::RawAttendanceLog::new(
        Uuid::new_v4(),
        ws_id,
        employment_id,
        punch_in_1_utc,
        LogSource::Device,
        None,
        None,
        -1,
        -1,
        Utc::now(),
    );
    let log_out_1 = apiserver::models::RawAttendanceLog::new(
        Uuid::new_v4(),
        ws_id,
        employment_id,
        punch_out_1_utc,
        LogSource::Device,
        None,
        None,
        -1,
        -1,
        Utc::now(),
    );
    let log_in_2 = apiserver::models::RawAttendanceLog::new(
        Uuid::new_v4(),
        ws_id,
        employment_id,
        punch_in_2_utc,
        LogSource::Device,
        None,
        None,
        -1,
        -1,
        Utc::now(),
    );
    let log_out_2 = apiserver::models::RawAttendanceLog::new(
        Uuid::new_v4(),
        ws_id,
        employment_id,
        punch_out_2_utc,
        LogSource::Device,
        None,
        None,
        -1,
        -1,
        Utc::now(),
    );

    let raw_repo =
        apiserver::repositories::DieselRawAttendanceLogRepository::new(diesel_pools.clone());
    use apiserver::repositories::raw_attendance_log_repository::RawAttendanceLogRepository;
    raw_repo.create_raw_log(ws_id, &log_in_1).await.unwrap();
    raw_repo.create_raw_log(ws_id, &log_out_1).await.unwrap();
    raw_repo.create_raw_log(ws_id, &log_in_2).await.unwrap();
    raw_repo.create_raw_log(ws_id, &log_out_2).await.unwrap();

    // Manually simulate the bug state first:
    // Day 2 has claimed 06:25 as a punch-in and 17:05 as punch-out (stolen punch).
    let fake_day2_entry = apiserver::models::TimeEntry::new(
        Uuid::new_v4(),
        employment_id,
        ws_id,
        Some(shift_id_2),
        punch_out_1_utc,
        Some(punch_out_2_utc),
        None,
        false,
        "work".to_string(),
        None,
        Some(serde_json::json!({
            "log_ids": vec![log_out_1.id().to_string(), log_out_2.id().to_string()]
        })),
        Utc::now(),
        Utc::now(),
    );

    // Save the fake Day 2 entry directly to database
    use apiserver::repositories::time_entry_repository::TimeEntryRepository;
    let auth_token = apiserver::diesel_pool::TenantAuthToken::system_override(ws_id);
    time_entry_repo
        .replace_time_entries(
            &auth_token,
            employment_id,
            day2_date.and_hms_opt(0, 0, 0).unwrap().and_utc(),
            day2_date.and_hms_opt(23, 59, 59).unwrap().and_utc(),
            vec![fake_day2_entry],
        )
        .await
        .unwrap();

    // Now reprocess Day 1 ONLY.
    // It should automatically trigger reprocessing of Day 2 because Day 1 has an overnight shift.
    service
        .reprocess_daily_attendance(ws_id, employment_id, shift_date, tz)
        .await
        .unwrap();

    // Day 1 should have successfully claimed the 06:25 punch-out, despite Day 2 having it previously
    let day1_entries = time_entry_repo
        .get_employment_time_entries(
            &auth_token,
            employment_id,
            shift_date.and_hms_opt(0, 0, 0).unwrap().and_utc(),
            shift_date.and_hms_opt(23, 59, 59).unwrap().and_utc(),
        )
        .await
        .unwrap();

    assert_eq!(day1_entries.len(), 1);
    assert_eq!(day1_entries[0].punch_out_time(), Some(punch_out_1_utc));

    // Day 2 should now correctly have punch-in 08:05 and punch-out 17:05 (clearing the stolen 06:25 punch)
    let day2_entries = time_entry_repo
        .get_employment_time_entries(
            &auth_token,
            employment_id,
            day2_date.and_hms_opt(0, 0, 0).unwrap().and_utc(),
            day2_date.and_hms_opt(23, 59, 59).unwrap().and_utc(),
        )
        .await
        .unwrap();
    assert_eq!(day2_entries.len(), 1);
    assert_eq!(day2_entries[0].punch_in_time(), punch_in_2_utc);
    assert_eq!(day2_entries[0].punch_out_time(), Some(punch_out_2_utc));
}

#[tokio::test]
async fn test_consecutive_overnight_shifts_propagation() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.diesel_pools.clone();
    let factory = TestFactory::new(pool.clone());
    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&test_db.url());

    let user_id = factory.user().create().await;
    let ws_id = factory
        .workspace()
        .with_timezone("Asia/Ho_Chi_Minh")
        .create_with_owner(user_id)
        .await;
    let employee_id = factory.employee().with_workspace(ws_id).create().await;
    let employment_id: Uuid = crate::test_query_scalar_uuid!(
        &pool.api,
        "SELECT id FROM employments WHERE employee_id = $1 LIMIT 1",
        diesel::sql_types::Uuid,
        employee_id
    );

    // 1. Overnight Shift on Day 1 & Day 2 (18:00 -> 02:00 next day)
    let shift_id = factory
        .shift()
        .with_workspace(ws_id)
        .with_name("Overnight Shift 18-02")
        .with_times(
            NaiveTime::from_hms_opt(18, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(2, 0, 0).unwrap(),
        )
        .create()
        .await;

    let shift_date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
    let day2_date = shift_date + Duration::days(1);
    let day3_date = shift_date + Duration::days(2);

    // Assign shifts
    let shift_repo = apiserver::repositories::DieselShiftRepository::new(diesel_pools.clone());
    use apiserver::repositories::shift_repository::ShiftRepository;
    shift_repo
        .assign_employment_shift(
            &apiserver::diesel_pool::TenantAuthToken::system_override(ws_id),
            employment_id,
            shift_id,
            shift_date,
            None,
        )
        .await
        .unwrap();
    shift_repo
        .assign_employment_shift(
            &apiserver::diesel_pool::TenantAuthToken::system_override(ws_id),
            employment_id,
            shift_id,
            day2_date,
            None,
        )
        .await
        .unwrap();

    // Setup Service
    let time_entry_repo = std::sync::Arc::new(
        apiserver::repositories::DieselTimeEntryRepository::new(diesel_pools.clone()),
    );
    let summary_service = std::sync::Arc::new(apiserver::services::AttendanceSummaryService::new(
        apiserver::services::attendance_summary_service::AttendanceSummaryServiceArgs {
            time_entry_repo: time_entry_repo.clone(),
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
            employment_repo: std::sync::Arc::new(
                apiserver::repositories::DieselEmploymentRepository::new(diesel_pools.clone()),
            ),
            holiday_repo: std::sync::Arc::new(
                apiserver::repositories::DieselHolidayRepository::new(diesel_pools.clone()),
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

    let service = apiserver::services::pairing_service::PairingService::new(
        apiserver::services::pairing_service::PairingServiceArgs::new(
            time_entry_repo.clone(),
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
            std::sync::Arc::new(apiserver::repositories::DieselShiftRepository::new(
                diesel_pools.clone(),
            )),
            summary_service as std::sync::Arc<dyn apiserver::services::SummaryRecalculator>,
        ),
    );

    let tz: Tz = "Asia/Ho_Chi_Minh".parse().unwrap();
    let punch_in_1_utc = tz
        .from_local_datetime(&shift_date.and_hms_opt(18, 0, 0).unwrap())
        .single()
        .unwrap()
        .with_timezone(&Utc);

    let punch_out_1_utc = tz
        .from_local_datetime(&day2_date.and_hms_opt(6, 25, 0).unwrap())
        .single()
        .unwrap()
        .with_timezone(&Utc);

    let punch_in_2_utc = tz
        .from_local_datetime(&day2_date.and_hms_opt(18, 5, 0).unwrap())
        .single()
        .unwrap()
        .with_timezone(&Utc);

    let punch_out_2_utc = tz
        .from_local_datetime(&day3_date.and_hms_opt(6, 10, 0).unwrap())
        .single()
        .unwrap()
        .with_timezone(&Utc);

    let log_in_1 = apiserver::models::RawAttendanceLog::new(
        Uuid::new_v4(),
        ws_id,
        employment_id,
        punch_in_1_utc,
        LogSource::Device,
        None,
        None,
        -1,
        -1,
        Utc::now(),
    );
    let log_out_1 = apiserver::models::RawAttendanceLog::new(
        Uuid::new_v4(),
        ws_id,
        employment_id,
        punch_out_1_utc,
        LogSource::Device,
        None,
        None,
        -1,
        -1,
        Utc::now(),
    );
    let log_in_2 = apiserver::models::RawAttendanceLog::new(
        Uuid::new_v4(),
        ws_id,
        employment_id,
        punch_in_2_utc,
        LogSource::Device,
        None,
        None,
        -1,
        -1,
        Utc::now(),
    );
    let log_out_2 = apiserver::models::RawAttendanceLog::new(
        Uuid::new_v4(),
        ws_id,
        employment_id,
        punch_out_2_utc,
        LogSource::Device,
        None,
        None,
        -1,
        -1,
        Utc::now(),
    );

    let raw_repo =
        apiserver::repositories::DieselRawAttendanceLogRepository::new(diesel_pools.clone());
    use apiserver::repositories::raw_attendance_log_repository::RawAttendanceLogRepository;
    raw_repo.create_raw_log(ws_id, &log_in_1).await.unwrap();
    raw_repo.create_raw_log(ws_id, &log_out_1).await.unwrap();
    raw_repo.create_raw_log(ws_id, &log_in_2).await.unwrap();
    raw_repo.create_raw_log(ws_id, &log_out_2).await.unwrap();

    // Simulating bug state:
    // Day 2 has a fake entry claiming log_out_1 (06:25) as punch_in and log_in_2 (18:05) as punch_out.
    let fake_day2_entry = apiserver::models::TimeEntry::new(
        Uuid::new_v4(),
        employment_id,
        ws_id,
        Some(shift_id),
        punch_out_1_utc,
        Some(punch_in_2_utc),
        None,
        false,
        "work".to_string(),
        None,
        Some(serde_json::json!({
            "log_ids": vec![log_out_1.id().to_string(), log_in_2.id().to_string()]
        })),
        Utc::now(),
        Utc::now(),
    );

    // Day 3 has a fake entry claiming log_out_2 (06:10) as punch_in.
    let fake_day3_entry = apiserver::models::TimeEntry::new(
        Uuid::new_v4(),
        employment_id,
        ws_id,
        None,
        punch_out_2_utc,
        None,
        None,
        false,
        "work".to_string(),
        None,
        Some(serde_json::json!({
            "log_ids": vec![log_out_2.id().to_string()]
        })),
        Utc::now(),
        Utc::now(),
    );

    use apiserver::repositories::time_entry_repository::TimeEntryRepository;
    let auth_token = apiserver::diesel_pool::TenantAuthToken::system_override(ws_id);
    time_entry_repo
        .replace_time_entries(
            &auth_token,
            employment_id,
            day2_date.and_hms_opt(0, 0, 0).unwrap().and_utc(),
            day2_date.and_hms_opt(23, 59, 59).unwrap().and_utc(),
            vec![fake_day2_entry],
        )
        .await
        .unwrap();

    time_entry_repo
        .replace_time_entries(
            &auth_token,
            employment_id,
            day3_date.and_hms_opt(0, 0, 0).unwrap().and_utc(),
            day3_date.and_hms_opt(23, 59, 59).unwrap().and_utc(),
            vec![fake_day3_entry],
        )
        .await
        .unwrap();

    // Action: Reprocess Day 1 daily attendance ONLY.
    // This should trigger the overnight cascade, propagating reprocessing to Day 2 and Day 3.
    service
        .reprocess_daily_attendance(ws_id, employment_id, shift_date, tz)
        .await
        .unwrap();

    // Day 1 should have successfully claimed the 06:25 punch-out
    let day1_entries = time_entry_repo
        .get_employment_time_entries(
            &auth_token,
            employment_id,
            shift_date.and_hms_opt(0, 0, 0).unwrap().and_utc(),
            shift_date.and_hms_opt(23, 59, 59).unwrap().and_utc(),
        )
        .await
        .unwrap();
    assert_eq!(day1_entries.len(), 1);
    assert_eq!(day1_entries[0].punch_in_time(), punch_in_1_utc);
    assert_eq!(day1_entries[0].punch_out_time(), Some(punch_out_1_utc));

    // Day 2 should have correctly paired [18:05 - 06:10] (clearing 06:25 and reclaiming 06:10)
    let day2_entries = time_entry_repo
        .get_employment_time_entries(
            &auth_token,
            employment_id,
            day2_date.and_hms_opt(0, 0, 0).unwrap().and_utc(),
            day2_date.and_hms_opt(23, 59, 59).unwrap().and_utc(),
        )
        .await
        .unwrap();
    assert_eq!(day2_entries.len(), 1);
    assert_eq!(day2_entries[0].punch_in_time(), punch_in_2_utc);
    assert_eq!(day2_entries[0].punch_out_time(), Some(punch_out_2_utc));

    // Day 3 should have zero entries (clearing the stolen 06:10)
    let day3_entries = time_entry_repo
        .get_employment_time_entries(
            &auth_token,
            employment_id,
            day3_date.and_hms_opt(0, 0, 0).unwrap().and_utc(),
            day3_date.and_hms_opt(23, 59, 59).unwrap().and_utc(),
        )
        .await
        .unwrap();
    assert_eq!(day3_entries.len(), 0);
}
