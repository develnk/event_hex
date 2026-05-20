use derive_getters::Getters;
use event_hex::domain::EntityId;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

#[derive(Clone, Debug, Serialize, PartialEq, Deserialize, Getters, TypedBuilder)]
pub struct UserId {
    entity_id: EntityId,
}

impl UserId {
    pub fn new() -> Self {
        Self {
            entity_id: EntityId::new(),
        }
    }
}

impl Default for UserId {
    fn default() -> Self {
        Self {
            entity_id: EntityId::default(),
        }
    }
}

impl From<EntityId> for UserId {
    fn from(value: EntityId) -> Self {
        UserId::builder().entity_id(value).build()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Role {
    ADMIN,
    CUSTOMER,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Permissions {
    BASE,
}
