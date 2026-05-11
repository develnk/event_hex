use crate::{
    core::identity_access_management::identity::aggregate::value_object::identity::AccountStatus,
    shared_kernel::model::common::{Permissions, Role},
};
use async_trait::async_trait;

use crate::application::ports::projections::projection::ProjectionDtoEventApplier;
use crate::core::identity_access_management::identity::event::access_account_events::{
    AccessAccountCreatedV1, AccessAccountEvents, AccessAccountStatusUpdatedV1,
    AccessAccountUpdatedV1,
};
use crate::shared_kernel::domain_event::DomainEvent;
use crate::shared_kernel::errors::ProjectionError;
use bson::serde_helpers::uuid_1;
use derive_getters::Getters;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use uuid::Uuid;

#[serde_as]
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AccessAccountProjectionDto {
    #[serde(rename = "_id")]
    #[serde_as(as = "uuid_1::AsBinary")]
    pub id: Uuid,
    pub user: UserProjectionDto,
    pub roles: Vec<Role>,
    pub permission: Vec<Permissions>,
    pub status: AccountStatus,
}

#[serde_as]
#[derive(Clone, Debug, Default, Serialize, Deserialize, Getters)]
pub struct UserProjectionDto {
    #[serde_as(as = "uuid_1::AsBinary")]
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub active: bool,
}

#[async_trait]
impl ProjectionDtoEventApplier for AccessAccountProjectionDto {
    async fn apply_event_to_dto(&mut self, event: &dyn DomainEvent) -> Result<(), ProjectionError> {
        if let Some(converted_event) = event.as_any().downcast_ref::<AccessAccountEvents>() {
            match converted_event {
                AccessAccountEvents::Created(e) => self.apply_access_account_create(e.to_latest()),
                AccessAccountEvents::Updated(e) => self.apply_access_account_update(e.to_latest()),
                AccessAccountEvents::StatusUpdated(e) => {
                    self.apply_access_account_status_update(e.to_latest())
                },
            }
        }

        Ok(())
    }
}

impl AccessAccountProjectionDto {
    fn apply_access_account_create(&mut self, event: AccessAccountCreatedV1) {
        self.id = event.id().as_uuid();
        self.user = UserProjectionDto::from(event.user().clone());
        self.roles = event.roles().clone();
        self.permission = event.permission().clone();
        self.status = event.status().clone();
    }

    fn apply_access_account_update(&mut self, event: AccessAccountUpdatedV1) {
        self.id = event.id().as_uuid();
        self.roles = event.roles().clone();
        self.permission = event.permission().clone();
        self.status = event.status().clone();
    }

    fn apply_access_account_status_update(&mut self, event: AccessAccountStatusUpdatedV1) {
        self.id = event.id().as_uuid();
        self.status = event.status().clone();
    }
}
