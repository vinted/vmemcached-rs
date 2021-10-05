use bb8::PooledConnection;
use bytes::BytesMut;
use futures_util::TryFutureExt;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::collections::HashMap;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::manager::ConnectionManager;
use crate::parser::{self, Response};
use crate::{codec, driver, ClientError, MemcacheError, Pool};

/// Stats type
pub type Stats = HashMap<String, String>;

/// Client wrapping r2d2 memcached connection pool
#[derive(Clone, Debug)]
pub struct Client(Pool);

pub(crate) fn check_key_len<K: AsRef<[u8]>>(key: K) -> Result<(), MemcacheError> {
    if key.as_ref().len() > 250 {
        Err(ClientError::KeyTooLong.into())
    } else {
        Ok(())
    }
}

impl Client {
    /// Initialize Client with given connection pool
    pub fn with_pool(pool: Pool) -> Self {
        Self(pool)
    }

    /// Get pool connection
    pub async fn get_connection<'f>(
        &'f self,
    ) -> Result<PooledConnection<'f, ConnectionManager>, MemcacheError> {
        Ok(self.0.get().await?)
    }

    /// Get ConnectionManager pool
    pub fn get_pool(&self) -> Pool {
        self.0.clone()
    }

    // /// Get the memcached server version.
    pub async fn version(&self) -> Result<String, MemcacheError> {
        let mut conn = self.get_connection().await?;
        driver::version(&mut conn).await
    }

    /// Get a key from memcached server.
    pub async fn get<K: AsRef<[u8]>, V: DeserializeOwned>(
        &self,
        key: K,
    ) -> Result<Option<V>, MemcacheError> {
        check_key_len(&key)?;

        let keys = &[key];

        // <command name> <key> <flags> <exptime> <bytes> [noreply]\r\n
        self.get_connection()
            .and_then(|conn| driver::retrieve(conn, driver::RetrievalCommand::Get, keys))
            .and_then(|response| async {
                if let Some(mut values) = response {
                    let value = values.swap_remove(0);
                    codec::decode(value.data)
                } else {
                    Ok(None)
                }
            })
            .await
    }

    /// Get keys from memcached server.
    pub async fn gets<K: AsRef<[u8]>, V: DeserializeOwned>(
        &self,
        keys: &[K],
    ) -> Result<Option<HashMap<String, V>>, MemcacheError> {
        for key in keys.iter() {
            check_key_len(&key)?;
        }

        // <command name> <key> <flags> <exptime> <bytes> [noreply]\r\n
        self.get_connection()
            .and_then(|conn| driver::retrieve(conn, driver::RetrievalCommand::Gets, keys))
            .and_then(|response| async {
                if let Some(values) = response {
                    let mut map: HashMap<String, V> = HashMap::with_capacity(values.len());

                    for value in values.into_iter() {
                        let decoded: V = codec::decode(value.data)?;

                        let _ = map.insert(String::from_utf8(value.key)?, decoded);
                    }
                    Ok(Some(map))
                } else {
                    Ok(None)
                }
            })
            .await
    }

    /// Set a key with associate value into memcached server with expiration seconds.
    pub async fn set<K: AsRef<[u8]>, T: Serialize>(
        &self,
        key: K,
        value: T,
        expiration: impl Into<Option<Duration>>,
    ) -> Result<parser::Status, MemcacheError> {
        check_key_len(&key)?;

        let encoded = codec::encode(value)?;

        // <command name> <key> <flags> <exptime> <bytes> [noreply]\r\n
        self.get_connection()
            .and_then(|conn| {
                driver::storage(
                    conn,
                    driver::StorageCommand::Set,
                    key,
                    0,
                    expiration,
                    encoded,
                    false,
                )
            })
            .and_then(|response| async {
                match response {
                    Response::Status(s) => Ok(s),
                    Response::Error(e) => Err(e.into()),
                    _ => unreachable!(),
                }
            })
            .await
    }

    /// Delete a key with associate value into memcached server
    pub async fn delete<K: AsRef<[u8]>>(&self, key: K) -> Result<parser::Status, MemcacheError> {
        check_key_len(&key)?;

        // <command name> <key> <flags> <exptime> <bytes> [noreply]\r\n
        self.get_connection()
            .and_then(|conn| driver::delete(conn, key, false))
            .and_then(|response| async {
                match response {
                    Response::Status(s) => Ok(s),
                    Response::Error(e) => Err(e.into()),
                    _ => unreachable!(),
                }
            })
            .await
    }

    /// Delete a key with associate value into memcached server
    pub async fn touch<K: AsRef<[u8]>>(
        &self,
        key: K,
        expiration: impl Into<Option<Duration>>,
    ) -> Result<parser::Status, MemcacheError> {
        check_key_len(&key)?;

        // <command name> <key> <flags> <exptime> <bytes> [noreply]\r\n
        self.get_connection()
            .and_then(|conn| driver::touch(conn, key, expiration, false))
            .and_then(|response| async {
                match response {
                    Response::Status(s) => Ok(s),
                    Response::Error(e) => Err(e.into()),
                    _ => unreachable!(),
                }
            })
            .await
    }

    // /// Add a key with associate value into memcached server with expiration seconds.
    // ///
    // /// Example:
    // ///
    // /// ```rust
    // /// let pool = vmemcached::Pool::builder()
    // ///     .connection_timeout(std::time::Duration::from_secs(1))
    // ///     .build(vmemcached::ConnectionManager::new("memcache://localhost:11211").unwrap())
    // ///     .unwrap();
    // ///
    // /// let client = vmemcached::Client::with_pool(pool);
    // /// let key = "add_test";
    // ///
    // /// client.delete(key).unwrap();
    // /// client.add(key, "bar", std::time::Duration::from_secs(100000000)).unwrap();
    // /// # client.flush().unwrap();
    // /// ```
    // pub fn add<K: AsRef<[u8]>, T: Serialize>(
    //     &self,
    //     key: K,
    //     value: T,
    //     expiration: Duration,
    // ) -> Result<(), MemcacheError> {
    //     check_key_len(&key)?;
    //     self.get_connection()?.add(key, value, expiration)
    // }
    //
    // /// Replace a key with associate value into memcached server with expiration seconds.
    // ///
    // /// Example:
    // ///
    // /// ```rust
    // /// let pool = vmemcached::Pool::builder()
    // ///     .connection_timeout(std::time::Duration::from_secs(1))
    // ///     .build(vmemcached::ConnectionManager::new("memcache://localhost:11211").unwrap())
    // ///     .unwrap();
    // ///
    // /// let client = vmemcached::Client::with_pool(pool);
    // /// let key = "replace_test";
    // ///
    // /// client.set(key, "bar", std::time::Duration::from_secs(0)).unwrap();
    // /// client.replace(key, "baz", std::time::Duration::from_secs(100000000)).unwrap();
    // /// # client.flush().unwrap();
    // /// ```
    // pub fn replace<K: AsRef<[u8]>, T: Serialize>(
    //     &self,
    //     key: K,
    //     value: T,
    //     expiration: Duration,
    // ) -> Result<(), MemcacheError> {
    //     check_key_len(&key)?;
    //     self.get_connection()?.replace(key, value, expiration)
    // }
    //
    // /// Delete a key from memcached server.
    // ///
    // /// Example:
    // ///
    // /// ```rust
    // /// let pool = vmemcached::Pool::builder()
    // ///     .connection_timeout(std::time::Duration::from_secs(1))
    // ///     .build(vmemcached::ConnectionManager::new("memcache://localhost:11211").unwrap())
    // ///     .unwrap();
    // ///
    // /// let client = vmemcached::Client::with_pool(pool);
    // ///
    // /// client.delete("foo").unwrap();
    // /// # client.flush().unwrap();
    // /// ```
    // pub fn delete<K: AsRef<[u8]>>(&self, key: K) -> Result<bool, MemcacheError> {
    //     check_key_len(&key)?;
    //     self.get_connection()?.delete(key)
    // }
    //
    // /// Set a new expiration time for a exist key.
    // ///
    // /// Example:
    // ///
    // /// ```rust
    // /// let pool = vmemcached::Pool::builder()
    // ///     .connection_timeout(std::time::Duration::from_secs(1))
    // ///     .build(vmemcached::ConnectionManager::new("memcache://localhost:11211").unwrap())
    // ///     .unwrap();
    // ///
    // /// let client = vmemcached::Client::with_pool(pool);
    // /// assert_eq!(client.touch("not_exists_key", std::time::Duration::from_secs(11211)).unwrap(), false);
    // ///
    // /// client.set("foo", "bar", std::time::Duration::from_secs(123)).unwrap();
    // /// assert_eq!(client.touch("foo", std::time::Duration::from_secs(11211)).unwrap(), true);
    // /// # client.flush().unwrap();
    // /// ```
    // pub fn touch<K: AsRef<[u8]>>(&self, key: K, expiration: Duration) -> Result<bool, MemcacheError> {
    //     check_key_len(&key)?;
    //     self.get_connection()?.touch(key, expiration)
    // }
    //
    // /// Get all servers' statistics.
    // ///
    // /// Example:
    // /// ```rust
    // /// let pool = vmemcached::Pool::builder()
    // ///     .connection_timeout(std::time::Duration::from_secs(1))
    // ///     .build(vmemcached::ConnectionManager::new("memcache://localhost:11211").unwrap())
    // ///     .unwrap();
    // ///
    // /// let client = vmemcached::Client::with_pool(pool);
    // /// let stats = client.stats().unwrap();
    // /// ```
    // pub async fn stats(&self) -> Result<Stats, MemcacheError> {
    //     self.get_connection().and_then(|conn| {
    //         conn.stats()
    //     })
    // }
}
