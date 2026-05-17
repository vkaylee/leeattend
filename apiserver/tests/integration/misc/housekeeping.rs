#![allow(missing_docs)]

use apiserver::queue::{HousekeepingPayload, Job, JobPayload, JobPriority, TaskType};
use apiserver::worker::handlers::housekeeping::NightlyHousekeepingHandler;
use chrono::NaiveDate;

use uuid::Uuid;

#[tokio::test]
async fn test_housekeeping_execution() {
    let test_db = crate::common::db::TestDatabase::new().await;
    let pool = test_db.diesel_pools.clone();

    let job = Job::with_priority(
        Uuid::new_v4(),
        0,
        TaskType::NightlyHousekeeping,
        Uuid::new_v4(),
        JobPayload::NightlyHousekeeping(HousekeepingPayload {
            reference_date: NaiveDate::from_ymd_opt(2026, 5, 15).unwrap(),
        }),
        JobPriority::Low,
    );

    let diesel = test_db.diesel_pools.clone();
    let deps = apiserver::worker::WorkerDeps::new(&diesel);
    if let Err(e) = NightlyHousekeepingHandler::handle(&deps, &job).await {
        panic!("Handler failed: {}", e);
    }

    // Verify partitions are created for target date and next 3 months (May, Jun, Jul, Aug)
    let expected_months = [5, 6, 7, 8];
    for month in expected_months {
        let audit_table = format!("audit_logs_y2026m{:02}", month);
        let exists: bool = crate::test_query_scalar_bool!(
            &pool.api,
            "SELECT EXISTS(SELECT 1 FROM pg_class WHERE relname = $1) as val",
            diesel::sql_types::Text,
            &audit_table
        );

        assert!(exists, "Partition {} should exist", audit_table);
    }
}
