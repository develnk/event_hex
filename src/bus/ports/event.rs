use crate::domain_event::{DomainEvent, DomainEventHandlerFactory};
use crate::errors::DomainEventHandlerError;
use async_trait::async_trait;

/// Port for the event bus.
///
/// Publishes domain events to their registered handlers.
#[async_trait]
pub trait EventBusPort: Send + Sync {
    /// Registers an event handler factory for the event type `E`.
    async fn register_handler<E, F>(&self, factory: F)
    where
        E: DomainEvent + Clone + Send + Sync + 'static,
        F: DomainEventHandlerFactory<E> + 'static;

    /// Publishes an event to all handlers registered for its type.
    async fn publish(&self, event: &dyn DomainEvent) -> Result<(), DomainEventHandlerError>;
}
