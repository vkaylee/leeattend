//! Employee pagination and search tests

use crate::common::app::setup_app_tower;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::models::{Employee, PaginatedResponse};
use apiserver::AppState;
use axum::http::StatusCode;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

#[tokio::test]
async fn test_employee_pagination_logic() {
    let (test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;
    let user_id: Uuid = factory.user().create().await;
    let ws_id: Uuid = factory.workspace().create_with_owner(user_id).await;
    let pool = test_db.diesel_pools.clone();

    let hq_loc_id = {
        let mut conn = pool.tenants_tx.get_connection(&ws_id).await.unwrap();
        use apiserver::schema::work_locations::dsl::*;
        use diesel::prelude::*;
        use diesel_async::RunQueryDsl;
        work_locations
            .filter(workspace_id.eq(ws_id))
            .filter(is_headquarters.eq(true))
            .select(id)
            .first::<uuid::Uuid>(&mut *conn)
            .await
            .unwrap()
    };

    // Create 25 employees (sorted by name)
    for i in 0..25 {
        let employee_id = Uuid::new_v4();
        let name = format!("User {i:02}");
        let id_num = format!("EMP{i:02}");

        crate::test_execute!(
            &pool.api,
            r"INSERT INTO employees (id, workspace_id, full_name, employee_id_number, is_active, original_hire_date, hire_date, primary_location_id, created_at, updated_at)
             VALUES ($1, $2, $3, $4, true, CURRENT_DATE, CURRENT_DATE, $5, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
            diesel::sql_types::Uuid,
            employee_id,
            diesel::sql_types::Uuid,
            ws_id,
            diesel::sql_types::Text,
            &name,
            diesel::sql_types::Text,
            &id_num,
            diesel::sql_types::Uuid,
            hq_loc_id
        );
    }

    // Request Page 1, Limit 10
    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri(format!("/api/v1/workspaces/{ws_id}/employees?page=1&limit=10"))
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let res: PaginatedResponse<Employee> = serde_json::from_slice(&body).unwrap();
    assert_eq!(res.data.len(), 10);
    assert_eq!(res.total, 25);

    // Request Page 3 (Last page with 5 records)
    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri(format!("/api/v1/workspaces/{ws_id}/employees?page=3&limit=10"))
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let res: PaginatedResponse<Employee> = serde_json::from_slice(&body).unwrap();
    assert_eq!(res.data.len(), 5);
}

#[tokio::test]
async fn test_employee_search_limit() {
    let (test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;
    let user_id: Uuid = factory.user().create().await;
    let ws_id: Uuid = factory.workspace().create_with_owner(user_id).await;
    let pool = test_db.diesel_pools.clone();

    let hq_loc_id = {
        let mut conn = pool.tenants_tx.get_connection(&ws_id).await.unwrap();
        use apiserver::schema::work_locations::dsl::*;
        use diesel::prelude::*;
        use diesel_async::RunQueryDsl;
        work_locations
            .filter(workspace_id.eq(ws_id))
            .filter(is_headquarters.eq(true))
            .select(id)
            .first::<uuid::Uuid>(&mut *conn)
            .await
            .unwrap()
    };

    for i in 0..60 {
        let employee_id = Uuid::new_v4();
        let name = format!("SearchTest User {i:02}");
        let id_num = format!("ST{i:02}");

        crate::test_execute!(
            &pool.api,
            r"INSERT INTO employees (id, workspace_id, full_name, employee_id_number, is_active, original_hire_date, hire_date, primary_location_id, created_at, updated_at)
             VALUES ($1, $2, $3, $4, true, CURRENT_DATE, CURRENT_DATE, $5, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
            diesel::sql_types::Uuid,
            employee_id,
            diesel::sql_types::Uuid,
            ws_id,
            diesel::sql_types::Text,
            &name,
            diesel::sql_types::Text,
            &id_num,
            diesel::sql_types::Uuid,
            hq_loc_id
        );
    }

    let response = app
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri(format!("/api/v1/workspaces/{ws_id}/employees?q=SearchTest"))
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let response_data: PaginatedResponse<Employee> = serde_json::from_slice(&body).unwrap();
    assert_eq!(
        response_data.data.len(),
        50,
        "Search results should be limited to 50 by default"
    );
    assert_eq!(response_data.total, 60);
}
