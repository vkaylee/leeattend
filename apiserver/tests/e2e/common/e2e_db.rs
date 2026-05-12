use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::collections::HashSet;
use std::sync::{Arc, Mutex, OnceLock};
use uuid::Uuid;

type ActiveDbSet = Arc<Mutex<HashSet<(String, String)>>>;
static ACTIVE_DBS: OnceLock<ActiveDbSet> = OnceLock::new();
static SIGNAL_HANDLER_INITIALIZED: OnceLock<()> = OnceLock::new();

fn get_active_dbs() -> &'static ActiveDbSet {
    ACTIVE_DBS.get_or_init(|| Arc::new(Mutex::new(HashSet::new())))
}

#[allow(dead_code)]
pub struct E2eTestDatabase {
    pub pool: PgPool,
    db_name: String,
    base_url: String, // The direct postgres URL used to drop it
    tenant_id: String,
}

impl E2eTestDatabase {
    pub async fn new() -> Option<Self> {
        let base_url = std::env::var("DATABASE_URL_FOR_TEST_GENERATION")
            .or_else(|_| std::env::var("DATABASE_URL"))
            .expect("DATABASE_URL_FOR_TEST_GENERATION or DATABASE_URL must be set");

        assert!(
            !base_url.contains("leeattzkt"),
            "SAFETY TRIGGERED: Tests are attempting to run against production!"
        );

        let mut url_parts: Vec<&str> = base_url.split('/').collect();
        let _ = url_parts.pop().unwrap_or("postgres");
        let base_connection_string = url_parts.join("/");

        let admin_url = format!("{base_connection_string}/test_database");
        let db_name = format!("e2e_test_{}", Uuid::new_v4().simple());
        let tenant_id = db_name.clone();

        Self::init_trap();

        let api_url = std::env::var("SUPAVISOR_API_URL")
            .unwrap_or_else(|_| "http://supavisor:4000".to_string());

        let client = reqwest::Client::new();
        if client.get(&api_url).send().await.is_err() {
            println!(
                "Supavisor API is unreachable at {}. Skipping E2E test database creation.",
                api_url
            );
            return None;
        }

        // 1. Create Physical Database via Direct Connection
        let mut connection = Self::connect_with_retries(&admin_url).await;
        let sql = format!("CREATE DATABASE \"{db_name}\" TEMPLATE leeattend_test_template");

        let mut create_result = Err(sqlx::Error::PoolClosed);
        for attempt in 0..5 {
            create_result = connection.execute(sql.as_str()).await;
            if create_result.is_ok() {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(200 * (1 << attempt))).await;
        }

        if let Err(e) = create_result {
            eprintln!("Warning: Failed to create database from template: {e}. Falling back to manual migration.");
            connection
                .execute(format!("CREATE DATABASE \"{db_name}\"").as_str())
                .await
                .expect("Fallback failed");

            let direct_db_url = format!("{base_connection_string}/{db_name}");
            let pool = PgPool::connect(&direct_db_url).await.unwrap();
            sqlx::migrate!("../migrations").run(&pool).await.unwrap();
            pool.close().await;
        }

        {
            let mut dbs = get_active_dbs().lock().unwrap();
            dbs.insert((tenant_id.clone(), base_url.clone()));
        }

        // 2. Register Tenant in Supavisor
        Self::register_tenant(&db_name, &tenant_id).await;

        // 3. Connect via Supavisor Pooler
        let supavisor_port = std::env::var("SUPAVISOR_PORT").unwrap_or_else(|_| "5452".to_string());
        let supavisor_host =
            std::env::var("SUPAVISOR_HOST").unwrap_or_else(|_| "supavisor".to_string());

        let parsed_url = reqwest::Url::parse(&base_url).unwrap();
        let pooler_user = parsed_url.username().to_string();
        let pooler_pass = parsed_url.password().unwrap_or("").to_string();

        // Supavisor connection string: postgres://{user}.{tenant}:{password}@{host}:{port}/{dbname}
        let pooler_url = format!(
            "postgres://{}.{}:{}@{}:{}/{}",
            pooler_user, tenant_id, pooler_pass, supavisor_host, supavisor_port, db_name
        );

        let pool = PgPool::connect(&pooler_url)
            .await
            .unwrap_or_else(|_| panic!("Failed to connect to Supavisor Pooler at {}", pooler_url));

        Some(Self {
            pool,
            db_name,
            base_url: base_url.clone(),
            tenant_id,
        })
    }

    async fn connect_with_retries(url: &str) -> PgConnection {
        for attempt in 0..5 {
            if let Ok(conn) = PgConnection::connect(url).await {
                return conn;
            }
            tokio::time::sleep(std::time::Duration::from_millis(200 * (1 << attempt))).await;
        }
        panic!("Failed to connect to Postgres admin after 5 retries");
    }

    async fn register_tenant(db_name: &str, tenant_id: &str) {
        let api_url = std::env::var("SUPAVISOR_API_URL")
            .unwrap_or_else(|_| "http://supavisor:4000".to_string());
        let jwt_secret = std::env::var("API_JWT_SECRET").unwrap_or_else(|_| {
            "your-super-secret-jwt-token-with-at-least-32-bytes-long".to_string()
        });

        let token = Self::generate_jwt(&jwt_secret);
        let payload = Self::build_payload(db_name);

        let client = reqwest::Client::new();
        let url = format!("{}/api/tenants/{tenant_id}", api_url.trim_end_matches('/'));

        let resp = client
            .put(&url)
            .header("Authorization", format!("Bearer {token}"))
            .json(&payload)
            .send()
            .await
            .expect("Failed to reach Supavisor API");

        assert!(
            resp.status().is_success(),
            "Failed to register tenant: {:?}",
            resp.text().await
        );

        // Sleep a tiny bit to ensure Supavisor propagates the new tenant config internally
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }

    async fn delete_tenant(tenant_id: &str) {
        let api_url = std::env::var("SUPAVISOR_API_URL")
            .unwrap_or_else(|_| "http://supavisor:4000".to_string());
        let jwt_secret = std::env::var("API_JWT_SECRET").unwrap_or_else(|_| {
            "your-super-secret-jwt-token-with-at-least-32-bytes-long".to_string()
        });
        let token = Self::generate_jwt(&jwt_secret);

        let client = reqwest::Client::new();
        let url = format!("{}/api/tenants/{tenant_id}", api_url.trim_end_matches('/'));

        let _ = client
            .delete(&url)
            .header("Authorization", format!("Bearer {token}"))
            .send()
            .await;
    }

    fn generate_jwt(secret: &str) -> String {
        #[derive(Debug, Serialize, Deserialize)]
        struct Claims {
            iat: usize,
            exp: usize,
        }
        let iat = chrono::Utc::now().timestamp() as usize;
        let claims = Claims {
            iat,
            exp: iat + 3600,
        };
        encode(
            &Header::new(Algorithm::HS256),
            &claims,
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .unwrap()
    }

    fn build_payload(db_name: &str) -> serde_json::Value {
        let base_url = std::env::var("DATABASE_URL").unwrap();
        let parsed_url = reqwest::Url::parse(&base_url).unwrap();

        let db_user = parsed_url.username().to_string();
        let db_password = parsed_url.password().unwrap_or("").to_string();
        let db_host = parsed_url.host_str().unwrap_or("db").to_string();
        let db_port = parsed_url.port().unwrap_or(5432);

        json!({
            "tenant": {
                "db_host": db_host,
                "db_port": db_port,
                "db_database": db_name,
                "ip_version": "auto",
                "enforce_ssl": false,
                "require_user": false,
                "auth_query": "SELECT rolname, rolpassword FROM pg_authid WHERE rolname=$1;",
                "users": [
                    {
                        "db_user": db_user,
                        "db_password": db_password,
                        "pool_size": 10,
                        "mode_type": "session",
                        "is_manager": true
                    }
                ]
            }
        })
    }

    fn init_trap() {
        SIGNAL_HANDLER_INITIALIZED.get_or_init(|| {
            tokio::spawn(async move {
                use tokio::signal::unix::{signal, SignalKind};
                let mut sigint = signal(SignalKind::interrupt()).unwrap();
                let mut sigterm = signal(SignalKind::terminate()).unwrap();

                tokio::select! {
                    _ = sigint.recv() => {},
                    _ = sigterm.recv() => {},
                };

                let dbs_to_cleanup = get_active_dbs().lock().unwrap().drain().collect::<Vec<_>>();
                for (tenant_id, admin_url) in dbs_to_cleanup {
                    let _ = Self::cleanup_database_async(&tenant_id, &admin_url).await;
                }
                std::process::exit(1);
            });
        });
    }

    async fn cleanup_database_async(tenant_id: &str, admin_url: &str) -> Result<(), sqlx::Error> {
        Self::delete_tenant(tenant_id).await;

        let mut url_parts: Vec<&str> = admin_url.split('/').collect();
        url_parts.pop();
        let base_admin_url = format!("{}/postgres", url_parts.join("/"));

        let mut conn = PgConnection::connect(&base_admin_url).await?;
        let kill_sql = format!(
            "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = '{tenant_id}' AND pid <> pg_backend_pid()"
        );
        let _ = conn.execute(kill_sql.as_str()).await;
        let drop_sql = format!("DROP DATABASE IF EXISTS \"{tenant_id}\"");
        conn.execute(drop_sql.as_str()).await?;
        Ok(())
    }

    fn cleanup_blocking(tenant_id: String, admin_url: String) {
        let handle = std::thread::spawn(move || {
            if let Ok(rt) = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            {
                let _ = rt.block_on(Self::cleanup_database_async(&tenant_id, &admin_url));
            }
        });
        let _ = handle.join();
    }
}

impl Drop for E2eTestDatabase {
    fn drop(&mut self) {
        if let Ok(mut dbs) = get_active_dbs().lock() {
            dbs.remove(&(self.tenant_id.clone(), self.base_url.clone()));
        }
        Self::cleanup_blocking(self.tenant_id.clone(), self.base_url.clone());
    }
}
