//! Integration tests for Employee Employments API and Multi-Employer conflict checks.

use crate::common::app::setup_app_tower;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::worker::OutboxSweepScheduler;
use apiserver::AppState;
use axum::http::StatusCode;
use serde_json::json;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

fn make_req(
    method: &str,
    uri: &str,
    ws_id: Uuid,
    user_id: Uuid,
    body: Option<serde_json::Value>,
) -> axum::http::Request<axum::body::Body> {
    let builder = axum::http::Request::builder()
        .method(method)
        .uri(uri)
        .header("X-Workspace-Id", ws_id.to_string())
        .header("X-User-Id", user_id.to_string());

    if let Some(b) = body {
        builder
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(serde_json::to_string(&b).unwrap()))
            .unwrap()
    } else {
        builder.body(axum::body::Body::empty()).unwrap()
    }
}

#[tokio::test]
async fn test_same_employer_overlap_fails_across_employment_types() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    let employer_id = Uuid::new_v4();
    crate::test_execute!(
        &_test_db.diesel_pools.api,
        "INSERT INTO employers (id, workspace_id, name, tax_code, legal_address, created_at, updated_at) VALUES ($1, $2, 'Test Employer', 'MST-1', 'Addr', NOW(), NOW())",
        diesel::sql_types::Uuid,
        employer_id,
        diesel::sql_types::Uuid,
        ws_id
    );

    // 1. Create first primary employment (2026-01-01 to 2026-06-30)
    let payload_1 = json!({
        "employee_id": emp_id.to_string(),
        "employer_id": employer_id.to_string(),
        "status": "active",
        "start_date": "2026-01-01",
        "end_date": "2026-06-30",
        "employment_type": "primary",
        "job_title": "Software Engineer"
    });

    let res1 = app
        .clone()
        .oneshot(make_req(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/employees/{emp_id}/employments"),
            ws_id,
            user_id,
            Some(payload_1),
        ))
        .await
        .unwrap();

    assert_eq!(
        res1.status(),
        StatusCode::CREATED,
        "Should allow creating first primary employment"
    );

    // 2. Try creating overlapping primary employment (2026-06-01 to 2026-12-31)
    let payload_2 = json!({
        "employee_id": emp_id.to_string(),
        "employer_id": employer_id.to_string(),
        "status": "active",
        "start_date": "2026-06-01",
        "end_date": "2026-12-31",
        "employment_type": "contractor",
        "job_title": "Tech Lead"
    });

    let res2 = app
        .clone()
        .oneshot(make_req(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/employees/{emp_id}/employments"),
            ws_id,
            user_id,
            Some(payload_2),
        ))
        .await
        .unwrap();

    assert_eq!(
        res2.status(),
        StatusCode::CONFLICT,
        "Should return conflict (409) for overlapping employment with the same employer"
    );

    // 3. The same dates for a different employer remain valid.
    let second_employer_id = Uuid::new_v4();
    crate::test_execute!(
        &_test_db.diesel_pools.api,
        "INSERT INTO employers (id, workspace_id, name, tax_code, legal_address, created_at, updated_at) VALUES ($1, $2, 'Second Test Employer', 'MST-2', 'Addr', NOW(), NOW())",
        diesel::sql_types::Uuid,
        second_employer_id,
        diesel::sql_types::Uuid,
        ws_id
    );
    let different_employer_payload = json!({
        "employee_id": emp_id.to_string(),
        "employer_id": second_employer_id.to_string(),
        "status": "active",
        "start_date": "2026-06-01",
        "end_date": "2026-12-31",
        "employment_type": "contractor"
    });
    let different_employer_response = app
        .clone()
        .oneshot(make_req(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/employees/{emp_id}/employments"),
            ws_id,
            user_id,
            Some(different_employer_payload),
        ))
        .await
        .unwrap();
    assert_eq!(different_employer_response.status(), StatusCode::CREATED);

    // 4. Create non-overlapping primary employment (2026-07-01 to open-ended)
    let payload_3 = json!({
        "employee_id": emp_id.to_string(),
        "employer_id": employer_id.to_string(),
        "status": "active",
        "start_date": "2026-07-01",
        "end_date": null,
        "employment_type": "primary",
        "job_title": "Staff Engineer"
    });

    let res3 = app
        .clone()
        .oneshot(make_req(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/employees/{emp_id}/employments"),
            ws_id,
            user_id,
            Some(payload_3),
        ))
        .await
        .unwrap();

    assert_eq!(
        res3.status(),
        StatusCode::CREATED,
        "Should allow creating non-overlapping primary employment"
    );

    let body3 = axum::body::to_bytes(res3.into_body(), usize::MAX)
        .await
        .unwrap();
    let created3: serde_json::Value = serde_json::from_slice(&body3).unwrap();
    let emp3_id = created3["id"].as_str().unwrap().to_string();

    // 5. Try updating the non-overlapping employment start_date to overlap (2026-06-15)
    let update_payload = json!({
        "start_date": "2026-06-15"
    });

    let res4 = app
        .clone()
        .oneshot(make_req(
            "PATCH",
            &format!("/api/v1/workspaces/{ws_id}/employees/{emp_id}/employments/{emp3_id}"),
            ws_id,
            user_id,
            Some(update_payload),
        ))
        .await
        .unwrap();

    assert_eq!(
        res4.status(),
        StatusCode::CONFLICT,
        "Should return conflict when patching start_date to cause overlap"
    );
}

#[tokio::test]
async fn test_employment_crud_lifecycle() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    let employer_id = Uuid::new_v4();
    crate::test_execute!(
        &_test_db.diesel_pools.api,
        "INSERT INTO employers (id, workspace_id, name, tax_code, legal_address, created_at, updated_at) VALUES ($1, $2, 'Test Employer', 'MST-1', 'Addr', NOW(), NOW())",
        diesel::sql_types::Uuid,
        employer_id,
        diesel::sql_types::Uuid,
        ws_id
    );

    // 1. POST - Create Employment
    let payload = json!({
        "employee_id": emp_id.to_string(),
        "employer_id": employer_id.to_string(),
        "status": "active",
        "start_date": "2026-01-01",
        "end_date": "2026-12-31",
        "employment_type": "primary",
        "job_title": "Software Engineer"
    });

    let res = app
        .clone()
        .oneshot(make_req(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/employees/{emp_id}/employments"),
            ws_id,
            user_id,
            Some(payload),
        ))
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::CREATED);

    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let created: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let employment_id = created["id"].as_str().unwrap().to_string();

    // 2. GET - List Employments
    let res_list = app
        .clone()
        .oneshot(make_req(
            "GET",
            &format!("/api/v1/workspaces/{ws_id}/employees/{emp_id}/employments"),
            ws_id,
            user_id,
            None,
        ))
        .await
        .unwrap();

    assert_eq!(res_list.status(), StatusCode::OK);

    let body_list = axum::body::to_bytes(res_list.into_body(), usize::MAX)
        .await
        .unwrap();
    let list: serde_json::Value = serde_json::from_slice(&body_list).unwrap();
    assert!(list.is_array());
    assert!(!list.as_array().unwrap().is_empty());

    // 3. GET - Detail Employment
    let res_detail = app
        .clone()
        .oneshot(make_req(
            "GET",
            &format!("/api/v1/workspaces/{ws_id}/employees/{emp_id}/employments/{employment_id}"),
            ws_id,
            user_id,
            None,
        ))
        .await
        .unwrap();

    assert_eq!(res_detail.status(), StatusCode::OK);

    let body_detail = axum::body::to_bytes(res_detail.into_body(), usize::MAX)
        .await
        .unwrap();
    let detail: serde_json::Value = serde_json::from_slice(&body_detail).unwrap();
    assert_eq!(detail["job_title"].as_str().unwrap(), "Software Engineer");

    // 4. GET - Detail NotFound
    let res_not_found = app
        .clone()
        .oneshot(make_req(
            "GET",
            &format!(
                "/api/v1/workspaces/{ws_id}/employees/{emp_id}/employments/{}",
                Uuid::new_v4()
            ),
            ws_id,
            user_id,
            None,
        ))
        .await
        .unwrap();

    assert_eq!(res_not_found.status(), StatusCode::NOT_FOUND);

    // 5. PATCH - Update Employment
    let update_payload = json!({
        "job_title": "Senior Software Engineer"
    });

    let res_update = app
        .clone()
        .oneshot(make_req(
            "PATCH",
            &format!("/api/v1/workspaces/{ws_id}/employees/{emp_id}/employments/{employment_id}"),
            ws_id,
            user_id,
            Some(update_payload),
        ))
        .await
        .unwrap();

    assert_eq!(res_update.status(), StatusCode::OK);

    let body_update = axum::body::to_bytes(res_update.into_body(), usize::MAX)
        .await
        .unwrap();
    let updated: serde_json::Value = serde_json::from_slice(&body_update).unwrap();
    assert_eq!(
        updated["job_title"].as_str().unwrap(),
        "Senior Software Engineer"
    );

    // 6. DELETE - Soft Delete Employment
    let res_delete = app
        .clone()
        .oneshot(make_req(
            "DELETE",
            &format!("/api/v1/workspaces/{ws_id}/employees/{emp_id}/employments/{employment_id}"),
            ws_id,
            user_id,
            None,
        ))
        .await
        .unwrap();

    assert_eq!(res_delete.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_employment_mutations_invalidate_roster_cache() {
    let (test_db, factory, state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;
    apiserver::diesel_pool::DieselPools::run_all_migrations_for_test(
        &apiserver::diesel_pool::DbPoolConfig {
            primary_url: test_db.url(),
            pooler_tx_url: String::new(),
            pooler_sess_url: String::new(),
            tenant_pooler_tx_url: String::new(),
            tenant_pooler_sess_url: String::new(),
            reader_url: None,
            api_max_conn: 1,
            api_acquire_timeout_secs: 5,
            worker_max_conn: 1,
            worker_acquire_timeout_secs: 5,
            tenant_max_conn: 1,
        },
    )
    .expect("test database migrations should be current");
    let trigger_count = crate::test_query_scalar_i64!(
        &test_db.diesel_pools.api,
        "SELECT COUNT(*) FROM pg_trigger WHERE tgname = 'trg_roster_cache_invalidation_employments'");
    if trigger_count == 0 {
        crate::test_batch_execute!(
            &test_db.diesel_pools.api,
            include_str!(
                "../../../migrations/tenant/20260810000001_roster_cache_invalidation_outbox/up.sql"
            )
        );
    }
    let trigger_count = crate::test_query_scalar_i64!(
        &test_db.diesel_pools.api,
        "SELECT COUNT(*) FROM pg_trigger WHERE tgname = 'trg_roster_cache_invalidation_employments'");
    assert!(
        trigger_count > 0,
        "roster invalidation migration did not install employment trigger"
    );
    let Some(cache) = state.infra().roster_cache() else {
        eprintln!("SKIPPED employment roster invalidation test: REDIS_URL is required");
        return;
    };
    let coordinator = state
        .reports()
        .coordinator()
        .cloned()
        .expect("Redis-backed test app must configure report coordinator");
    let scheduler = OutboxSweepScheduler::new(
        coordinator.outbox_service().clone(),
        coordinator,
        Some(Arc::clone(cache)),
    );

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;

    // Verify AFTER DELETE interval triggers can resolve the parent workspace
    // from OLD.version_id after the interval row has already been removed.
    let interval_template_id = factory.shift().with_workspace(ws_id).create().await;
    let interval_version_id = crate::test_query_scalar_uuid!(
        &state.infra().diesel().api,
        "SELECT current_version_id AS id FROM shift_templates WHERE id = $1",
        diesel::sql_types::Uuid,
        interval_template_id
    );
    let interval_id = crate::test_query_scalar_uuid!(
        &state.infra().diesel().api,
        "SELECT id FROM shift_default_intervals WHERE version_id = $1 ORDER BY sort_order LIMIT 1",
        diesel::sql_types::Uuid,
        interval_version_id
    );
    let pending_before_interval_delete = crate::test_query_scalar_i64!(
        &state.infra().diesel().api,
        "SELECT COUNT(*) FROM outbox_jobs WHERE workspace_id = $1 AND job_type = 'RosterCacheInvalidation' AND status = 'pending'",
        diesel::sql_types::Uuid,
        ws_id
    );
    crate::test_execute!(
        &state.infra().diesel().api,
        "DELETE FROM shift_default_intervals WHERE id = $1",
        diesel::sql_types::Uuid,
        interval_id
    );
    let pending_after_interval_delete = crate::test_query_scalar_i64!(
        &state.infra().diesel().api,
        "SELECT COUNT(*) FROM outbox_jobs WHERE workspace_id = $1 AND job_type = 'RosterCacheInvalidation' AND status = 'pending'",
        diesel::sql_types::Uuid,
        ws_id
    );
    assert!(
        pending_after_interval_delete > pending_before_interval_delete,
        "deleting a roster interval must enqueue durable invalidation"
    );

    let emp_id = factory.employee().with_workspace(ws_id).create().await;
    let department_a = factory
        .department()
        .with_workspace(ws_id)
        .with_name("Cache Department A")
        .create()
        .await;
    let department_b = factory
        .department()
        .with_workspace(ws_id)
        .with_name("Cache Department B")
        .create()
        .await;
    let employer_id = Uuid::new_v4();
    crate::test_execute!(
        &state.infra().diesel().api,
        "INSERT INTO employers (id, workspace_id, name, tax_code, legal_address, created_at, updated_at) VALUES ($1, $2, 'Cache Employer', 'CACHE-1', 'Addr', NOW(), NOW())",
        diesel::sql_types::Uuid,
        employer_id,
        diesel::sql_types::Uuid,
        ws_id
    );

    let replica_cache = cache.new_replica();
    let before = cache
        .get_roster_data(ws_id, state.infra().diesel())
        .await
        .expect("warming roster cache should succeed");
    let before_replica = replica_cache
        .get_roster_data(ws_id, state.infra().diesel())
        .await
        .expect("warming replica roster cache should succeed");
    assert!(!before.dept_history.contains_key(&emp_id));
    assert!(!before_replica.dept_history.contains_key(&emp_id));

    let pending_before_create = crate::test_query_scalar_i64!(
        &state.infra().diesel().api,
        "SELECT COUNT(*) FROM outbox_jobs WHERE workspace_id = $1 AND job_type = 'RosterCacheInvalidation' AND status = 'pending'",
        diesel::sql_types::Uuid,
        ws_id
    );
    let payload = json!({
        "employee_id": emp_id.to_string(),
        "employer_id": employer_id.to_string(),
        "department_id": department_a.to_string(),
        "status": "active",
        "start_date": "2026-01-01",
        "end_date": "2026-12-31",
        "employment_type": "primary",
        "job_title": "Cache Engineer"
    });
    let created = app
        .clone()
        .oneshot(make_req(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/employees/{emp_id}/employments"),
            ws_id,
            user_id,
            Some(payload),
        ))
        .await
        .expect("create request should complete");
    assert_eq!(created.status(), StatusCode::CREATED);
    let created_body = axum::body::to_bytes(created.into_body(), usize::MAX)
        .await
        .expect("created response body should be readable");
    let created_json: serde_json::Value =
        serde_json::from_slice(&created_body).expect("created response should be JSON");
    let employment_id = created_json["id"]
        .as_str()
        .expect("created employment should have an id")
        .to_string();

    let pending_after_create = crate::test_query_scalar_i64!(
        &state.infra().diesel().api,
        "SELECT COUNT(*) FROM outbox_jobs WHERE workspace_id = $1 AND job_type = 'RosterCacheInvalidation' AND status = 'pending'",
        diesel::sql_types::Uuid,
        ws_id
    );
    assert!(pending_after_create > pending_before_create);
    scheduler
        .sweep_once()
        .await
        .expect("durable roster invalidation sweep should succeed");
    let after_create = cache
        .get_roster_data(ws_id, state.infra().diesel())
        .await
        .expect("created roster snapshot should load");
    let after_create_replica = replica_cache
        .get_roster_data(ws_id, state.infra().diesel())
        .await
        .expect("replica should refresh after create generation bump");
    assert_eq!(
        after_create.dept_history[&emp_id][0].department_id,
        department_a
    );
    assert_eq!(
        after_create_replica.dept_history[&emp_id][0].department_id,
        department_a
    );

    let pending_before_update = crate::test_query_scalar_i64!(
        &state.infra().diesel().api,
        "SELECT COUNT(*) FROM outbox_jobs WHERE workspace_id = $1 AND job_type = 'RosterCacheInvalidation' AND status = 'pending'",
        diesel::sql_types::Uuid,
        ws_id
    );
    let updated = app
        .clone()
        .oneshot(make_req(
            "PATCH",
            &format!("/api/v1/workspaces/{ws_id}/employees/{emp_id}/employments/{employment_id}"),
            ws_id,
            user_id,
            Some(json!({
                "department_id": department_b,
                "job_title": "Senior Cache Engineer"
            })),
        ))
        .await
        .expect("update request should complete");
    assert_eq!(updated.status(), StatusCode::OK);

    let pending_after_update = crate::test_query_scalar_i64!(
        &state.infra().diesel().api,
        "SELECT COUNT(*) FROM outbox_jobs WHERE workspace_id = $1 AND job_type = 'RosterCacheInvalidation' AND status = 'pending'",
        diesel::sql_types::Uuid,
        ws_id
    );
    assert!(pending_after_update > pending_before_update);
    scheduler
        .sweep_once()
        .await
        .expect("durable roster invalidation sweep should succeed after update");
    let after_update = cache
        .get_roster_data(ws_id, state.infra().diesel())
        .await
        .expect("updated roster snapshot should load");
    let after_update_replica = replica_cache
        .get_roster_data(ws_id, state.infra().diesel())
        .await
        .expect("replica should refresh after update generation bump");
    assert_eq!(
        after_update.dept_history[&emp_id][0].department_id,
        department_b
    );
    assert_eq!(
        after_update_replica.dept_history[&emp_id][0].department_id,
        department_b
    );

    let pending_before_delete = crate::test_query_scalar_i64!(
        &state.infra().diesel().api,
        "SELECT COUNT(*) FROM outbox_jobs WHERE workspace_id = $1 AND job_type = 'RosterCacheInvalidation' AND status = 'pending'",
        diesel::sql_types::Uuid,
        ws_id
    );
    let deleted = app
        .oneshot(make_req(
            "DELETE",
            &format!("/api/v1/workspaces/{ws_id}/employees/{emp_id}/employments/{employment_id}"),
            ws_id,
            user_id,
            None,
        ))
        .await
        .expect("delete request should complete");
    assert_eq!(deleted.status(), StatusCode::NO_CONTENT);

    let pending_after_delete = crate::test_query_scalar_i64!(
        &state.infra().diesel().api,
        "SELECT COUNT(*) FROM outbox_jobs WHERE workspace_id = $1 AND job_type = 'RosterCacheInvalidation' AND status = 'pending'",
        diesel::sql_types::Uuid,
        ws_id
    );
    assert!(pending_after_delete > pending_before_delete);
    scheduler
        .sweep_once()
        .await
        .expect("durable roster invalidation sweep should succeed after delete");
    let after_delete = cache
        .get_roster_data(ws_id, state.infra().diesel())
        .await
        .expect("deleted roster snapshot should load");
    let after_delete_replica = replica_cache
        .get_roster_data(ws_id, state.infra().diesel())
        .await
        .expect("replica should refresh after delete generation bump");
    assert!(!after_delete.dept_history.contains_key(&emp_id));
    assert!(!after_delete_replica.dept_history.contains_key(&emp_id));
}
