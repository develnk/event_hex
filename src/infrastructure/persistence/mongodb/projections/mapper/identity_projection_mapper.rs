use crate::adapters::persistence::mongo::projections::dto::identity_projection_dto::IdentityProjectionDto;
use crate::application::ports::projections::models::identity::IdentityProjection;
use crate::core::identity_access_management::identity::event::identity_events::ProviderIdentityCreatedV1;

impl From<IdentityProjection> for IdentityProjectionDto {
    fn from(projection: IdentityProjection) -> Self {
        let all_external_ids =
            IdentityProjectionDto::get_external_ids(&projection.linked_identities);
        Self {
            id: projection.id,
            access_account_id: projection.access_account_id,
            linked_identities: projection.linked_identities,
            all_external_ids,
        }
    }
}

impl From<IdentityProjectionDto> for IdentityProjection {
    fn from(dto: IdentityProjectionDto) -> Self {
        Self {
            id: dto.id,
            access_account_id: dto.access_account_id,
            linked_identities: dto.linked_identities,
        }
    }
}

impl From<ProviderIdentityCreatedV1> for IdentityProjectionDto {
    fn from(event: ProviderIdentityCreatedV1) -> Self {
        let all_external_ids = IdentityProjectionDto::get_external_ids(&event.linked_identities());
        Self {
            id: event.id().entity_id().as_uuid(),
            access_account_id: *event.access_account_id(),
            linked_identities: event.linked_identities().clone(),
            all_external_ids,
        }
    }
}
