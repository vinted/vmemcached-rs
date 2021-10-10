use async_trait::async_trait;
use std::convert::TryFrom;
use std::io;
use tokio::io::{Interest, Ready};
use url::Url;

use crate::connection::Connection;
use crate::MemcacheError;

/// A `bb8::ManageConnection` for `memcache_async::ascii::Protocol`.
#[derive(Clone, Debug)]
pub struct ConnectionManager {
    url: Url,
}

impl ConnectionManager {
    /// Initialize ConnectionManager with given URL
    pub fn new(url: Url) -> ConnectionManager {
        ConnectionManager { url }
    }
}

impl TryFrom<&str> for ConnectionManager {
    type Error = url::ParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(Self::new(Url::parse(value)?))
    }
}

#[async_trait]
impl bb8::ManageConnection for ConnectionManager {
    type Connection = Connection;
    type Error = MemcacheError;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        Connection::connect(&*self.url.socket_addrs(|| None)?)
            .await
            .map_err(Into::into)
    }

    async fn is_valid(
        &self,
        conn: &mut bb8::PooledConnection<'_, Self>,
    ) -> Result<(), Self::Error> {
        let ready = conn
            .get_ref()
            .ready(Interest::READABLE | Interest::WRITABLE)
            .await?;

        // Check connection for all states: READABLE | WRITABLE | READ_CLOSED | WRITE_CLOSED
        if ready == Ready::ALL {
            Ok(())
        } else {
            Err(io::ErrorKind::UnexpectedEof.into())
        }
    }

    fn has_broken(&self, conn: &mut Self::Connection) -> bool {
        conn.has_broken()
    }
}
