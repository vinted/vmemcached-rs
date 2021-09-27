mod ascii;

use crate::client::Stats;
use crate::error::MemcacheError;
pub(crate) use crate::protocol::ascii::AsciiProtocol;
use crate::stream::Stream;
use crate::value::{FromMemcacheValueExt, ToMemcacheValue};
use std::collections::HashMap;

pub trait ProtocolTrait {
    fn auth(&mut self, username: &str, password: &str) -> Result<(), MemcacheError>;
    fn version(&mut self) -> Result<String, MemcacheError>;
    fn flush(&mut self) -> Result<(), MemcacheError>;
    fn flush_with_delay(&mut self, delay: u32) -> Result<(), MemcacheError>;
    fn get<K: AsRef<[u8]>, T: FromMemcacheValueExt>(&mut self, key: K) -> Result<Option<T>, MemcacheError>;
    fn gets<K, I, T>(&mut self, keys: I) -> Result<HashMap<String, T>, MemcacheError>
    where
        T: FromMemcacheValueExt,
        I: IntoIterator<Item = K>,
        K: AsRef<[u8]>;
    fn set<K: AsRef<[u8]>, T: ToMemcacheValue<Stream>>(
        &mut self,
        key: K,
        value: T,
        expiration: u32,
    ) -> Result<(), MemcacheError>;
    fn cas<K: AsRef<[u8]>, T: ToMemcacheValue<Stream>>(
        &mut self,
        key: K,
        value: T,
        expiration: u32,
        cas: u64,
    ) -> Result<bool, MemcacheError>;
    fn add<K: AsRef<[u8]>, T: ToMemcacheValue<Stream>>(
        &mut self,
        key: K,
        value: T,
        expiration: u32,
    ) -> Result<(), MemcacheError>;
    fn replace<K: AsRef<[u8]>, T: ToMemcacheValue<Stream>>(
        &mut self,
        key: K,
        value: T,
        expiration: u32,
    ) -> Result<(), MemcacheError>;
    fn append<K: AsRef<[u8]>, T: ToMemcacheValue<Stream>>(&mut self, key: K, value: T) -> Result<(), MemcacheError>;
    fn prepend<K: AsRef<[u8]>, T: ToMemcacheValue<Stream>>(&mut self, key: K, value: T) -> Result<(), MemcacheError>;
    fn delete<K: AsRef<[u8]>>(&mut self, key: K) -> Result<bool, MemcacheError>;
    fn increment<K: AsRef<[u8]>>(&mut self, key: K, amount: u64) -> Result<u64, MemcacheError>;
    fn decrement<K: AsRef<[u8]>>(&mut self, key: K, amount: u64) -> Result<u64, MemcacheError>;
    fn touch<K: AsRef<[u8]>>(&mut self, key: K, expiration: u32) -> Result<bool, MemcacheError>;
    fn stats(&mut self) -> Result<Stats, MemcacheError>;
}
