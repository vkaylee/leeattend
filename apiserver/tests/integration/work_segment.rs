//! Integration tests for WorkSegment repository — CRUD validation and
//! write-through persistence verification.
//!
//! These tests validate the `DieselWorkSegmentRepository` against a real
//! database, ensuring correct Diesel mappings, filter behavior, and
//! the immutable-segment contract (manual segments are never overwritten).

use crate::common::app::setup_app_tower;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::models::{CreateWorkSegmentRequest, UpdateWorkSegmentRequest, WorkSegmentFilter};
use apiserver::repositories::work_segment_repository::WorkSegmentRepository;
use apiserver::AppState;
use apiserver::diesel_pool::TenantAuthToken;
use chrono::{NaiveDate, NaiveTime};
use std::sync::Arc;
use uuid::Uuid;

// ── helpers ──────────────────────────────────────────────────────────────────

fn make_request(
    employment_id: Uuid,
    date: NaiveDate,
    template_id: Uuid,
    sort_order: i32,
) -> CreateWorkSegmentRequest {
    CreateWorkSegmentRequest {
        employment_id,
        date,
        template_id,
        interval_id: Some(Uuid::new_v4()),
        sort_order,
        scheduled_start: NaiveTime::from_hms_opt(8 + sort_order as u32, 0, 0).unwrap(),
        scheduled_end: NaiveTime::from_hms_opt(17 + sort_order as u32, 0, 0).unwrap(),
        segment_type: "work".to_string(),
        segment_status: "scheduled".to_string(),
        compensation_rate: 1.0,
        gap_compensation_before: "unpaid".to_string(),
        gap_compensation_rate_before: 0.0,
        break_config_override: None,
        ot_config_override: None,
        source: "manual".to_string(),
        source_detail: serde_json::json!({"test": true}),
        notes: None,
    }
}

fn system_token(workspace_id: Uuid) -> TenantAuthToken {
    TenantAuthToken::system_override(workspace_id)
}

// ── Test: CRUD through wired repository ──────────────────────────────────────

#[tokio::test]
async fn test_work_segment_crud_integration() {
    let (_test_db, factory, state, _app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let emp_id = factory.employee().with_workspace(ws_id).with_user(user_id).create().await;

    let auth = system_token(ws_id);
    let repo = state.shifts().work_segment_repo();

    let tmpl_id = Uuid::new_v4();
    let date = NaiveDate::from_ymd_opt(2026, 8, 1).unwrap();

    // 1. Create
    let req = make_request(emp_id, date, tmpl_id, 0);
    let created = repo.create_segment(&auth, &req).await.expect("create");
    assert_eq!(created.employment_id, emp_id);
    assert_eq!(created.date, date);
    assert_eq!(created.template_id, tmpl_id);

    // 2. Get by id
    let fetched = repo
        .get_segment_by_id(&auth, created.id)
        .await
        .expect("get")
        .expect("exists");
    assert_eq!(fetched.id, created.id);

    // 3. List — should include our segment
    let filter = WorkSegmentFilter {
        employment_ids: Some(vec![emp_id]),
        start_date: Some(date),
        end_date: Some(date),
        ..Default::default()
    };
    let (results, total) = repo.list_segments(&auth, &filter, 1, 10).await.expect("list");
    assert!(total >= 1);
    assert!(results.iter().any(|s| s.id == created.id));

    // 4. Update
    let update = UpdateWorkSegmentRequest {
        template_id: None,
        interval_id: None,
        sort_order: Some(42),
        scheduled_start: None,
        scheduled_end: None,
        segment_type: None,
        segment_status: None,
        compensation_rate: None,
        gap_compensation_before: None,
        gap_compensation_rate_before: None,
        break_config_override: None,
        ot_config_override: None,
        source: None,
        source_detail: None,
        notes: Some("integration test update".to_string()),
    };
    let updated = repo
        .update_segment(&auth, created.id, &update)
        .await
        .expect("update");
    assert_eq!(updated.sort_order, 42);

    // 5. Delete
    repo.delete_segment(&auth, created.id).await.expect("delete");
    let after = repo
        .get_segment_by_id(&auth, created.id)
        .await
        .expect("get after delete");
    assert!(after.is_none());
}

// ── Test: Manual segments are NOT overwritten by write-through ───────────────

#[tokio::test]
async fn test_manual_segment_not_overwritten() {
    let (_test_db, factory, state, _app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let emp_id = factory.employee().with_workspace(ws_id).with_user(user_id).create().await;

    let auth = system_token(ws_id);
    let repo = state.shifts().work_segment_repo();

    let tmpl_id = Uuid::new_v4();
    let interval_id = Uuid::new_v4();
    let date = NaiveDate::from_ymd_opt(2026, 8, 2).unwrap();

    // Manually create a segment (source="manual")
    let mut manual_req = make_request(emp_id, date, tmpl_id, 0);
    manual_req.interval_id = Some(interval_id);
    let manual = repo
        .create_segment(&auth, &manual_req)
        .await
        .expect("create manual");

    // Simulate what write-through does: try inserting same (employment_id, date, interval_id)
    // Since ON CONFLICT DO NOTHING is used, the existing manual segment should be preserved.
    let same_reqs = vec![
        CreateWorkSegmentRequest {
            interval_id: Some(interval_id), // SAME interval_id triggers unique index
            ..make_request(emp_id, date, tmpl_id, 99)
        },
    ];

    let _result = repo
        .bulk_create_segments(&auth, &same_reqs, false)
        .await
        .expect("bulk create without replace");

    // The original manual segment should still exist and be unchanged
    let refetched = repo
        .get_segment_by_id(&auth, manual.id)
        .await
        .expect("get")
        .expect("manual segment must still exist");

    // Original values must be preserved (ON CONFLICT DO NOTHING prevents overwrite)
    assert_eq!(refetched.sort_order, 0, "manual sort_order must be preserved");
    assert_eq!(refetched.source, "manual", "manual source must be preserved");

    // Cleanup
    let _ = repo.delete_segment(&auth, manual.id).await;
}

// ── Test: bulk_create with replace_existing clears old segments ──────────────

#[tokio::test]
async fn test_bulk_create_replace_deletes_old() {
    let (_test_db, factory, state, _app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let emp_id = factory.employee().with_workspace(ws_id).with_user(user_id).create().await;

    let auth = system_token(ws_id);
    let repo = state.shifts().work_segment_repo();

    let tmpl_id = Uuid::new_v4();
    let date = NaiveDate::from_ymd_opt(2026, 8, 3).unwrap();

    // Insert 3 segments manually
    let initial: Vec<CreateWorkSegmentRequest> = (0..3)
        .map(|i| make_request(emp_id, date, tmpl_id, i))
        .collect();
    let created = repo
        .bulk_create_segments(&auth, &initial, false)
        .await
        .expect("initial bulk create");
    assert_eq!(created.len(), 3);

    // Replace with 2 different segments
    let replacements: Vec<CreateWorkSegmentRequest> = (0..2)
        .map(|i| make_request(emp_id, date, tmpl_id, i * 10))
        .collect();
    let replaced = repo
        .bulk_create_segments(&auth, &replacements, true)
        .await
        .expect("replace bulk create");

    assert_eq!(replaced.len(), 2, "should have exactly 2 segments after replace");

    // Old segments should be deleted
    for old in &created {
        let check = repo
            .get_segment_by_id(&auth, old.id)
            .await
            .expect("get");
        assert!(
            check.is_none(),
            "old segment {} should be deleted after replace_existing=true",
            old.id
        );
    }

    // New segments should exist
    for new in &replaced {
        assert!(new.sort_order % 10 == 0, "new segment should have new sort_order");
        let _ = repo.delete_segment(&auth, new.id).await;
    }
}

// ── Test: filter by segment_type and date range ──────────────────────────────

#[tokio::test]
async fn test_list_segments_filter_by_type() {
    let (_test_db, factory, state, _app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let emp_id = factory.employee().with_workspace(ws_id).with_user(user_id).create().await;

    let auth = system_token(ws_id);
    let repo = state.shifts().work_segment_repo();

    let tmpl_id = Uuid::new_v4();
    let date = NaiveDate::from_ymd_opt(2026, 8, 4).unwrap();

    // Create a work segment
    let mut work_req = make_request(emp_id, date, tmpl_id, 0);
    work_req.segment_type = "work".to_string();
    let work_seg = repo.create_segment(&auth, &work_req).await.expect("create work");

    // Create a standby segment
    let mut stdby_req = make_request(emp_id, date, tmpl_id, 1);
    stdby_req.segment_type = "standby".to_string();
    let standby_seg = repo
        .create_segment(&auth, &stdby_req)
        .await
        .expect("create standby");

    // Filter by segment_type = "work"
    let filter = WorkSegmentFilter {
        segment_type: Some("work".to_string()),
        ..Default::default()
    };
    let (results, _) = repo.list_segments(&auth, &filter, 1, 10).await.expect("list");
    // All results should be work type (note: may include segments from other tests)
    for seg in &results {
        if seg.employment_id == emp_id && seg.date == date {
            assert_eq!(seg.segment_type, "work");
        }
    }

    // Filter by segment_status
    let filter = WorkSegmentFilter {
        segment_status: Some("scheduled".to_string()),
        employment_ids: Some(vec![emp_id]),
        start_date: Some(date),
        end_date: Some(date),
        ..Default::default()
    };
    let (results, _) = repo.list_segments(&auth, &filter, 1, 10).await.expect("list status");
    for seg in &results {
        assert_eq!(seg.segment_status, "scheduled");
    }

    // Cleanup
    let _ = repo.delete_segment(&auth, work_seg.id).await;
    let _ = repo.delete_segment(&auth, standby_seg.id).await;
}

// ── Test: get_segments_by_date_range returns correct subset ──────────────────

#[tokio::test]
async fn test_get_segments_by_date_range() {
    let (_test_db, factory, state, _app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let emp_id = factory.employee().with_workspace(ws_id).with_user(user_id).create().await;

    let auth = system_token(ws_id);
    let repo = state.shifts().work_segment_repo();

    let tmpl_id = Uuid::new_v4();
    let july1 = NaiveDate::from_ymd_opt(2026, 7, 1).unwrap();
    let july15 = NaiveDate::from_ymd_opt(2026, 7, 15).unwrap();
    let aug1 = NaiveDate::from_ymd_opt(2026, 8, 1).unwrap();

    // Create segments on 3 different dates
    let seg_july1 = repo
        .create_segment(&auth, &make_request(emp_id, july1, tmpl_id, 0))
        .await
        .expect("july1");
    let seg_july15 = repo
        .create_segment(&auth, &make_request(emp_id, july15, tmpl_id, 1))
        .await
        .expect("july15");
    let seg_aug1 = repo
        .create_segment(&auth, &make_request(emp_id, aug1, tmpl_id, 2))
        .await
        .expect("aug1");

    // Query July range
    let results = repo
        .get_segments_by_date_range(&auth, &[emp_id], july1, july15)
        .await
        .expect("date range");

    let ids: Vec<Uuid> = results.iter().map(|s| s.id).collect();
    assert!(ids.contains(&seg_july1.id), "should contain july1");
    assert!(ids.contains(&seg_july15.id), "should contain july15");
    assert!(
        !ids.contains(&seg_aug1.id),
        "should NOT contain aug1 (outside range)"
    );

    // Cleanup
    let _ = repo.delete_segment(&auth, seg_july1.id).await;
    let _ = repo.delete_segment(&auth, seg_july15.id).await;
    let _ = repo.delete_segment(&auth, seg_aug1.id).await;
}

// ── Test: template_id roundtrip (schema drift guard) ─────────────────────────

#[tokio::test]
async fn test_template_id_roundtrip_integration() {
    let (_test_db, factory, state, _app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let emp_id = factory.employee().with_workspace(ws_id).with_user(user_id).create().await;

    let auth = system_token(ws_id);
    let repo = state.shifts().work_segment_repo();

    let tmpl_id = Uuid::new_v4(); // known value
    let date = NaiveDate::from_ymd_opt(2026, 8, 10).unwrap();

    let req = make_request(emp_id, date, tmpl_id, 0);
    let created = repo.create_segment(&auth, &req).await.expect("create");

    assert_eq!(
        created.template_id, tmpl_id,
        "template_id must roundtrip; if this fails, check schema drift"
    );

    // Also verify through list
    let filter = WorkSegmentFilter {
        employment_ids: Some(vec![emp_id]),
        start_date: Some(date),
        end_date: Some(date),
        ..Default::default()
    };
    let (results, _) = repo.list_segments(&auth, &filter, 1, 10).await.expect("list");
    let found = results.iter().find(|s| s.id == created.id).expect("found");
    assert_eq!(
        found.template_id, tmpl_id,
        "template_id must survive list roundtrip"
    );

    // Cleanup
    let _ = repo.delete_segment(&auth, created.id).await;
}
