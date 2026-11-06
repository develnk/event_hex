use crate::domain::{AggregateRoot, EntityId};
use crate::domain_event::{Snapshot, StoredEvent};
use crate::errors::EventStoreError;
use crate::persistence::transaction::EventTransactionContext;
use async_trait::async_trait;

/// Abstraction of event store operations.
///
/// Decouples the event store repository from any concrete database, which
/// allows replacing MongoDB with a mock for testing or adding new backends.
#[async_trait]
pub trait EventStorage<A: AggregateRoot + Default>: Send + Sync {
    /// Returns the most recent event stored for an aggregate.
    async fn find_last_event(&self, ctx: &mut dyn EventTransactionContext, aggregate_id: &EntityId) -> Result<Option<StoredEvent>, EventStoreError>;

    /// Inserts a batch of events for an aggregate.
    async fn insert_events(&self, ctx: &mut dyn EventTransactionContext, events: Vec<StoredEvent>) -> Result<(), EventStoreError>;

    /// Deletes the snapshot of an aggregate.
    async fn delete_snapshot(&self, ctx: &mut dyn EventTransactionContext, aggregate_id: &EntityId) -> Result<(), EventStoreError>;

    /// Inserts a snapshot for an aggregate.
    async fn insert_snapshot(&self, ctx: &mut dyn EventTransactionContext, snapshot: Snapshot<A>) -> Result<(), EventStoreError>;

    /// Returns all events for an aggregate starting at the given version.
    async fn find_events_since_version(&self, ctx: &mut dyn EventTransactionContext, id: &EntityId, min_version: u32) -> Result<Vec<StoredEvent>, EventStoreError>;

    /// Returns the most recent snapshot for an aggregate, if any.
    async fn find_latest_snapshot(&self, ctx: &mut dyn EventTransactionContext, aggregate_id: &EntityId) -> Result<Option<Snapshot<A>>, EventStoreError>;
}
