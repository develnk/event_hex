use crate::shared_kernel::auditable::Auditable;
use crate::shared_kernel::domain_event::{DomainEvent, StoredEvent};
use crate::shared_kernel::errors::DomainError;
#[cfg(feature = "mongo")]
use bson::Bson;
use serde::de::DeserializeOwned;
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct EntityId(Uuid);

impl EntityId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }

    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl Default for EntityId {
    fn default() -> Self {
        Self(Uuid::nil())
    }
}

#[cfg(feature = "mongo")]
impl From<EntityId> for Bson {
    fn from(id: EntityId) -> Self {
        Bson::String(id.as_uuid().to_string())
    }
}

#[cfg(feature = "mongo")]
impl From<EntityId> for bson::Uuid {
    fn from(id: EntityId) -> Self {
        bson::Uuid::from_bytes(id.as_uuid().into_bytes())
    }
}


impl std::fmt::Display for EntityId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for EntityId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl From<EntityId> for Uuid {
    fn from(id: EntityId) -> Self {
        id.0
    }
}

pub trait AggregateRoot: Sized + Send + Sync + Serialize {
    type Event: DomainEvent + Clone + Serialize + DeserializeOwned;

    /// Returns the unique identifier of the aggregate.
    fn id(&self) -> &EntityId;

    // Prepare a string representation for each aggregate
    fn aggregate_type() -> &'static str;

    /// Applies an event to restore state.
    fn apply_event(&mut self, event: Self::Event);

    // Convert a stored event from the database into a domain event.
    fn convert_to_domain_event(stored_event: StoredEvent) -> Result<Self::Event, DomainError> {
        let mut val = stored_event.event.payload;
        // Add type field directly into the data object to deserialize the entire enum at once
        if let Some(obj) = val.as_object_mut() {
            obj.insert(
                "type".to_string(),
                serde_json::Value::String(stored_event.event.event_type),
            );
        }
        Ok(serde_json::from_value::<Self::Event>(val)?)
    }

    /// Creates a new aggregate with default state.
    fn first_state(id: &EntityId) -> Self;

    /// Returns the current version of the aggregate (number of applied events).
    fn get_version(&self) -> u32;

    fn set_version(&mut self, version: u32);

    fn increment_version(&mut self);
}

// This wrapper separates domain model data from infrastructure logic (events).
pub struct AggregateContainer<A: AggregateRoot> {
    pub aggregate: A,
    uncommitted_events: Vec<A::Event>,
    audit: Auditable,
}

impl<A: AggregateRoot> AggregateContainer<A> {
    /// Creates a new container (e.g., when creating a new aggregate)
    pub fn new(aggregate: A) -> Self {
        Self {
            aggregate,
            uncommitted_events: Vec::new(),
            // TODO Implement the logic later.
            audit: Default::default(),
        }
    }

    pub fn version(&self) -> u32 {
        self.aggregate.get_version()
    }

    /// Restores the aggregate from events from the very beginning.
    /// TODO Change stored_events: &Vec<StoredEvent> to events: Vec<A::Event> to avoid depending on the event storage structure in the DB
    pub fn restore_from_history(
        entity_id: &EntityId, stored_events: &Vec<StoredEvent>,
    ) -> Result<Self, DomainError> {
        if stored_events.is_empty() {
            return Err(DomainError::AggregateNotFound {
                aggregate_id: entity_id.as_uuid().into(),
                aggregate_type: String::from("Unknown"),
            });
        }

        let mut aggregate = A::first_state(entity_id);

        for stored_event in stored_events {
            let domain_event = A::convert_to_domain_event(stored_event.to_owned())?;
            aggregate.apply_event(domain_event);
        }

        Ok(Self {
            aggregate,
            uncommitted_events: Vec::new(),
            audit: Default::default(),
        })
    }

    pub fn restore_from_snapshot(
        &mut self, after_events: &Vec<StoredEvent>,
    ) -> Result<(), DomainError> {
        for stored_event in after_events {
            let domain_event = A::convert_to_domain_event(stored_event.to_owned())?;
            self.aggregate.apply_event(domain_event);
        }

        Ok(())
    }

    /// Main method for executing commands.
    /// Takes an event, applies it to the aggregate, and saves it to the queue.
    pub fn push_event(&mut self, event: A::Event) {
        // 1. Apply to the internal state
        self.aggregate.apply_event(event.to_owned());

        // 2. Save for DB/EventStore
        self.uncommitted_events.push(event);
    }

    pub fn get_events(&self) -> Vec<A::Event> {
        self.uncommitted_events.clone()
    }

    pub fn get_erased_events(&self) -> Vec<Box<dyn DomainEvent>> {
        let events = self.uncommitted_events.clone();
        events
            .into_iter() // consume the vector, ownership moves to the iterator
            .map(|event| Box::new(event) as Box<dyn DomainEvent>)
            .collect()
    }

    pub fn take_events(&mut self) -> Vec<A::Event> {
        std::mem::take(&mut self.uncommitted_events)
    }

    pub fn take_erased_events(&mut self) -> Vec<Box<dyn DomainEvent>> {
        let events = std::mem::take(&mut self.uncommitted_events);
        events
            .into_iter() // consume the vector, ownership moves to the iterator
            .map(|event| Box::new(event) as Box<dyn DomainEvent>)
            .collect()
    }
}
