pub mod domain;
pub mod domain_event;
pub mod errors;
pub mod auditable;
pub mod bus;
pub mod event_store;
pub mod persistence;
pub mod domain_event_handlers;
pub mod event_store_repository;
pub mod cqrs;
pub mod projection;

pub mod prelude {
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
    pub use crate::projection::{ProjectionDtoEventApplier, ProjectionRepository};
}

pub mod types {
    pub type SequenceNumber = u32;
}
