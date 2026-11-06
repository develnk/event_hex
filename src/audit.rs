use chrono::{DateTime, Utc};
use derive_getters::Getters;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Audit information attached to a persisted domain event.
///
/// The shared kernel does not interpret [`EventAudit::metadata()`]: it is an
/// application-defined JSON document (for example "who changed the aggregate and
/// when") which is stored in `Event::metadata` verbatim. This keeps the generic
/// event store free of any particular audit model.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventAudit {
    /// Application-defined audit payload.
    metadata: serde_json::Value,
    /// Point in time at which the change happened.
    occurred_at: DateTime<Utc>,
}

impl EventAudit {
    /// Creates audit information from an opaque metadata document and a timestamp.
    pub fn new(metadata: serde_json::Value, occurred_at: DateTime<Utc>) -> Self {
        Self { metadata, occurred_at }
    }

    /// Returns the application-defined audit payload.
    pub fn metadata(&self) -> &serde_json::Value {
        &self.metadata
    }

    /// Returns the moment of the change.
    pub fn occurred_at(&self) -> DateTime<Utc> {
        self.occurred_at
    }
}

/// Structure of the JSON document written to `Event::metadata`.
///
/// The event store does not interpret these fields; they are provided for
/// applications that want to describe the actor, the time and the lifecycle of
/// an aggregate change in a uniform way.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Getters)]
pub struct EventMetadata {
    /// Who produced the event.
    pub actor_id: Uuid,
    /// When the event was produced.
    pub occurred_at: DateTime<Utc>,
    /// When the aggregate was created.
    pub created_at: DateTime<Utc>,
    /// Who created the aggregate.
    pub created_by: Uuid,
    /// When the aggregate was last changed before this event.
    pub updated_at: DateTime<Utc>,
    /// Who performed the last change before this event.
    pub updated_by: Uuid,
    /// Whether the event deleted the aggregate.
    pub deleted: bool,
}

impl EventMetadata {
    /// Serializes metadata to the JSON value stored in `Event::metadata`.
    pub fn to_value(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or(serde_json::Value::Null)
    }

    /// Parses metadata from the JSON value stored in `Event::metadata`.
    pub fn from_value(value: &serde_json::Value) -> Option<Self> {
        serde_json::from_value(value.clone()).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_metadata_round_trips_through_json() {
        let metadata = EventMetadata {
            actor_id: Uuid::now_v7(),
            occurred_at: Utc::now(),
            created_at: Utc::now(),
            created_by: Uuid::now_v7(),
            updated_at: Utc::now(),
            updated_by: Uuid::now_v7(),
            deleted: true,
        };

        let restored = EventMetadata::from_value(&metadata.to_value()).expect("Metadata should be parsed");

        assert_eq!(restored, metadata);
        assert_eq!(*restored.actor_id(), metadata.actor_id);
        assert!(restored.deleted());
    }

    #[test]
    fn null_metadata_is_not_parsed() {
        assert!(EventMetadata::from_value(&serde_json::Value::Null).is_none());
    }
}
