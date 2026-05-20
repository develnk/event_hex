use crate::cqrs::{Query, QueryHandlerFactory};
use crate::errors::QueryHandlerError;
use async_trait::async_trait;
use std::any::Any;

#[async_trait]
pub trait QueryBusPort: Send + Sync {
    async fn register_handler<Q, F>(&self, factory: F)
    where
        Q: Query + 'static,
        F: QueryHandlerFactory<Q> + 'static;
    async fn dispatch(&self, query: Box<dyn Query>) -> Result<Box<dyn Any + Send + Sync + 'static>, QueryHandlerError>;
}
