use crate::shared_kernel::errors::EventHexError;
use async_trait::async_trait;
use futures::future::BoxFuture;
use std::any::Any;

/// Alias for erased type.
pub type ErasedResult = Box<dyn Any + Send>;

/// Abstract transaction context.
#[async_trait]
pub trait TransactionContext: Send {
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub type TransactionHandler = Box<dyn for<'a> FnOnce(&'a mut dyn TransactionContext) -> BoxFuture<'a, Result<ErasedResult, EventHexError>> + Send>;

/// Port for transaction management.
#[async_trait]
pub trait TransactionManager: Send + Sync {
    // Используем TransactionHandler<'a> с временем жизни из аргумента
    async fn run_transaction(&self, handler: TransactionHandler) -> Result<ErasedResult, EventHexError>;
}

impl dyn TransactionManager {
    pub async fn run<T, F>(&self, f: F) -> Result<T, EventHexError>
    where
        T: Any + Send + 'static,
        F: FnOnce(&mut dyn TransactionContext) -> BoxFuture<'_, Result<T, EventHexError>> + Send + 'static,
    {
        // Оборачиваем пользовательский handler в ErasedResult
        let handler: TransactionHandler = Box::new(|ctx| {
            Box::pin(async move {
                let res = f(ctx).await?;
                let erased: ErasedResult = Box::new(res);
                Ok(erased)
            })
        });

        let erased_result = self.run_transaction(handler).await?;

        // Attempting to cast the result to the required type T
        erased_result
            .downcast::<T>()
            .map(|boxed_t| *boxed_t)
            .map_err(|_| EventHexError::DownCastError("Downcast failed in transaction".into()))
    }
}
