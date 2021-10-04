use std::time::Duration;
use std::convert::TryFrom;
use vmemcached::{Client, ConnectionManager, MemcacheError, Pool};

// Connect to memcache
pub async fn connect(target: &str) -> Result<Client, MemcacheError> {
    let pool = Pool::builder()
        .max_size(20)
        .connection_timeout(Duration::from_millis(500))
        .build(ConnectionManager::try_from(target)?).await?;

    Ok(Client::with_pool(pool))
}
