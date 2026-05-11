use crate::application::ports::projections::projection::ProjectionDtoEventApplier;
use crate::core::identity_access_management::identity::aggregate::value_object::identity::ProviderIdentity;
use crate::core::identity_access_management::identity::event::identity_events::{
    ProviderIdentityAccessAccountLinkedV1, ProviderIdentityCreatedV1, ProviderIdentityEvents,
    ProviderIdentityMetadataUpdatedV1,
};
use crate::shared_kernel::domain_event::DomainEvent;
use crate::shared_kernel::errors::ProjectionError;
use async_trait::async_trait;
use bson::serde_helpers::uuid_1;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use std::collections::HashSet;
use uuid::Uuid;

#[serde_as]
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct IdentityProjectionDto {
    #[serde(rename = "_id")]
    #[serde_as(as = "uuid_1::AsBinary")]
    pub id: Uuid,
    #[serde_as(as = "uuid_1::AsBinary")]
    pub access_account_id: Uuid,
    pub linked_identities: HashSet<ProviderIdentity>,
    // Денормализованное поле для быстрого поиска
    // TODO создать Multikey Index в Mongo для этого поля
    pub all_external_ids: Vec<String>,
}

#[async_trait]
impl ProjectionDtoEventApplier for IdentityProjectionDto {
    async fn apply_event_to_dto(&mut self, event: &dyn DomainEvent) -> Result<(), ProjectionError> {
        if let Some(converted_event) = event.as_any().downcast_ref::<ProviderIdentityEvents>() {
            match converted_event {
                ProviderIdentityEvents::Created(e) => {
                    self.apply_provider_identity_create(e.to_latest())
                },
                ProviderIdentityEvents::ProviderIdentityMetadataUpdated(e) => {
                    self.apply_provider_identity_metadata_update(e.to_latest())
                },
                ProviderIdentityEvents::AccessAccountLinked(e) => {
                    self.apply_access_account_link(e.to_latest())
                },
            }
        }

        Ok(())
    }
}

impl IdentityProjectionDto {
    pub fn get_external_ids(linked_identities: &HashSet<ProviderIdentity>) -> Vec<String> {
        let mut ids: Vec<String> =
            linked_identities.iter().map(|id| id.get_external_id().to_string()).collect();
        ids.sort();
        ids
    }

    fn apply_provider_identity_create(&mut self, event: ProviderIdentityCreatedV1) {
        self.id = event.id().entity_id().as_uuid().into();
        self.linked_identities = event.linked_identities().clone();
        self.access_account_id = *event.access_account_id();
    }

    fn apply_provider_identity_metadata_update(
        &mut self, event: ProviderIdentityMetadataUpdatedV1,
    ) {
        self.id = event.id().entity_id().as_uuid().into();
        self.linked_identities = event.linked_identities().clone();
    }

    fn apply_access_account_link(&mut self, event: ProviderIdentityAccessAccountLinkedV1) {
        self.id = event.id().entity_id().as_uuid().into();
        self.access_account_id = *event.access_account_id();
    }
}
