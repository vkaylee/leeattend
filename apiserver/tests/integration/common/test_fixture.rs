use apiserver::diesel_pool::DieselPool;
use diesel::sql_query;
use diesel_async::RunQueryDsl;

/// Helper macros to execute raw SQL queries without SQLx.
/// These macros use diesel::sql_query under the hood and handle the
/// QueryableByName boilerplate for scalar returns.

#[macro_export]
macro_rules! test_execute {
    ($pool:expr, $query:expr $(, $bind_type:ty, $bind_val:expr)*) => {{
        let mut conn = $pool.get().await.expect("Failed to get connection from pool");
        let q = diesel::sql_query($query)
        $(
            .bind::<$bind_type, _>($bind_val)
        )*;
        diesel_async::RunQueryDsl::execute(q, &mut *conn).await.expect("Failed to execute test_execute")
    }};
}

/// Execute a SQL batch containing multiple statements.
///
/// Diesel's prepared `sql_query` path accepts one statement only; migration
/// fixtures contain several DDL statements and must use the connection's
/// simple-protocol batch executor instead.
#[macro_export]
macro_rules! test_batch_execute {
    ($pool:expr, $query:expr) => {{
        let mut conn = $pool
            .get()
            .await
            .expect("Failed to get connection from pool");
        diesel_async::SimpleAsyncConnection::batch_execute(&mut *conn, $query)
            .await
            .expect("Failed to execute test_batch_execute")
    }};
}

#[macro_export]
macro_rules! test_query_scalar_uuid {
    ($pool:expr, $query:expr $(, $bind_type:ty, $bind_val:expr)*) => {{
        #[derive(diesel::query_builder::QueryId, diesel::deserialize::QueryableByName)]
        struct ScalarResult {
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            id: uuid::Uuid,
        }
        let mut conn = $pool.get().await.expect("Failed to get connection from pool");
        let q = diesel::sql_query($query)
        $(
            .bind::<$bind_type, _>($bind_val)
        )*;
        diesel_async::RunQueryDsl::get_result::<ScalarResult>(q, &mut *conn)
            .await
            .expect("Failed to execute test_query_scalar_uuid")
            .id
    }};
}

#[macro_export]
macro_rules! test_query_scalar_uuid_optional {
    ($pool:expr, $query:expr $(, $bind_type:ty, $bind_val:expr)*) => {{
        #[derive(diesel::query_builder::QueryId, diesel::deserialize::QueryableByName)]
        struct ScalarResult {
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Uuid>)]
            id: Option<uuid::Uuid>,
        }
        let mut conn = $pool.get().await.expect("Failed to get connection from pool");
        let q = diesel::sql_query($query)
        $(
            .bind::<$bind_type, _>($bind_val)
        )*;
        diesel_async::RunQueryDsl::get_result::<ScalarResult>(q, &mut *conn)
            .await
            .ok()
            .and_then(|r| r.id)
    }};
}

#[macro_export]
macro_rules! test_query_scalar_i32 {
    ($pool:expr, $query:expr $(, $bind_type:ty, $bind_val:expr)*) => {{
        #[derive(diesel::query_builder::QueryId, diesel::deserialize::QueryableByName)]
        struct ScalarResult {
            #[diesel(sql_type = diesel::sql_types::Integer)]
            count: i32,
        }
        let mut conn = $pool.get().await.expect("Failed to get connection from pool");
        let q = diesel::sql_query($query)
        $(
            .bind::<$bind_type, _>($bind_val)
        )*;
        diesel_async::RunQueryDsl::get_result::<ScalarResult>(q, &mut *conn)
            .await
            .expect("Failed to execute test_query_scalar_i32")
            .count
    }};
}

#[macro_export]
macro_rules! test_query_scalar_i64 {
    ($pool:expr, $query:expr $(, $bind_type:ty, $bind_val:expr)*) => {{
        #[derive(diesel::query_builder::QueryId, diesel::deserialize::QueryableByName)]
        struct ScalarResult {
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            count: i64,
        }
        let mut conn = $pool.get().await.expect("Failed to get connection from pool");
        let q = diesel::sql_query($query)
        $(
            .bind::<$bind_type, _>($bind_val)
        )*;
        diesel_async::RunQueryDsl::get_result::<ScalarResult>(q, &mut *conn)
            .await
            .expect("Failed to execute test_query_scalar_i64")
            .count
    }};
}

#[macro_export]
macro_rules! test_query_scalar_string {
    ($pool:expr, $query:expr $(, $bind_type:ty, $bind_val:expr)*) => {{
        #[derive(diesel::query_builder::QueryId, diesel::deserialize::QueryableByName)]
        struct ScalarResult {
            #[diesel(sql_type = diesel::sql_types::Text)]
            val: String,
        }
        let mut conn = $pool.get().await.expect("Failed to get connection from pool");
        let q = diesel::sql_query($query)
        $(
            .bind::<$bind_type, _>($bind_val)
        )*;
        diesel_async::RunQueryDsl::get_result::<ScalarResult>(q, &mut *conn)
            .await
            .expect("Failed to execute test_query_scalar_string")
            .val
    }};
}

#[macro_export]
macro_rules! test_query_scalar_bool {
    ($pool:expr, $query:expr $(, $bind_type:ty, $bind_val:expr)*) => {{
        #[derive(diesel::query_builder::QueryId, diesel::deserialize::QueryableByName)]
        struct ScalarResult {
            #[diesel(sql_type = diesel::sql_types::Bool)]
            val: bool,
        }
        let mut conn = $pool.get().await.expect("Failed to get connection from pool");
        let q = diesel::sql_query($query)
        $(
            .bind::<$bind_type, _>($bind_val)
        )*;
        diesel_async::RunQueryDsl::get_result::<ScalarResult>(q, &mut *conn)
            .await
            .expect("Failed to execute test_query_scalar_bool")
            .val
    }};
}

#[macro_export]
macro_rules! test_query_scalar_bigdecimal {
    ($pool:expr, $query:expr $(, $bind_type:ty, $bind_val:expr)*) => {{
        #[derive(diesel::query_builder::QueryId, diesel::deserialize::QueryableByName)]
        struct ScalarResult {
            #[diesel(sql_type = diesel::sql_types::Numeric)]
            val: bigdecimal::BigDecimal,
        }
        let mut conn = $pool.get().await.expect("Failed to get connection from pool");
        let q = diesel::sql_query($query)
        $(
            .bind::<$bind_type, _>($bind_val)
        )*;
        diesel_async::RunQueryDsl::get_result::<ScalarResult>(q, &mut *conn)
            .await
            .expect("Failed to execute test_query_scalar_bigdecimal")
            .val
    }};
}
