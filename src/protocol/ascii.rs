use serde::de::DeserializeOwned;
use serde::Serialize;
use std::collections::HashMap;
use std::fmt;
use std::io::{Read, Write};
use std::time::Duration;

use super::ProtocolTrait;
use crate::client::Stats;
use crate::codec;
use crate::error::{ClientError, CommandError, MemcacheError, ServerError};
use crate::stream::Stream;
use std::borrow::Cow;

#[derive(Default)]
pub(crate) struct Options {
    pub(crate) noreply: bool,
    pub(crate) exptime: Duration,
}

#[derive(PartialEq)]
enum StoreCommand {
    Set,
    Add,
    Replace,
}

const END: &str = "END\r\n";

impl fmt::Display for StoreCommand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            StoreCommand::Set => write!(f, "set"),
            StoreCommand::Add => write!(f, "add"),
            StoreCommand::Replace => write!(f, "replace"),
        }
    }
}

#[derive(Debug)]
struct CappedLineReader<C> {
    inner: C,
    filled: usize,
    buf: [u8; 2048],
}

fn get_line(buf: &[u8]) -> Option<usize> {
    for (i, r) in buf.iter().enumerate() {
        if *r == b'\r' && buf.get(i + 1) == Some(&b'\n') {
            return Some(i + 2);
        }
    }
    None
}

impl<C: Read> CappedLineReader<C> {
    fn new(inner: C) -> Self {
        Self {
            inner,
            filled: 0,
            buf: [0x0; 2048],
        }
    }

    pub(crate) fn get_mut(&mut self) -> &mut C {
        &mut self.inner
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), MemcacheError> {
        let min = std::cmp::min(buf.len(), self.filled);
        let (to_fill, rest) = buf.split_at_mut(min);
        to_fill.copy_from_slice(&self.buf[..min]);
        self.consume(min);
        if !rest.is_empty() {
            self.inner.read_exact(&mut *rest)?;
        }
        Ok(())
    }

    /// Try to read a CRLF terminated line from the underlying reader.
    /// The length of the line is expected to be <= the length of the
    /// internal buffer, suited for reading headers or short responses.
    fn read_line<T, F>(&mut self, mut cb: F) -> Result<T, MemcacheError>
    where
        F: FnMut(&str) -> Result<T, MemcacheError>,
    {
        // check if the buf already has a new line
        if let Some(n) = get_line(&self.buf[..self.filled]) {
            let result = cb(std::str::from_utf8(&self.buf[..n])?);
            self.consume(n);
            return result;
        }
        loop {
            let (filled, buf) = self.buf.split_at_mut(self.filled);
            if buf.is_empty() {
                return Err(ClientError::Error(Cow::Borrowed("Ascii protocol response too long")).into());
            }
            let filled = filled.len();
            let read = self.inner.read(&mut *buf)?;
            if read == 0 {
                return Err(ClientError::Error(Cow::Borrowed("Ascii protocol no line found")).into());
            }
            self.filled += read;
            if let Some(n) = get_line(&buf[..read]) {
                let result = cb(std::str::from_utf8(&self.buf[..filled + n])?);
                self.consume(n);
                return result;
            }
        }
    }

    fn consume(&mut self, amount: usize) {
        let amount = std::cmp::min(self.filled, amount);
        self.buf.copy_within(amount..self.filled, 0);
        self.filled -= amount;
    }
}

#[derive(Debug)]
pub struct AsciiProtocol<C: Read + Write + Sized> {
    reader: CappedLineReader<C>,
}

impl ProtocolTrait for AsciiProtocol<Stream> {
    fn auth(&mut self, username: &str, password: &str) -> Result<(), MemcacheError> {
        return self.set("auth", format!("{} {}", username, password), Duration::from_secs(0));
    }

    fn version(&mut self) -> Result<String, MemcacheError> {
        let _ = self.reader.get_mut().write(b"version\r\n")?;
        self.reader.get_mut().flush()?;
        self.reader.read_line(|response| {
            let response = MemcacheError::try_from(response)?;
            if !response.starts_with("VERSION") {
                return Err(ServerError::BadResponse(Cow::Owned(response.into())).into());
            }
            let version = response.trim_start_matches("VERSION ").trim_end_matches("\r\n");
            Ok(version.to_string())
        })
    }

    #[cfg(not(feature = "mcrouter"))]
    fn flush(&mut self) -> Result<(), MemcacheError> {
        write!(self.reader.get_mut(), "flush_all\r\n")?;
        self.parse_ok_response()
    }

    #[cfg(not(feature = "mcrouter"))]
    fn flush_with_delay(&mut self, delay: u32) -> Result<(), MemcacheError> {
        write!(self.reader.get_mut(), "flush_all {}\r\n", delay)?;
        self.reader.get_mut().flush()?;
        self.parse_ok_response()
    }

    fn get<K: AsRef<[u8]>, T: DeserializeOwned>(&mut self, key: K) -> Result<Option<T>, MemcacheError> {
        let reader = self.reader.get_mut();
        let _ = reader.write(b"get ");
        let _ = reader.write(key.as_ref());
        let _ = reader.write(b"\r\n");

        if let Some((k, v)) = self.parse_get_response()? {
            if k.as_bytes() != key.as_ref() {
                return Err(ServerError::BadResponse(Cow::Borrowed("key doesn't match in the response")).into());
            } else if self.parse_get_response::<T>()?.is_none() {
                Ok(Some(v))
            } else {
                return Err(ServerError::BadResponse(Cow::Borrowed("Expected end of get response")).into());
            }
        } else {
            Ok(None)
        }
    }

    fn set<K: AsRef<[u8]>, T: Serialize>(
        &mut self,
        key: K,
        value: T,
        expiration: Duration,
    ) -> Result<(), MemcacheError> {
        let options = Options {
            exptime: expiration,
            noreply: true,
            ..Default::default()
        };
        self.store(StoreCommand::Set, key.as_ref(), value, &options).map(|_| ())
    }

    fn add<K: AsRef<[u8]>, T: Serialize>(
        &mut self,
        key: K,
        value: T,
        expiration: Duration,
    ) -> Result<(), MemcacheError> {
        let options = Options {
            exptime: expiration,
            ..Default::default()
        };
        self.store(StoreCommand::Add, key, value, &options).map(|_| ())
    }

    fn replace<K: AsRef<[u8]>, T: Serialize>(
        &mut self,
        key: K,
        value: T,
        expiration: Duration,
    ) -> Result<(), MemcacheError> {
        let options = Options {
            exptime: expiration,
            ..Default::default()
        };
        self.store(StoreCommand::Replace, key, value, &options).map(|_| ())
    }

    fn delete<K: AsRef<[u8]>>(&mut self, key: K) -> Result<bool, MemcacheError> {
        let reader = self.reader.get_mut();
        let _ = reader.write(b"delete ");
        let _ = reader.write(key.as_ref())?;
        let _ = reader.write(b"\r\n");
        reader.flush()?;
        self.reader
            .read_line(|response| match MemcacheError::try_from(response) {
                Ok(s) => {
                    if s == "DELETED\r\n" {
                        Ok(true)
                    } else {
                        Err(ServerError::BadResponse(Cow::Owned(s.into())).into())
                    }
                }
                Err(MemcacheError::CommandError(CommandError::KeyNotFound)) => Ok(false),
                Err(e) => Err(e),
            })
    }

    fn touch<K: AsRef<[u8]>>(&mut self, key: K, expiration: Duration) -> Result<bool, MemcacheError> {
        let reader = self.reader.get_mut();
        let _ = reader.write(b"touch ")?;
        let _ = reader.write(key.as_ref())?;
        write!(reader, " {}\r\n", expiration.as_secs())?;
        reader.flush()?;
        self.reader
            .read_line(|response| match MemcacheError::try_from(response) {
                Ok(s) => {
                    if s == "TOUCHED\r\n" {
                        Ok(true)
                    } else {
                        Err(ServerError::BadResponse(Cow::Owned(s.into())).into())
                    }
                }
                Err(MemcacheError::CommandError(CommandError::KeyNotFound)) => Ok(false),
                Err(e) => Err(e),
            })
    }

    fn stats(&mut self) -> Result<Stats, MemcacheError> {
        let reader = self.reader.get_mut();
        let _ = reader.write(b"stats\r\n")?;
        reader.flush()?;

        enum Loop {
            Break,
            Continue,
        }

        let mut stats: Stats = HashMap::new();
        loop {
            let status = self.reader.read_line(|response| {
                if response != END {
                    return Ok(Loop::Break);
                }
                let s = MemcacheError::try_from(response)?;
                if !s.starts_with("STAT") {
                    return Err(ServerError::BadResponse(Cow::Owned(s.into())).into());
                }
                let stat: Vec<_> = s.trim_end_matches("\r\n").split(' ').collect();
                if stat.len() < 3 {
                    return Err(ServerError::BadResponse(Cow::Owned(s.into())).into());
                }
                let key = stat[1];
                let value = s.trim_start_matches(format!("STAT {}", key).as_str());
                let _ = stats.insert(key.into(), value.into());

                Ok(Loop::Continue)
            })?;

            if let Loop::Break = status {
                break Ok(stats);
            }
        }
    }
}

impl AsciiProtocol<Stream> {
    pub(crate) fn new(stream: Stream) -> Self {
        Self {
            reader: CappedLineReader::new(stream),
        }
    }

    fn store<K: AsRef<[u8]>, T: Serialize>(
        &mut self,
        command: StoreCommand,
        key: K,
        value: T,
        options: &Options,
    ) -> Result<bool, MemcacheError> {
        let encoded = codec::encode(value)?;

        let noreply = if options.noreply { " noreply" } else { "" };
        let reader = self.reader.get_mut();
        let _ = reader.write(format!("{} ", command).as_ref())?;
        let _ = reader.write(key.as_ref())?;
        write!(
            reader,
            " {flags} {exptime} {vlen}{noreply}\r\n",
            flags = 0,
            exptime = options.exptime.as_secs(),
            vlen = encoded.len(),
            noreply = noreply
        )?;

        let _ = reader.write_all(&encoded)?;
        let _ = reader.write(b"\r\n")?;
        let _ = reader.flush()?;

        if options.noreply {
            return Ok(true);
        }

        self.reader.read_line(|response| {
            let response = MemcacheError::try_from(response)?;
            match response {
                "STORED\r\n" => Ok(true),
                "NOT_STORED\r\n" => Ok(false),
                "EXISTS\r\n" => Err(CommandError::KeyExists.into()),
                "NOT_FOUND\r\n" => Err(CommandError::KeyNotFound.into()),
                response => return Err(ServerError::BadResponse(Cow::Owned(response.into())).into()),
            }
        })
    }

    #[cfg(not(feature = "mcrouter"))]
    fn parse_ok_response(&mut self) -> Result<(), MemcacheError> {
        self.reader.read_line(|response| {
            let response = MemcacheError::try_from(response)?;
            if response == "OK\r\n" {
                Ok(())
            } else {
                return Err(ServerError::BadResponse(Cow::Owned(response.into())).into());
            }
        })
    }

    // TODO: fix this with nom parser #589a806
    fn parse_get_response<T: DeserializeOwned>(&mut self) -> Result<Option<(String, T)>, MemcacheError> {
        let result = self.reader.read_line(|buf| {
            let buf = MemcacheError::try_from(buf)?;
            if buf == END {
                return Ok(None);
            }
            if !buf.starts_with("VALUE") {
                return Err(ServerError::BadResponse(Cow::Owned(buf.into())).into());
            }
            let mut header = buf.trim_end_matches("\r\n").split(' ');
            let mut next_or_err = || {
                header
                    .next()
                    .ok_or_else(|| ServerError::BadResponse(Cow::Owned(buf.into())))
            };
            let _ = next_or_err()?;
            let key = next_or_err()?;
            let flags: u32 = next_or_err()?.parse()?;
            let length: usize = next_or_err()?.parse()?;
            if header.next().is_some() {
                return Err(ServerError::BadResponse(Cow::Owned(buf.into())).into());
            }
            Ok(Some((key.to_string(), flags, length)))
        })?;
        match result {
            Some((key, _flags, length)) => {
                let mut value = vec![0u8; length + 2];
                self.reader.read_exact(value.as_mut_slice())?;
                if &value[length..] != b"\r\n" {
                    return Err(ServerError::BadResponse(Cow::Owned(String::from_utf8(value)?)).into());
                }
                // remove the trailing \r\n
                let _ = value.pop();
                let _ = value.pop();
                value.shrink_to_fit();
                let value = codec::decode(value)?;
                Ok(Some((key, value)))
            }
            None => Ok(None),
        }
    }
}
