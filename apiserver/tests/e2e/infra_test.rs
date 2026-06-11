use crate::common::E2eTestDatabase;
use diesel_async::RunQueryDsl;

#[derive(diesel::QueryableByName)]
struct DummyRow {
    #[diesel(sql_type = diesel::sql_types::Integer)]
    value: i32,
}

#[tokio::test]
async fn test_pgbouncer_transaction_mode_prepared_statements() {
    let is_pgbouncer = std::env::var("DB_POOLER_TYPE")
        .map(|v| v == "pgbouncer")
        .unwrap_or(false);
    if !is_pgbouncer {
        println!("PgBouncer not in use, skipping transaction mode prepared statements test.");
        return;
    }

    let base_url = std::env::var("DATABASE_URL").unwrap();
    let parsed_url = reqwest::Url::parse(&base_url).unwrap();
    let pooler_user = parsed_url.username().to_string();
    let pooler_pass = parsed_url.password().unwrap_or("").to_string();
    let pooler_host =
        std::env::var("SUPAVISOR_HOST").unwrap_or_else(|_| "pgbouncer_tx".to_string());

    // Connect to port 6543 (PgBouncer Transaction Pooler)
    let pooler_url =
        format!("postgres://{pooler_user}:{pooler_pass}@{pooler_host}:6543/test_database");

    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&pooler_url);
    let pool = diesel_pools.api.clone();

    // Verify simple connection
    let mut conn = pool.get().await.unwrap();
    let row: i32 = diesel::sql_query("SELECT 1 as value")
        .get_result::<DummyRow>(&mut conn)
        .await
        .expect("Failed simple query on PgBouncer Transaction Mode")
        .value;
    assert_eq!(row, 1);
    drop(conn);

    // Run concurrent queries with prepared statements on transaction port
    let mut tasks = vec![];
    for i in 0..10 {
        let pool_clone = pool.clone();
        tasks.push(tokio::spawn(async move {
            use diesel::sql_types::Integer;
            let mut conn = pool_clone.get().await.unwrap();
            let row: i32 = diesel::sql_query("SELECT $1::INT as value")
                .bind::<Integer, _>(i)
                .get_result::<DummyRow>(&mut conn)
                .await
                .expect("Failed concurrent query on PgBouncer Transaction Mode")
                .value;

            assert_eq!(row, i);
        }));
    }

    for task in tasks {
        task.await.unwrap();
    }
}
