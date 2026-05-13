use crate::{
    domain::identity_access_management::identity::aggregate::entity::user::User,
    shared_kernel::model::common::{Permissions, Role},
};
use derive_getters::Getters;
use event_hex::domain::domain::EntityId;
use event_hex::domain::domain_event::DomainEvent;
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(
    tag = "type",
    content = "data"
)] // This will create a structure { "type": "Created", "data": { ... } }
pub enum AccessAccountEvents {
    #[serde(rename = "AccessAccountCreatedEvent")]
    Created(AccessAccountCreateVersioned),
    #[serde(rename = "AccessAccountUpdatedEvent")]
    Updated(AccessAccountUpdateVersioned),
}

impl DomainEvent for AccessAccountEvents {
    fn event_type_name(&self) -> String {
        match self {
            AccessAccountEvents::Created { .. } => "AccessAccountCreatedEvent".to_string(),
            AccessAccountEvents::Updated { .. } => "AccessAccountUpdatedEvent".to_string(),
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_arc(self: Arc<Self>) -> Arc<dyn Any + Send + Sync> {
        self
    }
}

//----------------------------------------------------------------------------------------------------
#[derive(Debug, Deserialize, Serialize, Clone, Getters)]
pub struct AccessAccountCreatedV1 {
    id: EntityId,
    user: User,
    roles: Vec<Role>,
    permission: Vec<Permissions>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "version")]
pub enum AccessAccountCreateVersioned {
    #[serde(rename = "1")]
    V1(AccessAccountCreatedV1),
}

impl AccessAccountCreateVersioned {
    pub fn new_v1(
        id: EntityId, user: User, roles: Vec<Role>, permission: Vec<Permissions>,
    ) -> Self {
        Self::V1(AccessAccountCreatedV1 {
            id,
            user,
            roles,
            permission,
        })
    }

    pub fn id(&self) -> &EntityId {
        match self {
            Self::V1(v) => &v.id,
        }
    }

    pub fn to_latest(&self) -> AccessAccountCreatedV1 {
        match self {
            Self::V1(v) => v.clone(),
        }
    }
}

//----------------------------------------------------------------------------------------------------
#[derive(Debug, Deserialize, Serialize, Clone, Getters)]
pub struct AccessAccountUpdatedV1 {
    id: EntityId,
    roles: Vec<Role>,
    permission: Vec<Permissions>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "version")]
pub enum AccessAccountUpdateVersioned {
    #[serde(rename = "1")]
    V1(AccessAccountUpdatedV1),
}

impl AccessAccountUpdateVersioned {
    pub fn new_v1(
        id: EntityId, roles: Vec<Role>, permission: Vec<Permissions>,
    ) -> Self {
        Self::V1(AccessAccountUpdatedV1 {
            id,
            roles,
            permission,
        })
    }

    pub fn id(&self) -> &EntityId {
        match self {
            Self::V1(v) => &v.id,
        }
    }

    pub fn to_latest(&self) -> AccessAccountUpdatedV1 {
        match self {
            Self::V1(v) => v.clone(),
        }
    }
}
//----------------------------------------------------------------------------------------------------
