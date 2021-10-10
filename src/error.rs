use std::borrow::Cow;
use std::error;
use std::fmt;
use std::io;
use std::str;
use std::string;

/// Errors related to a memcached operation.
#[derive(Clone, Debug, PartialEq)]
pub enum ErrorKind {
    /// General error that may or may not have come from either the server or this crate.
    Generic(String),
    /// The command sent by the client does not exist.
    NonexistentCommand,
    /// Protocol-level error i.e. an invalid response from memcached for the given operation.
    Protocol(Option<String>),
    /// An error from memcached related to CLIENT_ERROR.
    Client(String),
    /// An error from memcached related to SERVER_ERROR.
    Server(String),
}

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
    Memcache(ErrorKind),
}

impl MemcacheError {
    /// Check if error type was time out
    pub fn is_timeout(&self) -> bool {
        match self {
            MemcacheError::Io(error) => error.kind() == io::ErrorKind::TimedOut,
            _ => false,
        }
    }
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

impl From<ErrorKind> for MemcacheError {
    fn from(e: ErrorKind) -> MemcacheError {
        MemcacheError::Memcache(e)
    }
}

impl From<io::Error> for MemcacheError {
    fn from(err: io::Error) -> MemcacheError {
        MemcacheError::Io(err)
    }
}

impl From<io::ErrorKind> for MemcacheError {
    fn from(err: io::ErrorKind) -> MemcacheError {
        MemcacheError::Io(io::Error::from(err))
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

impl From<bb8::RunError<MemcacheError>> for MemcacheError {
    fn from(e: bb8::RunError<MemcacheError>) -> Self {
        match e {
            bb8::RunError::User(e) => e,
            bb8::RunError::TimedOut => MemcacheError::Io(io::Error::from(io::ErrorKind::TimedOut)),
        }
    }
}
