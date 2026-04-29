//! Attendance Import integration tests

use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::models::{AttendanceImportJob, ImportJobStatus};
use apiserver::repositories::import_job_repository;
use apiserver::services::attendance_import_service::AttendanceImportService;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_import_csv_with_line_errors() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool().clone();
    let factory = TestFactory::new(pool.clone());

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;

    let emp_id_number = "ERR-001";
    factory
        .employee()
        .with_workspace(ws_id)
        .with_employee_id_number(emp_id_number)
        .create()
        .await;

    let csv_content = format!(
        "empNo,empName,empDt\n{emp_id_number},User 1,14/01/2026 08:00\n{emp_id_number},User 1,not-a-date\n{emp_id_number},User 1,"
    );

    let file_path = format!("storage/test_imports/{}.csv", uuid::Uuid::new_v4());
    tokio::fs::create_dir_all("storage/test_imports")
        .await
        .unwrap();
    tokio::fs::write(&file_path, csv_content).await.unwrap();

    let te_repo = std::sync::Arc::new(apiserver::repositories::SqlxTimeEntryRepository::new(
        pool.clone(),
    ));
    let import_repo: std::sync::Arc<
        dyn apiserver::repositories::import_job_repository::ImportJobRepository,
    > = std::sync::Arc::new(
        apiserver::repositories::import_job_repository::SqlxImportJobRepository::new(pool.clone()),
    );
    let adj_repo: std::sync::Arc<dyn apiserver::repositories::attendance_adjustment_repository::AttendanceAdjustmentRepository> = std::sync::Arc::new(apiserver::repositories::attendance_adjustment_repository::SqlxAttendanceAdjustmentRepository::new(pool.clone()));
    let employee_repo: std::sync::Arc<dyn apiserver::repositories::EmployeeRepository> =
        std::sync::Arc::new(apiserver::repositories::SqlxEmployeeRepository::new(
            pool.clone(),
        ));
    let job_id = AttendanceImportService::new(
        import_repo,
        adj_repo,
        employee_repo,
        te_repo,
        pool.clone(),
        None,
    )
    .batch_import_from_file(ws_id, user_id, file_path, "log".to_string())
    .await
    .unwrap();

    let mut job: Option<AttendanceImportJob> = None;
    for _ in 0..20 {
        sleep(Duration::from_millis(200)).await;
        if let Ok(Some(j)) = import_job_repository::get_job_by_id(&pool, job_id).await {
            if j.status == ImportJobStatus::Completed || j.status == ImportJobStatus::Failed {
                job = Some(j);
                break;
            }
        }
    }

    let job = job.expect("Job timed out");
    let errors = job.error_log.expect("No error log found");

    assert!(
        errors.contains("Missing employee ID or timestamp")
            || errors.contains("Thiếu mã nhân viên hoặc thời gian")
            || errors.contains("Raw:")
            || errors.contains("not-a-date"),
        "Should identify recognition error, got: {errors}"
    );
}
