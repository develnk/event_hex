use async_trait::async_trait;
use futures::future::BoxFuture;
use std::any::Any;
use crate::errors::EventHexError;

// Псевдоним для стертого типа
pub type ErasedResult = Box<dyn Any + Send>;

// Абстрактный контекст транзакции (чтобы не тянуть типы MongoDB в домен)
#[async_trait]
pub trait TransactionContext: Send {
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub type TransactionHandler = Box<
    dyn for<'a> FnOnce(
            &'a mut dyn TransactionContext,
        ) -> BoxFuture<'a, Result<ErasedResult, EventHexError>>
        + Send,
>;

// Порт для управления транзакциями
#[async_trait]
pub trait TransactionManager: Send + Sync {
    // Используем TransactionHandler<'a> с временем жизни из аргумента
    async fn run_transaction(&self, handler: TransactionHandler) -> Result<ErasedResult, EventHexError>;
}

// Добавляем удобный интерфейс для dyn TransactionManager
impl dyn TransactionManager {
    pub async fn run<T, F>(&self, f: F) -> Result<T, EventHexError>
    where
        T: Any + Send + 'static,
        F: FnOnce(&mut dyn TransactionContext) -> BoxFuture<'_, Result<T, EventHexError>>
            + Send
            + 'static,
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

        // Пытаемся привести результат к нужному типу T
        erased_result
            .downcast::<T>()
            .map(|boxed_t| *boxed_t)
            .map_err(|_| EventHexError::DownCastError("Downcast failed in transaction".into()))
    }
}
