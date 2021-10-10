//! Vinted Rust memcache
#![deny(
    bad_style,
    const_err,
    dead_code,
    deprecated,
    improper_ctypes,
    missing_debug_implementations,
    missing_docs,
    non_shorthand_field_patterns,
    no_mangle_generic_items,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    private_in_public,
    trivial_casts,
    trivial_numeric_casts,
    unconditional_recursion,
    unknown_lints,
    unreachable_code,
    unreachable_pub,
    unused,
    unused_allocation,
    unused_comparisons,
    unused_extern_crates,
    unused_import_braces,
    unused_mut,
    unused_parens,
    unused_qualifications,
    unused_results,
    warnings,
    while_true
)]

mod client;
mod codec;
mod connection;
mod error;
mod manager;
mod parser;

/// Driver access
pub mod driver;

pub use crate::client::Client;
pub use crate::error::{ClientError, ErrorKind, MemcacheError};
pub use crate::manager::ConnectionManager;
pub use bb8::{ErrorSink, State};
pub use connection::Connection;
pub use parser::Status;

/// R2D2 connection pool
pub type Pool = bb8::Pool<ConnectionManager>;

/// R2D2 connection pool
pub type PoolConnection<'c> = bb8::PooledConnection<'c, ConnectionManager>;
