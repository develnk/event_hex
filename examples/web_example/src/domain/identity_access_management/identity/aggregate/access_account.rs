use crate::domain::identity_access_management::identity::aggregate::value_object::email::Email;
use crate::domain::identity_access_management::identity::event::access_account_events::{
    AccessAccountCreateVersioned, AccessAccountCreatedV1, AccessAccountEvents,
    AccessAccountUpdateVersioned, AccessAccountUpdatedV1,
};
use crate::{
    domain::identity_access_management::identity::aggregate::{
        entity::user::User,
        value_object::account_vo::AccessAccountId,
    },
    shared_kernel::model::common::{Permissions, Role},
};
use derive_getters::Getters;
use event_hex::domain::domain::{AggregateRoot, EntityId};
use event_hex::shared_kernel::errors::DomainError;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use typed_builder::TypedBuilder;

#[serde_as]
#[derive(Debug, Serialize, Deserialize, TypedBuilder, Getters)]
pub struct AccessAccountAggregateRoot {
    pub id: AccessAccountId,
    pub user: User,
    pub roles: Vec<Role>,
    pub permission: Vec<Permissions>,
    // Current aggregate version (number of applied events)
    version: u32,
}

impl AccessAccountAggregateRoot {
    // Method for creating a new aggregate
    pub async fn new_access_account(&mut self, first_name: String, last_name: String, email: String) -> Result<AccessAccountEvents, DomainError> {
        // Inside Email invariants are checked
        let email_vo = Email::new(&self, email, "AccessAccountCreatedV1".to_string())?;

        let event = AccessAccountEvents::Created(AccessAccountCreateVersioned::new_v1(
            self.id().entity_id().to_owned(),
            User::new(first_name, last_name, email_vo.into_inner()),
            vec![Role::CUSTOMER],
            vec![Permissions::BASE],
        ));

        Ok(event)
    }

    pub async fn update_access_account(&mut self, roles: Vec<Role>, permission: Vec<Permissions>) -> Result<AccessAccountEvents, DomainError> {
        let event = AccessAccountEvents::Updated(AccessAccountUpdateVersioned::new_v1(self.id().entity_id().to_owned(), roles, permission));
        Ok(event)
    }

    fn apply_created(&mut self, event: AccessAccountCreatedV1) {
        self.user = event.user().clone();
        self.roles = event.roles().clone();
        self.permission = event.permission().clone();
        self.increment_version();
    }

    fn apply_updated(&mut self, event: AccessAccountUpdatedV1) {
        self.roles = event.roles().clone();
        self.permission = event.permission().clone();
        self.increment_version();
    }

    pub fn new() -> Self {
        Self::first_state(&EntityId::new())
    }
}

impl Default for AccessAccountAggregateRoot {
    fn default() -> Self {
        Self {
            id: Default::default(),
            user: Default::default(),
            roles: vec![Role::CUSTOMER],
            permission: vec![Permissions::BASE],
            version: 0,
        }
    }
}

impl AggregateRoot for AccessAccountAggregateRoot {
    type Event = AccessAccountEvents;

    fn id(&self) -> &EntityId {
        &self.id.entity_id()
    }

    fn aggregate_type() -> &'static str {
        "AccessAccountAggregate"
    }

    fn apply_event(&mut self, event: Self::Event) {
        match event.clone() {
            AccessAccountEvents::Created(versioned_event) => {
                let event = versioned_event.to_latest();
                self.apply_created(event);
            }
            AccessAccountEvents::Updated(versioned_event) => {
                let event = versioned_event.to_latest();
                self.apply_updated(event);
            }
        }
    }

    fn first_state(id: &EntityId) -> Self {
        let mut aggregate = AccessAccountAggregateRoot::default();
        aggregate.id = AccessAccountId::from(*id);
        aggregate
    }

    fn get_version(&self) -> u32 {
        self.version
    }

    fn set_version(&mut self, version: u32) {
        self.version = version
    }

    fn increment_version(&mut self) {
        self.version += 1;
    }
}
