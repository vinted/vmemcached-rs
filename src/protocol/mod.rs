use serde::de::DeserializeOwned;
use serde::Serialize;

mod ascii;

use crate::client::Stats;
use crate::error::MemcacheError;
pub(crate) use crate::protocol::ascii::AsciiProtocol;

pub(crate) trait ProtocolTrait {
    fn auth(&mut self, username: &str, password: &str) -> Result<(), MemcacheError>;
    fn version(&mut self) -> Result<String, MemcacheError>;
    #[cfg(not(feature = "mcrouter"))]
    fn flush(&mut self) -> Result<(), MemcacheError>;
    #[cfg(not(feature = "mcrouter"))]
    fn flush_with_delay(&mut self, delay: u32) -> Result<(), MemcacheError>;
    fn get<K: AsRef<[u8]>, T: DeserializeOwned>(&mut self, key: K) -> Result<Option<T>, MemcacheError>;
    fn set<K: AsRef<[u8]>, T: Serialize>(&mut self, key: K, value: T, expiration: u32) -> Result<(), MemcacheError>;
    fn add<K: AsRef<[u8]>, T: Serialize>(&mut self, key: K, value: T, expiration: u32) -> Result<(), MemcacheError>;
    fn replace<K: AsRef<[u8]>, T: Serialize>(&mut self, key: K, value: T, expiration: u32)
        -> Result<(), MemcacheError>;
    fn delete<K: AsRef<[u8]>>(&mut self, key: K) -> Result<bool, MemcacheError>;
    fn touch<K: AsRef<[u8]>>(&mut self, key: K, expiration: u32) -> Result<bool, MemcacheError>;
    fn stats(&mut self) -> Result<Stats, MemcacheError>;
}
