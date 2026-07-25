use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use crate::common::helpers;
use apiserver::models::{enums::ImportTargetType, ImportJobStatus};
use apiserver::repositories::{employee, import_job_repository};
use apiserver::services::AttendanceImportService;
use chrono::Utc;
use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;

#[tokio::test]
async fn test_import_employee_claims_staged_logs() -> Result<(), Box<dyn std::error::Error>> {
    let test_db = TestDatabase::new().await;
    let pool = test_db.diesel_pools.clone().clone();
    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&test_db.url());
    let factory = TestFactory::new(pool.clone());

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;

    // Seed default employer so that the employee import strategy can associate a default employer with the imported employee
    let employer_id = Uuid::new_v4();
    crate::test_execute!(
        &diesel_pools.api,
        "INSERT INTO employers (id, workspace_id, name, tax_code, legal_address, created_at, updated_at) VALUES ($1, $2, 'Default Test Employer', 'DEFAULT-TAX-1', 'Default Address', NOW(), NOW())",
        diesel::sql_types::Uuid, employer_id,
        diesel::sql_types::Uuid, ws_id
    );

    let emp_id_number = "CLAIM-001";
    let normalized_id = apiserver::utils::normalize_employee_id(emp_id_number);
    let punch_time = Utc::now();
    let fingerprint = format!("fp-{}", Uuid::new_v4());

    // 1. Create a staged log for a non-existent employee
    helpers::create_staging_log(
        &diesel_pools,
        ws_id,
        Some("device-1".to_string()),
        &normalized_id,
        punch_time,
        None,
        &fingerprint,
    )
    .await?;

    // Verify it's pending
    let pending_count = helpers::count_pending_staging_logs(&diesel_pools, ws_id).await?;
    assert_eq!(pending_count, 1);

    // 2. Import employee with this ID
    let file_path = format!("storage/test_imports/{}.xlsx", Uuid::new_v4());
    tokio::fs::create_dir_all("storage/test_imports").await?;
    let mut workbook = rust_xlsxwriter::Workbook::new();
    let sheet = workbook.add_worksheet();
    sheet.write_string(0, 0, "employee_code").unwrap();
    sheet.write_string(0, 1, "full_name").unwrap();
    sheet.write_string(0, 2, "hire_date").unwrap();
    sheet.write_string(0, 3, "department_name").unwrap();
    sheet.write_string(1, 0, emp_id_number).unwrap();
    sheet.write_string(1, 1, "Imported User").unwrap();
    sheet.write_string(1, 2, "2026-01-01").unwrap();
    sheet.write_string(1, 3, "IT").unwrap();
    workbook.save(&file_path).unwrap();

    let te_repo = std::sync::Arc::new(apiserver::repositories::DieselTimeEntryRepository::new(
        diesel_pools.clone(),
    ));
    let import_repo: std::sync::Arc<
        dyn apiserver::repositories::import_job_repository::ImportJobRepository,
    > = std::sync::Arc::new(apiserver::repositories::DieselImportJobRepository::new(
        diesel_pools.clone(),
    ));
    let adj_repo: std::sync::Arc<dyn apiserver::repositories::attendance_adjustment_repository::AttendanceAdjustmentRepository> = std::sync::Arc::new(apiserver::repositories::DieselAttendanceAdjustmentRepository::new(diesel_pools.clone()));
    let emp_repo: std::sync::Arc<dyn apiserver::repositories::EmployeeRepository> =
        std::sync::Arc::new(apiserver::repositories::DieselEmployeeRepository::new(
            diesel_pools.clone(),
        ));
    let import_service = AttendanceImportService::new(
        import_repo.clone(),
        adj_repo,
        emp_repo.clone(),
        te_repo,
        std::sync::Arc::new(apiserver::repositories::DieselWorkspaceRepository::new(
            diesel_pools.clone(),
        )),
        std::sync::Arc::new(apiserver::repositories::DieselDepartmentRepository::new(
            diesel_pools.clone(),
        )),
        std::sync::Arc::new(apiserver::repositories::DieselStagingLogRepository::new(
            diesel_pools.clone(),
        )),
        apiserver::worker::deps::WorkerDeps::new(&diesel_pools.clone()),
        None, // log_svc
    );
    let job_id = import_service
        .batch_import_from_file(ws_id, user_id, file_path, ImportTargetType::Employee)
        .await
        .map_err(|s| format!("Import failed with status: {s}"))?;

    // Wait for job completion
    let mut job_finished = false;
    for _ in 0..20 {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        if let Ok(Some(j)) = import_repo
            .get_job_by_id(
                &apiserver::diesel_pool::TenantAuthToken::system_override(ws_id),
                job_id,
            )
            .await
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
    let employee_opt = emp_repo
        .get_employee_by_id_number(
            &normalized_id,
            &apiserver::diesel_pool::TenantAuthToken::system_override(ws_id),
        )
        .await?;

    let employee = employee_opt.ok_or("Employee should be created")?;

    // 4. Verify staged log was claimed
    let pending_count_after = helpers::count_pending_staging_logs(&diesel_pools, ws_id).await?;
    assert_eq!(pending_count_after, 0, "Staged log should be processed");

    // Get the employment ID for the employee
    let employment_id: Uuid = crate::test_query_scalar_uuid!(
        &diesel_pools.api,
        "SELECT id FROM employments WHERE employee_id = $1 LIMIT 1",
        diesel::sql_types::Uuid,
        employee.id()
    );

    // 5. Verify raw log exists for this employee
    let raw_logs = helpers::get_paginated_raw_logs(
        &diesel_pools,
        ws_id,
        Some(employment_id),
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
    let pool = test_db.diesel_pools.clone().clone();
    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&test_db.url());
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
    helpers::create_staging_log(
        &diesel_pools,
        ws_id,
        Some("device-1".to_string()),
        &normalized_id,
        punch_time,
        None,
        &fingerprint,
    )
    .await?;

    // Verify it's pending
    let pending_count = helpers::count_pending_staging_logs(&diesel_pools, ws_id).await?;
    assert_eq!(pending_count, 1);

    // 2. Import employee with same ID (will be skipped for creation but should trigger claim)
    let file_path = format!("storage/test_imports/{}.xlsx", Uuid::new_v4());
    tokio::fs::create_dir_all("storage/test_imports").await?;
    let mut workbook = rust_xlsxwriter::Workbook::new();
    let sheet = workbook.add_worksheet();
    sheet.write_string(0, 0, "employee_code").unwrap();
    sheet.write_string(0, 1, "full_name").unwrap();
    sheet.write_string(0, 2, "hire_date").unwrap();
    sheet.write_string(0, 3, "department_name").unwrap();
    sheet.write_string(1, 0, emp_id_number).unwrap();
    sheet.write_string(1, 1, "Existing User").unwrap();
    sheet.write_string(1, 2, "2026-01-01").unwrap();
    sheet.write_string(1, 3, "IT").unwrap();
    workbook.save(&file_path).unwrap();

    let te_repo = std::sync::Arc::new(apiserver::repositories::DieselTimeEntryRepository::new(
        diesel_pools.clone(),
    ));
    let import_repo: std::sync::Arc<
        dyn apiserver::repositories::import_job_repository::ImportJobRepository,
    > = std::sync::Arc::new(apiserver::repositories::DieselImportJobRepository::new(
        diesel_pools.clone(),
    ));
    let adj_repo: std::sync::Arc<dyn apiserver::repositories::attendance_adjustment_repository::AttendanceAdjustmentRepository> = std::sync::Arc::new(apiserver::repositories::DieselAttendanceAdjustmentRepository::new(diesel_pools.clone()));
    let emp_repo: std::sync::Arc<dyn apiserver::repositories::EmployeeRepository> =
        std::sync::Arc::new(apiserver::repositories::DieselEmployeeRepository::new(
            diesel_pools.clone(),
        ));
    let import_service = AttendanceImportService::new(
        import_repo.clone(),
        adj_repo,
        emp_repo.clone(),
        te_repo,
        std::sync::Arc::new(apiserver::repositories::DieselWorkspaceRepository::new(
            diesel_pools.clone(),
        )),
        std::sync::Arc::new(apiserver::repositories::DieselDepartmentRepository::new(
            diesel_pools.clone(),
        )),
        std::sync::Arc::new(apiserver::repositories::DieselStagingLogRepository::new(
            diesel_pools.clone(),
        )),
        apiserver::worker::deps::WorkerDeps::new(&diesel_pools.clone()),
        None, // log_svc
    );
    let job_id = import_service
        .batch_import_from_file(ws_id, user_id, file_path, ImportTargetType::Employee)
        .await
        .map_err(|s| format!("Import failed with status: {s}"))?;

    // Wait for job completion
    let mut job_finished = false;
    for _ in 0..20 {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        if let Ok(Some(j)) = import_repo
            .get_job_by_id(
                &apiserver::diesel_pool::TenantAuthToken::system_override(ws_id),
                job_id,
            )
            .await
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
    let pending_count_after = helpers::count_pending_staging_logs(&diesel_pools, ws_id).await?;
    assert_eq!(
        pending_count_after, 0,
        "Staged log should be processed for existing employee"
    );

    // Get the employment covering the punch date (same resolution as claim_staged_logs).
    // After the refactor, claim_staged_logs resolves per punch date rather than
    // picking an arbitrary active employment, so we must query the same way.
    let employment_id: Uuid = crate::test_query_scalar_uuid!(
        &diesel_pools.api,
        "SELECT id FROM employments WHERE employee_id = $1 AND start_date <= $2 AND (end_date IS NULL OR end_date >= $2) LIMIT 1",
        diesel::sql_types::Uuid, employee_id,
        diesel::sql_types::Date, punch_time.date_naive()
    );

    // 4. Verify raw log exists for this employee
    let raw_logs = helpers::get_paginated_raw_logs(
        &diesel_pools,
        ws_id,
        Some(employment_id),
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
