use crate::domain::{AggregateContainer, AggregateRoot, EntityId};
use crate::domain_event::{DomainEvent, Snapshot, StoredEvent};
use crate::errors::EventStoreError;
use crate::persistence::transaction::EventTransactionContext;
use async_trait::async_trait;

/// Shared event repository.
///
/// The application-level port used to persist and load aggregates. It is
/// implemented by [`crate::event_store::repository::EventRepositoryImpl`].
#[async_trait]
pub trait EventRepository<A>: Send + Sync
where
    A: AggregateRoot + Default + Send + Sync + 'static,
{
    /// Saves the aggregate's uncommitted events.
    async fn save_aggregate(&self, ctx: &mut dyn EventTransactionContext, aggregate: AggregateContainer<A>) -> Result<(), EventStoreError>
    where
        A::Event: DomainEvent;

    /// Loads an aggregate by its ID, replaying stored events and snapshots.
    async fn load_aggregate(&self, ctx: &mut dyn EventTransactionContext, id: &EntityId) -> Result<Option<AggregateContainer<A>>, EventStoreError>;

    /// Returns all events for an aggregate starting at the given version.
    async fn get_events_since_version(&self, ctx: &mut dyn EventTransactionContext, id: &EntityId, version: u32) -> Result<Vec<StoredEvent>, EventStoreError>;

    /// Returns the most recent snapshot for an aggregate, if any.
    async fn get_latest_snapshot(&self, ctx: &mut dyn EventTransactionContext, id: &EntityId) -> Result<Option<Snapshot<A>>, EventStoreError>;
}
