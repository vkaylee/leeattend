use crate::common::E2eTestDatabase;

#[tokio::test]
async fn test_supavisor_connection_and_prepared_statements() {
    let test_db = match E2eTestDatabase::new().await {
        Some(db) => db,
        None => {
            println!("Supavisor not available, skipping test_supavisor_connection_and_prepared_statements.");
            return;
        }
    };
    let pool = test_db.pool.clone();

    // Test simple query execution
    let row: (i32,) = sqlx::query_as("SELECT 1")
        .fetch_one(&pool)
        .await
        .expect("Failed to execute simple query");
    assert_eq!(row.0, 1);

    // Test concurrent prepared statement execution (multiplexing via Supavisor)
    // This specifically triggers the `sqlx_s_N already exists` error if Supavisor is misconfigured (e.g. Transaction mode on SQLx macros)
    let mut tasks = vec![];
    for i in 0..10 {
        let pool_clone = pool.clone();
        tasks.push(tokio::spawn(async move {
            let row: (Option<i32>,) = sqlx::query_as("SELECT $1::INT as value")
                .bind(i as i32)
                .fetch_one(&pool_clone)
                .await
                .expect("Failed concurrent query execution");

            assert_eq!(row.0, Some(i as i32));
        }));
    }

    for task in tasks {
        task.await.unwrap();
    }
}
