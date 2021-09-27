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
    fn set<V: ToMemcacheValue<Stream>>(&mut self, key: &str, value: V, expiration: u32) -> Result<(), MemcacheError>;
    fn cas<V: ToMemcacheValue<Stream>>(
        &mut self,
        key: &str,
        value: V,
        expiration: u32,
        cas: u64,
    ) -> Result<bool, MemcacheError>;
    fn add<V: ToMemcacheValue<Stream>>(&mut self, key: &str, value: V, expiration: u32) -> Result<(), MemcacheError>;
    fn replace<V: ToMemcacheValue<Stream>>(
        &mut self,
        key: &str,
        value: V,
        expiration: u32,
    ) -> Result<(), MemcacheError>;
    fn append<V: ToMemcacheValue<Stream>>(&mut self, key: &str, value: V) -> Result<(), MemcacheError>;
    fn prepend<V: ToMemcacheValue<Stream>>(&mut self, key: &str, value: V) -> Result<(), MemcacheError>;
    fn delete(&mut self, key: &str) -> Result<bool, MemcacheError>;
    fn increment(&mut self, key: &str, amount: u64) -> Result<u64, MemcacheError>;
    fn decrement(&mut self, key: &str, amount: u64) -> Result<u64, MemcacheError>;
    fn touch(&mut self, key: &str, expiration: u32) -> Result<bool, MemcacheError>;
    fn stats(&mut self) -> Result<Stats, MemcacheError>;
}
