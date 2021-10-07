#[cfg(feature = "compress")]
mod compress {
    use crate::error::MemcacheError;
    use serde::de::DeserializeOwned;
    use serde::Serialize;
    use std::io::{Cursor, Write};

    pub(crate) fn encode<T: Serialize>(value: T) -> Result<Vec<u8>, MemcacheError> {
        let encoded = simd_json::to_vec(&value)?;

        let mut writer = brotli::CompressorWriter::new(Vec::new(), 2048, 11, 22);
        let _ = writer.write_all(&encoded)?;
        Ok(writer.into_inner())
    }

    pub(crate) fn decode<T: DeserializeOwned>(input: Vec<u8>) -> Result<T, MemcacheError> {
        let mut output = Vec::new();
        let _ = brotli::BrotliDecompress(&mut Cursor::new(input), &mut output)?;
        Ok(simd_json::from_slice(&mut output)?)
    }
}

#[cfg(not(feature = "compress"))]
mod plain {
    use crate::error::MemcacheError;
    use serde::de::DeserializeOwned;
    use serde::Serialize;

    pub(crate) fn encode<T: Serialize>(value: T) -> Result<Vec<u8>, MemcacheError> {
        Ok(simd_json::to_vec(&value)?)
    }

    pub(crate) fn decode<T: DeserializeOwned>(mut value: Vec<u8>) -> Result<T, MemcacheError> {
        Ok(simd_json::from_slice(value.as_mut_slice())?)
    }
}

#[cfg(feature = "compress")]
pub(crate) use compress::*;

#[cfg(not(feature = "compress"))]
pub(crate) use plain::*;

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct A {
        field: String,
    }

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct B {
        field: String,
    }

    #[derive(Serialize, Deserialize, Debug, PartialEq, Default)]
    struct C {
        field: String,
        #[serde(default)]
        flag: bool,
    }

    #[test]
    fn test_simd_json_serde_annotations() {
        let a = A {
            field: "a_struct".into(),
        };

        let b = B {
            field: "B_struct".into(),
        };

        let mut a_encoded = simd_json::to_vec(&a).unwrap();
        let mut b_encoded = simd_json::to_vec(&b).unwrap();

        let b_decoded_from_a: Result<B, simd_json::Error> = simd_json::from_slice(&mut a_encoded);
        assert!(b_decoded_from_a.is_ok());

        let a_decoded_from_b: Result<A, simd_json::Error> = simd_json::from_slice(&mut b_encoded);
        assert!(a_decoded_from_b.is_ok());

        let c_decoded_from_b: Result<C, simd_json::Error> = simd_json::from_slice(&mut b_encoded);
        assert!(
            c_decoded_from_b.is_ok(),
            "simd_json::Error: {:?}",
            c_decoded_from_b
        );
    }
}
