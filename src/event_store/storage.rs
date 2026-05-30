use crate::domain::{AggregateRoot, EntityId};
use crate::domain_event::{Snapshot, StoredEvent};
use crate::errors::EventStoreError;
use crate::persistence::transaction::EventTransactionContext;
use async_trait::async_trait;

/// Abstraction of event store operations. Allows replacing MongoDB with a mock for testing.
#[async_trait]
pub trait EventStorage<A: AggregateRoot + Default>: Send + Sync {
    async fn find_last_event(&self, ctx: &mut dyn EventTransactionContext, aggregate_id: &EntityId) -> Result<Option<StoredEvent>, EventStoreError>;

    async fn insert_events(&self, ctx: &mut dyn EventTransactionContext, events: Vec<StoredEvent>) -> Result<(), EventStoreError>;

    async fn delete_snapshot(&self, ctx: &mut dyn EventTransactionContext, aggregate_id: &EntityId) -> Result<(), EventStoreError>;

    async fn insert_snapshot(&self, ctx: &mut dyn EventTransactionContext, snapshot: Snapshot<A>) -> Result<(), EventStoreError>;

    async fn find_events_since_version(&self, ctx: &mut dyn EventTransactionContext, id: &EntityId, min_version: u32) -> Result<Vec<StoredEvent>, EventStoreError>;

    async fn find_latest_snapshot(&self, ctx: &mut dyn EventTransactionContext, aggregate_id: &EntityId) -> Result<Option<Snapshot<A>>, EventStoreError>;
}
