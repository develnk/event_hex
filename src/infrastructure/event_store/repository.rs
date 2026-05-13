use crate::application::ports::event_store_repository::EventStoreRepository;
use crate::application::ports::transaction::TransactionContext;
use crate::domain::domain::{AggregateContainer, AggregateRoot, EntityId};
use crate::domain::domain_event::{calculate_hash, convert_event_to_event_pre_record, DomainEvent, Event, Snapshot, StoredEvent};
use crate::infrastructure::event_store::storage::EventStoreStorage;
use crate::shared_kernel::errors::EventStoreError;
use async_trait::async_trait;
use chrono::Utc;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::sync::Arc;
use uuid::Uuid;

/// Event Store repository working through a storage abstraction.
pub struct EventStoreRepositoryImpl<A: AggregateRoot + Send + Sync + 'static> {
    storage: Arc<dyn EventStoreStorage<A>>,
    snapshot_threshold: u8,
}

impl<A> EventStoreRepositoryImpl<A>
where
    A: AggregateRoot + Send + Sync + 'static,
{
    pub fn new(storage: Arc<dyn EventStoreStorage<A>>, threshold: u8) -> Self {
        Self {
            storage,
            snapshot_threshold: threshold,
        }
    }
}

#[async_trait]
impl<A> EventStoreRepository<A> for EventStoreRepositoryImpl<A>
where
    A: AggregateRoot + DeserializeOwned + 'static,
{
    async fn save_aggregate(&self, ctx: &mut dyn TransactionContext, container: AggregateContainer<A>) -> Result<(), EventStoreError>
    where
        A::Event: DomainEvent + Serialize,
    {
        let aggregate_id = container.aggregate.id().to_owned();
        let aggregate_type = A::aggregate_type();

        let current_version = container.version();
        let mut next_version = current_version.clone();

        let last_event = self.storage.find_last_event(ctx, &aggregate_id, aggregate_type).await?;
        let mut last_event_hash: Vec<u8> = last_event.map_or(Vec::default(), |e| e.hash);

        let new_event_records: Vec<StoredEvent> = container
            .get_events()
            .iter()
            .cloned()
            .enumerate()
            .map(|(_i, event)| {
                next_version += 1;
                let pre_event = convert_event_to_event_pre_record(&event);
                let new_event = Event {
                    id: Uuid::now_v7(),
                    aggregate_id: aggregate_id.into(),
                    aggregate_type: aggregate_type.to_string(),
                    sequence_number: next_version,
                    event_type: pre_event.event_type,
                    payload: pre_event.event,
                    metadata: pre_event.metadata,
                    timestamp: Utc::now(),
                    previous_hash: last_event_hash.clone(),
                };

                let current_hash = calculate_hash(&new_event).unwrap_or_default();
                let event_store = StoredEvent {
                    event: new_event,
                    hash: current_hash.clone(),
                };
                last_event_hash = current_hash;
                event_store
            })
            .collect::<Vec<StoredEvent>>();

        self.storage.insert_events(ctx, new_event_records).await?;

        if current_version > 0 && current_version % self.snapshot_threshold as u32 == 0 {
            let snapshot = Snapshot::<A> {
                aggregate_id: aggregate_id.as_uuid(),
                aggregate_type: aggregate_type.to_owned(),
                version: next_version,
                timestamp: Utc::now(),
                data: container.aggregate,
            };

            self.storage.delete_snapshot(ctx, &aggregate_id, aggregate_type).await?;
            self.storage.insert_snapshot(ctx, snapshot).await?;
        }

        Ok(())
    }

    async fn load_aggregate(&self, ctx: &mut dyn TransactionContext, id: &EntityId) -> Result<Option<AggregateContainer<A>>, EventStoreError> {
        let snapshot_result: Option<Snapshot<A>> = self.storage.find_latest_snapshot(ctx, id).await?;
        let version = snapshot_result.as_ref().map_or(0, |s| s.version);
        let events = <Self as EventStoreRepository<A>>::get_events_since_version(self, ctx, id, version).await?;

        let mut aggregate: AggregateContainer<A>;

        if let Some(snapshot) = snapshot_result {
            aggregate = AggregateContainer::new(snapshot.data);
            aggregate.restore_from_snapshot(events[1..].to_vec().as_ref())?;
        } else {
            aggregate = AggregateContainer::restore_from_history(id, events.as_ref())?;
        }

        let verifying = verify_event_chain(id, A::aggregate_type(), &events.to_owned());
        if Result::is_err(&verifying) {
            Err(verifying.err().unwrap())
        } else {
            Ok(Some(aggregate))
        }
    }

    async fn get_events_since_version(&self, ctx: &mut dyn TransactionContext, id: &EntityId, min_version: u32) -> Result<Vec<StoredEvent>, EventStoreError> {
        self.storage.find_events_since_version(ctx, id, min_version).await
    }

    async fn get_latest_snapshot(&self, ctx: &mut dyn TransactionContext, id: &EntityId) -> Result<Option<Snapshot<A>>, EventStoreError> {
        self.storage.find_latest_snapshot(ctx, id).await
    }
}

/// MongoDB Event Store repository — backward compatibility with existing code.
pub type MongoEventStoreRepository<A> = EventStoreRepositoryImpl<A>;

/// Event chain integrity verification.
pub fn verify_event_chain(aggregate_id: &EntityId, aggregate_type: &str, events: &[StoredEvent]) -> Result<(), EventStoreError> {
    for i in 0..events.len() {
        let current_record = &events[i];
        if current_record.event.previous_hash.is_empty() {
            continue;
        }

        if i > 0 {
            let previous_record = &events[i - 1];
            let expected_previous_hash = &previous_record.hash;

            if current_record.event.previous_hash != expected_previous_hash.to_owned() {
                return Err(EventStoreError::EventChainVerifyError {
                    aggregate_id: aggregate_id.as_uuid(),
                    aggregate_type: aggregate_type.to_string(),
                    version: current_record.event.sequence_number,
                });
            }
        }

        match calculate_hash(&current_record.event) {
            Ok(recalculated_hash) => {
                if recalculated_hash != current_record.hash {
                    return Err(EventStoreError::EventChainVerifyError {
                        aggregate_id: aggregate_id.as_uuid(),
                        aggregate_type: aggregate_type.to_string(),
                        version: current_record.event.sequence_number,
                    });
                }
            }
            Err(e) => {
                println!("Error recalculating hash for event at index {}: {}", i, e);
                return Err(EventStoreError::EventChainSerializeError {
                    aggregate_id: aggregate_id.as_uuid(),
                    aggregate_type: aggregate_type.to_string(),
                    version: current_record.event.sequence_number,
                });
            }
        }
    }

    Ok(())
}
