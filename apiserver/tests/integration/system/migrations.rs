use apiserver::diesel_pool::{DbPoolConfig, DieselPools};
use tokio_postgres::NoTls;
use uuid::Uuid;

#[tokio::test]
async fn test_system_migrations_only_create_system_tables() {
    let base_url = std::env::var("DATABASE_URL_FOR_TEST_GENERATION")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .expect("DATABASE_URL_FOR_TEST_GENERATION or DATABASE_URL must be set");

    let mut url_parts: Vec<&str> = base_url.split('/').collect();
    let _ = url_parts.pop().unwrap_or("postgres");
    let base_connection_string = url_parts.join("/");

    let admin_url = format!("{base_connection_string}/test_database");
    let db_name = format!("test_sys_mig_{}", Uuid::new_v4().simple());

    // 1. Connect to admin and create a completely blank database
    let (admin_client, admin_conn) = tokio_postgres::connect(&admin_url, NoTls)
        .await
        .expect("Failed to connect to admin DB");
    tokio::spawn(async move {
        let _ = admin_conn.await;
    });

    let create_sql = format!("CREATE DATABASE \"{db_name}\"");
    admin_client
        .execute(&create_sql, &[])
        .await
        .expect("Failed to create blank database");

    // 2. Run ONLY system migrations
    let db_url = format!("{base_connection_string}/{db_name}");
    let config = DbPoolConfig {
        primary_url: db_url.clone(),
        pooler_url: None,
        reader_url: None,
        api_max_conn: 1,
        api_acquire_timeout_secs: 5,
        worker_max_conn: 1,
        worker_acquire_timeout_secs: 5,
        tenant_max_conn: 1,
    };

    // Use tokio::task::spawn_blocking because run_system_migrations is blocking
    let config_clone = config.clone();
    tokio::task::spawn_blocking(move || {
        DieselPools::run_system_migrations(&config_clone);
    })
    .await
    .unwrap();

    // 3. Connect to the new database and inspect schema
    let (client, conn) = tokio_postgres::connect(&db_url, NoTls)
        .await
        .expect("Failed to connect to test DB");
    tokio::spawn(async move {
        let _ = conn.await;
    });

    let row = client
        .query_one("SELECT EXISTS (SELECT FROM pg_tables WHERE schemaname = 'public' AND tablename = 'workspaces');", &[])
        .await
        .unwrap();
    let has_workspaces: bool = row.get(0);
    assert!(
        has_workspaces,
        "workspaces table should exist because it is a system table"
    );

    let row = client
        .query_one("SELECT EXISTS (SELECT FROM pg_tables WHERE schemaname = 'public' AND tablename = 'raw_attendance_logs');", &[])
        .await
        .unwrap();
    let has_attendance_logs: bool = row.get(0);
    assert!(
        !has_attendance_logs,
        "raw_attendance_logs table should NOT exist yet, because tenant migrations were not run!"
    );

    // 4. Now run ALL migrations for test (which simulates test fallback or TenantProvisioner)
    tokio::task::spawn_blocking(move || {
        DieselPools::run_all_migrations_for_test(&config);
    })
    .await
    .unwrap();

    let row = client
        .query_one("SELECT EXISTS (SELECT FROM pg_tables WHERE schemaname = 'public' AND tablename = 'raw_attendance_logs');", &[])
        .await
        .unwrap();
    let has_attendance_logs_now: bool = row.get(0);
    assert!(
        has_attendance_logs_now,
        "raw_attendance_logs table should NOW exist because we ran tenant migrations!"
    );

    // 5. Cleanup
    drop(client);

    // Kill existing connections and drop DB
    let kill_sql = format!(
        "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = '{db_name}' AND pid <> pg_backend_pid()"
    );
    let _ = admin_client.execute(&kill_sql, &[]).await;
    let drop_sql = format!("DROP DATABASE IF EXISTS \"{db_name}\"");
    let _ = admin_client.execute(&drop_sql, &[]).await;
}
