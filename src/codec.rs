use crate::error::MemcacheError;
use serde::de::DeserializeOwned;
use serde::Serialize;

pub(crate) fn encode<T: Serialize>(value: T) -> Result<Vec<u8>, MemcacheError> {
    Ok(bincode::serialize(&value)?)
}

pub(crate) fn decode<T: DeserializeOwned>(value: Vec<u8>) -> Result<T, MemcacheError> {
    Ok(bincode::deserialize(&value)?)
}
