//! Vinted Rust memcache

#[cfg(feature = "tls")]
extern crate openssl;

mod client;
mod codec;
mod connection;
mod error;
mod protocol;
mod stream;
mod value;

pub use crate::client::Client;
pub use crate::connection::ConnectionManager;
pub use crate::error::{ClientError, CommandError, MemcacheError, ServerError};
pub use crate::stream::Stream;
pub use crate::value::{FromMemcacheValue, ToMemcacheValue};
pub use r2d2::Error as PoolError;

/// R2D2 connection pool
pub type Pool = r2d2::Pool<connection::ConnectionManager>;
