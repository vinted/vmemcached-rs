use std::borrow::Cow;
use std::error;
use std::fmt;
use std::io;
use std::str;
use std::string;

use crate::parser;

/// Stands for errors raised from vmemcached
#[derive(Debug)]
pub enum MemcacheError {
    /// URL parse error
    UrlError(url::ParseError),
    /// `std::io` related errors.
    Io(io::Error),
    /// Client Errors
    ClientError(ClientError),
    /// Parse errors
    Utf8Error(string::FromUtf8Error),
    /// ConnectionPool errors
    PoolError(bb8::RunError<io::Error>),
    /// SIMD JSON error
    Serde(simd_json::Error),
    /// Nom error
    Nom(String),
    /// Memcache error
    Memcache(parser::ErrorKind),
}

impl fmt::Display for MemcacheError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            MemcacheError::Io(ref err) => err.fmt(f),
            MemcacheError::Utf8Error(ref err) => err.fmt(f),
            MemcacheError::ClientError(ref err) => err.fmt(f),
            MemcacheError::PoolError(ref err) => err.fmt(f),
            MemcacheError::Serde(ref err) => err.fmt(f),
            MemcacheError::Nom(ref err) => err.fmt(f),
            MemcacheError::Memcache(ref err) => err.fmt(f),
            MemcacheError::UrlError(ref err) => err.fmt(f),
        }
    }
}

impl error::Error for MemcacheError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            MemcacheError::Io(ref err) => err.source(),
            MemcacheError::Utf8Error(ref p) => p.source(),
            MemcacheError::ClientError(_) => None,
            MemcacheError::PoolError(ref p) => p.source(),
            MemcacheError::Serde(ref p) => p.source(),
            MemcacheError::Nom(_) => None,
            MemcacheError::Memcache(_) => None,
            MemcacheError::UrlError(ref p) => p.source(),
        }
    }
}

impl From<string::FromUtf8Error> for MemcacheError {
    fn from(err: string::FromUtf8Error) -> MemcacheError {
        MemcacheError::Utf8Error(err)
    }
}

impl From<parser::ErrorKind> for MemcacheError {
    fn from(e: parser::ErrorKind) -> MemcacheError {
        MemcacheError::Memcache(e)
    }
}

impl From<io::Error> for MemcacheError {
    fn from(err: io::Error) -> MemcacheError {
        MemcacheError::Io(err)
    }
}

impl From<bb8::RunError<io::Error>> for MemcacheError {
    fn from(err: bb8::RunError<io::Error>) -> MemcacheError {
        MemcacheError::PoolError(err)
    }
}

impl From<simd_json::Error> for MemcacheError {
    fn from(e: simd_json::Error) -> MemcacheError {
        MemcacheError::Serde(e)
    }
}

impl From<url::ParseError> for MemcacheError {
    fn from(e: url::ParseError) -> MemcacheError {
        MemcacheError::UrlError(e)
    }
}

/// Client-side errors
#[derive(Debug, PartialEq)]
pub enum ClientError {
    /// The key provided was longer than 250 bytes.
    KeyTooLong,
    /// The server returned an error prefixed with CLIENT_ERROR in response to a command.
    Error(Cow<'static, str>),
}

impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ClientError::KeyTooLong => write!(f, "The provided key was too long."),
            ClientError::Error(s) => write!(f, "{}", s),
        }
    }
}

impl From<ClientError> for MemcacheError {
    fn from(err: ClientError) -> Self {
        MemcacheError::ClientError(err)
    }
}

impl From<String> for ClientError {
    fn from(s: String) -> Self {
        ClientError::Error(Cow::Owned(s))
    }
}
