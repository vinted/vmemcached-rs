use async_trait::async_trait;
use std::convert::TryFrom;
use std::io;
use std::net::SocketAddr;
use tokio::io::{Interest, Ready};
use trust_dns_resolver::TokioAsyncResolver;
use trust_dns_resolver::{
    config::{ResolverConfig, ResolverOpts},
    system_conf::read_system_conf,
};
use url::Url;

use crate::connection::Connection;
use crate::MemcacheError;

/// A `bb8::ManageConnection` for `memcache_async::ascii::Protocol`.
#[derive(Clone, Debug)]
pub struct ConnectionManager {
    url: Url,
    resolver: TokioAsyncResolver,
}

impl ConnectionManager {
    /// Initialize ConnectionManager with given URL
    pub fn new(url: Url, resolver: TokioAsyncResolver) -> ConnectionManager {
        ConnectionManager { url, resolver }
    }
}

impl TryFrom<Url> for ConnectionManager {
    type Error = MemcacheError;

    fn try_from(value: Url) -> Result<Self, Self::Error> {
        let (config, opts) = read_system_conf()?;

        let resolver = TokioAsyncResolver::tokio(config, opts)?;

        Ok(Self::new(value, resolver))
    }
}

impl TryFrom<&str> for ConnectionManager {
    type Error = MemcacheError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let (config, opts) = read_system_conf()?;

        let resolver = TokioAsyncResolver::tokio(config, opts)?;

        Ok(Self::new(Url::parse(value)?, resolver))
    }
}

impl TryFrom<(&str, ResolverConfig, ResolverOpts)> for ConnectionManager {
    type Error = MemcacheError;

    fn try_from(value: (&str, ResolverConfig, ResolverOpts)) -> Result<Self, Self::Error> {
        let resolver = TokioAsyncResolver::tokio(value.1, value.2)?;

        Ok(Self::new(Url::parse(value.0)?, resolver))
    }
}

impl TryFrom<(Url, ResolverConfig, ResolverOpts)> for ConnectionManager {
    type Error = MemcacheError;

    fn try_from(value: (Url, ResolverConfig, ResolverOpts)) -> Result<Self, Self::Error> {
        let resolver = TokioAsyncResolver::tokio(value.1, value.2)?;

        Ok(Self::new(value.0, resolver))
    }
}

#[async_trait]
impl bb8::ManageConnection for ConnectionManager {
    type Connection = Connection;
    type Error = MemcacheError;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        let addresses = match self.url.domain() {
            Some(domain) => {
                let response = self.resolver.lookup_ip(domain).await?;

                let port = self.url.port().unwrap_or(11211);

                response
                    .iter()
                    .map(|address| SocketAddr::new(address, port))
                    .collect()
            }
            None => self.url.socket_addrs(|| None)?,
        };

        Connection::connect(&*addresses).await.map_err(Into::into)
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

#[cfg(test)]
mod tests {
    use url::Url;

    #[test]
    fn test_url_domain() {
        let link = Url::parse("https://with.sub.example.org:2993/").unwrap();
        assert_eq!(link.domain().unwrap(), "with.sub.example.org");
    }
}
