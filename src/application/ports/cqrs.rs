use crate::application::ports::transaction::TransactionContext;
use crate::shared_kernel::domain::EntityId;
use crate::shared_kernel::domain_event::DomainEvent;
use crate::shared_kernel::errors::{CommandHandlerError, QueryHandlerError};
use async_trait::async_trait;
use downcast_rs::{impl_downcast, Downcast};
use std::any::Any;
use std::fmt::Debug;

// Trait for commands.
pub trait Command: Downcast + Send + Sync + Debug {}
impl_downcast!(Command);

// Trait for command handlers
#[async_trait]
pub trait CommandHandler<C>: Send + Sync + 'static
where
    C: Command,
{
    // The command changes the aggregate state. As a result, the aggregate generates an array
    // of domain events that need to be published in the application layer.
    // In some situations, it's useful to return the aggregate id that the command interacted with.
    async fn handle(&self, command: C, ctx: Option<&mut dyn TransactionContext>) -> Result<(EntityId, Vec<Box<dyn DomainEvent>>), CommandHandlerError>;
}

// Factory will create a specific CommandHandler.
#[async_trait]
pub trait CommandHandlerFactory<C>: Send + Sync
where
    C: Command,
{
    async fn create(&self) -> Result<Box<dyn CommandHandler<C>>, CommandHandlerError>;
}

//==================================================================================================

// Trait for queries (with associated response type)
#[async_trait]
pub trait Query: Downcast + Send + Sync + 'static {}
impl_downcast!(Query);

// Trait for query handlers
#[async_trait]
pub trait QueryHandler<Q>: Send + Sync + 'static
where
    Q: Query,
{
    async fn handle(&self, query: Q) -> Result<Box<dyn Any + Send + Sync + 'static>, QueryHandlerError>;
}

#[async_trait]
pub trait QueryHandlerFactory<Q>: Send + Sync
where
    Q: Query,
{
    async fn create(&self) -> Result<Box<dyn QueryHandler<Q>>, QueryHandlerError>;
}
