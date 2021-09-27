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
mod protocol;
mod stream;

pub use crate::client::{Client, Stats};
pub use crate::connection::ConnectionManager;
pub use crate::error::{ClientError, CommandError, MemcacheError, ServerError};
pub use crate::stream::Stream;
pub use r2d2::Error as PoolError;

/// R2D2 connection pool
pub type Pool = r2d2::Pool<ConnectionManager>;
