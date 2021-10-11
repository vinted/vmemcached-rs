use std::convert::TryFrom;
use std::time::Duration;
use vmemcached::{Client, ConnectionManager, MemcacheError, Pool};

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

    Ok(Client::with_pool(pool))
}
