//! Employee pagination and search tests

use crate::integration::common::app::setup_app_tower;
use crate::integration::common::db::TestDatabase;
use crate::integration::common::factory::TestFactory;
use apiserver::models::{Employee, PaginatedResponse};
use apiserver::AppState;
use axum::http::StatusCode;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

#[tokio::test]
async fn test_employee_pagination_logic() {
    let (test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) = setup_app_tower().await;
    let user_id: Uuid = factory.user().create().await;
    let ws_id: Uuid = factory.workspace().create_with_owner(user_id).await;
    let pool = test_db.pool();

    // Create 25 employees (sorted by name)
    for i in 0..25 {
        let employee_id = Uuid::new_v4();
        let name = format!("User {:02}", i);
        let id_num = format!("EMP{:02}", i);
        
        sqlx::query(
            r#"INSERT INTO employees (id, workspace_id, full_name, employee_id_number, is_active, created_at, updated_at) 
             VALUES ($1, $2, $3, $4, true, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"#
        )
        .bind(employee_id)
        .bind(ws_id)
        .bind(&name)
        .bind(&id_num)
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            r#"INSERT INTO employment_periods (employee_id, hire_date) VALUES ($1, CURRENT_DATE)"#
        )
        .bind(employee_id)
        .execute(&pool)
        .await
        .unwrap();
    }

    // Request Page 1, Limit 10
    let response = app.clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri(format!("/api/workspaces/{}/employees?page=1&limit=10", ws_id))
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let res: PaginatedResponse<Employee> = serde_json::from_slice(&body).unwrap();
    assert_eq!(res.data.len(), 10);
    assert_eq!(res.total, 25);

    // Request Page 3 (Last page with 5 records)
    let response = app.clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri(format!("/api/workspaces/{}/employees?page=3&limit=10", ws_id))
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let res: PaginatedResponse<Employee> = serde_json::from_slice(&body).unwrap();
    assert_eq!(res.data.len(), 5);
}

#[tokio::test]
async fn test_employee_search_limit() {
    let (test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) = setup_app_tower().await;
    let user_id: Uuid = factory.user().create().await;
    let ws_id: Uuid = factory.workspace().create_with_owner(user_id).await;
    let pool = test_db.pool();

    for i in 0..60 {
        let employee_id = Uuid::new_v4();
        let name = format!("SearchTest User {:02}", i);
        let id_num = format!("ST{:02}", i);
        
        sqlx::query(
            r#"INSERT INTO employees (id, workspace_id, full_name, employee_id_number, is_active, created_at, updated_at) 
             VALUES ($1, $2, $3, $4, true, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"#
        )
        .bind(employee_id)
        .bind(ws_id)
        .bind(&name)
        .bind(&id_num)
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            r#"INSERT INTO employment_periods (employee_id, hire_date) VALUES ($1, CURRENT_DATE)"#
        )
        .bind(employee_id)
        .execute(&pool)
        .await
        .unwrap();
    }

    let response = app
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri(format!("/api/workspaces/{}/employees?q=SearchTest", ws_id))
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let response_data: PaginatedResponse<Employee> = serde_json::from_slice(&body).unwrap();
    assert_eq!(response_data.data.len(), 50, "Search results should be limited to 50 by default");
    assert_eq!(response_data.total, 60);
}
