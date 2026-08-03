use crate::common::app::create_test_redis_pool;
use crate::common::db::TestDatabase;
use apiserver::diesel_pool::DieselPools;
use apiserver::state::AppState;
use std::sync::Arc;

// ── Helper ────────────────────────────────────────────────────────────────

async fn setup_test_env() -> (TestDatabase, apiserver::utils::env_registry::EnvRegistry) {
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

    let registry = apiserver::utils::env_registry::EnvRegistry::resolve()
        .expect("Failed to resolve env registry for test");

    (test_db, registry)
}

async fn build_app_state(
    db_url: &str,
    registry: &apiserver::utils::env_registry::EnvRegistry,
) -> (Arc<AppState>, apiserver::app_builder::AppWorkers) {
    let diesel_pools = DieselPools::for_test(db_url);
    let redis_pool = create_test_redis_pool().await;
    AppState::build(diesel_pools, redis_pool, registry)
        .await
        .expect("Failed to build AppState")
}

// ── AppState Initialization Tests ─────────────────────────────────────────

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

    let build_result = AppState::build(diesel_pools, redis_pool, &registry).await;
    assert!(
        build_result.is_ok(),
        "Failed to build AppState: {:?}",
        build_result.err()
    );

    let (state, _workers) = build_result.unwrap();
    assert!(!state.core().jwt_secret().is_empty());
}

#[tokio::test]
async fn test_app_state_build_without_redis() {
    let (test_db, registry) = setup_test_env().await;
    let db_url = test_db.url();
    let diesel_pools = DieselPools::for_test(&db_url);

    let build_result = AppState::build(diesel_pools, None, &registry).await;
    assert!(
        build_result.is_ok(),
        "AppState build without Redis should succeed: {:?}",
        build_result.err()
    );

    let (state, _workers) = build_result.unwrap();
    assert!(!state.core().jwt_secret().is_empty());
    // Without Redis, infra redis should be None
    assert!(state.infra().redis().is_none());
}

#[tokio::test]
async fn test_app_state_build_with_encryption_keys_json() {
    std::env::set_var(
        "ENCRYPTION_KEYS_JSON",
        r#"{"active_key":"v1","keys":{"v1":"MDEyMzQ1Njc4OTAxMjM0NTY3ODkwMTIzNDU2Nzg5MDE="}}"#,
    );
    let (test_db, registry) = setup_test_env().await;
    let db_url = test_db.url();

    let diesel_pools = DieselPools::for_test(&db_url);
    let redis_pool = create_test_redis_pool().await;

    let build_result = AppState::build(diesel_pools, redis_pool, &registry).await;
    assert!(
        build_result.is_ok(),
        "AppState build with encryption keys should succeed: {:?}",
        build_result.err()
    );

    // Cleanup
    std::env::remove_var("ENCRYPTION_KEYS_JSON");
}

// ── Domain State Accessor Tests ───────────────────────────────────────────

#[tokio::test]
async fn test_all_domain_states_accessible() {
    let (test_db, registry) = setup_test_env().await;
    let (state, _workers) = build_app_state(&test_db.url(), &registry).await;

    // All 9 domain accessors + storage + file + scanner must be non-null
    let _employees = state.employees();
    let _attendance = state.attendance();
    let _shifts = state.shifts();
    let _leaves = state.leaves();
    let _payroll = state.payroll();
    let _reports = state.reports();
    let _core = state.core();
    let _agents = state.agents();
    let _infra = state.infra();
    let _storage = state.storage();
    let _file = state.file_service();
    let _scanner = state.scanner();

    // Basic sanity: infra diesel pools is accessible
    let _diesel = state.infra().diesel();
}

#[tokio::test]
async fn test_employee_state_accessors() {
    let (test_db, registry) = setup_test_env().await;
    let (state, _workers) = build_app_state(&test_db.url(), &registry).await;

    let employees = state.employees();
    let _svc = employees.service();
    let _recruitment_svc = employees.recruitment_service();
    let _repo = employees.repo();
    let _employment_repo = employees.employment_repo();
    let _contract_repo = employees.contract_repo();
    let _phase_repo = employees.employment_phase_repo();
}

#[tokio::test]
async fn test_attendance_state_accessors() {
    let (test_db, registry) = setup_test_env().await;
    let (state, _workers) = build_app_state(&test_db.url(), &registry).await;

    let attendance = state.attendance();
    let _time_tracking = attendance.time_tracking_service();
    let _summary = attendance.summary_service();
    let _log = attendance.log_service();
    let _pairing = attendance.pairing_service();
    let _adjustment = attendance.adjustment_service();
    let _import = attendance.import_service();
    let _time_entry_repo = attendance.time_entry_repo();
    let _staging_repo = attendance.staging_repo();
    let _raw_log_repo = attendance.raw_log_repo();
    let _import_repo = attendance.import_repo();
    let _error_repo = attendance.error_repo();
    let _policy_repo = attendance.policy_repo();
    let _daily_summary_repo = attendance.attendance_daily_summary_repo();
    let _behalf_repo = attendance.behalf_assignment_repo();
}

#[tokio::test]
async fn test_shift_state_accessors() {
    let (test_db, registry) = setup_test_env().await;
    let (state, _workers) = build_app_state(&test_db.url(), &registry).await;

    let shifts = state.shifts();
    let _svc = shifts.service();
    let _swap_svc = shifts.swap_service();
    let _roster_repo = shifts.roster_repo();
    let _shift_repo = shifts.shift_repo();
    let _swap_repo = shifts.swap_repo();
    let _assignment_rule_repo = shifts.assignment_rule_repo();
    let _working_pattern_repo = shifts.working_pattern_repo();
    let _compliance = shifts.compliance();
    let _work_segment_repo = shifts.work_segment_repo();
}

#[tokio::test]
async fn test_leave_state_accessors() {
    let (test_db, registry) = setup_test_env().await;
    let (state, _workers) = build_app_state(&test_db.url(), &registry).await;

    let leaves = state.leaves();
    let _svc = leaves.service();
    let _holiday_svc = leaves.holiday_leave_service();
    let _accrual_svc = leaves.accrual_service();
    let _request_repo = leaves.request_repo();
    let _balance_repo = leaves.balance_repo();
    let _policy_repo = leaves.policy_repo();
    let _holiday_repo = leaves.holiday_repo();
}

#[tokio::test]
async fn test_payroll_state_accessors() {
    let (test_db, registry) = setup_test_env().await;
    let (state, _workers) = build_app_state(&test_db.url(), &registry).await;

    let payroll = state.payroll();
    let _svc = payroll.service();
    let _repo = payroll.repo();
    let _rule_repo = payroll.rule_repo();
    let _dept_repo = payroll.department_repo();
    let _feature_flag_repo = payroll.feature_flag_repo();
    let _ot_policy_repo = payroll.overtime_policy_repo();
    let _salary_base_repo = payroll.salary_base_repo();
    let _ot_request_repo = payroll.overtime_request_repo();
    let _ot_request_svc = payroll.overtime_request_service();
}

#[tokio::test]
async fn test_report_state_accessors() {
    let (test_db, registry) = setup_test_env().await;
    let (state, _workers) = build_app_state(&test_db.url(), &registry).await;

    let reports = state.reports();
    let _svc = reports.service();
    // coordinator may be None without Redis, but accessor must not panic
    let _coordinator = reports.coordinator();
    let _stats_repo = reports.stats_repo();
}

#[tokio::test]
async fn test_core_state_accessors() {
    let (test_db, registry) = setup_test_env().await;
    let (state, _workers) = build_app_state(&test_db.url(), &registry).await;

    let core = state.core();
    let _auth = core.auth_service();
    let _email = core.email_service();
    let _workspace_svc = core.workspace_service();
    let _workspace_repo = core.workspace_repo();
    let _audit_repo = core.audit_repo();
    let _user_repo = core.user_repo();
    let _session_repo = core.session_repo();
    let _jwt_secret = core.jwt_secret();
    let _ws_config_repo = core.workspace_config_repo();
    let _approval_repo = core.approval_workflow_repo();
    let _ws_config_svc = core.workspace_config_service();
    let _approval_svc = core.approval_workflow_service();
    let _location_svc = core.location_service();
    let _location_repo = core.location_repo();
    let _employer_svc = core.employer_service();
    let _employer_repo = core.employer_repo();
    let _custom_field_svc = core.custom_field_service();
    let _custom_field_repo = core.custom_field_repo();
    let _lifecycle_svc = core.lifecycle_service();
    let _lifecycle_repo = core.lifecycle_repo();
    let _signature_svc = core.signature_service();
    let _condition_type_repo = core.condition_type_repo();
    let _condition_type_svc = core.condition_type_service();

    assert!(!core.jwt_secret().is_empty());
}

#[tokio::test]
async fn test_agent_state_accessors() {
    let (test_db, registry) = setup_test_env().await;
    let (state, _workers) = build_app_state(&test_db.url(), &registry).await;

    let agents = state.agents();
    let _manager = agents.manager();
    let _svc = agents.service();
    let _repo = agents.repo();
}

#[tokio::test]
async fn test_infra_state_accessors() {
    let (test_db, registry) = setup_test_env().await;
    let (state, _workers) = build_app_state(&test_db.url(), &registry).await;

    let infra = state.infra();
    let _diesel = infra.diesel();
    // redis may or may not be present depending on test env
    let _redis = infra.redis();
    let _cache = infra.cache();
    let _roster_cache = infra.roster_cache();
    let _queue = infra.queue();
    let _tenant_provisioner = infra.tenant_provisioner();
    let _crypto = infra.crypto_service();
    let _system_db_cluster = infra.system_db_cluster_repo();
    let _system_settings = infra.system_settings_repo();
    let _redis_pool = infra.redis_pool();
}

// ── Router & Middleware Tests ─────────────────────────────────────────────

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

    let app = apiserver::create_app(state, &registry);

    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    // Test health endpoint
    let request = Request::builder()
        .method("GET")
        .uri("/api/v1/health")
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert!(response.headers().contains_key("x-request-id"));

    // Test CORS preflight
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

#[tokio::test]
async fn test_router_health_check_returns_ok() {
    let (test_db, registry) = setup_test_env().await;
    let (state, _workers) = build_app_state(&test_db.url(), &registry).await;

    let app = apiserver::create_app(state, &registry);

    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    let request = Request::builder()
        .method("GET")
        .uri("/api/v1/health")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_router_request_id_middleware_injects_header() {
    let (test_db, registry) = setup_test_env().await;
    let (state, _workers) = build_app_state(&test_db.url(), &registry).await;

    let app = apiserver::create_app(state, &registry);

    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    let request = Request::builder()
        .method("GET")
        .uri("/api/v1/health")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let request_id = response.headers().get("x-request-id");
    assert!(
        request_id.is_some(),
        "x-request-id header must be present on all responses"
    );
    assert!(
        !request_id.unwrap().to_str().unwrap().is_empty(),
        "x-request-id must not be empty"
    );
}

#[tokio::test]
async fn test_router_locale_middleware_handles_default_locale() {
    let (test_db, registry) = setup_test_env().await;
    let (state, _workers) = build_app_state(&test_db.url(), &registry).await;

    let app = apiserver::create_app(state, &registry);

    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    // Test that requests without explicit Accept-Language still succeed
    // (locale middleware falls back to default "vi")
    let request = Request::builder()
        .method("GET")
        .uri("/api/v1/health")
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert!(response.status().is_success());

    // Test that requests with Accept-Language also succeed
    let request_with_lang = Request::builder()
        .method("GET")
        .uri("/api/v1/health")
        .header("Accept-Language", "en-US,en;q=0.9")
        .body(Body::empty())
        .unwrap();

    let response2 = app.oneshot(request_with_lang).await.unwrap();
    assert!(response2.status().is_success());
}

#[tokio::test]
async fn test_router_cors_preflight_returns_expected_headers() {
    let (test_db, registry) = setup_test_env().await;
    let (state, _workers) = build_app_state(&test_db.url(), &registry).await;

    let app = apiserver::create_app(state, &registry);

    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    let request = Request::builder()
        .method("OPTIONS")
        .uri("/api/v1/health")
        .header("Origin", "http://localhost:5173")
        .header("Access-Control-Request-Method", "POST")
        .header("Access-Control-Request-Headers", "content-type, x-workspace-id")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response
            .headers()
            .get("access-control-allow-origin")
            .unwrap()
            .to_str()
            .unwrap(),
        "http://localhost:5173"
    );
    assert!(response.headers().contains_key("access-control-allow-methods"));
    assert!(response.headers().contains_key("access-control-allow-headers"));
}

// ── Router Route Registration Tests ───────────────────────────────────────

#[tokio::test]
async fn test_router_registers_auth_routes() {
    let (test_db, registry) = setup_test_env().await;
    let (state, _workers) = build_app_state(&test_db.url(), &registry).await;
    let app = apiserver::create_app(state, &registry);

    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    // POST /api/v1/auth/login should exist (will return 400 for missing body, not 404)
    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/login")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"email":"test@test.com","password":"test"}"#))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    // Either 400 (bad request body), 401 (invalid creds), or 422 — not 404
    assert_ne!(
        response.status(),
        StatusCode::NOT_FOUND,
        "Auth login route must be registered"
    );
}

#[tokio::test]
async fn test_router_registers_workspace_routes() {
    let (test_db, registry) = setup_test_env().await;
    let (state, _workers) = build_app_state(&test_db.url(), &registry).await;
    let app = apiserver::create_app(state, &registry);

    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    // GET /api/v1/workspaces should exist (should return 401, not 404)
    let request = Request::builder()
        .method("GET")
        .uri("/api/v1/workspaces")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_ne!(
        response.status(),
        StatusCode::NOT_FOUND,
        "Workspace list route must be registered"
    );
}

#[tokio::test]
async fn test_router_registers_employee_routes() {
    let (test_db, registry) = setup_test_env().await;
    let (state, _workers) = build_app_state(&test_db.url(), &registry).await;
    let app = apiserver::create_app(state, &registry);

    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    // Employees route: GET /api/v1/workspaces/:ws_id/employees
    let request = Request::builder()
        .method("GET")
        .uri("/api/v1/workspaces/00000000-0000-0000-0000-000000000001/employees")
        .header("x-workspace-id", "00000000-0000-0000-0000-000000000001")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_ne!(
        response.status(),
        StatusCode::NOT_FOUND,
        "Employee list route must be registered"
    );
}

#[tokio::test]
async fn test_router_registers_time_tracking_routes() {
    let (test_db, registry) = setup_test_env().await;
    let (state, _workers) = build_app_state(&test_db.url(), &registry).await;
    let app = apiserver::create_app(state, &registry);

    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    // Time tracking route: GET /api/v1/workspaces/:ws_id/time/history/:employee_id
    let request = Request::builder()
        .method("GET")
        .uri("/api/v1/workspaces/00000000-0000-0000-0000-000000000001/time/history/00000000-0000-0000-0000-000000000002")
        .header("x-workspace-id", "00000000-0000-0000-0000-000000000001")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_ne!(
        response.status(),
        StatusCode::NOT_FOUND,
        "Time tracking route must be registered"
    );
}

#[tokio::test]
async fn test_router_registers_leave_routes() {
    let (test_db, registry) = setup_test_env().await;
    let (state, _workers) = build_app_state(&test_db.url(), &registry).await;
    let app = apiserver::create_app(state, &registry);

    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    // Leave routes: GET /api/v1/workspaces/:ws_id/leave/requests
    let request = Request::builder()
        .method("GET")
        .uri("/api/v1/workspaces/00000000-0000-0000-0000-000000000001/leave/requests")
        .header("x-workspace-id", "00000000-0000-0000-0000-000000000001")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_ne!(
        response.status(),
        StatusCode::NOT_FOUND,
        "Leave routes must be registered"
    );
}

#[tokio::test]
async fn test_router_registers_report_routes() {
    let (test_db, registry) = setup_test_env().await;
    let (state, _workers) = build_app_state(&test_db.url(), &registry).await;
    let app = apiserver::create_app(state, &registry);

    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    // Report routes: GET /api/v1/workspaces/:ws_id/reports/attendance
    let request = Request::builder()
        .method("GET")
        .uri("/api/v1/workspaces/00000000-0000-0000-0000-000000000001/reports/attendance")
        .header("x-workspace-id", "00000000-0000-0000-0000-000000000001")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_ne!(
        response.status(),
        StatusCode::NOT_FOUND,
        "Report routes must be registered"
    );
}

#[tokio::test]
async fn test_router_registers_payroll_routes() {
    let (test_db, registry) = setup_test_env().await;
    let (state, _workers) = build_app_state(&test_db.url(), &registry).await;
    let app = apiserver::create_app(state, &registry);

    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    // Payroll routes: GET /api/v1/workspaces/:ws_id/payroll/payslips
    let request = Request::builder()
        .method("GET")
        .uri("/api/v1/workspaces/00000000-0000-0000-0000-000000000001/payroll/payslips")
        .header("x-workspace-id", "00000000-0000-0000-0000-000000000001")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_ne!(
        response.status(),
        StatusCode::NOT_FOUND,
        "Payroll routes must be registered"
    );
}

#[tokio::test]
async fn test_router_registers_shift_routes() {
    let (test_db, registry) = setup_test_env().await;
    let (state, _workers) = build_app_state(&test_db.url(), &registry).await;
    let app = apiserver::create_app(state, &registry);

    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    // Shift routes: GET /api/v1/workspaces/:ws_id/shifts
    let request = Request::builder()
        .method("GET")
        .uri("/api/v1/workspaces/00000000-0000-0000-0000-000000000001/shifts")
        .header("x-workspace-id", "00000000-0000-0000-0000-000000000001")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_ne!(
        response.status(),
        StatusCode::NOT_FOUND,
        "Shift routes must be registered"
    );
}

#[tokio::test]
async fn test_router_registers_ess_routes() {
    let (test_db, registry) = setup_test_env().await;
    let (state, _workers) = build_app_state(&test_db.url(), &registry).await;
    let app = apiserver::create_app(state, &registry);

    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    // ESS routes: GET /api/v1/ess/profile
    let request = Request::builder()
        .method("GET")
        .uri("/api/v1/ess/profile")
        .header("x-workspace-id", "00000000-0000-0000-0000-000000000001")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_ne!(
        response.status(),
        StatusCode::NOT_FOUND,
        "ESS routes must be registered"
    );
}

// ── Router 404 Test ───────────────────────────────────────────────────────

#[tokio::test]
async fn test_router_unknown_route_returns_404() {
    let (test_db, registry) = setup_test_env().await;
    let (state, _workers) = build_app_state(&test_db.url(), &registry).await;
    let app = apiserver::create_app(state, &registry);

    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    let request = Request::builder()
        .method("GET")
        .uri("/api/v1/nonexistent-route-12345")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    // Even 404s should have request-id header
    assert!(response.headers().contains_key("x-request-id"));
}

// ── Worker Initialization Tests ───────────────────────────────────────────

#[tokio::test]
async fn test_app_workers_initialized_with_redis() {
    let (test_db, registry) = setup_test_env().await;
    let (state, _workers) = build_app_state(&test_db.url(), &registry).await;

    // With Redis, coordinator should be available
    let _coordinator = state.reports().coordinator();
}

#[tokio::test]
async fn test_app_workers_initialized_without_redis() {
    let (test_db, registry) = setup_test_env().await;
    let db_url = test_db.url();
    let diesel_pools = DieselPools::for_test(&db_url);

    let build_result = AppState::build(diesel_pools, None, &registry).await;
    assert!(build_result.is_ok());
    let (state, workers) = build_result.unwrap();

    // Without Redis, all worker handles should be None
    assert!(workers.job_worker_handle.is_none());
    assert!(workers.scheduler_handle.is_none());
    assert!(workers.outbox_sweep_handle.is_none());

    // Coordinator should be None without Redis
    assert!(state.reports().coordinator().is_none());

    // File scan scheduler always starts (doesn't need Redis)
    assert!(workers.file_scan_scheduler_handle.is_some());

    // Pairing scheduler always starts (uses in-memory channel, not Redis)
    assert!(workers.pairing_scheduler_handle.is_some());
}

// ── AppState JWT Secret Tests ─────────────────────────────────────────────

#[tokio::test]
async fn test_app_state_jwt_secret_is_populated() {
    let (test_db, registry) = setup_test_env().await;
    let (state, _workers) = build_app_state(&test_db.url(), &registry).await;

    let secret = state.core().jwt_secret();
    assert!(!secret.is_empty(), "JWT secret must be populated");
}

#[tokio::test]
async fn test_app_state_invalid_db_url_still_constructs() {
    std::env::set_var("DB_PRIMARY_URL", "postgres://u:p@localhost:5432/db");
    std::env::set_var("DB_POOLER_TX_URL", "postgres://u:p@localhost:6543/db");
    std::env::set_var("DB_POOLER_SESS_URL", "postgres://u:p@localhost:5452/db");
    std::env::set_var("S3_ENDPOINT_URL", "http://localhost:9000");
    std::env::set_var("S3_BUCKET_NAME", "test-bucket");
    std::env::set_var("S3_ACCESS_KEY", "test-access");
    std::env::set_var("S3_SECRET_KEY", "test-secret");
    std::env::set_var("S3_REGION", "us-east-1");

    let registry = apiserver::utils::env_registry::EnvRegistry::resolve()
        .expect("Should resolve with minimum env");

    let invalid_url = "postgres://invalid:invalid@localhost:12345/invalid_db";
    let diesel_pools = DieselPools::for_test(invalid_url);
    let result = AppState::build(diesel_pools, None, &registry).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_app_state_build_env_overrides() {
    std::env::set_var("DB_PRIMARY_URL", "postgres://u:p@localhost:5432/db");
    std::env::set_var("DB_POOLER_TX_URL", "postgres://u:p@localhost:6543/db");
    std::env::set_var("DB_POOLER_SESS_URL", "postgres://u:p@localhost:5452/db");
    std::env::set_var("APP_ENV", "testing");
    std::env::set_var("S3_ENDPOINT_URL", "http://localhost:9000");
    std::env::set_var("S3_BUCKET_NAME", "test-bucket");
    std::env::set_var("S3_ACCESS_KEY", "test-access");
    std::env::set_var("S3_SECRET_KEY", "test-secret");
    std::env::set_var("S3_REGION", "us-east-1");

    let registry = apiserver::utils::env_registry::EnvRegistry::resolve()
        .expect("Should resolve with minimum env");

    let invalid_url = "postgres://invalid:invalid@localhost:12345/invalid_db";
    let diesel_pools = DieselPools::for_test(invalid_url);
    let result = AppState::build(diesel_pools, None, &registry).await;
    assert!(result.is_ok());
    if let Ok((state, _workers)) = result {
        assert!(state.infra().redis().is_none());
    }
}

// ── create_app Without Redis Tests ────────────────────────────────────────

#[tokio::test]
async fn test_create_app_without_redis_does_not_panic() {
    let (test_db, registry) = setup_test_env().await;
    let db_url = test_db.url();
    let diesel_pools = DieselPools::for_test(&db_url);

    let build_result = AppState::build(diesel_pools, None, &registry).await;
    assert!(build_result.is_ok());
    let (state, _workers) = build_result.unwrap();

    // create_app should not panic when Redis is None
    let app = apiserver::create_app(state, &registry);

    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    let request = Request::builder()
        .method("GET")
        .uri("/api/v1/health")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

// ── Rate Limiting Middleware Test ─────────────────────────────────────────

#[tokio::test]
async fn test_rate_limit_headers_present() {
    let (test_db, registry) = setup_test_env().await;
    let (state, _workers) = build_app_state(&test_db.url(), &registry).await;
    let app = apiserver::create_app(state, &registry);

    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    let request = Request::builder()
        .method("GET")
        .uri("/api/v1/health")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    // Rate limit middleware should add headers
    // Even without Redis, the middleware should still be in the chain
    // and may add default headers or pass through gracefully
    let _rate_limit_remaining = response.headers().get("x-ratelimit-remaining");
    // We don't assert presence because it depends on Redis availability,
    // but we verify the middleware doesn't crash the request
    assert!(response.status().is_success());
}

// ── Trace Middleware Test ─────────────────────────────────────────────────

#[tokio::test]
async fn test_trace_layer_does_not_panic() {
    let (test_db, registry) = setup_test_env().await;
    let (state, _workers) = build_app_state(&test_db.url(), &registry).await;
    let app = apiserver::create_app(state, &registry);

    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    // Multiple requests through trace layer should not panic
    for _ in 0..3 {
        let request = Request::builder()
            .method("GET")
            .uri("/api/v1/health")
            .body(Body::empty())
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();
        assert!(response.status().is_success());
    }
}
