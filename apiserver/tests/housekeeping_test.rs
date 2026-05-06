#![allow(missing_docs)]

use apiserver::queue::{HousekeepingPayload, Job, JobPayload, JobPriority, TaskType};
use apiserver::worker::handlers::housekeeping::NightlyHousekeepingHandler;
use chrono::NaiveDate;
use sqlx::PgPool;
use uuid::Uuid;

#[sqlx::test]
async fn test_housekeeping_execution(pool: PgPool) {
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

    let result = NightlyHousekeepingHandler::handle(&pool, &job).await;
    assert!(result.is_ok(), "Handler should execute successfully");

    // Verify partitions are created for target date and next 3 months (May, Jun, Jul, Aug)
    let expected_months = [5, 6, 7, 8];
    for month in expected_months {
        let audit_table = format!("audit_logs_y2026m{:02}", month);
        let exists: (bool,) =
            sqlx::query_as("SELECT EXISTS(SELECT 1 FROM pg_class WHERE relname = $1)")
                .bind(&audit_table)
                .fetch_one(&pool)
                .await
                .unwrap();

        assert!(exists.0, "Partition {} should exist", audit_table);
    }
}
