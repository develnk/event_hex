//! In-memory and port abstractions for the command, query and event buses.

/// In-memory bus implementations backed by `tokio` primitives.
pub mod in_memory;

/// Port (trait) definitions for the buses, to be implemented by concrete
/// transports.
pub mod ports;
