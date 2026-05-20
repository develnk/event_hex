use crate::domain_event::{DomainEvent, DomainEventHandlerFactory};
use crate::errors::DomainEventHandlerError;
use async_trait::async_trait;

#[async_trait]
pub trait EventBusPort: Send + Sync {
    async fn register_handler<E, F>(&self, factory: F)
    where
        E: DomainEvent + Clone + Send + Sync + 'static,
        F: DomainEventHandlerFactory<E> + 'static;
    async fn publish(&self, event: &dyn DomainEvent) -> Result<(), DomainEventHandlerError>;
}