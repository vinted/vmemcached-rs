use bb8::PooledConnection;
use futures_util::TryFutureExt;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::collections::HashMap;
use std::time::Duration;

use crate::driver::{RetrievalCommand, StorageCommand};
use crate::manager::ConnectionManager;
use crate::parser::{self, Response};
use crate::{codec, driver, ClientError, MemcacheError, Pool};

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
    pub async fn get_connection(
        &self,
    ) -> Result<PooledConnection<'_, ConnectionManager>, MemcacheError> {
        Ok(self.0.get().await?)
    }

    /// Get clone of ConnectionManager pool
    pub fn get_pool(&self) -> Pool {
        self.0.clone()
    }

    /// Get the server version
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
            .and_then(|conn| driver::retrieve(conn, RetrievalCommand::Get, keys))
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
            .and_then(|conn| driver::retrieve(conn, RetrievalCommand::Gets, keys))
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

    #[inline]
    async fn store<K: AsRef<[u8]>, T: Serialize, E>(
        &self,
        cmd: StorageCommand,
        key: K,
        value: T,
        expiration: E,
    ) -> Result<parser::Status, MemcacheError>
    where
        E: Into<Option<Duration>>,
    {
        check_key_len(&key)?;

        let encoded = codec::encode(value)?;

        // <command name> <key> <flags> <exptime> <bytes> [noreply]\r\n
        self.get_connection()
            .and_then(|conn| driver::storage(conn, cmd, key, 0, expiration, encoded, false))
            .and_then(|response| async {
                match response {
                    Response::Status(s) => Ok(s),
                    Response::Error(e) => Err(e.into()),
                    _ => unreachable!(),
                }
            })
            .await
    }

    /// Set a key with associate value into memcached server with expiration seconds.
    pub async fn set<K: AsRef<[u8]>, T: Serialize, E>(
        &self,
        key: K,
        value: T,
        expiration: E,
    ) -> Result<parser::Status, MemcacheError>
    where
        E: Into<Option<Duration>>,
    {
        self.store(driver::StorageCommand::Set, key, value, expiration)
            .await
    }

    /// Add means "store this data, but only if the server *doesn't* already
    /// hold data for this key".
    pub async fn add<K: AsRef<[u8]>, T: Serialize, E>(
        &self,
        key: K,
        value: T,
        expiration: E,
    ) -> Result<parser::Status, MemcacheError>
    where
        E: Into<Option<Duration>>,
    {
        self.store(driver::StorageCommand::Add, key, value, expiration)
            .await
    }

    /// "replace" means "store this data, but only if the server *does*
    /// already hold data for this key".
    pub async fn replace<K: AsRef<[u8]>, T: Serialize, E>(
        &self,
        key: K,
        value: T,
        expiration: E,
    ) -> Result<parser::Status, MemcacheError>
    where
        E: Into<Option<Duration>>,
    {
        self.store(driver::StorageCommand::Replace, key, value, expiration)
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
    pub async fn touch<K: AsRef<[u8]>, E>(
        &self,
        key: K,
        expiration: E,
    ) -> Result<parser::Status, MemcacheError>
    where
        E: Into<Option<Duration>>,
    {
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
}
