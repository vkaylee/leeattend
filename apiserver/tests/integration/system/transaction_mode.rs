use apiserver::errors::AppError;
use apiserver::{crud_sql_query, stateful_session_query};
use diesel::prelude::*;
use diesel_async::{AsyncConnection, RunQueryDsl};
use uuid::Uuid;

use crate::common::app::spawn_app_with_factory;

#[tokio::test]
async fn test_transaction_rollback_on_error() {
    let (_addr, workspace_id, _user_id, factory, _db) = spawn_app_with_factory().await;

    let pool = factory.diesel_pools.clone();
    let token = apiserver::diesel_pool::TenantAuthToken::system_override(workspace_id);
    let mut conn = pool.tenant_transaction_conn(&token).await.unwrap();
    let dept_id = Uuid::new_v4();

    let result: Result<(), AppError> = conn
        .transaction::<_, AppError, _>(|mut t_conn| {
            Box::pin(async move {
                // Insert a dummy department using TransactionConn and crud_sql_query
                crud_sql_query!(&mut t_conn, "INSERT INTO departments (id, workspace_id, name, created_at, updated_at) VALUES ($1, $2, 'Temp Dept Rollback', NOW(), NOW())")
                    .bind::<diesel::sql_types::Uuid, _>(dept_id)
                    .bind::<diesel::sql_types::Uuid, _>(workspace_id)
                    .execute(&mut t_conn)
                    .await
                    .map_err(|e| AppError::Internal(e.to_string()))?;

                // Intentionally return an error to force a rollback
                Err(AppError::Internal("Intentional error to trigger rollback".into()))
            })
        })
        .await;

    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("Intentional error to trigger rollback"));

    // Verify the record was actually rolled back by querying it
    let mut session_conn = pool.tenant_session_conn(&token).await.unwrap();

    // Note: We use stateful_session_query! here with SessionConn
    #[derive(QueryableByName, Debug)]
    struct CountResult {
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        count: i64,
    }

    let rows: Vec<CountResult> = stateful_session_query!(
        &mut session_conn,
        "SELECT COUNT(*) as count FROM departments WHERE name = 'Temp Dept Rollback'"
    )
    .load(&mut session_conn)
    .await
    .expect("Query failed");

    assert_eq!(
        rows[0].count, 0,
        "Transaction should have rolled back the insert"
    );
}

#[tokio::test]
async fn test_transaction_commit_on_success() {
    let (_addr, workspace_id, _user_id, factory, _db) = spawn_app_with_factory().await;

    let pool = factory.diesel_pools.clone();
    let token = apiserver::diesel_pool::TenantAuthToken::system_override(workspace_id);
    let mut conn = pool.tenant_transaction_conn(&token).await.unwrap();
    let dept_id = Uuid::new_v4();

    let result: Result<(), AppError> = conn
        .transaction::<_, AppError, _>(|mut t_conn| {
            Box::pin(async move {
                crud_sql_query!(&mut t_conn, "INSERT INTO departments (id, workspace_id, name, created_at, updated_at) VALUES ($1, $2, 'Temp Dept Commit', NOW(), NOW())")
                    .bind::<diesel::sql_types::Uuid, _>(dept_id)
                    .bind::<diesel::sql_types::Uuid, _>(workspace_id)
                    .execute(&mut t_conn)
                    .await
                    .map_err(|e| AppError::Internal(e.to_string()))?;

                Ok(())
            })
        })
        .await;

    assert!(result.is_ok(), "Transaction should commit successfully");

    // Verify the record was committed
    let mut session_conn = pool.tenant_session_conn(&token).await.unwrap();

    #[derive(QueryableByName, Debug)]
    struct CountResult {
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        count: i64,
    }

    let rows: Vec<CountResult> = stateful_session_query!(
        &mut session_conn,
        "SELECT COUNT(*) as count FROM departments WHERE name = 'Temp Dept Commit'"
    )
    .load(&mut session_conn)
    .await
    .expect("Query failed");

    assert_eq!(
        rows[0].count, 1,
        "Transaction should have committed the insert"
    );
}
