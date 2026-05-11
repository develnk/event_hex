use crate::application::ports::transaction::TransactionContext;
use crate::shared_kernel::domain::EntityId;
use crate::shared_kernel::domain_event::DomainEvent;
use crate::shared_kernel::errors::{CommandHandlerError, QueryHandlerError};
use async_trait::async_trait;
use downcast_rs::{impl_downcast, Downcast};
use std::any::Any;
use std::fmt::Debug;

// Трейт для команд.
pub trait Command: Downcast + Send + Sync + Debug {}
impl_downcast!(Command);

// Трейт для обработчиков команд
#[async_trait]
pub trait CommandHandler<C>: Send + Sync + 'static
where
    C: Command,
{
    // Команда меняет состояние агрегата. В результате этого процесса агрегат генерирует массив
    // доменных событий, которые нужно будет опубликовать в application слое.
    // В некоторых ситуациях полезно возвращать id агрегата, с которым взаимодействовала команда.
    async fn handle(&self, command: C, ctx: Option<&mut dyn TransactionContext>) -> Result<(EntityId, Vec<Box<dyn DomainEvent>>), CommandHandlerError>;
}

// Фабрика будет создавать конкретный CommandHandler.
#[async_trait]
pub trait CommandHandlerFactory<C>: Send + Sync
where
    C: Command,
{
    async fn create(&self) -> Result<Box<dyn CommandHandler<C>>, CommandHandlerError>;
}

//==================================================================================================

// Трейт для запросов (с ассоциированным типом ответа)
#[async_trait]
pub trait Query: Downcast + Send + Sync + 'static {}
impl_downcast!(Query);

// Трейт для обработчиков запросов
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
