use r2d2::Pool;
use r2d2::PooledConnection;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::collections::HashMap;
use std::time::Duration;

use crate::connection::ConnectionManager;
use crate::error::{ClientError, MemcacheError};
use crate::protocol::ProtocolTrait;

/// Stats type
pub type Stats = HashMap<String, String>;

/// Client wrapping r2d2 memcached connection pool
#[derive(Clone, Debug)]
pub struct Client(Pool<ConnectionManager>);

pub(crate) fn check_key_len<K: AsRef<[u8]>>(key: K) -> Result<(), MemcacheError> {
    if key.as_ref().len() > 250 {
        Err(ClientError::KeyTooLong.into())
    } else {
        Ok(())
    }
}

impl Client {
    /// Initialize Client with given connection pool
    pub fn with_pool(pool: Pool<ConnectionManager>) -> Self {
        Self(pool)
    }

    /// Get pool connection
    pub fn get_connection(&self) -> Result<PooledConnection<ConnectionManager>, MemcacheError> {
        Ok(self.0.get()?)
    }

    /// Get ConnectionManager pool
    pub fn get_pool(&self) -> Pool<ConnectionManager> {
        self.0.clone()
    }

    /// Get the memcached server version.
    ///
    /// Example:
    ///
    /// ```rust
    /// let pool = vmemcached::Pool::builder()
    ///     .connection_timeout(std::time::Duration::from_secs(1))
    ///     .build(vmemcached::ConnectionManager::new("memcache://localhost:11211").unwrap())
    ///     .unwrap();
    ///
    /// let client = vmemcached::Client::with_pool(pool);
    ///
    /// client.version().unwrap();
    /// ```
    pub fn version(&self) -> Result<String, MemcacheError> {
        self.get_connection()?.version()
    }

    /// Flush all cache on memcached server immediately.
    ///
    /// Example:
    ///
    /// ```rust
    /// let pool = vmemcached::Pool::builder()
    ///     .connection_timeout(std::time::Duration::from_secs(1))
    ///     .build(vmemcached::ConnectionManager::new("memcache://localhost:11211").unwrap())
    ///     .unwrap();
    ///
    /// let client = vmemcached::Client::with_pool(pool);
    ///
    /// client.flush().unwrap();
    /// ```
    #[cfg(not(feature = "mcrouter"))]
    pub fn flush(&self) -> Result<(), MemcacheError> {
        self.get_connection()?.flush()
    }

    /// Flush all cache on memcached server with a delay seconds.
    ///
    /// Example:
    ///
    /// ```rust
    /// let pool = vmemcached::Pool::builder()
    ///     .connection_timeout(std::time::Duration::from_secs(1))
    ///     .build(vmemcached::ConnectionManager::new("memcache://localhost:11211").unwrap())
    ///     .unwrap();
    ///
    /// let client = vmemcached::Client::with_pool(pool);
    ///
    /// client.flush_with_delay(10).unwrap();
    /// ```
    #[cfg(not(feature = "mcrouter"))]
    pub fn flush_with_delay(&self, delay: u32) -> Result<(), MemcacheError> {
        self.get_connection()?.flush_with_delay(delay)
    }

    /// Get a key from memcached server.
    ///
    /// Example:
    ///
    /// ```rust
    /// let pool = vmemcached::Pool::builder()
    ///     .connection_timeout(std::time::Duration::from_secs(1))
    ///     .build(vmemcached::ConnectionManager::new("memcache://localhost:11211").unwrap())
    ///     .unwrap();
    ///
    /// let client = vmemcached::Client::with_pool(pool);
    ///
    /// let _: Option<String> = client.get("foo").unwrap();
    /// ```
    pub fn get<K: AsRef<[u8]>, T: DeserializeOwned>(&self, key: K) -> Result<Option<T>, MemcacheError> {
        check_key_len(&key)?;
        self.get_connection()?.get(key)
    }

    /// Set a key with associate value into memcached server with expiration seconds.
    ///
    /// Example:
    ///
    /// ```rust
    /// let pool = vmemcached::Pool::builder()
    ///     .connection_timeout(std::time::Duration::from_secs(1))
    ///     .build(vmemcached::ConnectionManager::new("memcache://localhost:11211").unwrap())
    ///     .unwrap();
    ///
    /// let client = vmemcached::Client::with_pool(pool);
    ///
    /// client.set("foo", "bar", std::time::Duration::from_secs(10)).unwrap();
    /// # client.flush().unwrap();
    /// ```
    pub fn set<K: AsRef<[u8]>, T: Serialize>(
        &self,
        key: K,
        value: T,
        expiration: Duration,
    ) -> Result<(), MemcacheError> {
        check_key_len(&key)?;
        self.get_connection()?.set(key, value, expiration)
    }

    /// Add a key with associate value into memcached server with expiration seconds.
    ///
    /// Example:
    ///
    /// ```rust
    /// let pool = vmemcached::Pool::builder()
    ///     .connection_timeout(std::time::Duration::from_secs(1))
    ///     .build(vmemcached::ConnectionManager::new("memcache://localhost:11211").unwrap())
    ///     .unwrap();
    ///
    /// let client = vmemcached::Client::with_pool(pool);
    /// let key = "add_test";
    ///
    /// client.delete(key).unwrap();
    /// client.add(key, "bar", std::time::Duration::from_secs(100000000)).unwrap();
    /// # client.flush().unwrap();
    /// ```
    pub fn add<K: AsRef<[u8]>, T: Serialize>(
        &self,
        key: K,
        value: T,
        expiration: Duration,
    ) -> Result<(), MemcacheError> {
        check_key_len(&key)?;
        self.get_connection()?.add(key, value, expiration)
    }

    /// Replace a key with associate value into memcached server with expiration seconds.
    ///
    /// Example:
    ///
    /// ```rust
    /// let pool = vmemcached::Pool::builder()
    ///     .connection_timeout(std::time::Duration::from_secs(1))
    ///     .build(vmemcached::ConnectionManager::new("memcache://localhost:11211").unwrap())
    ///     .unwrap();
    ///
    /// let client = vmemcached::Client::with_pool(pool);
    /// let key = "replace_test";
    ///
    /// client.set(key, "bar", std::time::Duration::from_secs(0)).unwrap();
    /// client.replace(key, "baz", std::time::Duration::from_secs(100000000)).unwrap();
    /// # client.flush().unwrap();
    /// ```
    pub fn replace<K: AsRef<[u8]>, T: Serialize>(
        &self,
        key: K,
        value: T,
        expiration: Duration,
    ) -> Result<(), MemcacheError> {
        check_key_len(&key)?;
        self.get_connection()?.replace(key, value, expiration)
    }

    /// Delete a key from memcached server.
    ///
    /// Example:
    ///
    /// ```rust
    /// let pool = vmemcached::Pool::builder()
    ///     .connection_timeout(std::time::Duration::from_secs(1))
    ///     .build(vmemcached::ConnectionManager::new("memcache://localhost:11211").unwrap())
    ///     .unwrap();
    ///
    /// let client = vmemcached::Client::with_pool(pool);
    ///
    /// client.delete("foo").unwrap();
    /// # client.flush().unwrap();
    /// ```
    pub fn delete<K: AsRef<[u8]>>(&self, key: K) -> Result<bool, MemcacheError> {
        check_key_len(&key)?;
        self.get_connection()?.delete(key)
    }

    /// Set a new expiration time for a exist key.
    ///
    /// Example:
    ///
    /// ```rust
    /// let pool = vmemcached::Pool::builder()
    ///     .connection_timeout(std::time::Duration::from_secs(1))
    ///     .build(vmemcached::ConnectionManager::new("memcache://localhost:11211").unwrap())
    ///     .unwrap();
    ///
    /// let client = vmemcached::Client::with_pool(pool);
    /// assert_eq!(client.touch("not_exists_key", std::time::Duration::from_secs(11211)).unwrap(), false);
    ///
    /// client.set("foo", "bar", std::time::Duration::from_secs(123)).unwrap();
    /// assert_eq!(client.touch("foo", std::time::Duration::from_secs(11211)).unwrap(), true);
    /// # client.flush().unwrap();
    /// ```
    pub fn touch<K: AsRef<[u8]>>(&self, key: K, expiration: Duration) -> Result<bool, MemcacheError> {
        check_key_len(&key)?;
        self.get_connection()?.touch(key, expiration)
    }

    /// Get all servers' statistics.
    ///
    /// Example:
    /// ```rust
    /// let pool = vmemcached::Pool::builder()
    ///     .connection_timeout(std::time::Duration::from_secs(1))
    ///     .build(vmemcached::ConnectionManager::new("memcache://localhost:11211").unwrap())
    ///     .unwrap();
    ///
    /// let client = vmemcached::Client::with_pool(pool);
    /// let stats = client.stats().unwrap();
    /// ```
    pub fn stats(&self) -> Result<Stats, MemcacheError> {
        self.get_connection()?.stats()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn connect(target: &str) -> Result<Client, MemcacheError> {
        let pool = r2d2::Pool::builder()
            .max_size(20)
            .connection_timeout(Duration::from_millis(500))
            .build(ConnectionManager::new(target)?)?;

        Ok(Client::with_pool(pool))
    }

    #[test]
    fn delete() {
        let mcrouter = connect("memcache://localhost:11311").unwrap();
        mcrouter.set("an_exists_key", "value", Duration::from_secs(0)).unwrap();
        assert_eq!(mcrouter.delete("an_exists_key").unwrap(), true);
        assert_eq!(mcrouter.delete("a_not_exists_key").unwrap(), false);
    }
}
