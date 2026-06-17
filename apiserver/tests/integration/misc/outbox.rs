use crate::common::app::setup_app_tower;
use apiserver::diesel_pool::TenantAuthToken;
use apiserver::repositories::outbox_job_repository::OutboxJobRepository;
use apiserver::repositories::DieselOutboxJobRepository;
use chrono::{Duration, Utc};
use diesel_async::RunQueryDsl;
use serde_json::json;
use uuid::Uuid;

#[tokio::test]
async fn test_outbox_job_concurrent_claiming() {
    let (test_db, _factory, _state, _app) = setup_app_tower().await;
    let pools = test_db.diesel_pools.clone();
    let repo = DieselOutboxJobRepository::new(pools.clone());

    // 1. Insert a workspace record so get_all_stuck_jobs will process it
    let ws_id = Uuid::new_v4();
    let mut system_conn = pools.api.get().await.unwrap();

    diesel::sql_query(
        format!(
            "INSERT INTO workspaces (id, name, date_format, time_format, timezone, theme_primary_color, default_language) \
             VALUES ('{}', 'Test Workspace', 'YYYY-MM-DD', '24h', 'UTC', '#000000', 'en')",
            ws_id
        )
    )
    .execute(&mut system_conn)
    .await
    .unwrap();

    // 2. Insert 10 pending outbox jobs with updated_at in the past using raw SQL
    // to bypass BEFORE UPDATE triggers that would override updated_at to NOW().
    let auth_token = TenantAuthToken::system_override(ws_id);
    let mut job_ids = std::collections::HashSet::new();
    let ten_mins_ago = Utc::now() - Duration::minutes(10);
    {
        let mut tenant_conn = pools.tenant_transaction_conn(&auth_token).await.unwrap();
        for _ in 0..10 {
            let job_id = Uuid::new_v4();
            job_ids.insert(job_id);
            diesel::sql_query(
                "INSERT INTO outbox_jobs (id, workspace_id, job_type, payload, status, created_at, updated_at) \
                 VALUES ($1, $2, $3, $4, $5, $6, $7)"
            )
            .bind::<diesel::sql_types::Uuid, _>(job_id)
            .bind::<diesel::sql_types::Uuid, _>(ws_id)
            .bind::<diesel::sql_types::Text, _>("concurrent_job")
            .bind::<diesel::sql_types::Jsonb, _>(json!({}))
            .bind::<diesel::sql_types::Text, _>("pending")
            .bind::<diesel::sql_types::Timestamptz, _>(ten_mins_ago)
            .bind::<diesel::sql_types::Timestamptz, _>(ten_mins_ago)
            .execute(&mut *tenant_conn)
            .await
            .unwrap();
        }
    }

    // 3. Spawn 3 concurrent tasks to claim the jobs
    let repo_clone = repo.clone();
    let older_than = Utc::now() - Duration::minutes(5); // jobs updated 10 minutes ago will match this

    let t1 = tokio::spawn({
        let repo = repo_clone.clone();
        async move { repo.get_all_stuck_jobs(older_than, 5).await.unwrap() }
    });

    let t2 = tokio::spawn({
        let repo = repo_clone.clone();
        async move { repo.get_all_stuck_jobs(older_than, 5).await.unwrap() }
    });

    let t3 = tokio::spawn({
        let repo = repo_clone;
        async move { repo.get_all_stuck_jobs(older_than, 5).await.unwrap() }
    });

    // 4. Gather results
    let res1 = t1.await.unwrap();
    let res2 = t2.await.unwrap();
    let res3 = t3.await.unwrap();

    let all_claimed = [res1, res2, res3];
    let mut claimed_ids = std::collections::HashSet::new();
    let mut duplicate_found = false;

    for chunk in &all_claimed {
        for job in chunk {
            if job.workspace_id() == ws_id {
                if !claimed_ids.insert(job.id()) {
                    duplicate_found = true;
                }
            }
        }
    }

    // 5. Assertions
    assert_eq!(
        claimed_ids.len(),
        10,
        "Expected exactly 10 jobs to be claimed, but claimed {}",
        claimed_ids.len()
    );
    assert!(
        !duplicate_found,
        "Race condition detected! Duplicate jobs were claimed."
    );

    // 6. Clean up database
    {
        let mut tenant_conn = pools.tenant_transaction_conn(&auth_token).await.unwrap();
        diesel::sql_query(format!(
            "DELETE FROM outbox_jobs WHERE workspace_id = '{}'",
            ws_id
        ))
        .execute(&mut *tenant_conn)
        .await
        .unwrap();
    }
    diesel::sql_query(format!("DELETE FROM workspaces WHERE id = '{}'", ws_id))
        .execute(&mut system_conn)
        .await
        .unwrap();
}
