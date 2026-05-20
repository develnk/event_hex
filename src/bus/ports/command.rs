use crate::cqrs::{Command, CommandHandlerFactory};
use crate::domain::EntityId;
use crate::domain_event::DomainEvent;
use crate::errors::CommandHandlerError;
use crate::persistence::transaction::EventTransactionContext;
use async_trait::async_trait;

#[async_trait]
pub trait CommandBusPort: Send + Sync {
    async fn register<C, F>(&self, factory: F)
    where
        C: Command + 'static,
        F: CommandHandlerFactory<C> + 'static;
    async fn dispatch(&self, command: Box<dyn Command>, ctx: Option<&mut dyn EventTransactionContext>) -> Result<(EntityId, Vec<Box<dyn DomainEvent>>), CommandHandlerError>;
}