use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::AsyncPgConnection;
use diesel::connection::CacheSize;

pub fn test() {
    let mut config = diesel_async::pooled_connection::ManagerConfig::default();
    config.custom_setup = Box::new(|conn| {
        Box::pin(async move {
            diesel_async::AsyncConnection::set_prepared_statement_cache_size(conn, CacheSize::Disabled);
            Ok(())
        })
    });
    let manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new_with_config("url", config);
}
