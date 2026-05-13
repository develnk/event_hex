use crate::application::ports::transaction::TransactionContext;
use crate::domain::domain::{AggregateRoot, EntityId};
use crate::domain::domain_event::{Snapshot, StoredEvent};
use crate::shared_kernel::errors::EventStoreError;
use async_trait::async_trait;

/// Abstraction of event store operations. Allows replacing MongoDB with a mock for testing.
#[async_trait]
pub trait EventStoreStorage<A: AggregateRoot>: Send + Sync {
    async fn find_last_event(&self, ctx: &mut dyn TransactionContext, aggregate_id: &EntityId, aggregate_type: &str) -> Result<Option<StoredEvent>, EventStoreError>;

    async fn insert_events(&self, ctx: &mut dyn TransactionContext, events: Vec<StoredEvent>) -> Result<(), EventStoreError>;

    async fn delete_snapshot(&self, ctx: &mut dyn TransactionContext, aggregate_id: &EntityId, aggregate_type: &str) -> Result<(), EventStoreError>;

    async fn insert_snapshot(&self, ctx: &mut dyn TransactionContext, snapshot: Snapshot<A>) -> Result<(), EventStoreError>;

    async fn find_events_since_version(&self, ctx: &mut dyn TransactionContext, id: &EntityId, min_version: u32) -> Result<Vec<StoredEvent>, EventStoreError>;

    async fn find_latest_snapshot(&self, ctx: &mut dyn TransactionContext, id: &EntityId) -> Result<Option<Snapshot<A>>, EventStoreError>;
}
