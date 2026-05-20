use crate::domain::{AggregateRoot, EntityId};
use crate::domain_event::{Snapshot, StoredEvent};
use crate::errors::DomainError;
use crate::errors::EventStoreError;
use crate::event_store::storage::EventStoreStorage;
use crate::persistence::mongodb::mongo_transaction::MongoContext;
use crate::persistence::transaction::EventTransactionContext;
use async_trait::async_trait;
use bson::doc;
use futures::StreamExt;
use mongodb::error::ErrorKind;
use mongodb::options::{FindOneAndDeleteOptions, FindOneOptions, FindOptions, InsertManyOptions};
use mongodb::{Client, Collection};
use serde::de::DeserializeOwned;
use std::sync::Arc;

/// MongoDB event store implementation.
pub struct MongoEventStoreStorage<A: AggregateRoot + Send + Sync + 'static> {
    events_collection: Collection<StoredEvent>,
    snapshots_collection: Collection<Snapshot<A>>,
}

impl<A> MongoEventStoreStorage<A>
where
    A: AggregateRoot + Send + Sync + 'static,
{
    pub fn new(client: Arc<Client>, db_name: &str) -> Self {
        Self {
            events_collection: client.database(db_name).collection("events"),
            snapshots_collection: client.database(db_name).collection("snapshots"),
        }
    }
}

#[async_trait]
impl<A> EventStoreStorage<A> for MongoEventStoreStorage<A>
where
    A: AggregateRoot + DeserializeOwned + Send + Sync + 'static,
{
    async fn find_last_event(
        &self,
        ctx: &mut dyn EventTransactionContext,
        aggregate_id: &EntityId,
        aggregate_type: &str,
    ) -> Result<Option<StoredEvent>, EventStoreError> {
        let filter = doc! {
            "event.aggregate_id": aggregate_id.as_uuid(),
            "event.aggregate_type": aggregate_type
        };
        let find_options =
            FindOneOptions::builder().sort(doc! { "event.sequence_number": -1 }).build();

        if let Some(mongo_ctx) = ctx.as_any_mut().downcast_mut::<MongoContext>() {
            self.events_collection
                .find_one(filter)
                .session(&mut mongo_ctx.session)
                .with_options(find_options)
                .await
                .map_err(|e| EventStoreError::StoreError(e.to_string()))
        } else {
            self.events_collection
                .find_one(filter)
                .with_options(find_options)
                .await
                .map_err(|e| EventStoreError::StoreError(e.to_string()))
        }
    }

    async fn insert_events(
        &self,
        ctx: &mut dyn EventTransactionContext,
        events: Vec<StoredEvent>,
    ) -> Result<(), EventStoreError> {
        let aggregate_id = events[0].event.aggregate_id;
        let aggregate_type = events[0].event.aggregate_type.clone();
        let sequence_number = events[0].event.sequence_number;

        let options = InsertManyOptions::builder().ordered(true).build();

        let result = if let Some(mongo_ctx) = ctx.as_any_mut().downcast_mut::<MongoContext>() {
            self.events_collection
                .insert_many(events)
                .session(&mut mongo_ctx.session)
                .with_options(options)
                .await
        } else {
            self.events_collection
                .insert_many(events)
                .with_options(options)
                .await
        };

        result.map_err(|e| {
            let is_duplicate_key = match *e.kind {
                ErrorKind::InsertMany(many_err_box) => many_err_box
                    .write_errors
                    .unwrap()
                    .iter()
                    .any(|vc| vc.code == 11000),
                _ => false,
            };

            if is_duplicate_key {
                EventStoreError::DomainEventStoreError(DomainError::ConcurrencyConflict {
                    aggregate_id,
                    aggregate_type,
                    expected: sequence_number,
                    actual: sequence_number - 1,
                })
            } else {
                EventStoreError::StoreError("Error while inserting new event".into())
            }
        })?;

        Ok(())
    }

    async fn delete_snapshot(
        &self,
        ctx: &mut dyn EventTransactionContext,
        aggregate_id: &EntityId,
        aggregate_type: &str,
    ) -> Result<(), EventStoreError> {
        let filter = doc! {
            "aggregate_id": aggregate_id.as_uuid(),
            "aggregate_type": aggregate_type
        };
        let options = FindOneAndDeleteOptions::builder().build();

        let result = if let Some(mongo_ctx) = ctx.as_any_mut().downcast_mut::<MongoContext>() {
            self.snapshots_collection
                .find_one_and_delete(filter)
                .session(&mut mongo_ctx.session)
                .with_options(options)
                .await
        } else {
            self.snapshots_collection
                .find_one_and_delete(filter)
                .with_options(options)
                .await
        };

        result.map_err(|e| {
            EventStoreError::SnapshotStoreError(format!("Failed to delete old snapshot: {}", e))
        })?;

        Ok(())
    }

    async fn insert_snapshot(
        &self,
        ctx: &mut dyn EventTransactionContext,
        snapshot: Snapshot<A>,
    ) -> Result<(), EventStoreError> {
        let result = if let Some(mongo_ctx) = ctx.as_any_mut().downcast_mut::<MongoContext>() {
            self.snapshots_collection
                .insert_one(snapshot)
                .session(&mut mongo_ctx.session)
                .await
        } else {
            self.snapshots_collection
                .insert_one(snapshot)
                .await
        };

        result.map_err(|e| {
            EventStoreError::SnapshotStoreError(format!("Failed to insert new snapshot: {}", e))
        })?;

        Ok(())
    }

    async fn find_events_since_version(
        &self,
        ctx: &mut dyn EventTransactionContext,
        id: &EntityId,
        min_version: u32,
    ) -> Result<Vec<StoredEvent>, EventStoreError> {
        let filter = doc! {
            "event.aggregate_id": id.as_uuid(),
            "event.sequence_number": { "$gte": min_version },
        };
        let find_options = FindOptions::builder()
            .sort(doc! { "event.sequence_number": 1 })
            .limit(None)
            .batch_size(100)
            .build();

        if let Some(mongo_ctx) = ctx.as_any_mut().downcast_mut::<MongoContext>() {
            let mut cursor = self
                .events_collection
                .find(filter)
                .session(&mut mongo_ctx.session)
                .with_options(find_options)
                .await?;
            let mut events = Vec::new();
            while let Some(event_record) = cursor.next(&mut mongo_ctx.session).await {
                events.push(event_record?);
            }
            Ok(events)
        } else {
            let mut cursor = self
                .events_collection
                .find(filter)
                .with_options(find_options)
                .await?;
            let mut events = Vec::new();
            while let Some(event_record) = cursor.next().await {
                events.push(event_record?);
            }
            Ok(events)
        }
    }

    async fn find_latest_snapshot(
        &self,
        ctx: &mut dyn EventTransactionContext,
        id: &EntityId,
    ) -> Result<Option<Snapshot<A>>, EventStoreError> {
        let filter = doc! { "aggregate_id": id.as_uuid() };
        let find_options = FindOneOptions::builder().sort(doc! { "version": -1 }).build();

        let result = if let Some(mongo_ctx) = ctx.as_any_mut().downcast_mut::<MongoContext>() {
            self.snapshots_collection
                .find_one(filter)
                .session(&mut mongo_ctx.session)
                .with_options(find_options)
                .await
        } else {
            self.snapshots_collection
                .find_one(filter)
                .with_options(find_options)
                .await
        };

        result.map_err(|e| {
            EventStoreError::SnapshotStoreError(format!("Snapshot load failed: {}", e))
        })
    }
}
