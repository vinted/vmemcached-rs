use std::convert::TryFrom;
use std::time::Duration;
use vmemcached::{Client, ConnectionManager, MemcacheError, Pool, Settings};

// Connect to memcache
pub async fn connect(target: &str) -> Result<Client, MemcacheError> {
    let pool = Pool::builder()
        .max_size(40)
        .min_idle(Some(2))
        .test_on_check_out(true)
        .max_lifetime(Some(Duration::from_secs(60 * 30)))
        .idle_timeout(Some(Duration::from_secs(60 * 10)))
        .connection_timeout(Duration::from_millis(40))
        .build(ConnectionManager::try_from(target)?)
        .await?;

    let options = Settings::new();

    Ok(Client::with_pool(pool, options))
}

// Connect to memcache with custom settings
pub async fn connect_with_custom_settings(
    target: &str,
    settings: Settings,
) -> Result<Client, MemcacheError> {
    let pool = Pool::builder()
        .max_size(40)
        .min_idle(Some(2))
        .test_on_check_out(true)
        .max_lifetime(Some(Duration::from_secs(60 * 30)))
        .idle_timeout(Some(Duration::from_secs(60 * 10)))
        .connection_timeout(Duration::from_millis(40))
        .build(ConnectionManager::try_from(target)?)
        .await?;

    Ok(Client::with_pool(pool, settings))
}
