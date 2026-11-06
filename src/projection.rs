use crate::domain::EntityId;
use crate::domain_event::DomainEvent;
use crate::errors::ProjectionError;
use async_trait::async_trait;
use std::any::Any;
use std::fmt::Debug;

/// Port for a general projection repository.
///
/// A projection (read model) consumes domain events and maintains a
/// denormalized view of the data that is optimized for queries.
#[async_trait]
pub trait ProjectionRepository: Debug + Any + Send + Sync + 'static {
    /// Applies a single event to the projection.
    async fn apply_event(&mut self, aggregate_id: &EntityId, event: &dyn DomainEvent) -> Result<(), ProjectionError>;

    /// Fully rebuilds the projection from a stream of events.
    ///
    /// Necessary when the read model structure changes.
    async fn rebuild(&mut self, aggregate_id: &EntityId, stream: Vec<&dyn DomainEvent>) -> Result<(), ProjectionError>;
}

/// Applies a domain event to a projection DTO.
#[async_trait]
pub trait ProjectionDtoEventApplier: Debug + Any + Send + Sync + 'static {
    /// Applies a single event to this DTO.
    async fn apply_event_to_dto(&mut self, event: &dyn DomainEvent) -> Result<(), ProjectionError>;
}
