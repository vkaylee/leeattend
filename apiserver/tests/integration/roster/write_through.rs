use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::models::diesel::work_segment::{
    GapCompensationType, NewWorkSegmentEntity, SegmentStatusType, SegmentType,
};
use apiserver::repositories::diesel_roster_repository::DieselRosterRepository;
use bigdecimal::BigDecimal;
use chrono::{NaiveDate, NaiveTime};
use serde_json::json;
use uuid::Uuid;

async fn setup_db() -> (
    TestDatabase,
    TestFactory,
    apiserver::diesel_pool::DieselPools,
) {
    let test_db = TestDatabase::new().await;
    let pools = test_db.diesel_pools.clone();
    let factory = TestFactory::new(pools.clone());
    (test_db, factory, pools)
}

fn segment_entity(
    workspace_id: Uuid,
    employment_id: Uuid,
    date: NaiveDate,
    template_id: Uuid,
    interval_id: Option<Uuid>,
    source: &str,
) -> NewWorkSegmentEntity {
    NewWorkSegmentEntity {
        id: Uuid::new_v4(),
        workspace_id,
        employment_id,
        date,
        template_id,
        interval_id,
        sort_order: 0,
        scheduled_start: NaiveTime::from_hms_opt(8, 0, 0).expect("valid test time"),
        scheduled_end: NaiveTime::from_hms_opt(17, 0, 0).expect("valid test time"),
        segment_type: SegmentType::Work,
        segment_status: SegmentStatusType::Scheduled,
        compensation_rate: BigDecimal::from(1),
        gap_compensation_before: GapCompensationType::Unpaid,
        gap_compensation_rate_before: BigDecimal::from(0),
        break_config_override: None,
        ot_config_override: Some(json!({})),
        source: source.to_string(),
        source_detail: json!({}),
        notes: None,
        generation_metadata: json!({}),
    }
}

async fn shift_ids(pools: &apiserver::diesel_pool::DieselPools, template_id: Uuid) -> (Uuid, Uuid) {
    let version_id = crate::test_query_scalar_uuid!(
        &pools.api,
        "SELECT current_version_id AS id FROM shift_templates WHERE id = $1",
        diesel::sql_types::Uuid,
        template_id
    );
    let old_interval_id = crate::test_query_scalar_uuid!(
        &pools.api,
        "SELECT id FROM shift_default_intervals WHERE version_id = $1 AND sort_order = 0",
        diesel::sql_types::Uuid,
        version_id
    );
    let new_interval_id = crate::test_query_scalar_uuid!(
        &pools.api,
        "INSERT INTO shift_default_intervals (version_id, sort_order, start_time, end_time, break_duration_minutes, break_is_overtime, early_punch_in_grace_period_minutes, allow_early_ot) VALUES ($1, 1, $2, $3, $4, $5, $6, $7) RETURNING id",
        diesel::sql_types::Uuid,
        version_id,
        diesel::sql_types::Time,
        NaiveTime::from_hms_opt(9, 0, 0).expect("valid test time"),
        diesel::sql_types::Time,
        NaiveTime::from_hms_opt(18, 0, 0).expect("valid test time"),
        diesel::sql_types::Integer,
        0_i32,
        diesel::sql_types::Bool,
        false,
        diesel::sql_types::Integer,
        0_i32,
        diesel::sql_types::Bool,
        false
    );
    (old_interval_id, new_interval_id)
}

async fn install_delete_failure_trigger(
    pools: &apiserver::diesel_pool::DieselPools,
    date: NaiveDate,
) -> (String, String) {
    let suffix = Uuid::new_v4().simple().to_string();
    let function_name = format!("test_wt_delete_failure_{suffix}");
    let trigger_name = format!("test_wt_delete_trigger_{suffix}");
    crate::test_execute!(
        &pools.api,
        format!(
            "CREATE OR REPLACE FUNCTION public.\"{function_name}\"() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN IF OLD.date = DATE '{date}' THEN RAISE EXCEPTION 'test cleanup failure'; END IF; RETURN OLD; END; $$"
        )
    );
    crate::test_execute!(
        &pools.api,
        format!(
            "CREATE TRIGGER \"{trigger_name}\" BEFORE DELETE ON public.work_segments FOR EACH ROW EXECUTE FUNCTION public.\"{function_name}\"()"
        )
    );
    (function_name, trigger_name)
}

async fn remove_delete_failure_trigger(
    pools: &apiserver::diesel_pool::DieselPools,
    function_name: &str,
    trigger_name: &str,
) {
    crate::test_execute!(
        &pools.api,
        format!("DROP TRIGGER IF EXISTS \"{trigger_name}\" ON public.work_segments")
    );
    crate::test_execute!(
        &pools.api,
        format!("DROP FUNCTION IF EXISTS public.\"{function_name}\"()")
    );
}

#[tokio::test]
async fn write_through_success_replaces_stale_rows_and_preserves_manual_rows() {
    let (_test_db, factory, pools) = setup_db().await;
    let workspace_id = factory.workspace().create().await;
    let employee_id = factory
        .employee()
        .with_workspace(workspace_id)
        .create()
        .await;
    let employment_id = crate::test_query_scalar_uuid!(
        &pools.api,
        "SELECT id FROM employments WHERE employee_id = $1 AND workspace_id = $2 LIMIT 1",
        diesel::sql_types::Uuid,
        employee_id,
        diesel::sql_types::Uuid,
        workspace_id
    );
    let template_id = factory.shift().with_workspace(workspace_id).create().await;
    let (old_interval_id, new_interval_id) = shift_ids(&pools, template_id).await;
    let date = NaiveDate::from_ymd_opt(2026, 3, 10).expect("valid test date");

    DieselRosterRepository::persist_workspace_segments(
        pools.clone(),
        workspace_id,
        vec![segment_entity(
            workspace_id,
            employment_id,
            date,
            template_id,
            Some(old_interval_id),
            "roster",
        )],
    )
    .await
    .expect("initial write-through should succeed");
    DieselRosterRepository::persist_workspace_segments(
        pools.clone(),
        workspace_id,
        vec![
            segment_entity(
                workspace_id,
                employment_id,
                date,
                template_id,
                Some(new_interval_id),
                "roster",
            ),
            segment_entity(
                workspace_id,
                employment_id,
                date,
                template_id,
                None,
                "manual",
            ),
        ],
    )
    .await
    .expect("replacement write-through should succeed");

    let old_count = crate::test_query_scalar_i64!(
        &pools.api,
        "SELECT COUNT(*) AS count FROM work_segments WHERE employment_id = $1 AND date = $2 AND interval_id = $3",
        diesel::sql_types::Uuid,
        employment_id,
        diesel::sql_types::Date,
        date,
        diesel::sql_types::Uuid,
        old_interval_id
    );
    let new_count = crate::test_query_scalar_i64!(
        &pools.api,
        "SELECT COUNT(*) AS count FROM work_segments WHERE employment_id = $1 AND date = $2 AND interval_id = $3",
        diesel::sql_types::Uuid,
        employment_id,
        diesel::sql_types::Date,
        date,
        diesel::sql_types::Uuid,
        new_interval_id
    );
    let manual_count = crate::test_query_scalar_i64!(
        &pools.api,
        "SELECT COUNT(*) AS count FROM work_segments WHERE employment_id = $1 AND date = $2 AND source = 'manual'",
        diesel::sql_types::Uuid,
        employment_id,
        diesel::sql_types::Date,
        date
    );
    assert_eq!(old_count, 0);
    assert_eq!(new_count, 1);
    assert_eq!(manual_count, 1);
}

#[tokio::test]
async fn write_through_delete_failure_rolls_back_all_cleanup_deletes() {
    let (_test_db, factory, pools) = setup_db().await;
    let workspace_id = factory.workspace().create().await;
    let employee_id = factory
        .employee()
        .with_workspace(workspace_id)
        .create()
        .await;
    let employment_id = crate::test_query_scalar_uuid!(
        &pools.api,
        "SELECT id FROM employments WHERE employee_id = $1 AND workspace_id = $2 LIMIT 1",
        diesel::sql_types::Uuid,
        employee_id,
        diesel::sql_types::Uuid,
        workspace_id
    );
    let template_id = factory.shift().with_workspace(workspace_id).create().await;
    let (old_interval_id, new_interval_id) = shift_ids(&pools, template_id).await;
    let first_date = NaiveDate::from_ymd_opt(2026, 3, 10).expect("valid test date");
    let second_date = NaiveDate::from_ymd_opt(2026, 3, 11).expect("valid test date");

    let stale_entities = [first_date, second_date]
        .into_iter()
        .map(|date| {
            segment_entity(
                workspace_id,
                employment_id,
                date,
                template_id,
                Some(old_interval_id),
                "roster",
            )
        })
        .collect();
    DieselRosterRepository::persist_workspace_segments(pools.clone(), workspace_id, stale_entities)
        .await
        .expect("initial write-through should succeed");
    let (function_name, trigger_name) = install_delete_failure_trigger(&pools, second_date).await;

    let replacement_entities = [first_date, second_date]
        .into_iter()
        .map(|date| {
            segment_entity(
                workspace_id,
                employment_id,
                date,
                template_id,
                Some(new_interval_id),
                "roster",
            )
        })
        .collect();
    let result = DieselRosterRepository::persist_workspace_segments(
        pools.clone(),
        workspace_id,
        replacement_entities,
    )
    .await;
    remove_delete_failure_trigger(&pools, &function_name, &trigger_name).await;

    assert!(
        result.is_err(),
        "the trigger must fail the second cleanup delete"
    );
    let stale_count = crate::test_query_scalar_i64!(
        &pools.api,
        "SELECT COUNT(*) AS count FROM work_segments WHERE employment_id = $1 AND interval_id = $2",
        diesel::sql_types::Uuid,
        employment_id,
        diesel::sql_types::Uuid,
        old_interval_id
    );
    let replacement_count = crate::test_query_scalar_i64!(
        &pools.api,
        "SELECT COUNT(*) AS count FROM work_segments WHERE employment_id = $1 AND interval_id = $2",
        diesel::sql_types::Uuid,
        employment_id,
        diesel::sql_types::Uuid,
        new_interval_id
    );
    assert_eq!(stale_count, 2, "all cleanup deletes must roll back");
    assert_eq!(
        replacement_count, 0,
        "the failed transaction must not insert rows"
    );
}

#[tokio::test]
async fn write_through_insert_failure_rolls_back_cleanup() {
    let (_test_db, factory, pools) = setup_db().await;
    let workspace_id = factory.workspace().create().await;
    let employee_id = factory
        .employee()
        .with_workspace(workspace_id)
        .create()
        .await;
    let employment_id = crate::test_query_scalar_uuid!(
        &pools.api,
        "SELECT id FROM employments WHERE employee_id = $1 AND workspace_id = $2 LIMIT 1",
        diesel::sql_types::Uuid,
        employee_id,
        diesel::sql_types::Uuid,
        workspace_id
    );
    let template_id = factory.shift().with_workspace(workspace_id).create().await;
    let (old_interval_id, new_interval_id) = shift_ids(&pools, template_id).await;
    let date = NaiveDate::from_ymd_opt(2026, 3, 12).expect("valid test date");

    DieselRosterRepository::persist_workspace_segments(
        pools.clone(),
        workspace_id,
        vec![segment_entity(
            workspace_id,
            employment_id,
            date,
            template_id,
            Some(old_interval_id),
            "roster",
        )],
    )
    .await
    .expect("initial write-through should succeed");

    let result = DieselRosterRepository::persist_workspace_segments(
        pools.clone(),
        workspace_id,
        vec![segment_entity(
            workspace_id,
            employment_id,
            date,
            Uuid::new_v4(),
            Some(new_interval_id),
            "roster",
        )],
    )
    .await;

    assert!(
        result.is_err(),
        "the template foreign key must fail the insert"
    );
    let stale_count = crate::test_query_scalar_i64!(
        &pools.api,
        "SELECT COUNT(*) AS count FROM work_segments WHERE employment_id = $1 AND interval_id = $2",
        diesel::sql_types::Uuid,
        employment_id,
        diesel::sql_types::Uuid,
        old_interval_id
    );
    let replacement_count = crate::test_query_scalar_i64!(
        &pools.api,
        "SELECT COUNT(*) AS count FROM work_segments WHERE employment_id = $1 AND interval_id = $2",
        diesel::sql_types::Uuid,
        employment_id,
        diesel::sql_types::Uuid,
        new_interval_id
    );
    assert_eq!(stale_count, 1, "cleanup must roll back when insert fails");
    assert_eq!(replacement_count, 0);
}
