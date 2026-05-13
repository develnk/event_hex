use crate::domain::domain::{AggregateRoot, EntityId};
use crate::shared_kernel::errors::DomainEventHandlerError;
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: Uuid,
    /// The aggregate instance that emitted the event.
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

/// The structure stored in the database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredEvent {
    pub event: Event,
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
    pub metadata: Value,
    pub event: Value,
    pub event_type: String,
}

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

pub fn convert_event_to_event_pre_record<E: DomainEvent + Serialize>(event: &E) -> EventPreRecord {
    EventPreRecord {
        // @TODO temporarily empty metadata.
        metadata: Value::Null,
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
    fn aggregate_id(&self) -> EntityId {
        EntityId::new()
    }
    /// Event creation time
    fn occurred_on(&self) -> DateTime<Utc> {
        Utc::now()
    }
    /// Event type (string representation, useful for logging/dispatching)
    fn event_type_name(&self) -> String;

    fn event_type_id(&self) -> TypeId {
        TypeId::of::<Self>()
    }

    // Additional method to get a reference to Any
    fn as_any(&self) -> &dyn Any;
    fn as_any_arc(self: Arc<Self>) -> Arc<dyn Any + Send + Sync>;
}
erased_serde::serialize_trait_object!(DomainEvent);

/// Common trait for event handlers.
#[async_trait]
pub trait DomainEventHandler<E>: Debug + Send + Sync + 'static
where
    E: DomainEvent,
{
    async fn handle(&self, event: &E);
}

#[async_trait]
pub trait DomainEventHandlerFactory<E>: Send + Sync
where
    E: DomainEvent,
{
    async fn create(&self) -> Result<Box<dyn DomainEventHandler<E>>, DomainEventHandlerError>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Snapshot<A>
where
    A: AggregateRoot,
{
    pub aggregate_id: Uuid,
    pub aggregate_type: String,
    pub version: u32,
    pub timestamp: DateTime<Utc>,
    pub data: A,
}
