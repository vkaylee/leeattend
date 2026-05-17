#![allow(missing_docs)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::panic)]
#![allow(clippy::todo)]
#![allow(clippy::unimplemented)]
#![allow(warnings)]
#![allow(clippy::all)]
#![allow(clippy::pedantic)]
#![allow(clippy::nursery)]
use apiserver::diesel_pool::{DbPoolConfig, DieselPools};

#[tokio::test]
#[ignore]
async fn run_migrations() {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    println!("Migrating database: {database_url}");

    DieselPools::run_migrations(&DbPoolConfig {
        primary_url: database_url,
        pooler_url: None,
        reader_url: None,
        api_max_conn: 1,
        api_acquire_timeout_secs: 5,
        worker_max_conn: 1,
        worker_acquire_timeout_secs: 5,
        tenant_max_conn: 1,
    });

    println!("Migrations complete!");
}
