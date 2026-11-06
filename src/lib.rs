//! A pragmatic Rust toolkit for Domain-Driven Design (DDD) with first-class
//! support for event sourcing and CQRS.
//!
//! The crate is organized around a few core concepts:
//!
//! - [`domain`]: aggregates and their containers.
//! - [`domain_event`]: the domain event abstraction and event-store record types.
//! - [`event_store`]: event store repositories and storage backends.
//! - [`bus`] and [`cqrs`]: in-memory command, query and event buses.
//! - [`projection`]: read-model projections.

#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

/// Audit information attached to persisted domain events.
pub mod audit;

/// In-memory command, query and event buses together with their ports.
pub mod bus;

/// Command and query abstractions (CQRS).
pub mod cqrs;

/// Aggregate root abstraction and aggregate containers.
pub mod domain;

/// Domain event abstraction and event-store record types.
pub mod domain_event;

/// Built-in domain event handlers (for example projection updaters).
pub mod domain_event_handlers;

/// Error types produced across the crate.
pub mod errors;

/// Event store repositories and storage backends.
pub mod event_store;

/// Application-level event repository port.
pub mod event_store_repository;

/// Transaction management ports and implementations.
pub mod persistence;

/// Projection (read model) ports.
pub mod projection;

/// Re-exports of the most commonly used types.
///
/// Importing `event_hex::prelude::*` brings the core traits, buses and error
/// types into scope for typical application code.
pub mod prelude {
    pub use crate::audit::{EventAudit, EventMetadata};
    pub use crate::bus::in_memory::command_bus::CommandBus;
    pub use crate::bus::in_memory::event_bus::EventBus;
    pub use crate::bus::in_memory::query_bus::QueryBus;
    pub use crate::bus::ports::command::CommandBusPort;
    pub use crate::bus::ports::event::EventBusPort;
    pub use crate::bus::ports::query::QueryBusPort;
    pub use crate::cqrs::{Command, CommandHandler, CommandHandlerFactory, Query, QueryHandler, QueryHandlerFactory};
    pub use crate::domain::{AggregateContainer, AggregateRoot, EntityId};
    pub use crate::domain_event::{DomainEvent, DomainEventHandler, DomainEventHandlerFactory, Snapshot, StoredEvent};
    pub use crate::domain_event_handlers::{ProjectionUpdaterEventHandler, ProjectionUpdaterEventHandlerFactory};
    pub use crate::errors::{CommandHandlerError, DomainError, DomainEventHandlerError, ProjectionError, QueryHandlerError};
    pub use crate::event_store_repository::EventRepository;
    pub use crate::persistence::transaction::EventTransactionContext;
    pub use crate::projection::{ProjectionDtoEventApplier, ProjectionRepository};
}

/// Shared type aliases used across the crate.
pub mod types {
    /// Sequence number of an event within a single aggregate instance.
    pub type SequenceNumber = u32;
}
