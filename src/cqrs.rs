use crate::domain_event::DomainEvent;
use crate::errors::{CommandHandlerError, QueryHandlerError};
use crate::persistence::transaction::EventTransactionContext;
use crate::prelude::*;
use async_trait::async_trait;
use downcast_rs::{impl_downcast, Downcast};
use std::any::Any;
use std::fmt::Debug;

/// Marker trait for commands.
///
/// A command is an intent to change state. It is dispatched through a
/// [`CommandHandler`], which changes the aggregate state and produces one or
/// more domain events.
pub trait Command: Downcast + Send + Sync + Debug {}
impl_downcast!(Command);

/// Handles a single command type.
///
/// The handler changes the aggregate state; as a result, the aggregate
/// generates an array of domain events that must be published in the
/// application layer. In some situations it is useful to return the aggregate
/// id that the command interacted with.
#[async_trait]
pub trait CommandHandler<C>: Send + Sync + 'static
where
    C: Command,
{
    /// Executes the command and returns the affected aggregate id together with
    /// the domain events produced by it.
    async fn handle(&self, command: C, ctx: Option<&mut dyn EventTransactionContext>) -> Result<(EntityId, Vec<Box<dyn DomainEvent>>), CommandHandlerError>;
}

/// Creates a [`CommandHandler`] for a given command type.
#[async_trait]
pub trait CommandHandlerFactory<C>: Send + Sync
where
    C: Command,
{
    /// Creates a new handler instance.
    async fn create(&self) -> Result<Box<dyn CommandHandler<C>>, CommandHandlerError>;
}

/// Marker trait for queries.
///
/// A query is a read-only request that does not change state. Each query is
/// associated with a response type through its [`QueryHandler`].
pub trait Query: Downcast + Send + Sync + 'static {}
impl_downcast!(Query);

/// Handles a single query type.
#[async_trait]
pub trait QueryHandler<Q>: Send + Sync + 'static
where
    Q: Query,
{
    /// Executes the query and returns its type-erased response.
    async fn handle(&self, query: Q) -> Result<Box<dyn Any + Send + Sync + 'static>, QueryHandlerError>;
}

/// Creates a [`QueryHandler`] for a given query type.
#[async_trait]
pub trait QueryHandlerFactory<Q>: Send + Sync
where
    Q: Query,
{
    /// Creates a new handler instance.
    async fn create(&self) -> Result<Box<dyn QueryHandler<Q>>, QueryHandlerError>;
}
