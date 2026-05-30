use std::collections::HashSet;
use std::sync::{Arc, Mutex, OnceLock};
use tokio_postgres::NoTls;
use uuid::Uuid;

type ActiveDbSet = Arc<Mutex<HashSet<(String, String)>>>;

static ACTIVE_DBS: OnceLock<ActiveDbSet> = OnceLock::new();
static SIGNAL_HANDLER_INITIALIZED: OnceLock<()> = OnceLock::new();

fn get_active_dbs() -> &'static ActiveDbSet {
    ACTIVE_DBS.get_or_init(|| Arc::new(Mutex::new(HashSet::new())))
}

#[allow(dead_code)]
pub struct TestDatabaseInner {
    pub diesel_pools: apiserver::diesel_pool::DieselPools,
    db_name: String,
    base_url: String,
}

#[allow(dead_code)]
pub struct TestDatabase {
    inner: Option<TestDatabaseInner>,
}

impl std::ops::Deref for TestDatabase {
    type Target = TestDatabaseInner;
    fn deref(&self) -> &Self::Target {
        self.inner.as_ref().unwrap()
    }
}

#[allow(dead_code)]
impl TestDatabase {
    pub async fn connect_with_retries(url: &str) -> tokio_postgres::Client {
        for attempt in 0..15 {
            if let Ok((client, conn)) = tokio_postgres::connect(url, NoTls).await {
                tokio::spawn(async move {
                    if let Err(e) = conn.await {
                        eprintln!("connection error: {}", e);
                    }
                });
                return client;
            }
            tokio::time::sleep(std::time::Duration::from_millis(200 * ((attempt % 5) + 1))).await;
        }
        panic!("Failed to connect to Postgres admin after 15 retries");
    }

    pub async fn new() -> Self {
        let base_url = std::env::var("DATABASE_URL_FOR_TEST_GENERATION")
            .or_else(|_| std::env::var("DATABASE_URL"))
            .expect("DATABASE_URL_FOR_TEST_GENERATION or DATABASE_URL must be set");

        assert!(!base_url.contains("leeattzkt"), "SAFETY TRIGGERED: Tests are attempting to run against the production database (leeattzkt)! Check your .env vs .env.test configuration.");

        let mut url_parts: Vec<&str> = base_url.split('/').collect();
        let _database_name = url_parts.pop().unwrap_or("postgres");
        let base_connection_string = url_parts.join("/");

        // Connect to admin database (NOT the template!) to avoid blocking
        // CREATE DATABASE ... TEMPLATE. PostgreSQL requires zero active
        // connections on the template database.
        let admin_url = format!("{base_connection_string}/test_database");

        let db_name = format!("test_db_{}", Uuid::new_v4().simple());

        Self::init_trap();

        // Retry with backoff — parallel test workers can cause transient DNS
        // resolution failures when many containers resolve the same hostname.
        let connection = Self::connect_with_retries(&admin_url).await;

        // Use template database to avoid running migrations.
        // Retry with backoff since parallel tests may briefly connect to the
        // template (e.g., during fallback migration), causing transient blocks.
        let sql = format!("CREATE DATABASE \"{db_name}\" TEMPLATE leeattend_test_template");
        let mut create_result: Option<Result<u64, tokio_postgres::Error>> = None;
        for attempt in 0..5 {
            create_result = Some(connection.execute(sql.as_str(), &[]).await);
            if let Some(Ok(_)) = create_result {
                break;
            }
            if attempt < 4 {
                let delay = std::time::Duration::from_millis(200 * (1 << attempt));
                tokio::time::sleep(delay).await;
            }
        }

        if let Some(Err(e)) = create_result {
            eprintln!("Warning: Failed to create database from template after retries: {e}. Falling back to manual migration.");
            let sql_fallback = format!("CREATE DATABASE \"{db_name}\"");
            connection
                .execute(sql_fallback.as_str(), &[])
                .await
                .expect("Failed to create test database even with fallback");

            let db_url = format!("{base_connection_string}/{db_name}");
            apiserver::diesel_pool::DieselPools::run_all_migrations_for_test(
                &apiserver::diesel_pool::DbPoolConfig {
                    primary_url: db_url.clone(),
                    pooler_tx_url: "".to_string(),
                    pooler_sess_url: "".to_string(),
                    reader_url: None,
                    api_max_conn: 1,
                    api_acquire_timeout_secs: 5,
                    worker_max_conn: 1,
                    worker_acquire_timeout_secs: 5,
                    tenant_max_conn: 1,
                },
            );

            {
                let mut dbs = get_active_dbs().lock().unwrap();
                dbs.insert((db_name.clone(), base_url.clone()));
            }

            return Self {
                inner: Some(TestDatabaseInner {
                    diesel_pools: apiserver::diesel_pool::DieselPools::for_test(&db_url),
                    db_name,
                    base_url: base_url.clone(),
                }),
            };
        }

        {
            let mut dbs = get_active_dbs().lock().unwrap();
            dbs.insert((db_name.clone(), base_url.clone()));
        }

        let db_url = format!("{base_connection_string}/{db_name}");

        Self {
            inner: Some(TestDatabaseInner {
                diesel_pools: apiserver::diesel_pool::DieselPools::for_test(&db_url),
                db_name,
                base_url: base_url.clone(),
            }),
        }
    }

    pub fn url(&self) -> String {
        let mut url_parts: Vec<&str> = self.base_url.split('/').collect();
        url_parts.pop();
        let base = url_parts.join("/");
        format!("{}/{}", base, self.db_name)
    }

    fn init_trap() {
        SIGNAL_HANDLER_INITIALIZED.get_or_init(|| {
            tokio::spawn(async move {
                use tokio::signal::unix::{signal, SignalKind};
                let mut sigint = signal(SignalKind::interrupt()).unwrap();
                let mut sigterm = signal(SignalKind::terminate()).unwrap();

                tokio::select! {
                    _ = sigint.recv() => println!("\nReceived SIGINT, cleaning up..."),
                    _ = sigterm.recv() => println!("\nReceived SIGTERM, cleaning up..."),
                };

                let dbs_to_cleanup = {
                    let mut dbs = get_active_dbs().lock().unwrap();
                    dbs.drain().collect::<Vec<_>>()
                };

                for (db_name, admin_url) in dbs_to_cleanup {
                    if let Err(e) = Self::cleanup_database_async(&db_name, &admin_url).await {
                        eprintln!("Signal cleanup failed for {db_name}: {e}");
                    }
                }
                std::process::exit(1);
            });
        });
    }

    /// Async database cleanup using tokio_postgres
    async fn cleanup_database_async(
        db_name: &str,
        admin_url: &str,
    ) -> Result<(), tokio_postgres::Error> {
        let mut url_parts: Vec<&str> = admin_url.split('/').collect();
        url_parts.pop();
        let base_admin_url = format!("{}/postgres", url_parts.join("/"));

        let (client, connection) = tokio_postgres::connect(&base_admin_url, NoTls).await?;
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });

        // Terminate all connections to the target database
        let kill_sql = format!(
            "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = '{db_name}' AND pid <> pg_backend_pid()"
        );
        let _ = client.execute(kill_sql.as_str(), &[]).await;

        // Drop the database
        let drop_sql = format!("DROP DATABASE IF EXISTS \"{db_name}\"");
        client.execute(drop_sql.as_str(), &[]).await?;

        Ok(())
    }

    /// Blocking cleanup wrapper for use in Drop trait.
    /// Runs cleanup on a dedicated thread with its own runtime to avoid
    /// panics from `block_in_place` (requires multi-threaded runtime, but
    /// nextest uses current-thread). Joins the thread to ensure cleanup
    /// completes before the test process exits.
    fn cleanup_database_blocking(db_name: String, admin_url: String) {
        let handle = std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build();
            if let Ok(rt) = rt {
                // cleanup_database_async already uses pg_terminate_backend
                // to forcefully kill all connections before DROP DATABASE.
                if let Err(e) = rt.block_on(Self::cleanup_database_async(&db_name, &admin_url)) {
                    eprintln!("Cleanup failed for {db_name}: {e}");
                }
            }
        });
        // Wait for the cleanup thread to finish
        let _ = handle.join();
    }

    #[must_use]
    pub fn db_name(&self) -> &str {
        &self.db_name
    }
}

impl Drop for TestDatabase {
    fn drop(&mut self) {
        if let Some(inner) = self.inner.take() {
            let db_name = inner.db_name.clone();
            let base_url = inner.base_url.clone();

            // Drop connection pools BEFORE trying to delete the database
            // to prevent connection pools from reconnecting during DROP DATABASE
            drop(inner);

            if let Ok(mut dbs) = get_active_dbs().lock() {
                dbs.remove(&(db_name.clone(), base_url.clone()));
            }
            Self::cleanup_database_blocking(db_name, base_url);
        }
    }
}
