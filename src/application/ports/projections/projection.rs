use crate::shared_kernel::domain::EntityId;
use crate::shared_kernel::domain_event::DomainEvent;
use crate::shared_kernel::errors::ProjectionError;
use async_trait::async_trait;
use std::any::Any;
use std::fmt::Debug;

//Порт: Общий репозиторий проекции
#[async_trait]
pub trait ProjectionRepository: Debug + Any + Send + Sync + 'static {
    //Метод для обработки одного события
    async fn apply_event(&mut self, aggregate_id: &EntityId, event: &dyn DomainEvent) -> Result<(), ProjectionError>;

    // Метод для полного перестроения  — необходимо, если мы меняем структуру Read Model
    async fn rebuild(&mut self, aggregate_id: &EntityId, stream: Vec<&dyn DomainEvent>) -> Result<(), ProjectionError>;
}

#[async_trait]
pub trait ProjectionDtoEventApplier: Debug + Any + Send + Sync + 'static {
    async fn apply_event_to_dto(&mut self, event: &dyn DomainEvent) -> Result<(), ProjectionError>;
}
