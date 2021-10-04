use bytes::BytesMut;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::parser::{Response, Status};
use crate::{codec, parser, MemcacheError, PoolConnection};

const EMPTY_BYTES: &[u8; 1] = b" ";
const NEW_LINE_BYTES: &[u8; 2] = b"\r\n";
const NO_REPLY_BYTES: &[u8; 10] = b" noreply\r\n";

pub enum Command {
    Set,
    Get,
    Add,
    Replace,
}

impl From<Command> for &'static [u8] {
    fn from(c: Command) -> &'static [u8] {
        match c {
            Command::Set => b"set ",
            Command::Get => b"get ",
            Command::Add => b"add ",
            Command::Replace => b"replace ",
        }
    }
}

// <command name> <key> <flags> <exptime> <bytes> [noreply]\r\n
pub async fn store<K>(
    mut conn: PoolConnection<'_>,
    command: Command,
    key: K,
    flags: u32,
    expiration: impl Into<Option<Duration>>,
    bytes: Vec<u8>,
    noreply: bool,
) -> Result<Status, MemcacheError>
where
    K: AsRef<[u8]>,
{
    // <command name>
    let _ = conn.write(command.into()).await?;
    // <key>
    let _ = conn.write_all(key.as_ref()).await?;
    let _ = conn.write(EMPTY_BYTES).await?;

    // <flags>
    let _ = conn.write(flags.to_string().as_ref()).await?;
    let _ = conn.write(EMPTY_BYTES).await?;

    // <exptime>
    let exptime = expiration.into().map(|d| d.as_secs()).unwrap_or(0);
    let _ = conn.write(exptime.to_string().as_ref()).await?;
    let _ = conn.write(EMPTY_BYTES).await?;

    // <bytes>
    let _ = conn.write(bytes.len().to_string().as_bytes()).await?;

    // [noreply]
    if noreply {
        let _ = conn.write(NO_REPLY_BYTES).await?;
    } else {
        let _ = conn.write(NEW_LINE_BYTES).await?;
    }

    // <data block>
    let _ = conn.write_all(&bytes).await?;
    let _ = conn.write(NEW_LINE_BYTES).await?;

    // Flush command
    let _ = conn.flush().await?;

    let mut buffer: BytesMut = BytesMut::with_capacity(64);

    let _ = conn.read_buf(&mut buffer).await?;

    match parser::parse_ascii_status(&buffer) {
        Ok((_left, result)) => match result {
            Response::Status(s) => Ok(s),
            _ => unreachable!(),
        },
        Err(e) => Err(MemcacheError::Nom(format!("{}", e))),
    }
}

// get <key>*\r\n
// gets <key>*\r\n
//
//
// VALUE <key> <flags> <bytes> [<cas unique>]\r\n
// <data block>\r\n
// "END\r\n"
pub async fn retrieve<K, V: DeserializeOwned>(
    mut conn: PoolConnection<'_>,
    command: Command,
    key: K,
) -> Result<Option<V>, MemcacheError>
where
    K: AsRef<[u8]>,
{
    // <command name>
    let _ = conn.write(command.into()).await?;
    // <key>
    let _ = conn.write_all(key.as_ref()).await?;
    let _ = conn.write(NEW_LINE_BYTES).await?;

    // Flush command
    let _ = conn.flush().await?;

    let mut buffer: BytesMut = BytesMut::with_capacity(1024);

    loop {
        let _bytes_read = conn.read_buf(&mut buffer).await?;

        match parser::parse_ascii_response(&buffer) {
            Ok(Some((_n, response))) => match response {
                Response::Data(Some(mut values)) => {
                    let value = values.remove(0);
                    return Ok(Some(codec::decode(value.data)?));
                }
                Response::Data(None) => return Ok(None),
                _ => unreachable!(),
            },
            Ok(None) => {
                buffer.reserve(1024);
                continue;
            }
            Err(e) => {
                //

                return Err(MemcacheError::Nom(format!("{}", e)));
            }
        }
    }
}
