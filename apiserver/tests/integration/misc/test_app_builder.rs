use crate::common::app::create_test_redis_pool;
use crate::common::db::TestDatabase;
use apiserver::diesel_pool::DieselPools;
use apiserver::state::AppState;
use std::sync::Arc;

#[tokio::test]
async fn test_app_state_build_works() {
    let test_db = TestDatabase::new().await;
    let diesel_pools = DieselPools::for_test(&test_db.url());
    let redis_pool = create_test_redis_pool().await;

    // Call build
    let build_result = AppState::build(diesel_pools, redis_pool).await;
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
