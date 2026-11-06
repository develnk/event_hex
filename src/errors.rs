use crate::cqrs::Query;
use crate::domain_event::DomainEvent;
use thiserror::Error;
use uuid::Uuid;

/// Top-level error type for the whole crate.
///
/// Aggregates all domain-specific and infrastructure errors that can occur
/// while handling commands, queries and events.
#[derive(Error, Debug)]
pub enum EventHexError {
    /// A command handler failed.
    #[error("Command handler error: {0}")]
    CommandHandler(#[from] CommandHandlerError),

    /// A domain event handler failed.
    #[error("Domain event handler error: {0}")]
    DomainEventHandler(#[from] DomainEventHandlerError),

    /// A downcast to the expected type failed.
    #[error("Error casting to the required type {0}")]
    DownCastError(String),

    /// The event store reported an error.
    #[error("EventStore error: {0}")]
    EventStore(#[from] EventStoreError),

    /// MongoDB reported an error.
    #[cfg(feature = "mongo")]
    #[error("MongoDB error: {0}")]
    MongoError(#[from] mongodb::error::Error),

    /// A query handler failed.
    #[error("Query handler error: {0}")]
    QueryHandler(#[from] QueryHandlerError),

    /// Serialization to JSON failed.
    #[error("Serialize JSON error: {0}")]
    SerdeError(#[from] serde_json::Error),

    /// The operation requires a transaction context but none was provided.
    #[error("Transaction context is required here")]
    TransactionContextRequired(),

    /// A projection reported an error.
    #[error("Domain projection error: {0}")]
    ProjectionError(#[from] ProjectionError),
}

/// Errors produced while handling domain events.
#[derive(Error, Debug, Clone)]
pub enum DomainEventHandlerError {
    /// A generic domain event handling failure.
    #[error("Common domain event handling error: {0}")]
    DomainEventHandlerCommon(String),

    /// The event could not be downcast to the expected type.
    #[error("Failed to downcast event type")]
    DomainEventHandlerDownCast(),

    /// No handler was registered for the given event type.
    #[error("No handler registered for event {0}")]
    DomainEventHandlerNotRegistered(String),
}

impl From<Box<dyn DomainEvent>> for DomainEventHandlerError {
    fn from(_err: Box<dyn DomainEvent>) -> Self {
        DomainEventHandlerError::DomainEventHandlerDownCast()
    }
}

impl From<ProjectionError> for DomainEventHandlerError {
    fn from(value: ProjectionError) -> Self {
        DomainEventHandlerError::DomainEventHandlerCommon(value.to_string())
    }
}

/// Errors produced while handling commands.
#[derive(Error, Debug, Clone)]
pub enum CommandHandlerError {
    /// The command could not be downcast to the expected type.
    #[error("Failed to downcast command type: {0}")]
    CommandHandlerDownCast(String),

    /// The command was not found.
    #[error("Command not found {0}")]
    CommandNotFound(String),

    /// The repository could not be downcast to the expected type.
    #[error("Failed to downcast repository type")]
    RepoDowncastFailed(),

    /// No handler was registered for the given command type.
    #[error("No handler registered for command {0}")]
    CommandHandlerNotRegistered(String),

    /// A generic command handling failure.
    #[error("Generic Command Handler error: {0}")]
    GenericCommandHandler(String),

    /// The event store reported an error while executing the command.
    #[error("Found event event_store error during the command execution: {0}")]
    EventStoreError(#[from] EventStoreError),

    /// A domain error occurred while executing the command.
    #[error("Found domain error during the command execution: {0}")]
    DomainError(#[from] DomainError),
}

impl From<EventHexError> for CommandHandlerError {
    fn from(error: EventHexError) -> Self {
        Self::GenericCommandHandler(error.to_string())
    }
}

/// Errors produced while handling queries.
#[derive(Error, Debug, Clone)]
pub enum QueryHandlerError {
    /// No handler was registered for the given query type.
    #[error("No handler registered for query {0}")]
    QueryHandlerNotRegistered(String),

    /// The query could not be downcast to the expected type.
    #[error("Failed to downcast query type")]
    QueryDowncastFailed(),

    /// The repository could not be downcast to the expected type.
    #[error("Failed to downcast repository type")]
    RepoDowncastFailed(),

    /// The event store reported an error.
    #[error("Failed to downcast repository type")]
    FromEventStoreError(#[from] EventStoreError),
}

impl From<Box<dyn Query>> for QueryHandlerError {
    fn from(_err: Box<dyn Query>) -> Self {
        // todo figure out somehow which QueryHandlerError to pass
        QueryHandlerError::QueryDowncastFailed()
    }
}

/// Errors produced by the event store.
#[derive(Error, Debug, Clone)]
pub enum EventStoreError {
    /// Publishing an event failed.
    #[error("Failed to publish event: {0}")]
    PublishEventError(String),

    /// Deserializing an event stored in the database failed.
    #[error("Error serializing event stored in  DB: {0}")]
    DeSerializationError(String),

    /// Serializing an event or aggregate failed.
    #[error("Error during event/aggregate serialization")]
    SerializationError(),

    /// The hash chain verification failed for an event.
    #[error("Integrity check for event #{version} failed for aggregate {aggregate_type}: {aggregate_id}")]
    EventChainVerifyError {
        /// The aggregate whose event failed verification.
        aggregate_id: Uuid,
        /// The type of the aggregate.
        aggregate_type: String,
        /// The sequence number of the event that failed verification.
        version: u32,
    },

    /// Serialization or hashing failed for an event in the chain.
    #[error("Serialization/hashing error for event #{version} of aggregate {aggregate_type}:{aggregate_id}")]
    EventChainSerializeError {
        /// The aggregate whose event failed to serialize.
        aggregate_id: Uuid,
        /// The type of the aggregate.
        aggregate_type: String,
        /// The sequence number of the event that failed to serialize.
        version: u32,
    },

    /// Applying an event to the aggregate failed.
    #[error("Event application error: {0}")]
    EventApplicationError(String),

    /// A transaction error occurred.
    #[error("Event Store transaction error")]
    TransactionError,

    /// The transaction could not be finalized.
    #[error("Event Store end of transaction error")]
    EndTransactionError,

    /// A generic store error occurred.
    #[error("Error in the event event_store: {0}")]
    StoreError(String),

    /// The snapshot store reported an error.
    #[error("Snapshot store error: {0}")]
    SnapshotStoreError(String),

    /// A domain-specific error occurred in the event store.
    #[error("Store Domain specific error: {0}")]
    DomainEventStoreError(#[from] DomainError),
}

impl From<serde_json::Error> for EventStoreError {
    fn from(err: serde_json::Error) -> Self {
        // todo depending on the error type (serialization or deserialization), return the appropriate error
        EventStoreError::DeSerializationError(err.to_string())
    }
}

#[cfg(feature = "mongo")]
impl From<mongodb::error::Error> for EventStoreError {
    fn from(value: mongodb::error::Error) -> Self {
        EventStoreError::StoreError(value.to_string())
    }
}

/// Domain-level errors.
#[derive(Error, Debug, Clone)]
pub enum DomainError {
    /// The aggregate was not found.
    #[error("Aggregate not found: {aggregate_type}:{aggregate_id}")]
    AggregateNotFound {
        /// The id of the missing aggregate.
        aggregate_id: Uuid,
        /// The type of the missing aggregate.
        aggregate_type: String,
    },

    /// Two writers tried to save conflicting versions of the same aggregate.
    #[error("Concurrency conflict for aggregate {aggregate_type}:{aggregate_id} expected version {expected}, found {actual}")]
    ConcurrencyConflict {
        /// The id of the conflicting aggregate.
        aggregate_id: Uuid,
        /// The type of the conflicting aggregate.
        aggregate_type: String,
        /// The version the writer expected.
        expected: u32,
        /// The version actually found in the store.
        actual: u32,
    },

    /// A domain object could not be deserialized.
    #[error("Error deserializing domain {0}")]
    DeSerializationError(String),

    /// An event of an unknown type was found while replaying.
    #[error("Unknown event found: {event_name}")]
    UnknownEvent {
        /// The name of the unknown event type.
        event_name: String,
    },

    /// A domain invariant was violated.
    #[error("Error validation domain invariant for aggregate {aggregate_type}:{aggregate_id}. Actual data:{actual} at event: {event_name}. Message: {message}")]
    DomainValidationError {
        /// The name of the event that triggered the validation.
        event_name: String,
        /// The id of the aggregate that failed validation.
        aggregate_id: Uuid,
        /// The type of the aggregate that failed validation.
        aggregate_type: String,
        /// The actual data that failed validation.
        actual: String,
        /// A human-readable explanation of the failure.
        message: String,
    },
}

impl From<serde_json::Error> for DomainError {
    fn from(err: serde_json::Error) -> Self {
        DomainError::DeSerializationError(err.to_string())
    }
}

/// Errors produced while updating projections.
#[derive(Error, Debug, Clone)]
pub enum ProjectionError {
    /// Serializing projection data failed.
    #[error("Data serialization error: {0}")]
    ProjectionSerializeError(String),

    /// The projection store reported an error.
    #[error("Projection store error: {0}")]
    StoreProjectionError(String),

    /// Applying an event to a projection failed.
    #[error("Error applying event to projection {0}")]
    ApplyEventToProjectionError(String),

    /// The projection was not found.
    #[error("Projection {0} not found. Its id: {1}")]
    ProjectionNotFound(String, String),

    /// Processing a domain event for the projection failed.
    #[error("Domain event processing error for Projection")]
    DomainEventHandlerError,
}

impl From<DomainEventHandlerError> for ProjectionError {
    fn from(_value: DomainEventHandlerError) -> Self {
        ProjectionError::DomainEventHandlerError
    }
}

#[cfg(feature = "mongo")]
impl From<mongodb::error::Error> for ProjectionError {
    fn from(value: mongodb::error::Error) -> Self {
        ProjectionError::StoreProjectionError(value.to_string())
    }
}
