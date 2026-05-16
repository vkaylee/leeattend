use crate::common::E2eTestDatabase;
use diesel_async::RunQueryDsl;

#[derive(diesel::QueryableByName)]
struct DummyRow {
    #[diesel(sql_type = diesel::sql_types::Integer)]
    value: i32,
}

#[tokio::test]
async fn test_supavisor_connection_and_prepared_statements() {
    let test_db = match E2eTestDatabase::new().await {
        Some(db) => db,
        None => {
            println!("Supavisor not available, skipping test_supavisor_connection_and_prepared_statements.");
            return;
        }
    };
    let pool = test_db.diesel_pools.api.clone();

    let mut conn = pool.get().await.unwrap();
    let row: i32 = diesel::sql_query("SELECT 1 as value")
        .get_result::<DummyRow>(&mut conn)
        .await
        .expect("Failed to execute simple query")
        .value;
    assert_eq!(row, 1);

    let mut tasks = vec![];
    for i in 0..10 {
        let pool_clone = pool.clone();
        tasks.push(tokio::spawn(async move {
            use diesel::sql_types::Integer;
            let mut conn = pool_clone.get().await.unwrap();
            let row: i32 = diesel::sql_query("SELECT $1::INT as value")
                .bind::<Integer, _>(i as i32)
                .get_result::<DummyRow>(&mut conn)
                .await
                .expect("Failed concurrent query execution")
                .value;

            assert_eq!(row, i as i32);
        }));
    }

    for task in tasks {
        task.await.unwrap();
    }
}
