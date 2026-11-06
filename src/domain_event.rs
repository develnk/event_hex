use crate::audit::EventAudit;
use crate::domain::{AggregateRoot, EntityId};
use crate::errors::DomainEventHandlerError;
use crate::types::SequenceNumber;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use erased_serde::Serialize as ErasedSerialize;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::any::{Any, TypeId};
use std::fmt;
use std::fmt::Debug;
use std::sync::Arc;
use uuid::Uuid;

/// A domain event persisted in the event store.
///
/// Carries the event payload, metadata and enough information to reconstruct
/// the aggregate and verify the hash chain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// The aggregate instance that emitted the event.
    #[cfg_attr(
        feature = "mongo",
        serde(with = "bson::serde_helpers::uuid_1::AsBinary")
    )]
    pub aggregate_id: Uuid,
    /// The aggregate type that emitted the event.
    pub aggregate_type: String,
    /// The sequence number of the event, within its specific aggregate instance.
    pub sequence_number: SequenceNumber,
    /// Type of event.
    pub event_type: String,
    /// Event Payload.
    pub payload: Value,
    /// Event Metadata.
    pub metadata: Value,
    /// Time when the event was created.
    pub timestamp: DateTime<Utc>,
    /// Hash of the previous event in the chain.
    pub previous_hash: Vec<u8>,
}

/// The structure stored in the database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredEvent {
    /// The event itself.
    pub event: Event,
    /// SHA-256 hash of the event, used to verify the event chain integrity.
    pub hash: Vec<u8>,
}

/// Takes an event, serializes it, and computes a SHA-256 hash.
pub fn calculate_hash(event: &Event) -> Result<Vec<u8>, bincode::error::EncodeError> {
    // Event serialization to bytes. It is important to use a deterministic format.
    let config = bincode::config::standard();
    let serialized_event = bincode::serde::encode_to_vec(event.to_owned(), config)?;

    // Calculate SHA-256 hash
    let mut hasher = Sha256::new();
    hasher.update(serialized_event);
    let hash = hasher.finalize();

    Ok(hash.as_slice().to_vec())
}

impl fmt::Display for StoredEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let hash_hex = self.hash.iter().map(|byte| format!("{:02x}", byte)).collect::<String>();

        let prev_hash_hex = self
            .event
            .previous_hash
            .clone()
            .into_iter()
            .map(|byte| format!("{:02x}", byte))
            .collect::<String>();

        write!(
            f,
            "Event ID: {}\nTimestamp: {}\nPayload: \"{}\"\nPrevious Hash: {}\nCurrent Hash: {}\n---",
            self.event.sequence_number,
            self.event.timestamp,
            self.event.payload,
            prev_hash_hex,
            hash_hex
        )
    }
}

/// Structure representing a new event before saving to the database.
/// Used to pass data to the append method.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventPreRecord {
    /// Metadata attached to the event.
    pub metadata: Value,
    /// Serialized event payload.
    pub event: Value,
    /// Type of the event.
    pub event_type: String,
}

/// Converts a batch of domain events into event-store records.
pub fn get_event_records<T: DomainEvent + Serialize>(events: Vec<&T>) -> Vec<EventPreRecord> {
    events
        .into_iter()
        .map(|event| {
            // @TODO temporarily empty metadata.
            let metadata: Value = serde_json::to_value("").unwrap();

            EventPreRecord {
                metadata,
                event: serde_json::to_value(event).unwrap_or(Value::Null),
                event_type: event.event_type_name().to_string(),
            }
        })
        .collect()
}

/// Converts a single domain event into an event-store record.
pub fn convert_event_to_event_pre_record<E: DomainEvent + Serialize>(event: &E) -> EventPreRecord {
    EventPreRecord {
        // @TODO temporarily empty metadata.
        metadata: Value::Null,
        event: serde_json::to_value(event).unwrap_or(Value::Null),
        event_type: event.event_type_name().to_string(),
    }
}

/// Converts a domain event to an event-store record together with its audit metadata.
pub fn convert_event_to_event_pre_record_with_metadata<E: DomainEvent + Serialize>(event: &E, metadata: Value) -> EventPreRecord {
    EventPreRecord {
        metadata,
        event: serde_json::to_value(event).unwrap_or(Value::Null),
        event_type: event.event_type_name().to_string(),
    }
}

/// Common trait for all domain events.
#[async_trait]
pub trait DomainEvent: Debug + ErasedSerialize + Send + Sync + 'static {
    /// Unique event ID
    fn new_event_id(&self) -> Uuid {
        Uuid::now_v7()
    }
    /// Returns the id of the aggregate this event belongs to.
    fn aggregate_id(&self) -> EntityId {
        EntityId::new()
    }
    /// Event creation time
    fn occurred_on(&self) -> DateTime<Utc> {
        Utc::now()
    }
    /// Event type (string representation, useful for logging/dispatching)
    fn event_type_name(&self) -> String;

    /// Returns the [`TypeId`] of the concrete event type.
    ///
    /// Used to match events with their registered handlers.
    fn event_type_id(&self) -> TypeId {
        TypeId::of::<Self>()
    }

    /// Returns a reference to the concrete event behind `dyn Any`.
    fn as_any(&self) -> &dyn Any;
    /// Returns the concrete event behind `Arc<dyn Any + Send + Sync>`.
    fn as_any_arc(self: Arc<Self>) -> Arc<dyn Any + Send + Sync>;
}
erased_serde::serialize_trait_object!(DomainEvent);

/// Common trait for event handlers.
#[async_trait]
pub trait DomainEventHandler<E>: Debug + Send + Sync + 'static
where
    E: DomainEvent,
{
    /// Handles a single domain event.
    async fn handle(&self, event: &E);
}

/// Creates a [`DomainEventHandler`] for a given event type.
#[async_trait]
pub trait DomainEventHandlerFactory<E>: Send + Sync
where
    E: DomainEvent,
{
    /// Creates a new handler instance.
    async fn create(&self) -> Result<Box<dyn DomainEventHandler<E>>, DomainEventHandlerError>;
}

/// A snapshot of an aggregate's state at a point in time.
///
/// Snapshots avoid replaying all events from the beginning when loading an
/// aggregate.
#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Snapshot<A>
where
    A: AggregateRoot,
    A: Default,
{
    /// The id of the aggregate this snapshot belongs to.
    #[cfg_attr(
        feature = "mongo",
        serde(with = "bson::serde_helpers::uuid_1::AsBinary")
    )]
    pub aggregate_id: Uuid,
    /// The type of the aggregate.
    pub aggregate_type: String,
    /// The aggregate version at the moment the snapshot was taken.
    pub version: u32,
    /// The point in time at which the snapshot was taken.
    pub timestamp: DateTime<Utc>,
    /// The serialized aggregate state.
    pub data: A,
    /// Audit information of the aggregate at the moment the snapshot was taken.
    #[serde(default)]
    pub audit: Option<EventAudit>,
}
