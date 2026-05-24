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
        pooler_tx_url: None,
        pooler_sess_url: None,
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

#[tokio::test]
async fn test_run_tenant_migrations_on_existing_workspace() {
    let base_url = std::env::var("DATABASE_URL_FOR_TEST_GENERATION")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .expect("DATABASE_URL_FOR_TEST_GENERATION or DATABASE_URL must be set");

    let mut url_parts: Vec<&str> = base_url.split('/').collect();
    let _ = url_parts.pop().unwrap_or("postgres");
    let base_connection_string = url_parts.join("/");

    let admin_url = format!("{base_connection_string}/test_database");
    let workspace_id = Uuid::new_v4();
    let sys_db_name = format!("test_sys_ten_{}", Uuid::new_v4().simple());
    let tenant_db_name = format!("tenant_{}", workspace_id.to_string().replace('-', ""));

    // 1. Create system database and tenant database
    let (admin_client, admin_conn) = tokio_postgres::connect(&admin_url, NoTls)
        .await
        .expect("Failed to connect to admin DB");
    tokio::spawn(async move {
        let _ = admin_conn.await;
    });

    admin_client
        .execute(&format!("CREATE DATABASE \"{sys_db_name}\""), &[])
        .await
        .expect("Failed to create system DB");

    admin_client
        .execute(&format!("CREATE DATABASE \"{tenant_db_name}\""), &[])
        .await
        .expect("Failed to create tenant DB");

    // 2. Run system migrations and insert a workspace
    let sys_db_url = format!("{base_connection_string}/{sys_db_name}");
    let tenant_db_url = format!("{base_connection_string}/{tenant_db_name}");

    let config = DbPoolConfig {
        primary_url: sys_db_url.clone(),
        pooler_tx_url: None,
        pooler_sess_url: None,
        reader_url: None,
        api_max_conn: 1,
        api_acquire_timeout_secs: 5,
        worker_max_conn: 1,
        worker_acquire_timeout_secs: 5,
        tenant_max_conn: 1,
    };

    let sys_url_clone = sys_db_url.clone();
    let ws_id = workspace_id;
    tokio::task::spawn_blocking(move || {
        DieselPools::run_system_migrations(&DbPoolConfig {
            primary_url: sys_url_clone.clone(),
            pooler_tx_url: None,
            pooler_sess_url: None,
            reader_url: None,
            api_max_conn: 1,
            api_acquire_timeout_secs: 5,
            worker_max_conn: 1,
            worker_acquire_timeout_secs: 5,
            tenant_max_conn: 1,
        });

        // Insert a workspace so run_tenant_migrations can find it
        use diesel::{Connection, RunQueryDsl};
        let mut conn = diesel::PgConnection::establish(&sys_url_clone).expect("connect");
        diesel::sql_query(format!(
            "INSERT INTO workspaces (id, name) VALUES ('{ws_id}', 'test-ws')"
        ))
        .execute(&mut conn)
        .expect("insert workspace");
    })
    .await
    .unwrap();

    // 3. Verify tenant DB is empty (no tenant tables yet)
    let (tenant_client, tenant_conn) = tokio_postgres::connect(&tenant_db_url, NoTls)
        .await
        .unwrap();
    tokio::spawn(async move {
        let _ = tenant_conn.await;
    });

    let row = tenant_client
        .query_one(
            "SELECT EXISTS (SELECT FROM pg_tables WHERE schemaname = 'public' AND tablename = 'raw_attendance_logs');",
            &[],
        )
        .await
        .unwrap();
    assert!(
        !row.get::<_, bool>(0),
        "raw_attendance_logs should NOT exist before run_tenant_migrations"
    );
    drop(tenant_client);

    // 4. Run tenant migrations
    let config_clone = config.clone();
    tokio::task::spawn_blocking(move || {
        DieselPools::run_tenant_migrations(&config_clone);
    })
    .await
    .unwrap();

    // 5. Verify tenant tables now exist
    let (tenant_client2, tenant_conn2) = tokio_postgres::connect(&tenant_db_url, NoTls)
        .await
        .unwrap();
    tokio::spawn(async move {
        let _ = tenant_conn2.await;
    });

    let row = tenant_client2
        .query_one(
            "SELECT EXISTS (SELECT FROM pg_tables WHERE schemaname = 'public' AND tablename = 'raw_attendance_logs');",
            &[],
        )
        .await
        .unwrap();
    assert!(
        row.get::<_, bool>(0),
        "raw_attendance_logs should exist after run_tenant_migrations"
    );

    // Also verify a system table does NOT exist in tenant DB (cross-contamination check)
    let row = tenant_client2
        .query_one(
            "SELECT EXISTS (SELECT FROM pg_tables WHERE schemaname = 'public' AND tablename = 'users');",
            &[],
        )
        .await
        .unwrap();
    assert!(
        !row.get::<_, bool>(0),
        "users table should NOT exist in tenant DB — it belongs to system DB"
    );

    drop(tenant_client2);

    // 6. Cleanup both databases
    for db in &[&sys_db_name, &tenant_db_name] {
        let kill = format!(
            "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = '{db}' AND pid <> pg_backend_pid()"
        );
        let _ = admin_client.execute(&kill, &[]).await;
        let _ = admin_client
            .execute(&format!("DROP DATABASE IF EXISTS \"{db}\""), &[])
            .await;
    }
}

#[tokio::test]
async fn test_notifications_table_in_system_db() {
    let base_url = std::env::var("DATABASE_URL_FOR_TEST_GENERATION")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .expect("DATABASE_URL_FOR_TEST_GENERATION or DATABASE_URL must be set");

    let mut url_parts: Vec<&str> = base_url.split('/').collect();
    let _ = url_parts.pop().unwrap_or("postgres");
    let base_connection_string = url_parts.join("/");

    let admin_url = format!("{base_connection_string}/test_database");
    let db_name = format!("test_notif_sys_{}", Uuid::new_v4().simple());

    let (admin_client, admin_conn) = tokio_postgres::connect(&admin_url, NoTls)
        .await
        .expect("Failed to connect to admin DB");
    tokio::spawn(async move {
        let _ = admin_conn.await;
    });

    admin_client
        .execute(&format!("CREATE DATABASE \"{db_name}\""), &[])
        .await
        .expect("Failed to create test DB");

    let db_url = format!("{base_connection_string}/{db_name}");

    // 1. Run system migrations (creates users + notifications tables)
    let url_clone = db_url.clone();
    tokio::task::spawn_blocking(move || {
        DieselPools::run_system_migrations(&DbPoolConfig {
            primary_url: url_clone,
            pooler_tx_url: None,
            pooler_sess_url: None,
            reader_url: None,
            api_max_conn: 1,
            api_acquire_timeout_secs: 5,
            worker_max_conn: 1,
            worker_acquire_timeout_secs: 5,
            tenant_max_conn: 1,
        });
    })
    .await
    .unwrap();

    let (client, conn) = tokio_postgres::connect(&db_url, NoTls).await.unwrap();
    tokio::spawn(async move {
        let _ = conn.await;
    });

    // 2. Verify notifications table exists in system DB
    let row = client
        .query_one(
            "SELECT EXISTS (SELECT FROM pg_tables WHERE schemaname = 'public' AND tablename = 'notifications');",
            &[],
        )
        .await
        .unwrap();
    assert!(
        row.get::<_, bool>(0),
        "notifications table should exist in system DB after system migrations"
    );

    // 3. Verify FK cascade: insert user + notification, delete user → notification cascade-deleted
    let user_id = Uuid::new_v4();
    let notif_id = Uuid::new_v4();

    client
        .execute(
            &format!("INSERT INTO users (id, email, password_hash) VALUES ('{user_id}', 'test@test.com', 'hash')"),
            &[],
        )
        .await
        .unwrap();

    client
        .execute(
            &format!("INSERT INTO notifications (id, user_id, type, title, content) VALUES ('{notif_id}', '{user_id}', 'test', 'Test', 'Body')"),
            &[],
        )
        .await
        .unwrap();

    // Verify notification exists
    let row = client
        .query_one(
            &format!("SELECT COUNT(*) FROM notifications WHERE id = '{notif_id}'"),
            &[],
        )
        .await
        .unwrap();
    let count: i64 = row.get(0);
    assert_eq!(count, 1, "notification should exist");

    // Delete user → notification should be cascade-deleted
    client
        .execute(&format!("DELETE FROM users WHERE id = '{user_id}'"), &[])
        .await
        .unwrap();

    let row = client
        .query_one(
            &format!("SELECT COUNT(*) FROM notifications WHERE id = '{notif_id}'"),
            &[],
        )
        .await
        .unwrap();
    let count: i64 = row.get(0);
    assert_eq!(count, 0, "notification should be cascade-deleted with user");

    drop(client);

    // Cleanup
    let kill = format!(
        "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = '{db_name}' AND pid <> pg_backend_pid()"
    );
    let _ = admin_client.execute(&kill, &[]).await;
    let _ = admin_client
        .execute(&format!("DROP DATABASE IF EXISTS \"{db_name}\""), &[])
        .await;
}

#[tokio::test]
async fn test_authorized_agents_only_in_system_db() {
    let base_url = std::env::var("DATABASE_URL_FOR_TEST_GENERATION")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .expect("DATABASE_URL_FOR_TEST_GENERATION or DATABASE_URL must be set");

    let mut url_parts: Vec<&str> = base_url.split('/').collect();
    let _ = url_parts.pop().unwrap_or("postgres");
    let base_connection_string = url_parts.join("/");

    let admin_url = format!("{base_connection_string}/test_database");
    let sys_db_name = format!("test_agent_sys_{}", Uuid::new_v4().simple());
    let tenant_db_name = format!("test_agent_ten_{}", Uuid::new_v4().simple());

    let (admin_client, admin_conn) = tokio_postgres::connect(&admin_url, NoTls)
        .await
        .expect("Failed to connect to admin DB");
    tokio::spawn(async move {
        let _ = admin_conn.await;
    });

    admin_client
        .execute(&format!("CREATE DATABASE \"{sys_db_name}\""), &[])
        .await
        .expect("Failed to create system DB");
    admin_client
        .execute(&format!("CREATE DATABASE \"{tenant_db_name}\""), &[])
        .await
        .expect("Failed to create tenant DB");

    let sys_db_url = format!("{base_connection_string}/{sys_db_name}");

    // 1. Run system migrations only
    let url_clone = sys_db_url.clone();
    tokio::task::spawn_blocking(move || {
        DieselPools::run_system_migrations(&DbPoolConfig {
            primary_url: url_clone,
            pooler_tx_url: None,
            pooler_sess_url: None,
            reader_url: None,
            api_max_conn: 1,
            api_acquire_timeout_secs: 5,
            worker_max_conn: 1,
            worker_acquire_timeout_secs: 5,
            tenant_max_conn: 1,
        });
    })
    .await
    .unwrap();

    let (sys_client, sys_conn) = tokio_postgres::connect(&sys_db_url, NoTls).await.unwrap();
    tokio::spawn(async move {
        let _ = sys_conn.await;
    });

    // 2. authorized_agents exists in system DB
    let row = sys_client
        .query_one(
            "SELECT EXISTS (SELECT FROM pg_tables WHERE schemaname = 'public' AND tablename = 'authorized_agents');",
            &[],
        )
        .await
        .unwrap();
    assert!(
        row.get::<_, bool>(0),
        "authorized_agents should exist in system DB after system migrations"
    );

    // 3. Insert a workspace and an agent, verify FK works
    let ws_id = Uuid::new_v4();
    sys_client
        .execute(
            &format!("INSERT INTO workspaces (id, name) VALUES ('{ws_id}', 'test-ws')"),
            &[],
        )
        .await
        .unwrap();

    let agent_id = "test-agent-001";
    let token = Uuid::new_v4().to_string();
    sys_client
        .execute(
            &format!(
                "INSERT INTO authorized_agents (id, name, token, workspace_id) VALUES ('{agent_id}', 'Test Agent', '{token}', '{ws_id}')"
            ),
            &[],
        )
        .await
        .unwrap();

    let row = sys_client
        .query_one(
            &format!("SELECT COUNT(*) FROM authorized_agents WHERE workspace_id = '{ws_id}'"),
            &[],
        )
        .await
        .unwrap();
    let count: i64 = row.get(0);
    assert_eq!(count, 1, "agent should be queryable in system DB");

    drop(sys_client);

    // 4. authorized_agents does NOT exist in tenant DB
    let tenant_db_url = format!("{base_connection_string}/{tenant_db_name}");
    let (ten_client, ten_conn) = tokio_postgres::connect(&tenant_db_url, NoTls)
        .await
        .unwrap();
    tokio::spawn(async move {
        let _ = ten_conn.await;
    });

    let row = ten_client
        .query_one(
            "SELECT EXISTS (SELECT FROM pg_tables WHERE schemaname = 'public' AND tablename = 'authorized_agents');",
            &[],
        )
        .await
        .unwrap();
    assert!(
        !row.get::<_, bool>(0),
        "authorized_agents should NOT exist in tenant DB"
    );

    drop(ten_client);

    // Cleanup
    for db in &[&sys_db_name, &tenant_db_name] {
        let kill = format!(
            "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = '{db}' AND pid <> pg_backend_pid()"
        );
        let _ = admin_client.execute(&kill, &[]).await;
        let _ = admin_client
            .execute(&format!("DROP DATABASE IF EXISTS \"{db}\""), &[])
            .await;
    }
}

#[tokio::test]
async fn test_workspace_tables_fk_cascade_in_system_db() {
    let base_url = std::env::var("DATABASE_URL_FOR_TEST_GENERATION")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .expect("DATABASE_URL_FOR_TEST_GENERATION or DATABASE_URL must be set");

    let mut url_parts: Vec<&str> = base_url.split('/').collect();
    let _ = url_parts.pop().unwrap_or("postgres");
    let base_connection_string = url_parts.join("/");

    let admin_url = format!("{base_connection_string}/test_database");
    let db_name = format!("test_ws_fk_{}", Uuid::new_v4().simple());

    let (admin_client, admin_conn) = tokio_postgres::connect(&admin_url, NoTls)
        .await
        .expect("Failed to connect to admin DB");
    tokio::spawn(async move {
        let _ = admin_conn.await;
    });

    admin_client
        .execute(&format!("CREATE DATABASE \"{db_name}\""), &[])
        .await
        .expect("Failed to create test DB");

    let db_url = format!("{base_connection_string}/{db_name}");

    // Run system migrations
    let url_clone = db_url.clone();
    tokio::task::spawn_blocking(move || {
        DieselPools::run_system_migrations(&DbPoolConfig {
            primary_url: url_clone,
            pooler_tx_url: None,
            pooler_sess_url: None,
            reader_url: None,
            api_max_conn: 1,
            api_acquire_timeout_secs: 5,
            worker_max_conn: 1,
            worker_acquire_timeout_secs: 5,
            tenant_max_conn: 1,
        });
    })
    .await
    .unwrap();

    let (client, conn) = tokio_postgres::connect(&db_url, NoTls).await.unwrap();
    tokio::spawn(async move {
        let _ = conn.await;
    });

    // Seed: user + workspace + role
    let user_id = Uuid::new_v4();
    let ws_id = Uuid::new_v4();
    let role_id = Uuid::new_v4();

    client
        .execute(&format!("INSERT INTO users (id, email, password_hash) VALUES ('{user_id}', 'fk@test.com', 'hash')"), &[])
        .await.unwrap();
    client
        .execute(
            &format!("INSERT INTO workspaces (id, name) VALUES ('{ws_id}', 'fk-ws')"),
            &[],
        )
        .await
        .unwrap();
    client
        .execute(&format!("INSERT INTO workspace_roles (id, workspace_id, name) VALUES ('{role_id}', NULL, 'fk-role')"), &[])
        .await.unwrap();

    // --- Test 1: workspace_members FK cascade ---
    client
        .execute(&format!("INSERT INTO workspace_members (workspace_id, user_id, role_id) VALUES ('{ws_id}', '{user_id}', '{role_id}')"), &[])
        .await.unwrap();

    // Delete user → workspace_members cascade
    client
        .execute(&format!("DELETE FROM users WHERE id = '{user_id}'"), &[])
        .await
        .unwrap();
    let row = client
        .query_one(
            &format!("SELECT COUNT(*) FROM workspace_members WHERE workspace_id = '{ws_id}'"),
            &[],
        )
        .await
        .unwrap();
    let count: i64 = row.get(0);
    assert_eq!(
        count, 0,
        "workspace_members should be cascade-deleted with user"
    );

    // Re-seed user + member for next test
    let user2 = Uuid::new_v4();
    client.execute(&format!("INSERT INTO users (id, email, password_hash) VALUES ('{user2}', 'fk2@test.com', 'hash')"), &[]).await.unwrap();
    client.execute(&format!("INSERT INTO workspace_members (workspace_id, user_id, role_id) VALUES ('{ws_id}', '{user2}', '{role_id}')"), &[]).await.unwrap();

    // Delete workspace → workspace_members cascade
    client
        .execute(&format!("DELETE FROM workspaces WHERE id = '{ws_id}'"), &[])
        .await
        .unwrap();
    let count: i64 = client
        .query_one("SELECT COUNT(*) FROM workspace_members", &[])
        .await
        .unwrap()
        .get(0);
    assert_eq!(
        count, 0,
        "workspace_members should be cascade-deleted with workspace"
    );

    // --- Test 2: workspace_role_permissions FK cascade ---
    let role2 = Uuid::new_v4();
    client.execute(&format!("INSERT INTO workspace_roles (id, workspace_id, name) VALUES ('{role2}', NULL, 'fk-role2')"), &[]).await.unwrap();
    client.execute(&format!("INSERT INTO workspace_role_permissions (role_id, permission_name) VALUES ('{role2}', 'test:perm')"), &[]).await.unwrap();

    // Delete role → permissions cascade
    client
        .execute(
            &format!("DELETE FROM workspace_roles WHERE id = '{role2}'"),
            &[],
        )
        .await
        .unwrap();
    let count: i64 = client
        .query_one(
            &format!("SELECT COUNT(*) FROM workspace_role_permissions WHERE role_id = '{role2}'"),
            &[],
        )
        .await
        .unwrap()
        .get(0);
    assert_eq!(
        count, 0,
        "workspace_role_permissions should be cascade-deleted with role"
    );

    // --- Test 3: member_extra_roles FK cascades ---
    let ws2 = Uuid::new_v4();
    let role3 = Uuid::new_v4();
    let extra_id = Uuid::new_v4();
    client
        .execute(
            &format!("INSERT INTO workspaces (id, name) VALUES ('{ws2}', 'fk-ws2')"),
            &[],
        )
        .await
        .unwrap();
    client.execute(&format!("INSERT INTO workspace_roles (id, workspace_id, name) VALUES ('{role3}', NULL, 'fk-role3')"), &[]).await.unwrap();
    client.execute(&format!("INSERT INTO member_extra_roles (id, workspace_id, user_id, role_id, expires_at) VALUES ('{extra_id}', '{ws2}', '{user2}', '{role3}', NOW() + INTERVAL '30 days')"), &[]).await.unwrap();

    // Delete user → extra_roles cascade
    client
        .execute(&format!("DELETE FROM users WHERE id = '{user2}'"), &[])
        .await
        .unwrap();
    let count: i64 = client
        .query_one(
            &format!("SELECT COUNT(*) FROM member_extra_roles WHERE id = '{extra_id}'"),
            &[],
        )
        .await
        .unwrap()
        .get(0);
    assert_eq!(
        count, 0,
        "member_extra_roles should be cascade-deleted with user"
    );

    drop(client);

    // Cleanup
    let kill = format!(
        "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = '{db_name}' AND pid <> pg_backend_pid()"
    );
    let _ = admin_client.execute(&kill, &[]).await;
    let _ = admin_client
        .execute(&format!("DROP DATABASE IF EXISTS \"{db_name}\""), &[])
        .await;
}

#[tokio::test]
async fn test_run_tenant_migrations_falls_back_to_primary_url() {
    // Verify run_tenant_migrations works when pooler_url is None
    // (dev/test mode or no Supavisor). It falls back to primary_url.
    let base_url = std::env::var("DATABASE_URL_FOR_TEST_GENERATION")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .expect("DATABASE_URL_FOR_TEST_GENERATION or DATABASE_URL must be set");

    let mut url_parts: Vec<&str> = base_url.split('/').collect();
    let _ = url_parts.pop().unwrap_or("postgres");
    let base_connection_string = url_parts.join("/");

    let admin_url = format!("{base_connection_string}/test_database");
    let workspace_id = Uuid::new_v4();
    let sys_db_name = format!("test_fallback_ten_{}", Uuid::new_v4().simple());
    let tenant_db_name = format!("tenant_{}", workspace_id.to_string().replace('-', ""));

    let (admin_client, admin_conn) = tokio_postgres::connect(&admin_url, NoTls)
        .await
        .expect("Failed to connect to admin DB");
    tokio::spawn(async move {
        let _ = admin_conn.await;
    });

    admin_client
        .execute(&format!("CREATE DATABASE \"{sys_db_name}\""), &[])
        .await
        .expect("Failed to create system DB");
    admin_client
        .execute(&format!("CREATE DATABASE \"{tenant_db_name}\""), &[])
        .await
        .expect("Failed to create tenant DB");

    let sys_db_url = format!("{base_connection_string}/{sys_db_name}");

    // Run system migrations and insert a workspace
    let url_clone = sys_db_url.clone();
    let ws_id = workspace_id;
    tokio::task::spawn_blocking(move || {
        use diesel::{Connection, RunQueryDsl};

        let config = DbPoolConfig {
            primary_url: url_clone.clone(),
            pooler_tx_url: None,
            pooler_sess_url: None,
            reader_url: None,
            api_max_conn: 1,
            api_acquire_timeout_secs: 5,
            worker_max_conn: 1,
            worker_acquire_timeout_secs: 5,
            tenant_max_conn: 1,
        };
        DieselPools::run_system_migrations(&config);

        let mut conn = diesel::PgConnection::establish(&url_clone).expect("connect");
        diesel::sql_query(format!(
            "INSERT INTO workspaces (id, name) VALUES ('{ws_id}', 'test-ws')"
        ))
        .execute(&mut conn)
        .expect("insert workspace");
    })
    .await
    .unwrap();

    // Run tenant migrations with pooler_tx_url: None, pooler_sess_url: None — should fall back to primary_url
    let config_no_pooler = DbPoolConfig {
        primary_url: sys_db_url.clone(),
        pooler_tx_url: None,
        pooler_sess_url: None,
        reader_url: None,
        api_max_conn: 1,
        api_acquire_timeout_secs: 5,
        worker_max_conn: 1,
        worker_acquire_timeout_secs: 5,
        tenant_max_conn: 1,
    };

    let config_clone = config_no_pooler.clone();
    tokio::task::spawn_blocking(move || {
        DieselPools::run_tenant_migrations(&config_clone);
    })
    .await
    .unwrap();

    // Verify tenant tables were created via direct connection fallback
    let tenant_db_url = format!("{base_connection_string}/{tenant_db_name}");
    let (client, conn) = tokio_postgres::connect(&tenant_db_url, NoTls)
        .await
        .unwrap();
    tokio::spawn(async move {
        let _ = conn.await;
    });

    let row = client
        .query_one(
            "SELECT EXISTS (SELECT FROM pg_tables WHERE schemaname = 'public' AND tablename = 'raw_attendance_logs');",
            &[],
        )
        .await
        .unwrap();
    assert!(
        row.get::<_, bool>(0),
        "tenant migrations should have run via primary_url fallback when no pooler configured"
    );

    drop(client);

    // Cleanup
    for db in &[&sys_db_name, &tenant_db_name] {
        let kill = format!(
            "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = '{db}' AND pid <> pg_backend_pid()"
        );
        let _ = admin_client.execute(&kill, &[]).await;
        let _ = admin_client
            .execute(&format!("DROP DATABASE IF EXISTS \"{db}\""), &[])
            .await;
    }
}
