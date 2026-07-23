use super::*;
use crate::diesel_pool::DieselPools;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

#[tokio::test]
async fn test_repo_initialization() {
    let test_db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://user:pass@localhost:5432/leeattend_test_db".to_string());

    let pools = DieselPools::for_test(&test_db_url);
    let _repo = DieselPayrollRepository::new(pools);
    // DI connection test passes if it compiles
}

#[tokio::test]
async fn test_create_salary_component_definition() {
    let test_db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://user:pass@localhost:5432/leeattend_test_db".to_string());

    let pools = DieselPools::for_test(&test_db_url);
    let repo = DieselPayrollRepository::new(pools.clone());
    let ws_id = Uuid::new_v4();
    let token = crate::diesel_pool::TenantAuthToken::system_override(ws_id);

    let mut conn = if let Ok(c) = pools.tenant_transaction_conn(&token).await {
        c
    } else {
        println!("Database not available, skipping integration assertions");
        return;
    };

    // Insert Workspace to satisfy FK constraints
    let workspace_entity = crate::models::diesel::core::WorkspaceEntity {
        id: ws_id,
        name: "Test Workspace".to_string(),
        created_at: Some(chrono::Utc::now()),
        date_format: "YYYY-MM-DD".to_string(),
        time_format: "HH:mm".to_string(),
        timezone: "UTC".to_string(),
        theme_primary_color: None,
        default_language: "en".to_string(),
        logo_url: None,
        tagline: None,
        custom_db_url_encrypted: None,
    };
    diesel::insert_into(crate::schema::workspaces::table)
        .values(&workspace_entity)
        .execute(&mut *conn)
        .await
        .expect("insert workspace");

    // Insert a base group
    let group_id1 = Uuid::new_v4();
    let group_id2 = Uuid::new_v4();

    diesel::sql_query("INSERT INTO salary_base_groups (id, workspace_id, name, slug, description, is_system, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)")
        .bind::<diesel::sql_types::Uuid, _>(group_id1)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Uuid>, _>(Some(ws_id))
        .bind::<diesel::sql_types::Text, _>("Test Group 1")
        .bind::<diesel::sql_types::Text, _>("test-group-1")
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(None::<String>)
        .bind::<diesel::sql_types::Bool, _>(false)
        .bind::<diesel::sql_types::Timestamptz, _>(chrono::Utc::now())
        .bind::<diesel::sql_types::Timestamptz, _>(chrono::Utc::now())
        .execute(&mut *conn)
        .await
        .expect("insert group 1");

    diesel::sql_query("INSERT INTO salary_base_groups (id, workspace_id, name, slug, description, is_system, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)")
        .bind::<diesel::sql_types::Uuid, _>(group_id2)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Uuid>, _>(Some(ws_id))
        .bind::<diesel::sql_types::Text, _>("Test Group 2")
        .bind::<diesel::sql_types::Text, _>("test-group-2")
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(None::<String>)
        .bind::<diesel::sql_types::Bool, _>(false)
        .bind::<diesel::sql_types::Timestamptz, _>(chrono::Utc::now())
        .bind::<diesel::sql_types::Timestamptz, _>(chrono::Utc::now())
        .execute(&mut *conn)
        .await
        .expect("insert group 2");

    drop(conn);

    // 1. Success Path: create definition with valid groups
    let res = repo
        .create_salary_component_definition(
            ws_id,
            "Base Salary".to_string(),
            "BASE".to_string(),
            Some(vec![group_id1, group_id2]),
        )
        .await;
    assert!(res.is_ok());
    let def = res.unwrap();
    assert_eq!(def.name, "Base Salary");
    assert_eq!(def.code, "BASE");

    // Verify members are created in DB
    let mut conn = pools.tenant_transaction_conn(&token).await.unwrap();
    let count: i64 = crate::schema::salary_component_base_members::table
        .filter(crate::schema::salary_component_base_members::component_definition_id.eq(def.id))
        .count()
        .get_result(&mut *conn)
        .await
        .unwrap();
    assert_eq!(count, 2);

    // 2. Failure Path: invalid base group (Forbidden error)
    let bad_group = Uuid::new_v4();
    let res_err = repo
        .create_salary_component_definition(
            ws_id,
            "Allowance".to_string(),
            "ALLOW".to_string(),
            Some(vec![bad_group]),
        )
        .await;
    assert!(res_err.is_err());
    assert!(matches!(res_err.unwrap_err(), AppError::Forbidden(_)));
}
