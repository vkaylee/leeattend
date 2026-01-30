use apiserver::agent_manager::AgentManager;
use apiserver::create_app;
use apiserver::models::{BatchCalculatePayrollRequest, BatchCalculatePayrollResponse};
use apiserver::services::AttendanceLogService;
use apiserver::AppState;
use std::sync::Arc;

mod common;
use common::db::TestDatabase;
use common::factory::TestFactory;

async fn spawn_app() -> (String, TestDatabase) {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();

    let agent_manager = AgentManager::new();
    let agent_repo = Arc::new(apiserver::repositories::SqlxAgentRepository { pool: pool.clone() });
    let workspace_repo =
        Arc::new(apiserver::repositories::SqlxWorkspaceRepository { pool: pool.clone() });
    let agent_service = Arc::new(apiserver::services::AgentService::new(
        agent_repo.clone(),
        agent_manager.clone(),
    ));

    let state = Arc::new(AppState {
        agent_manager,
        db: pool.clone(),
        agent_repo,
        workspace_repo,
        agent_service,
        jwt_secret: "test_secret_123".to_string(),
        attendance_service: Arc::new(AttendanceLogService::new(pool.clone(), 100, 10, 1000).0),
        redis: None,
        coordinator: None,
    });

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let app = create_app(state);

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    (format!("http://127.0.0.1:{}", port), test_db)
}

#[tokio::test]
async fn test_batch_calculate_payroll() {
    let (address, test_db) = spawn_app().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());
    let client = reqwest::Client::new();

    // 1. Setup Workspace & Admin
    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let auth_token = factory.auth().create_token(user_id);

    // 2. Create 2 employees
    let emp1_id = factory.employee()
        .with_workspace(ws_id)
        .with_full_name("Emp1")
        .with_employee_id_number("E001")
        .create().await;
        
    let emp2_id = factory.employee()
        .with_workspace(ws_id)
        .with_full_name("Emp2")
        .with_employee_id_number("E002")
        .create().await;

    // 3. Setup base salary
    let sc_def_id: uuid::Uuid = sqlx::query_scalar(
        "INSERT INTO salary_component_definitions (workspace_id, name, code) VALUES ($1, 'Base', 'BASE') RETURNING id"
    ).bind(ws_id).fetch_one(&pool).await.unwrap();

    sqlx::query("INSERT INTO employee_salary_components (employee_id, component_definition_id, amount, effective_date) VALUES ($1, $2, $3, $4)")
        .bind(emp1_id).bind(sc_def_id).bind(rust_decimal::Decimal::from(10_000_000)).bind(chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap())
        .execute(&pool).await.unwrap();
        
    sqlx::query("INSERT INTO employee_salary_components (employee_id, component_definition_id, amount, effective_date) VALUES ($1, $2, $3, $4)")
        .bind(emp2_id).bind(sc_def_id).bind(rust_decimal::Decimal::from(12_000_000)).bind(chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap())
        .execute(&pool).await.unwrap();

    // 4. Test Batch Calculation
    let req_body = BatchCalculatePayrollRequest {
        employee_ids: vec![emp1_id, emp2_id],
        year: 2026,
        month: 1,
    };

    let res = client
        .post(format!("{}/api/workspaces/{}/payroll/calculate-batch", address, ws_id))
        .header("Authorization", format!("Bearer {}", auth_token))
        .json(&req_body)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    
    let batch_res: BatchCalculatePayrollResponse = res.json().await.unwrap();
    assert_eq!(batch_res.results.len(), 2);
    
    for result in batch_res.results {
        assert_eq!(result.status, "success");
        assert!(result.payslip.is_some());
    }
}
