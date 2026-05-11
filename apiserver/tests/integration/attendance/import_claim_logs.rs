use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::models::{enums::ImportTargetType, ImportJobStatus};
use apiserver::repositories::{
    employee, import_job_repository, raw_attendance_log_repository, staging_log_repository,
};
use apiserver::services::AttendanceImportService;
use chrono::Utc;
use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;

#[tokio::test]
async fn test_import_employee_claims_staged_logs() -> Result<(), Box<dyn std::error::Error>> {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool().clone();
    let factory = TestFactory::new(pool.clone());

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;

    let emp_id_number = "CLAIM-001";
    let normalized_id = apiserver::utils::normalize_employee_id(emp_id_number);
    let punch_time = Utc::now();
    let fingerprint = format!("fp-{}", Uuid::new_v4());

    // 1. Create a staged log for a non-existent employee
    staging_log_repository::create_staging_log(
        &pool,
        ws_id,
        Some("device-1".to_string()),
        &normalized_id,
        punch_time,
        None,
        &fingerprint,
    )
    .await?;

    // Verify it's pending
    let pending_count = staging_log_repository::count_pending_staging_logs(&pool, ws_id).await?;
    assert_eq!(pending_count, 1);

    // 2. Import employee with this ID
    let csv_content = format!("employee_code,full_name,hire_date,department_name\n{emp_id_number},Imported User,2026-01-01,IT");

    let file_path = format!("storage/test_imports/{}.csv", Uuid::new_v4());
    tokio::fs::create_dir_all("storage/test_imports").await?;
    tokio::fs::write(&file_path, csv_content).await?;

    let te_repo = std::sync::Arc::new(apiserver::repositories::SqlxTimeEntryRepository::new(
        pool.clone(),
    ));
    let import_repo: std::sync::Arc<
        dyn apiserver::repositories::import_job_repository::ImportJobRepository,
    > = std::sync::Arc::new(
        apiserver::repositories::import_job_repository::SqlxImportJobRepository::new(pool.clone()),
    );
    let adj_repo: std::sync::Arc<dyn apiserver::repositories::attendance_adjustment_repository::AttendanceAdjustmentRepository> = std::sync::Arc::new(apiserver::repositories::attendance_adjustment_repository::SqlxAttendanceAdjustmentRepository::new(pool.clone()));
    let emp_repo: std::sync::Arc<dyn apiserver::repositories::EmployeeRepository> =
        std::sync::Arc::new(
            apiserver::repositories::diesel_employee_repository::DieselEmployeeRepository::new(
                pool.clone(),
                apiserver::diesel_pool::DieselPools::for_test(&test_db.url()),
            ),
        );
    let import_service = AttendanceImportService::new(
        import_repo,
        adj_repo,
        emp_repo,
        te_repo,
        std::sync::Arc::new(apiserver::repositories::SqlxWorkspaceRepository {
            pool: pool.clone(),
        }),
        pool.clone(),
    );
    let job_id = import_service
        .batch_import_from_file(ws_id, user_id, file_path, ImportTargetType::Employee)
        .await
        .map_err(|s| format!("Import failed with status: {s}"))?;

    // Wait for job completion
    let mut job_finished = false;
    for _ in 0..20 {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        if let Ok(Some(j)) =
            apiserver::repositories::import_job_repository::get_job_by_id(&pool, job_id).await
        {
            if j.status == ImportJobStatus::Completed {
                job_finished = true;
                break;
            }
            if j.status == ImportJobStatus::Failed {
                return Err(format!("Job failed: {:?}", j.error_log).into());
            }
        }
    }
    assert!(job_finished, "Job timed out");

    let normalized_id = apiserver::utils::normalize_employee_id(emp_id_number);

    // 3. Verify employee was created
    let employee = employee::get_employee_by_id_number(&pool, &normalized_id, ws_id)
        .await?
        .ok_or("Employee should be created")?;

    // 4. Verify staged log was claimed
    let pending_count_after =
        staging_log_repository::count_pending_staging_logs(&pool, ws_id).await?;
    assert_eq!(pending_count_after, 0, "Staged log should be processed");

    // 5. Verify raw log exists for this employee
    let raw_logs = raw_attendance_log_repository::get_paginated_raw_logs(
        &pool,
        ws_id,
        Some(employee.id),
        None,
        None,
        10,
        0,
    )
    .await?;
    assert_eq!(
        raw_logs.len(),
        1,
        "One raw log should exist for the employee"
    );
    assert_eq!(raw_logs[0].punch_time.timestamp(), punch_time.timestamp());
    Ok(())
}

#[tokio::test]
async fn test_import_existing_employee_claims_staged_logs() -> Result<(), Box<dyn std::error::Error>>
{
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool().clone();
    let factory = TestFactory::new(pool.clone());

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;

    let emp_id_number = "CLAIM-002";

    // Create employee first
    let employee_id = factory
        .employee()
        .with_workspace(ws_id)
        .with_employee_id_number(emp_id_number)
        .create()
        .await;

    let normalized_id = apiserver::utils::normalize_employee_id(emp_id_number);
    let punch_time = Utc::now();
    let fingerprint = format!("fp-{}", Uuid::new_v4());

    // 1. Create a staged log for this existing employee
    staging_log_repository::create_staging_log(
        &pool,
        ws_id,
        Some("device-1".to_string()),
        &normalized_id,
        punch_time,
        None,
        &fingerprint,
    )
    .await?;

    // Verify it's pending
    let pending_count = staging_log_repository::count_pending_staging_logs(&pool, ws_id).await?;
    assert_eq!(pending_count, 1);

    // 2. Import employee with same ID (will be skipped for creation but should trigger claim)
    let csv_content = format!("employee_code,full_name,hire_date,department_name\n{emp_id_number},Existing User,2026-01-01,IT");

    let file_path = format!("storage/test_imports/{}.csv", Uuid::new_v4());
    tokio::fs::create_dir_all("storage/test_imports").await?;
    tokio::fs::write(&file_path, csv_content).await?;

    let te_repo = std::sync::Arc::new(apiserver::repositories::SqlxTimeEntryRepository::new(
        pool.clone(),
    ));
    let import_repo: std::sync::Arc<
        dyn apiserver::repositories::import_job_repository::ImportJobRepository,
    > = std::sync::Arc::new(
        apiserver::repositories::import_job_repository::SqlxImportJobRepository::new(pool.clone()),
    );
    let adj_repo: std::sync::Arc<dyn apiserver::repositories::attendance_adjustment_repository::AttendanceAdjustmentRepository> = std::sync::Arc::new(apiserver::repositories::attendance_adjustment_repository::SqlxAttendanceAdjustmentRepository::new(pool.clone()));
    let emp_repo: std::sync::Arc<dyn apiserver::repositories::EmployeeRepository> =
        std::sync::Arc::new(
            apiserver::repositories::diesel_employee_repository::DieselEmployeeRepository::new(
                pool.clone(),
                apiserver::diesel_pool::DieselPools::for_test(&test_db.url()),
            ),
        );
    let import_service = AttendanceImportService::new(
        import_repo,
        adj_repo,
        emp_repo,
        te_repo,
        std::sync::Arc::new(apiserver::repositories::SqlxWorkspaceRepository {
            pool: pool.clone(),
        }),
        pool.clone(),
    );
    let job_id = import_service
        .batch_import_from_file(ws_id, user_id, file_path, ImportTargetType::Employee)
        .await
        .map_err(|s| format!("Import failed with status: {s}"))?;

    // Wait for job completion
    let mut job_finished = false;
    for _ in 0..20 {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        if let Ok(Some(j)) =
            apiserver::repositories::import_job_repository::get_job_by_id(&pool, job_id).await
        {
            if j.status == ImportJobStatus::Completed {
                job_finished = true;
                break;
            }
            if j.status == ImportJobStatus::Failed {
                return Err(format!("Job failed: {:?}", j.error_log).into());
            }
        }
    }
    assert!(job_finished, "Job timed out");

    // 3. Verify staged log was claimed even though employee already existed
    let pending_count_after =
        staging_log_repository::count_pending_staging_logs(&pool, ws_id).await?;
    assert_eq!(
        pending_count_after, 0,
        "Staged log should be processed for existing employee"
    );

    // 4. Verify raw log exists for this employee
    let raw_logs = raw_attendance_log_repository::get_paginated_raw_logs(
        &pool,
        ws_id,
        Some(employee_id),
        None,
        None,
        10,
        0,
    )
    .await?;
    assert_eq!(
        raw_logs.len(),
        1,
        "One raw log should exist for the employee"
    );
    Ok(())
}
