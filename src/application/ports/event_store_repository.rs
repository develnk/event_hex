use crate::application::ports::transaction::TransactionContext;
use crate::shared_kernel::domain::{AggregateContainer, AggregateRoot, EntityId};
use crate::shared_kernel::domain_event::{DomainEvent, Snapshot, StoredEvent};
use crate::shared_kernel::errors::EventStoreError;
use async_trait::async_trait;

/// Shared Event Repository
#[async_trait]
pub trait EventStoreRepository<A>: Send + Sync
where
    A: AggregateRoot + Send + Sync + 'static,
{
    // Method for saving aggregate changes.
    async fn save_aggregate(&self, ctx: &mut dyn TransactionContext, aggregate: AggregateContainer<A>) -> Result<(), EventStoreError>
    where
        A::Event: DomainEvent;
    // Method for loading an aggregate by its ID
    async fn load_aggregate(&self, ctx: &mut dyn TransactionContext, id: &EntityId) -> Result<Option<AggregateContainer<A>>, EventStoreError>;
    async fn get_events_since_version(&self, ctx: &mut dyn TransactionContext, id: &EntityId, version: u32) -> Result<Vec<StoredEvent>, EventStoreError>;
    async fn get_latest_snapshot(&self, ctx: &mut dyn TransactionContext, id: &EntityId) -> Result<Option<Snapshot<A>>, EventStoreError>;
}
