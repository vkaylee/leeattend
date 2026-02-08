use sqlx::postgres::PgPoolOptions;

#[tokio::test]
#[ignore]
async fn run_migrations() {
    dotenvy::from_filename(".env.test").ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    println!("Migrating database: {}", database_url);

    let pool = PgPoolOptions::new()
        .connect(&database_url)
        .await
        .expect("Failed to connect to Postgres for migrations");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    println!("Migrations complete!");
}
