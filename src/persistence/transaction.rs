use crate::errors::EventHexError;
use async_trait::async_trait;
use futures::future::BoxFuture;
use std::any::Any;

/// Alias for a type-erased transaction result.
pub type ErasedResult = Box<dyn Any + Send>;

/// Abstract transaction context.
///
/// Concrete implementations wrap a database session (for example a MongoDB
/// `ClientSession`) and expose it through downcasting.
#[async_trait]
pub trait EventTransactionContext: Send {
    /// Returns this context as `&mut dyn Any` so that storage backends can
    /// downcast it to their concrete session type.
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// Type-erased transaction handler.
///
/// A boxed closure that runs a piece of work against a transaction context and
/// returns an [`ErasedResult`].
pub type EventTransactionHandler = Box<dyn for<'a> FnOnce(&'a mut dyn EventTransactionContext) -> BoxFuture<'a, Result<ErasedResult, EventHexError>> + Send>;

/// Port for transaction management.
#[async_trait]
pub trait EventTransactionManager: Send + Sync {
    /// Runs the given handler inside a transaction (when supported) and returns
    /// its type-erased result.
    async fn run_transaction(&self, handler: EventTransactionHandler) -> Result<ErasedResult, EventHexError>;
}

impl dyn EventTransactionManager {
    /// Runs a closure inside a transaction and returns a strongly typed result.
    ///
    /// The closure receives a mutable transaction context and returns a future
    /// resolving to `T`. The result is downcast back to `T` on success.
    pub async fn run<T, F>(&self, f: F) -> Result<T, EventHexError>
    where
        T: Any + Send + 'static,
        F: FnOnce(&mut dyn EventTransactionContext) -> BoxFuture<'_, Result<T, EventHexError>> + Send + 'static,
    {
        // Wrap the user-defined handler in ErasedResult.
        let handler: EventTransactionHandler = Box::new(|ctx| {
            Box::pin(async move {
                let res = f(ctx).await?;
                let erased: ErasedResult = Box::new(res);
                Ok(erased)
            })
        });

        let erased_result = self.run_transaction(handler).await?;

        // Attempting to cast the result to the required type T.
        erased_result
            .downcast::<T>()
            .map(|boxed_t| *boxed_t)
            .map_err(|_| EventHexError::DownCastError("Downcast failed in transaction".into()))
    }
}
