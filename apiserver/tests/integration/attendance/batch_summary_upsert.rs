//! Integration tests for `batch_upsert_calculated_summaries` — verifies that the
//! batch repository method persists multiple daily summaries in a single transaction.

use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::models::OtStatus;
use apiserver::repositories::attendance_daily_summary_repository::{
    AttendanceDailySummaryRepository, BatchSummaryInput,
};
use apiserver::repositories::DieselAttendanceDailySummaryRepository;
use chrono::NaiveDate;
use uuid::Uuid;

fn make_input(date: NaiveDate, actual_minutes: i32) -> BatchSummaryInput {
    BatchSummaryInput {
        date,
        actual_work_minutes: actual_minutes,
        system_detected_ot_minutes: 0,
        standard_minutes: actual_minutes,
        early_ot_minutes: 0,
        late_ot_minutes: 0,
        break_ot_minutes: 0,
        is_weekend: false,
        ot_status: OtStatus::None,
        is_holiday: false,
        holiday_name: None,
    }
}

struct TestCtx {
    _test_db: TestDatabase,
    repo: DieselAttendanceDailySummaryRepository,
    employment_id: Uuid,
    auth: apiserver::diesel_pool::TenantAuthToken,
}

async fn setup() -> TestCtx {
    let test_db = TestDatabase::new().await;
    let pool = test_db.diesel_pools.clone();
    let factory = TestFactory::new(pool.clone());
    let repo = DieselAttendanceDailySummaryRepository::new(pool);

    let workspace_id = factory.workspace().create().await;
    let employee_id = factory
        .employee()
        .with_workspace(workspace_id)
        .create()
        .await;
    let employment_id: Uuid = crate::test_query_scalar_uuid!(
        &factory.diesel_pools.api,
        "SELECT id FROM employments WHERE employee_id = $1 LIMIT 1",
        diesel::sql_types::Uuid,
        employee_id
    );

    let auth = apiserver::diesel_pool::TenantAuthToken::system_override(workspace_id);
    TestCtx {
        _test_db: test_db,
        repo,
        employment_id,
        auth,
    }
}

#[tokio::test]
async fn test_batch_upsert_empty_input_succeeds() {
    let ctx = setup().await;

    let result = ctx
        .repo
        .batch_upsert_calculated_summaries(&ctx.auth, ctx.employment_id, &[])
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_batch_upsert_persists_multiple_days() {
    let ctx = setup().await;

    let date1 = NaiveDate::from_ymd_opt(2026, 7, 1).unwrap();
    let date2 = NaiveDate::from_ymd_opt(2026, 7, 2).unwrap();
    let date3 = NaiveDate::from_ymd_opt(2026, 7, 3).unwrap();

    let inputs = vec![
        make_input(date1, 480),
        make_input(date2, 420),
        make_input(date3, 360),
    ];

    ctx.repo
        .batch_upsert_calculated_summaries(&ctx.auth, ctx.employment_id, &inputs)
        .await
        .unwrap();

    let summaries = ctx
        .repo
        .get_summaries(&ctx.auth, Some(ctx.employment_id), date1, date3)
        .await
        .unwrap();

    assert_eq!(summaries.len(), 3);
    assert_eq!(
        summaries.iter().find(|s| s.date == date1).unwrap().actual_work_minutes,
        480
    );
    assert_eq!(
        summaries.iter().find(|s| s.date == date2).unwrap().actual_work_minutes,
        420
    );
    assert_eq!(
        summaries.iter().find(|s| s.date == date3).unwrap().actual_work_minutes,
        360
    );
}

#[tokio::test]
async fn test_batch_upsert_overwrites_existing() {
    let ctx = setup().await;
    let date = NaiveDate::from_ymd_opt(2026, 7, 15).unwrap();

    // First upsert: 480 minutes
    ctx.repo
        .batch_upsert_calculated_summaries(&ctx.auth, ctx.employment_id, &[make_input(date, 480)])
        .await
        .unwrap();

    // Second upsert: 300 minutes (should overwrite)
    ctx.repo
        .batch_upsert_calculated_summaries(&ctx.auth, ctx.employment_id, &[make_input(date, 300)])
        .await
        .unwrap();

    let summaries = ctx
        .repo
        .get_summaries(&ctx.auth, Some(ctx.employment_id), date, date)
        .await
        .unwrap();

    assert_eq!(summaries.len(), 1);
    assert_eq!(summaries[0].actual_work_minutes, 300);
}
