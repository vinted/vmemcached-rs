use std::convert::TryFrom;
use std::time::Duration;
use vmemcached::{Client, ConnectionManager, MemcacheError, Pool};

// Connect to memcache
pub async fn connect(target: &str) -> Result<Client, MemcacheError> {
    let pool = Pool::builder()
        .max_size(20)
        .test_on_check_out(true)
        .connection_timeout(Duration::from_millis(500))
        .build(ConnectionManager::try_from(target)?)
        .await?;

    Ok(Client::with_pool(pool))
}
