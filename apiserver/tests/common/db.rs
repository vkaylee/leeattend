use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;

#[allow(dead_code)]
pub struct TestDatabase {
    pub pool: PgPool,
    db_name: String,
    base_url: String,
}

#[allow(dead_code)]
impl TestDatabase {
    pub async fn new() -> Self {
        dotenvy::from_filename(".env.test").ok();
        let base_url = std::env::var("DATABASE_URL_FOR_TEST_GENERATION")
            .or_else(|_| std::env::var("DATABASE_URL"))
            .expect("DATABASE_URL_FOR_TEST_GENERATION or DATABASE_URL must be set");
        
        // Parse base URL to get the connection string to 'postgres' database
        let mut url_parts: Vec<&str> = base_url.split('/').collect();
        let _database_name = url_parts.pop().unwrap_or("postgres");
        let base_connection_string = url_parts.join("/");
        
        let db_name = format!("test_db_{}", Uuid::new_v4().simple());

        // 1. Create the new database
        let mut connection = PgConnection::connect(&base_url)
            .await
            .expect("Failed to connect to Postgres admin");
        
        connection
            .execute(format!(r#"CREATE DATABASE "{}""#, db_name).as_str())
            .await
            .expect("Failed to create test database");

        // 2. Connect to the new database
        let db_url = format!("{}/{}", base_connection_string, db_name);
        let pool = PgPool::connect(&db_url)
            .await
            .expect("Failed to connect to the new test database");

        // 3. Run migrations
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("Failed to run migrations");

        Self {
            pool,
            db_name,
            base_url: base_url.to_string(),
        }
    }

    pub fn pool(&self) -> PgPool {
        self.pool.clone()
    }
}

impl Drop for TestDatabase {
    fn drop(&mut self) {
        let db_name = self.db_name.clone();
        let admin_url = self.base_url.clone();

        // We need to spawn a separate thread because Drop is synchronous,
        // but dropping a database is an asynchronous operation over the network.
        std::thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();

            runtime.block_on(async move {
                // 1. Connect to admin database
                let mut connection = match PgConnection::connect(&admin_url).await {
                    Ok(conn) => conn,
                    Err(_) => return, // Silent fail if cannot connect
                };

                // 2. Force terminate all other connections to the test database
                let _ = connection
                    .execute(
                        format!(
                            r#"
                            SELECT pg_terminate_backend(pg_stat_activity.pid)
                            FROM pg_stat_activity
                            WHERE pg_stat_activity.datname = '{}'
                              AND pid <> pg_backend_pid();
                            "#,
                            db_name
                        )
                        .as_str(),
                    )
                    .await;

                // 3. Drop the database
                let _ = connection
                    .execute(format!(r#"DROP DATABASE "{}""#, db_name).as_str())
                    .await;
            });
        });
    }
}
