use crate::cqrs::{Query, QueryHandlerFactory};
use crate::errors::QueryHandlerError;
use async_trait::async_trait;
use std::any::Any;

/// Port for the query bus.
///
/// Dispatches queries to their registered handlers and returns the response.
#[async_trait]
pub trait QueryBusPort: Send + Sync {
    /// Registers a query handler factory for the query type `Q`.
    async fn register_handler<Q, F>(&self, factory: F)
    where
        Q: Query + 'static,
        F: QueryHandlerFactory<Q> + 'static;

    /// Dispatches a query to its handler and returns the type-erased response.
    async fn dispatch(&self, query: Box<dyn Query>) -> Result<Box<dyn Any + Send + Sync + 'static>, QueryHandlerError>;
}
