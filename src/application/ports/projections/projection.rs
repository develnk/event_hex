use crate::domain::domain::EntityId;
use crate::domain::domain_event::DomainEvent;
use crate::shared_kernel::errors::ProjectionError;
use async_trait::async_trait;
use std::any::Any;
use std::fmt::Debug;

// Port: General projection repository
#[async_trait]
pub trait ProjectionRepository: Debug + Any + Send + Sync + 'static {
    // Method for processing a single event
    async fn apply_event(&mut self, aggregate_id: &EntityId, event: &dyn DomainEvent) -> Result<(), ProjectionError>;

    // Method for full rebuild — necessary if we change the Read Model structure
    async fn rebuild(&mut self, aggregate_id: &EntityId, stream: Vec<&dyn DomainEvent>) -> Result<(), ProjectionError>;
}

#[async_trait]
pub trait ProjectionDtoEventApplier: Debug + Any + Send + Sync + 'static {
    async fn apply_event_to_dto(&mut self, event: &dyn DomainEvent) -> Result<(), ProjectionError>;
}
