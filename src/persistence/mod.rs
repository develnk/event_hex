//! Persistence adapters and transaction management.

/// MongoDB persistence adapters.
#[cfg(feature = "mongo")]
pub mod mongodb;

/// PostgreSQL persistence adapters.
#[cfg(feature = "postgres")]
pub mod postgres;

/// Transaction context and manager ports.
pub mod transaction;
