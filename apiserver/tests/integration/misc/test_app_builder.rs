use crate::common::app::create_test_redis_pool;
use crate::common::db::TestDatabase;
use apiserver::diesel_pool::DieselPools;
use apiserver::state::AppState;
use std::sync::Arc;

#[tokio::test]
async fn test_app_state_build_works() {
    std::env::set_var("S3_ENDPOINT_URL", "http://localhost:9000");
    std::env::set_var("S3_BUCKET_NAME", "test-bucket");
    std::env::set_var("S3_ACCESS_KEY", "test-access");
    std::env::set_var("S3_SECRET_KEY", "test-secret");
    std::env::set_var("S3_REGION", "us-east-1");

    let test_db = TestDatabase::new().await;
    let db_url = test_db.url();
    std::env::set_var("DB_PRIMARY_URL", &db_url);
    std::env::set_var("DB_POOLER_TX_URL", &db_url);
    std::env::set_var("DB_POOLER_SESS_URL", &db_url);

    let diesel_pools = DieselPools::for_test(&db_url);
    let redis_pool = create_test_redis_pool().await;

    let registry = apiserver::utils::env_registry::EnvRegistry::resolve()
        .expect("Failed to resolve env registry for test");

    // Call build
    let build_result = AppState::build(diesel_pools, redis_pool, &registry).await;
    assert!(
        build_result.is_ok(),
        "Failed to build AppState: {:?}",
        build_result.err()
    );

    let (state, _workers) = build_result.unwrap();
    // Verify that JWT secret has a fallback or is correctly populated in state
    assert!(!state.core().jwt_secret().is_empty());
}

#[tokio::test]
async fn test_binary_gen_openapi() {
    let binary_path = env!("CARGO_BIN_EXE_apiserver");
    let output = std::process::Command::new(binary_path)
        .arg("gen-openapi")
        .output()
        .expect("Failed to execute binary");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("\"openapi\""));
}

#[tokio::test]
async fn test_create_app_router_config() {
    std::env::set_var("S3_ENDPOINT_URL", "http://localhost:9000");
    std::env::set_var("S3_BUCKET_NAME", "test-bucket");
    std::env::set_var("S3_ACCESS_KEY", "test-access");
    std::env::set_var("S3_SECRET_KEY", "test-secret");
    std::env::set_var("S3_REGION", "us-east-1");

    let test_db = TestDatabase::new().await;
    let db_url = test_db.url();
    std::env::set_var("DB_PRIMARY_URL", &db_url);
    std::env::set_var("DB_POOLER_TX_URL", &db_url);
    std::env::set_var("DB_POOLER_SESS_URL", &db_url);

    let diesel_pools = DieselPools::for_test(&db_url);
    let redis_pool = create_test_redis_pool().await;

    let registry = apiserver::utils::env_registry::EnvRegistry::resolve()
        .expect("Failed to resolve env registry for test");
    let build_result = AppState::build(diesel_pools, redis_pool, &registry).await;
    assert!(build_result.is_ok());
    let (state, _workers) = build_result.unwrap();

    // Create the router
    let app = apiserver::create_app(state, &registry);

    // Verify routing and middleware setup by sending a request using tower::ServiceExt
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    // Test a simple public route: /api/v1/health
    let request = Request::builder()
        .method("GET")
        .uri("/api/v1/health")
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Verify request_id_middleware successfully injected "x-request-id" header into the response
    assert!(response.headers().contains_key("x-request-id"));

    // Test CORS preflight configuration
    let cors_request = Request::builder()
        .method("OPTIONS")
        .uri("/api/v1/health")
        .header("Origin", "http://localhost:3000")
        .header("Access-Control-Request-Method", "GET")
        .header(
            "Access-Control-Request-Headers",
            "content-type, x-workspace-id",
        )
        .body(Body::empty())
        .unwrap();

    let cors_response = app.oneshot(cors_request).await.unwrap();
    assert_eq!(cors_response.status(), StatusCode::OK);

    // Check CORS headers are present
    assert_eq!(
        cors_response
            .headers()
            .get("access-control-allow-origin")
            .unwrap()
            .to_str()
            .unwrap(),
        "http://localhost:3000"
    );
    assert_eq!(
        cors_response
            .headers()
            .get("access-control-allow-credentials")
            .unwrap()
            .to_str()
            .unwrap(),
        "true"
    );
}
