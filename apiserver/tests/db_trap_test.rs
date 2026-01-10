mod common;
use sqlx::{Connection, PgConnection};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_database_cleanup_lifecycle() {
    dotenvy::from_filename(".env.test").ok();
    let db_name: String;
    let base_url: String;

    {
        let db = common::db::TestDatabase::new().await;
        db_name = db.db_name().to_string();
        base_url = std::env::var("DATABASE_URL_FOR_TEST_GENERATION")
            .or_else(|_| std::env::var("DATABASE_URL"))
            .expect("DATABASE_URL must be set");

        let mut conn = PgConnection::connect(&base_url).await.unwrap();
        let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM pg_database WHERE datname = $1)")
            .bind(&db_name)
            .fetch_one(&mut conn)
            .await
            .unwrap();
        
        assert!(exists, "Arrange: Database {} should exist after creation", db_name);
    } 

    println!("Act: Dropped TestDatabase, waiting for background worker...");
    sleep(Duration::from_millis(5000)).await;

    let mut conn = PgConnection::connect(&base_url).await.unwrap();
    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM pg_database WHERE datname = $1)")
        .bind(&db_name)
        .fetch_one(&mut conn)
        .await
        .unwrap();

    assert!(!exists, "Assert: Database {} should be physically deleted from Postgres", db_name);
}

#[tokio::test]
async fn test_concurrent_databases_cleanup() {
    dotenvy::from_filename(".env.test").ok();
    let mut db_names = Vec::new();
    let base_url = std::env::var("DATABASE_URL_FOR_TEST_GENERATION")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .expect("DATABASE_URL must be set");

    {
        let db1 = common::db::TestDatabase::new().await;
        let db2 = common::db::TestDatabase::new().await;
        let db3 = common::db::TestDatabase::new().await;

        db_names.push(db1.db_name().to_string());
        db_names.push(db2.db_name().to_string());
        db_names.push(db3.db_name().to_string());

        let mut conn = PgConnection::connect(&base_url).await.unwrap();
        for name in &db_names {
            let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM pg_database WHERE datname = $1)")
                .bind(name)
                .fetch_one(&mut conn)
                .await
                .unwrap();
            assert!(exists);
        }
    } 

    println!("Act: Dropped 3 databases, waiting for concurrent background workers...");
    sleep(Duration::from_millis(5000)).await;

    let mut conn = PgConnection::connect(&base_url).await.unwrap();
    for name in &db_names {
        let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM pg_database WHERE datname = $1)")
            .bind(name)
            .fetch_one(&mut conn)
            .await
            .unwrap();
        assert!(!exists, "Database {} still exists after concurrent drop", name);
    }
}
