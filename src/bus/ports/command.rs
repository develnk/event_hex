use crate::cqrs::{Command, CommandHandlerFactory};
use crate::domain::EntityId;
use crate::domain_event::DomainEvent;
use crate::errors::CommandHandlerError;
use crate::persistence::transaction::EventTransactionContext;
use async_trait::async_trait;

/// Port for the command bus.
///
/// Dispatches commands to their registered handlers and returns the produced
/// domain events.
#[async_trait]
pub trait CommandBusPort: Send + Sync {
    /// Registers a command handler factory for the command type `C`.
    async fn register<C, F>(&self, factory: F)
    where
        C: Command + 'static,
        F: CommandHandlerFactory<C> + 'static;

    /// Dispatches a command to its handler.
    ///
    /// Returns the affected aggregate id and the domain events produced by the
    /// command.
    async fn dispatch(&self, command: Box<dyn Command>, ctx: Option<&mut dyn EventTransactionContext>) -> Result<(EntityId, Vec<Box<dyn DomainEvent>>), CommandHandlerError>;
}
