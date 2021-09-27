use std::time::Duration;
use vmemcached::{Client, ConnectionManager, MemcacheError};

// Connect to memcache
pub fn connect(target: &str) -> Result<Client, MemcacheError> {
    let pool = r2d2::Pool::builder()
        .max_size(20)
        .connection_timeout(Duration::from_millis(500))
        .build(ConnectionManager::new(target)?)?;

    Ok(Client::with_pool(pool))
}
