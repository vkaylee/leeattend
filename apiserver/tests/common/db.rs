use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;
use std::sync::{Arc, Mutex, OnceLock};
use std::collections::HashSet;

static ACTIVE_DBS: OnceLock<Arc<Mutex<HashSet<(String, String)>>>> = OnceLock::new();
static SIGNAL_HANDLER_INITIALIZED: OnceLock<()> = OnceLock::new();

fn get_active_dbs() -> &'static Arc<Mutex<HashSet<(String, String)>>> {
    ACTIVE_DBS.get_or_init(|| Arc::new(Mutex::new(HashSet::new())))
}

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
        
        let mut url_parts: Vec<&str> = base_url.split('/').collect();
        let _database_name = url_parts.pop().unwrap_or("postgres");
        let base_connection_string = url_parts.join("/");
        
        let db_name = format!("test_db_{}", Uuid::new_v4().simple());

        Self::init_trap();

        let mut connection = PgConnection::connect(&base_url)
            .await
            .expect("Failed to connect to Postgres admin");
        
        let sql = format!("CREATE DATABASE \"{}\"", db_name);
        connection
            .execute(sql.as_str())
            .await
            .expect("Failed to create test database");

        {
            let mut dbs = get_active_dbs().lock().unwrap();
            dbs.insert((db_name.clone(), base_url.clone()));
        }

        let db_url = format!("{}/{}", base_connection_string, db_name);
        let pool = PgPool::connect(&db_url)
            .await
            .expect("Failed to connect to the new test database");

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
                    Self::spawn_detached_cleanup(db_name, admin_url);
                }
                std::process::exit(1);
            });
        });
    }

    fn spawn_detached_cleanup(db_name: String, admin_url: String) {
        let mut url_parts: Vec<&str> = admin_url.split('/').collect();
        url_parts.pop();
        let base_admin_url = format!("{}/postgres", url_parts.join("/"));

        let kill_sql = format!("SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = '{}' AND pid <> pg_backend_pid();", db_name);
        let drop_sql = format!("DROP DATABASE \"{}\";", db_name);
        
        // Để debug, bạn có thể thay > /dev/null thành >> /tmp/cleanup.log
        let full_cmd = format!(
            "sleep 0.5; psql -X \"{}\" -c \"{}\" > /dev/null 2>&1; psql -X \"{}\" -c \"{}\" > /dev/null 2>&1",
            base_admin_url, kill_sql, base_admin_url, drop_sql
        );

        let _ = std::process::Command::new("sh")
            .arg("-c")
            .arg(format!("({}) &", full_cmd))
            .spawn();
    }

    pub fn pool(&self) -> PgPool { self.pool.clone() }
    pub fn db_name(&self) -> &str { &self.db_name }
}

impl Drop for TestDatabase {
    fn drop(&mut self) {
        {
            if let Ok(mut dbs) = get_active_dbs().lock() {
                dbs.remove(&(self.db_name.clone(), self.base_url.clone()));
            }
        }
        Self::spawn_detached_cleanup(self.db_name.clone(), self.base_url.clone());
    }
}