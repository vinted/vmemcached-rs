use std::fmt;

mod ascii;
pub(crate) use ascii::{parse_ascii_response, parse_ascii_status, parse_version};

use crate::ErrorKind;

/// A value from memcached.
#[derive(Clone, Debug, PartialEq)]
pub struct Value {
    /// The key.
    pub key: Vec<u8>,
    /// CAS identifier.
    pub cas: Option<u64>,
    /// Flags for this key.
    ///
    /// Defaults to 0.
    pub flags: u32,
    /// Data for this key.
    pub data: Vec<u8>,
}

/// Status of a memcached operation.
#[derive(Clone, Debug, PartialEq)]
pub enum Status {
    /// The value was stored.
    Stored,
    /// The value was not stored.
    NotStored,
    /// The key was deleted.
    Deleted,
    /// The key was touched.
    Touched,
    /// The key already exists.
    Exists,
    /// The key was not found.
    NotFound,
}

/// Response to a memcached operation.
#[derive(Clone, Debug, PartialEq)]
pub enum Response {
    /// The status of a given operation, which may or may not have succeeded.
    Status(Status),
    /// Data response, which is only returned for reads.
    Data(Vec<Value>),
    /// An error occurred for the given operation.
    Error(ErrorKind),
}

impl Response {
    pub fn is_server_error(&self) -> bool {
        matches!(self, Response::Error(ErrorKind::Server(_)))
    }
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Stored => "stored".fmt(f),
            Self::NotStored => "not stored".fmt(f),
            Self::Deleted => "deleted".fmt(f),
            Self::Touched => "touched".fmt(f),
            Self::Exists => "exists".fmt(f),
            Self::NotFound => "not found".fmt(f),
        }
    }
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Generic(s) => write!(f, "generic: {}", s),
            Self::NonexistentCommand => write!(f, "command does not exist"),
            Self::Protocol(s) => match s {
                Some(s) => write!(f, "protocol: {}", s),
                None => write!(f, "protocol"),
            },
            Self::Client(s) => write!(f, "client: {}", s),
            Self::Server(s) => write!(f, "server: {}", s),
        }
    }
}
