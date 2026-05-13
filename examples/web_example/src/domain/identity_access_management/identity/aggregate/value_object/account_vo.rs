use derive_getters::Getters;
use event_hex::domain::domain::EntityId;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize, Getters, TypedBuilder)]
pub struct AccessAccountId {
    entity_id: EntityId,
}

impl AccessAccountId {
    pub fn new() -> Self {
        Self {
            entity_id: EntityId::new(),
        }
    }
}

impl Default for AccessAccountId {
    fn default() -> Self {
        Self {
            entity_id: Default::default(),
        }
    }
}

impl From<EntityId> for AccessAccountId {
    fn from(entity_id: EntityId) -> Self {
        Self { entity_id }
    }
}

impl From<Uuid> for AccessAccountId {
    fn from(value: Uuid) -> Self {
        Self {
            entity_id: EntityId::from(value),
        }
    }
}
