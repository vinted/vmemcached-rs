#[cfg(feature = "compress")]
mod compress {
    use crate::error::MemcacheError;
    use serde::de::DeserializeOwned;
    use serde::Serialize;
    use std::io::{Cursor, Write};

    pub(crate) fn encode<T: Serialize>(value: T) -> Result<Vec<u8>, MemcacheError> {
        let encoded = serde_json::to_vec(&value)?;

        let mut writer = brotli::CompressorWriter::new(Vec::new(), 2048, 11, 22);
        let _ = writer.write_all(&encoded)?;
        Ok(writer.into_inner())
    }

    pub(crate) fn decode<T: DeserializeOwned>(input: Vec<u8>) -> Result<T, MemcacheError> {
        let mut output = Vec::new();
        let _ = brotli::BrotliDecompress(&mut Cursor::new(input), &mut output)?;
        Ok(serde_json::from_slice(&mut output)?)
    }
}

#[cfg(not(feature = "compress"))]
mod plain {
    use crate::error::MemcacheError;
    use serde::de::DeserializeOwned;
    use serde::Serialize;

    pub(crate) fn encode<T: Serialize>(value: T) -> Result<Vec<u8>, MemcacheError> {
        Ok(serde_json::to_vec(&value)?)
    }

    pub(crate) fn decode<T: DeserializeOwned>(value: Vec<u8>) -> Result<T, MemcacheError> {
        Ok(serde_json::from_slice(&value)?)
    }
}

#[cfg(feature = "compress")]
pub(crate) use compress::*;

#[cfg(not(feature = "compress"))]
pub(crate) use plain::*;
