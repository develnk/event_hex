//! Event store repositories and storage backends.

/// MongoDB-backed event store.
#[cfg(feature = "mongo")]
pub mod mongodb;

/// PostgreSQL-backed event store.
#[cfg(feature = "postgres")]
pub mod postgres;

/// Storage abstraction used by the event store repository.
pub mod storage;

/// Event store repository working through a storage abstraction.
pub mod repository;
