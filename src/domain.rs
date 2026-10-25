use crate::audit::EventAudit;
use crate::domain_event::{DomainEvent, StoredEvent};
use crate::errors::DomainError;
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

    /// Creates audit information for a newly constructed aggregate.
    ///
    /// Called once by [`AggregateContainer::new`]. The returned audit is attached
    /// to the first event of the aggregate. The content of
    /// [`EventAudit::metadata`] is opaque for the event store.
    ///
    /// The default implementation makes the aggregate non-auditable.
    fn init_audit(&mut self) -> Option<EventAudit> {
        None
    }

    /// Updates the audit information after a command produced an event.
    ///
    /// Called by [`AggregateContainer::push_event`] after the event has been
    /// applied. The returned audit is attached to that event.
    ///
    /// The default implementation makes the aggregate non-auditable.
    fn touch_audit(&mut self) -> Option<EventAudit> {
        None
    }

    /// Restores audit information persisted in the event store.
    ///
    /// Called by [`AggregateContainer`] when replaying stored events or restoring
    /// from a snapshot. `metadata` may be [`serde_json::Value::Null`] for events
    /// written before audit support, in which case the aggregate may fall back to
    /// [`EventAudit::occurred_at`].
    ///
    /// The default implementation is a no-op.
    fn restore_audit(&mut self, _audit: &EventAudit) {}

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
    /// Events that were applied to the aggregate but are not persisted yet.
    uncommitted_events: Vec<A::Event>,
    /// Audit information for each uncommitted event (same order as `uncommitted_events`).
    event_audits: Vec<Option<EventAudit>>,
    /// Audit information of the aggregate as a whole.
    audit: Option<EventAudit>,
}

impl<A: AggregateRoot> AggregateContainer<A> {
    /// Creates a new container for a brand-new aggregate.
    ///
    /// The aggregate is asked for its initial audit information (see
    /// [`AggregateRoot::init_audit`]); non-auditable aggregates simply return
    /// `None`.
    pub fn new(mut aggregate: A) -> Self {
        let audit = aggregate.init_audit();
        Self {
            aggregate,
            uncommitted_events: Vec::new(),
            event_audits: Vec::new(),
            audit,
        }
    }

    /// Restores a container from a snapshot with its persisted audit information.
    pub fn from_snapshot(mut aggregate: A, audit: Option<EventAudit>) -> Self {
        if let Some(audit) = &audit {
            aggregate.restore_audit(audit);
        }
        Self {
            aggregate,
            uncommitted_events: Vec::new(),
            event_audits: Vec::new(),
            audit,
        }
    }

    /// Returns the audit information of the aggregate as a whole.
    pub fn audit(&self) -> Option<&EventAudit> {
        self.audit.as_ref()
    }

    pub fn version(&self) -> u32 {
        self.aggregate.get_version()
    }

    /// Restores the aggregate from events from the very beginning.
    /// TODO Change stored_events: &Vec<StoredEvent> to events: Vec<A::Event> to avoid depending on the event storage structure in the DB
    pub fn restore_from_history(entity_id: &EntityId, stored_events: &Vec<StoredEvent>) -> Result<Self, DomainError> {
        if stored_events.is_empty() {
            return Err(DomainError::AggregateNotFound {
                aggregate_id: entity_id.as_uuid().into(),
                aggregate_type: String::from("Unknown"),
            });
        }

        let mut container = Self {
            aggregate: A::first_state(entity_id),
            uncommitted_events: Vec::new(),
            event_audits: Vec::new(),
            audit: None,
        };

        for stored_event in stored_events {
            container.apply_stored_event(stored_event)?;
        }

        Ok(container)
    }

    pub fn restore_from_snapshot(&mut self, after_events: &Vec<StoredEvent>) -> Result<(), DomainError> {
        for stored_event in after_events {
            self.apply_stored_event(stored_event)?;
        }

        Ok(())
    }

    fn apply_stored_event(&mut self, stored_event: &StoredEvent) -> Result<(), DomainError> {
        let domain_event = A::convert_to_domain_event(stored_event.to_owned())?;

        // Audit information is written to the event metadata when the event is stored.
        // Older events may not contain it, but the timestamp is always available.
        let audit = EventAudit::new(stored_event.event.metadata.clone(), stored_event.event.timestamp);

        self.aggregate.apply_event(domain_event);

        // Restore after applying the event so that the persisted audit information
        // wins over anything `apply_event` changed in the aggregate.
        self.aggregate.restore_audit(&audit);
        self.audit = Some(audit);

        Ok(())
    }

    /// Main method for executing commands.
    /// Takes an event, applies it to the aggregate, and saves it to the queue.
    pub fn push_event(&mut self, event: A::Event) {
        // 1. Apply to the internal state.
        self.aggregate.apply_event(event.to_owned());

        // 2. Update the audit information of the aggregate; it is snapshotted
        //    for the event so that the event store can persist it.
        let audit = self.aggregate.touch_audit();
        if let Some(audit) = &audit {
            self.audit = Some(audit.clone());
        }

        // 3. Save the event and its audit information for the event store.
        self.event_audits.push(audit);
        self.uncommitted_events.push(event);
    }

    pub fn get_events(&self) -> Vec<A::Event> {
        self.uncommitted_events.clone()
    }

    /// Returns audit information for the uncommitted events, in the same order.
    pub fn get_event_audits(&self) -> Vec<Option<EventAudit>> {
        self.event_audits.clone()
    }

    pub fn get_erased_events(&self) -> Vec<Box<dyn DomainEvent>> {
        let events = self.uncommitted_events.clone();
        events
            .into_iter() // consume the vector, ownership moves to the iterator
            .map(|event| Box::new(event) as Box<dyn DomainEvent>)
            .collect()
    }

    pub fn take_events(&mut self) -> Vec<A::Event> {
        self.event_audits.clear();
        std::mem::take(&mut self.uncommitted_events)
    }

    pub fn take_erased_events(&mut self) -> Vec<Box<dyn DomainEvent>> {
        self.event_audits.clear();
        let events = std::mem::take(&mut self.uncommitted_events);
        events
            .into_iter() // consume the vector, ownership moves to the iterator
            .map(|event| Box::new(event) as Box<dyn DomainEvent>)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::EventAudit;
    use crate::domain_event::DomainEvent;
    use chrono::{DateTime, Utc};
    use serde_json::{json, Value};
    use std::any::Any;
    use std::sync::Arc;

    const OCCURRED_AT_TIMESTAMP: i64 = 1_700_000_000;

    fn occurred_at() -> DateTime<Utc> {
        DateTime::from_timestamp(OCCURRED_AT_TIMESTAMP, 0).expect("Valid timestamp")
    }

    #[derive(Debug, Clone, Serialize, serde::Deserialize)]
    struct TestEvent {
        value: u32,
        occurred_at: DateTime<Utc>,
    }

    impl DomainEvent for TestEvent {
        fn event_type_name(&self) -> String {
            "TestEvent".to_string()
        }

        fn as_any(&self) -> &dyn Any {
            self
        }

        fn as_any_arc(self: Arc<Self>) -> Arc<dyn Any + Send + Sync> {
            Arc::new(self)
        }
    }

    /// Aggregate which stores its audit information as opaque JSON metadata.
    #[derive(Debug, Clone, Default, Serialize, serde::Deserialize)]
    struct TestAggregate {
        id: EntityId,
        value: u32,
        audit_metadata: Value,
    }

    impl TestAggregate {
        fn audit(&self) -> EventAudit {
            EventAudit::new(self.audit_metadata.clone(), occurred_at())
        }
    }

    impl AggregateRoot for TestAggregate {
        type Event = TestEvent;

        fn id(&self) -> &EntityId {
            &self.id
        }

        fn aggregate_type() -> &'static str {
            "TestAggregate"
        }

        fn apply_event(&mut self, event: Self::Event) {
            self.value = event.value;
        }

        fn init_audit(&mut self) -> Option<EventAudit> {
            self.audit_metadata = json!({ "actor": "creator", "value": self.value });
            Some(self.audit())
        }

        fn touch_audit(&mut self) -> Option<EventAudit> {
            self.audit_metadata = json!({ "actor": "updater", "value": self.value });
            Some(self.audit())
        }

        fn restore_audit(&mut self, audit: &EventAudit) {
            self.audit_metadata = audit.metadata().clone();
        }

        fn first_state(id: &EntityId) -> Self {
            Self {
                id: *id,
                ..Default::default()
            }
        }

        fn get_version(&self) -> u32 {
            0
        }

        fn set_version(&mut self, _version: u32) {}

        fn increment_version(&mut self) {}
    }

    fn stored_event(aggregate_id: &EntityId, sequence_number: u32, value: u32, metadata: Value) -> StoredEvent {
        StoredEvent {
            event: crate::domain_event::Event {
                aggregate_id: aggregate_id.as_uuid(),
                aggregate_type: TestAggregate::aggregate_type().to_string(),
                sequence_number,
                event_type: "TestEvent".to_string(),
                payload: serde_json::to_value(TestEvent { value, occurred_at: occurred_at() }).unwrap(),
                metadata,
                timestamp: occurred_at(),
                previous_hash: Vec::new(),
            },
            hash: Vec::new(),
        }
    }

    #[test]
    fn push_event_snapshots_audit_for_every_event() {
        let mut container = AggregateContainer::new(TestAggregate::first_state(&EntityId::new()));

        container.push_event(TestEvent {
            value: 1,
            occurred_at: occurred_at(),
        });
        container.push_event(TestEvent {
            value: 2,
            occurred_at: occurred_at(),
        });

        let audits = container.get_event_audits();
        assert_eq!(audits.len(), 2);
        assert_eq!(audits[0].as_ref().unwrap().metadata()["value"], json!(1));
        assert_eq!(audits[1].as_ref().unwrap().metadata()["value"], json!(2));
        assert_eq!(container.audit().unwrap().metadata()["actor"], json!("updater"));
    }

    #[test]
    fn audit_is_restored_from_stored_events() {
        let aggregate_id = EntityId::new();
        let events = vec![
            stored_event(&aggregate_id, 1, 1, json!({ "actor": "creator", "value": 1 })),
            stored_event(&aggregate_id, 2, 2, json!({ "actor": "updater", "value": 2 })),
        ];

        let container = AggregateContainer::<TestAggregate>::restore_from_history(&aggregate_id, &events).expect("Should restore");

        assert_eq!(container.aggregate.value, 2);
        assert_eq!(container.aggregate.audit_metadata["actor"], json!("updater"));
        assert_eq!(container.audit().unwrap().metadata()["value"], json!(2));
    }

    #[test]
    fn audit_is_restored_from_a_snapshot() {
        let aggregate_id = EntityId::new();
        let mut aggregate = TestAggregate::first_state(&aggregate_id);
        aggregate.value = 7;

        let snapshot_audit = EventAudit::new(json!({ "actor": "creator", "value": 7 }), occurred_at());
        let container = AggregateContainer::from_snapshot(aggregate, Some(snapshot_audit.clone()));

        assert_eq!(container.aggregate.audit_metadata, *snapshot_audit.metadata());
        assert_eq!(container.audit(), Some(&snapshot_audit));
    }

    /// Aggregate without audit support: the default hooks must keep it working.
    #[derive(Debug, Clone, Default, Serialize, serde::Deserialize)]
    struct PlainAggregate {
        id: EntityId,
        value: u32,
    }

    impl AggregateRoot for PlainAggregate {
        type Event = TestEvent;

        fn id(&self) -> &EntityId {
            &self.id
        }

        fn aggregate_type() -> &'static str {
            "PlainAggregate"
        }

        fn apply_event(&mut self, event: Self::Event) {
            self.value = event.value;
        }

        fn first_state(id: &EntityId) -> Self {
            Self {
                id: *id,
                ..Default::default()
            }
        }

        fn get_version(&self) -> u32 {
            0
        }

        fn set_version(&mut self, _version: u32) {}

        fn increment_version(&mut self) {}
    }

    #[test]
    fn plain_aggregates_do_not_produce_audit_metadata() {
        let mut container = AggregateContainer::new(PlainAggregate::first_state(&EntityId::new()));
        container.push_event(TestEvent {
            value: 1,
            occurred_at: occurred_at(),
        });

        assert!(container.audit().is_none());
        assert_eq!(container.get_event_audits(), vec![None]);

        // Stored events still carry their timestamp, which is enough to restore
        // audit information for aggregates that do not track it themselves.
        let aggregate_id = EntityId::new();
        let events = vec![stored_event(&aggregate_id, 1, 1, Value::Null)];
        let restored = AggregateContainer::<PlainAggregate>::restore_from_history(&aggregate_id, &events).expect("Should restore");
        assert_eq!(restored.audit().expect("Event timestamp is used").occurred_at(), occurred_at());
    }
}
