use crate::application::ports::cqrs::Query;
use crate::shared_kernel::domain_event::DomainEvent;
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum EventHexError {
    #[error("Command handler error: {0}")]
    CommandHandler(#[from] CommandHandlerError),

    #[error("Domain event handler error: {0}")]
    DomainEventHandler(#[from] DomainEventHandlerError),

    #[error("Ошибка приведения к нужному типу {0}")]
    DownCastError(String),

    #[error("EventStore error: {0}")]
    EventStore(#[from] EventStoreError),

    #[cfg(feature = "mongo")]
    #[error("MongoDB error: {0}")]
    MongoError(#[from] mongodb::error::Error),

    #[error("Query handler error: {0}")]
    QueryHandler(#[from] QueryHandlerError),

    #[error("Serialize JSON error: {0}")]
    SerdeError(#[from] serde_json::Error),

    #[error("В этом месте контекст транзакции обязательно нужен")]
    TransactionContextRequired(),

    #[error("Ошибка проекции домена: {0}")]
    ProjectionError(#[from] ProjectionError),
}

#[derive(Error, Debug, Clone)]
pub enum DomainEventHandlerError {
    #[error("Общая ошибка обработки доменного события: {0}")]
    DomainEventHandlerCommon(String),

    #[error("Failed to downcast event type")]
    DomainEventHandlerDownCast(),

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

#[derive(Error, Debug, Clone)]
pub enum CommandHandlerError {
    #[error("Failed to downcast command type: {0}")]
    CommandHandlerDownCast(String),

    #[error("Command not found {0}")]
    CommandNotFound(String),

    #[error("Failed to downcast repository type")]
    RepoDowncastFailed(),

    #[error("No handler registered for command {0}")]
    CommandHandlerNotRegistered(String),

    #[error("Generic Command Handler error: {0}")]
    GenericCommandHandler(String),

    #[error("Found event event_store error during the command execution: {0}")]
    EventStoreError(#[from] EventStoreError),

    #[error("Found domain error during the command execution: {0}")]
    DomainError(#[from] DomainError),
}

#[derive(Error, Debug, Clone)]
pub enum QueryHandlerError {
    #[error("No handler registered for query {0}")]
    QueryHandlerNotRegistered(String),

    #[error("Failed to downcast query type")]
    QueryDowncastFailed(),

    #[error("Failed to downcast repository type")]
    RepoDowncastFailed(),

    #[error("Failed to downcast repository type")]
    FromEventStoreError(#[from] EventStoreError),
}

impl From<Box<dyn Query>> for QueryHandlerError {
    fn from(_err: Box<dyn Query>) -> Self {
        // todo выяснить каким-либо образом какой именно QueryHandlerError передать
        QueryHandlerError::QueryDowncastFailed()
    }
}

#[derive(Error, Debug, Clone)]
pub enum EventStoreError {
    #[error("Failed to publish event: {0}")]
    PublishEventError(String),

    #[error("Error serializing event stored in  DB: {0}")]
    DeSerializationError(String),

    #[error("Ошибка во время сериализации события/агрегата")]
    SerializationError(),

    #[error("Проверка целостности события под номером: {version} выявила ошибку для агрегата {aggregate_type}: {aggregate_id}"
    )]
    EventChainVerifyError {
        aggregate_id: Uuid,
        aggregate_type: String,
        version: u32,
    },

    #[error("Ошибка сериалзации/хеширования события под номером: {version} для агрегата {aggregate_type}:{aggregate_id}"
    )]
    EventChainSerializeError {
        aggregate_id: Uuid,
        aggregate_type: String,
        version: u32,
    },

    #[error("Event application error: {0}")]
    EventApplicationError(String),

    #[error("Event Store transaction error")]
    TransactionError,

    #[error("Event Store end of transaction error")]
    EndTransactionError,

    #[error("Error in the event event_store: {0}")]
    StoreError(String),

    #[error("Ошибка в хранилище снапшотов: {0}")]
    SnapshotStoreError(String),

    #[error("Store Domain specific error: {0}")]
    DomainEventStoreError(#[from] DomainError),
}

impl From<serde_json::Error> for EventStoreError {
    fn from(err: serde_json::Error) -> Self {
        // todo в зависимости от типа ошибки(сериализация или десереализация) отдавать соотв. ошибку
        EventStoreError::DeSerializationError(err.to_string())
    }
}

#[cfg(feature = "mongo")]
impl From<mongodb::error::Error> for EventStoreError {
    fn from(value: mongodb::error::Error) -> Self {
        EventStoreError::StoreError(value.to_string())
    }
}

#[derive(Error, Debug, Clone)]
pub enum DomainError {
    #[error("Aggregate not found: {aggregate_type}:{aggregate_id}")]
    AggregateNotFound { aggregate_id: Uuid, aggregate_type: String },

    #[error("Concurrency conflict for aggregate {aggregate_type}:{aggregate_id} expected version {expected}, found {actual}"
    )]
    ConcurrencyConflict {
        aggregate_id: Uuid,
        aggregate_type: String,
        expected: u32,
        actual: u32,
    },

    #[error("Error deserializing domain {0}")]
    DeSerializationError(String),

    #[error("Unknown event found: {event_name}")]
    UnknownEvent { event_name: String },

    #[error(
        "Error validation domain invariant for aggregate {aggregate_type}:{aggregate_id}. Actual data:{actual} at event: {event_name}. Message: {message}"
    )]
    DomainValidationError {
        event_name: String,
        aggregate_id: Uuid,
        aggregate_type: String,
        actual: String,
        message: String,
    },
}

impl From<serde_json::Error> for DomainError {
    fn from(err: serde_json::Error) -> Self {
        DomainError::DeSerializationError(err.to_string())
    }
}

#[derive(Error, Debug, Clone)]
pub enum ProjectionError {
    #[error("Ошибка сериализации данных: {0}")]
    ProjectionSerializeError(String),

    #[error("Ошибка в хранилище проекций: {0}")]
    StoreProjectionError(String),

    #[error("Ошибка применения события к проекции {0}")]
    ApplyEventToProjectionError(String),

    #[error("Проекция {0} не найдена. Её id: {1}")]
    ProjectionNotFound(String, String),

    #[error("Ошибка обработки доменного события для Проекции")]
    DomainEventHandlerError,
}

impl From<DomainEventHandlerError> for ProjectionError {
    fn from(value: DomainEventHandlerError) -> Self {
        ProjectionError::DomainEventHandlerError
    }
}

