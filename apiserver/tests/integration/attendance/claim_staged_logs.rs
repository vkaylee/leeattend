//! Integration tests for `ClaimStagedLogsHandler`
//!
//! Tests the queue-based claiming of orphan attendance staging logs
//! when employees are created or imported.

use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use crate::common::helpers;
use apiserver::queue::{ClaimStagedLogsPayload, Job, JobPayload, TaskType};
use apiserver::worker::handlers::ClaimStagedLogsHandler;
use chrono::Utc;
use uuid::Uuid;

/// Handler claims pending staged logs for a matching employee.
#[tokio::test]
async fn test_handler_claims_staged_logs_for_employee() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool().clone();
    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&test_db.url());
    let deps = apiserver::worker::deps::WorkerDeps::new(&diesel_pools.clone());
    let factory = TestFactory::new(pool.clone());

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let emp_id_number = "QUEUE-CLAIM-001";
    let emp_id = factory
        .employee()
        .with_workspace(ws_id)
        .with_employee_id_number(emp_id_number)
        .create()
        .await;

    let punch_time = Utc::now();
    let fingerprint = format!("fp-{}", Uuid::new_v4());

    // 1. Create a staged log (orphan)
    helpers::create_staging_log(
        &diesel_pools,
        ws_id,
        Some("device-1".to_string()),
        emp_id_number,
        punch_time,
        None,
        &fingerprint,
    )
    .await
    .unwrap();

    let pending = helpers::count_pending_staging_logs(&diesel_pools, ws_id)
        .await
        .unwrap();
    assert_eq!(pending, 1, "One staged log should be pending");

    // 2. Construct and execute the ClaimStagedLogs job
    let job = Job::new(
        Uuid::new_v4(),
        0,
        TaskType::ClaimStagedLogs,
        ws_id,
        JobPayload::ClaimStagedLogs(ClaimStagedLogsPayload {
            workspace_id: ws_id,
            employee_id: emp_id,
            employee_id_number: emp_id_number.to_string(),
        }),
    );

    ClaimStagedLogsHandler::handle(&diesel_pools, &job, &deps)
        .await
        .unwrap();

    // 3. Verify staged log is no longer pending
    let pending_after = helpers::count_pending_staging_logs(&diesel_pools, ws_id)
        .await
        .unwrap();
    assert_eq!(pending_after, 0, "Staged log should be claimed");

    // 4. Verify raw log was created for this employee
    let raw_logs =
        helpers::get_paginated_raw_logs(&diesel_pools, ws_id, Some(emp_id), None, None, 10, 0)
            .await
            .unwrap();
    assert_eq!(raw_logs.len(), 1, "One raw log should exist");
    assert_eq!(raw_logs[0].punch_time.timestamp(), punch_time.timestamp());
}

/// Handler returns Ok when no staged logs exist for the employee.
#[tokio::test]
async fn test_handler_no_staged_logs_is_ok() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool().clone();
    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&test_db.url());
    let deps = apiserver::worker::deps::WorkerDeps::new(&diesel_pools.clone());
    let factory = TestFactory::new(pool.clone());

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let emp_id = factory
        .employee()
        .with_workspace(ws_id)
        .with_employee_id_number("QUEUE-CLAIM-EMPTY")
        .create()
        .await;

    let job = Job::new(
        Uuid::new_v4(),
        0,
        TaskType::ClaimStagedLogs,
        ws_id,
        JobPayload::ClaimStagedLogs(ClaimStagedLogsPayload {
            workspace_id: ws_id,
            employee_id: emp_id,
            employee_id_number: "QUEUE-CLAIM-EMPTY".to_string(),
        }),
    );

    // Should succeed even with no staged logs
    let result = ClaimStagedLogsHandler::handle(&diesel_pools, &job, &deps).await;
    assert!(result.is_ok(), "Handler should succeed with no staged logs");
}

/// Handler claims multiple staged logs for the same employee ID.
#[tokio::test]
async fn test_handler_claims_multiple_staged_logs() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool().clone();
    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&test_db.url());
    let deps = apiserver::worker::deps::WorkerDeps::new(&diesel_pools.clone());
    let factory = TestFactory::new(pool.clone());

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let emp_id_number = "QUEUE-CLAIM-MULTI";
    let emp_id = factory
        .employee()
        .with_workspace(ws_id)
        .with_employee_id_number(emp_id_number)
        .create()
        .await;

    // Create 3 staged logs at different times
    for i in 0..3 {
        let punch_time = Utc::now() + chrono::Duration::hours(i);
        let fingerprint = format!("fp-multi-{}-{}", i, Uuid::new_v4());
        helpers::create_staging_log(
            &diesel_pools,
            ws_id,
            Some("device-1".to_string()),
            emp_id_number,
            punch_time,
            None,
            &fingerprint,
        )
        .await
        .unwrap();
    }

    let pending = helpers::count_pending_staging_logs(&diesel_pools, ws_id)
        .await
        .unwrap();
    assert_eq!(pending, 3, "Three staged logs should be pending");

    // Execute handler
    let job = Job::new(
        Uuid::new_v4(),
        0,
        TaskType::ClaimStagedLogs,
        ws_id,
        JobPayload::ClaimStagedLogs(ClaimStagedLogsPayload {
            workspace_id: ws_id,
            employee_id: emp_id,
            employee_id_number: emp_id_number.to_string(),
        }),
    );

    ClaimStagedLogsHandler::handle(&diesel_pools, &job, &deps)
        .await
        .unwrap();

    // Verify all claimed
    let pending_after = helpers::count_pending_staging_logs(&diesel_pools, ws_id)
        .await
        .unwrap();
    assert_eq!(pending_after, 0, "All staged logs should be claimed");

    let raw_logs =
        helpers::get_paginated_raw_logs(&diesel_pools, ws_id, Some(emp_id), None, None, 10, 0)
            .await
            .unwrap();
    assert_eq!(raw_logs.len(), 3, "Three raw logs should exist");
}

/// Handler only claims logs matching the `employee_id_number`, not other employees.
#[tokio::test]
async fn test_handler_only_claims_matching_employee() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool().clone();
    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&test_db.url());
    let deps = apiserver::worker::deps::WorkerDeps::new(&diesel_pools.clone());
    let factory = TestFactory::new(pool.clone());

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;

    let emp_a_id_number = "QUEUE-CLAIM-A";
    let emp_b_id_number = "QUEUE-CLAIM-B";

    let emp_a_id = factory
        .employee()
        .with_workspace(ws_id)
        .with_employee_id_number(emp_a_id_number)
        .create()
        .await;
    let _emp_b_id = factory
        .employee()
        .with_workspace(ws_id)
        .with_employee_id_number(emp_b_id_number)
        .create()
        .await;

    // Create staged logs for both employees
    for (id_num, suffix) in [(emp_a_id_number, "a"), (emp_b_id_number, "b")] {
        let fingerprint = format!("fp-iso-{}-{}", suffix, Uuid::new_v4());
        helpers::create_staging_log(
            &diesel_pools,
            ws_id,
            Some("device-1".to_string()),
            id_num,
            Utc::now(),
            None,
            &fingerprint,
        )
        .await
        .unwrap();
    }

    let pending = helpers::count_pending_staging_logs(&diesel_pools, ws_id)
        .await
        .unwrap();
    assert_eq!(pending, 2, "Two staged logs should be pending");

    // Claim only employee A's logs
    let job = Job::new(
        Uuid::new_v4(),
        0,
        TaskType::ClaimStagedLogs,
        ws_id,
        JobPayload::ClaimStagedLogs(ClaimStagedLogsPayload {
            workspace_id: ws_id,
            employee_id: emp_a_id,
            employee_id_number: emp_a_id_number.to_string(),
        }),
    );

    ClaimStagedLogsHandler::handle(&diesel_pools, &job, &deps)
        .await
        .unwrap();

    // Employee B's staged log should still be pending
    let pending_after = helpers::count_pending_staging_logs(&diesel_pools, ws_id)
        .await
        .unwrap();
    assert_eq!(
        pending_after, 1,
        "Employee B's staged log should still be pending"
    );

    // Employee A should have a raw log
    let raw_logs_a =
        helpers::get_paginated_raw_logs(&diesel_pools, ws_id, Some(emp_a_id), None, None, 10, 0)
            .await
            .unwrap();
    assert_eq!(raw_logs_a.len(), 1, "Employee A should have 1 raw log");
}
