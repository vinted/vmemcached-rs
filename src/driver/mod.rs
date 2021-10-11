use bytes::BytesMut;
use std::io;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::parser::{Response, Value};
use crate::{parser, MemcacheError, PoolConnection};

const EMPTY_SPACE_BYTES: &[u8] = b" ";
const NEW_LINE_BYTES: &[u8] = b"\r\n";
const NO_REPLY_BYTES: &[u8] = b" noreply\r\n";
const COMMAND_DELETE: &[u8] = b"delete ";
const COMMAND_TOUCH: &[u8] = b"touch ";
const COMMAND_VERSION: &[u8] = b"version\r\n";

// 128 bytes should be enough to address all storage responses
const RESPONSE_BUFFER_BYTES: usize = 128;

/// Storage command
#[derive(Debug)]
pub enum StorageCommand {
    /// "set" means "store this data".
    Set,
    /// "add" means "store this data, but only if the server *doesn't* already
    /// hold data for this key".
    Add,
    /// "replace" means "store this data, but only if the server *does*
    /// already hold data for this key".
    Replace,
}

impl From<StorageCommand> for &'static [u8] {
    fn from(c: StorageCommand) -> &'static [u8] {
        match c {
            StorageCommand::Set => b"set ",
            StorageCommand::Add => b"add ",
            StorageCommand::Replace => b"replace ",
        }
    }
}

/// <command name> <key> <flags> <exptime> <bytes> [noreply]\r\n
///
///
/// - "STORED\r\n", to indicate success.
///
/// - "NOT_STORED\r\n" to indicate the data was not stored, but not
/// because of an error. This normally means that the
/// condition for an "add" or a "replace" command wasn't met.
///
/// - "EXISTS\r\n" to indicate that the item you are trying to store with
/// a "cas" command has been modified since you last fetched it.
///
/// - "NOT_FOUND\r\n" to indicate that the item you are trying to store
/// with a "cas" command did not exist.
pub async fn storage<K, E>(
    mut conn: PoolConnection<'_>,
    command: StorageCommand,
    key: K,
    flags: u32,
    expiration: E,
    bytes: Vec<u8>,
    noreply: bool,
) -> Result<Response, MemcacheError>
where
    K: AsRef<[u8]>,
    E: Into<Option<Duration>>,
{
    // <command name>
    let _ = conn.write(command.into()).await?;
    // <key>
    let _ = conn.write_all(key.as_ref()).await?;
    let _ = conn.write(EMPTY_SPACE_BYTES).await?;

    // <flags>
    let _ = conn.write(flags.to_string().as_ref()).await?;
    let _ = conn.write(EMPTY_SPACE_BYTES).await?;

    // <exptime>
    let exptime = expiration.into().map(|d| d.as_secs()).unwrap_or(0);
    let _ = conn.write(exptime.to_string().as_ref()).await?;
    let _ = conn.write(EMPTY_SPACE_BYTES).await?;

    // <bytes>
    let _ = conn.write(bytes.len().to_string().as_bytes()).await?;

    // [noreply]
    if noreply {
        // FYI: NO_REPLY_BYTES contains space before and new line after
        let _ = conn.write(NO_REPLY_BYTES).await?;
    } else {
        let _ = conn.write(NEW_LINE_BYTES).await?;
    }

    // <data block>
    let _ = conn.write_all(&bytes).await?;
    let _ = conn.write(NEW_LINE_BYTES).await?;

    // Flush command
    let _ = conn.flush().await?;

    let mut buffer: BytesMut = BytesMut::with_capacity(RESPONSE_BUFFER_BYTES);

    if conn.read_buf(&mut buffer).await? == 0 {
        return Err(io::ErrorKind::UnexpectedEof.into());
    }

    match parser::parse_ascii_status(&buffer) {
        Ok((_left, result)) => Ok(result),
        Err(e) => Err(MemcacheError::Nom(format!("{}", e))),
    }
}

/// Retrieval command
#[derive(Debug)]
pub enum RetrievalCommand {
    /// "get" means "get this data".
    Get,
    /// "gets" means "get multiple data".
    Gets,
}

impl From<RetrievalCommand> for &'static [u8] {
    fn from(c: RetrievalCommand) -> &'static [u8] {
        match c {
            RetrievalCommand::Get => b"get",
            RetrievalCommand::Gets => b"gets",
        }
    }
}

/// get <key>*\r\n
/// gets <key>*\r\n
///
///
/// VALUE <key> <flags> <bytes> [<cas unique>]\r\n
/// <data block>\r\n
/// VALUE <key> <flags> <bytes> [<cas unique>]\r\n
/// <data block>\r\n
/// VALUE <key> <flags> <bytes> [<cas unique>]\r\n
/// <data block>\r\n
/// "END\r\n"
pub async fn retrieve<K>(
    mut conn: PoolConnection<'_>,
    command: RetrievalCommand,
    keys: &[K],
) -> Result<Option<Vec<Value>>, MemcacheError>
where
    K: AsRef<[u8]>,
{
    debug_assert!(!keys.is_empty());
    // <command name>
    let _ = conn.write(command.into()).await?;

    // <key>
    for key in &*keys {
        let _ = conn.write(EMPTY_SPACE_BYTES).await?; // ends key without empty space
        let _ = conn.write_all(key.as_ref()).await?;
    }
    let _ = conn.write(NEW_LINE_BYTES).await?;

    // Flush command
    let _ = conn.flush().await?;

    let mut buffer: BytesMut = BytesMut::with_capacity(1024);

    loop {
        if conn.read_buf(&mut buffer).await? == 0 {
            return Err(io::ErrorKind::UnexpectedEof.into());
        }

        match parser::parse_ascii_response(&buffer) {
            Ok(Some((_n, response))) => match response {
                Response::Data(values) => {
                    return if values.is_empty() {
                        Ok(None)
                    } else {
                        Ok(Some(values))
                    };
                }
                Response::Error(e) => return Err(MemcacheError::Memcache(e)),
                _ => return Ok(None),
            },
            Ok(None) => {
                buffer.reserve(1024);
                continue;
            }
            Err(e) => return Err(MemcacheError::Nom(format!("{}", e))),
        }
    }
}

/// delete <key> [noreply]\r\n
///
///
/// - "DELETED\r\n" to indicate success
///
/// - "NOT_FOUND\r\n" to indicate that the item with this key was not
///   found.
pub async fn delete<K>(
    mut conn: PoolConnection<'_>,
    key: K,
    noreply: bool,
) -> Result<Response, MemcacheError>
where
    K: AsRef<[u8]>,
{
    // <command name>
    let _ = conn.write(COMMAND_DELETE).await?;
    // <key>
    let _ = conn.write_all(key.as_ref()).await?;

    // [noreply]
    if noreply {
        // FYI: NO_REPLY_BYTES contains space before and new line after
        let _ = conn.write(NO_REPLY_BYTES).await?;
    } else {
        let _ = conn.write(NEW_LINE_BYTES).await?;
    }

    // Flush command
    let _ = conn.flush().await?;

    let mut buffer: BytesMut = BytesMut::with_capacity(RESPONSE_BUFFER_BYTES);

    if conn.read_buf(&mut buffer).await? == 0 {
        return Err(io::ErrorKind::UnexpectedEof.into());
    }

    match parser::parse_ascii_status(&buffer) {
        Ok((_left, result)) => Ok(result),
        Err(e) => Err(MemcacheError::Nom(format!("{}", e))),
    }
}

/// touch <key> <exptime> [noreply]\r\n
///
///
/// The response line to this command can be one of:
///
/// - "TOUCHED\r\n" to indicate success
///
/// - "NOT_FOUND\r\n" to indicate that the item with this key was not
///   found.
pub async fn touch<K, E>(
    mut conn: PoolConnection<'_>,
    key: K,
    expiration: E,
    noreply: bool,
) -> Result<Response, MemcacheError>
where
    K: AsRef<[u8]>,

    E: Into<Option<Duration>>,
{
    // <command name>
    let _ = conn.write(COMMAND_TOUCH).await?;
    // <key>
    let _ = conn.write_all(key.as_ref()).await?;
    let _ = conn.write(EMPTY_SPACE_BYTES).await?;

    // <exptime>
    let exptime = expiration.into().map(|d| d.as_secs()).unwrap_or(0);
    let _ = conn.write(exptime.to_string().as_ref()).await?;
    let _ = conn.write(EMPTY_SPACE_BYTES).await?;

    // [noreply]
    if noreply {
        // FYI: NO_REPLY_BYTES contains space before and new line after
        let _ = conn.write(NO_REPLY_BYTES).await?;
    } else {
        let _ = conn.write(NEW_LINE_BYTES).await?;
    }

    // Flush command
    let _ = conn.flush().await?;

    let mut buffer: BytesMut = BytesMut::with_capacity(RESPONSE_BUFFER_BYTES);

    if conn.read_buf(&mut buffer).await? == 0 {
        return Err(io::ErrorKind::UnexpectedEof.into());
    }

    match parser::parse_ascii_status(&buffer) {
        Ok((_left, result)) => Ok(result),
        Err(e) => Err(MemcacheError::Nom(format!("{}", e))),
    }
}

/// version\r\n
///
///
/// "VERSION <version>\r\n", where <version> is the version string for the
pub async fn version(conn: &mut PoolConnection<'_>) -> Result<String, MemcacheError> {
    // <command name>
    let _ = conn.write(COMMAND_VERSION).await?;

    // Flush command
    let _ = conn.flush().await?;

    let mut buffer: BytesMut = BytesMut::with_capacity(RESPONSE_BUFFER_BYTES);

    if conn.read_buf(&mut buffer).await? == 0 {
        return Err(io::ErrorKind::UnexpectedEof.into());
    }

    match parser::parse_version(&buffer) {
        Ok((_left, result)) => Ok(result),
        Err(e) => Err(MemcacheError::Nom(format!("{}", e))),
    }
}
