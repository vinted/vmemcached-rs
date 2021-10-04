use bytes::BytesMut;
use serde::Serialize;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::parser::{ErrorKind, Status};
use crate::{parser, MemcacheError, PoolConnection};

const EMPTY_BYTES: &[u8; 1] = b" ";
const NEW_LINE_BYTES: &[u8; 2] = b"\r\n";
const NO_REPLY_BYTES: &[u8; 10] = b" noreply\r\n";

pub enum Command {
    Set,
    Get,
}

impl From<Command> for &'static [u8] {
    fn from(c: Command) -> &'static [u8] {
        match c {
            Command::Set => b"set ",
            Command::Get => b"get ",
        }
    }
}

// <command name> <key> <flags> <exptime> <bytes> [noreply]\r\n
pub async fn store_command<K>(
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
        Ok((_n, result)) => Ok(result),
        Err(e) => Err(MemcacheError::Nom(format!("{}", e))),
    }
}
