use crate::adapters::persistence::mongo::projections::dto::access_account_projection_dto::{
    AccessAccountProjectionDto, UserProjectionDto,
};
use crate::application::ports::projections::models::access_account::{
    AccessAccountProjection, UserProjection,
};
use crate::core::identity_access_management::identity::aggregate::entity::user::User;
use crate::core::identity_access_management::identity::event::access_account_events::AccessAccountCreatedV1;

impl From<AccessAccountProjection> for AccessAccountProjectionDto {
    fn from(projection: AccessAccountProjection) -> Self {
        Self {
            id: projection.id,
            user: projection.user.into(),
            roles: projection.roles,
            permission: projection.permission,
            status: projection.status,
        }
    }
}

impl From<UserProjection> for UserProjectionDto {
    fn from(projection: UserProjection) -> Self {
        Self {
            id: projection.id,
            first_name: projection.first_name,
            last_name: projection.last_name,
            email: projection.email,
            active: projection.active,
        }
    }
}

impl From<UserProjectionDto> for UserProjection {
    fn from(dto: UserProjectionDto) -> Self {
        Self {
            id: dto.id,
            first_name: dto.first_name,
            last_name: dto.last_name,
            email: dto.email,
            active: dto.active,
        }
    }
}

impl From<AccessAccountProjectionDto> for AccessAccountProjection {
    fn from(dto: AccessAccountProjectionDto) -> Self {
        Self {
            id: dto.id,
            user: dto.user.into(),
            roles: dto.roles,
            permission: dto.permission,
            status: dto.status,
        }
    }
}

impl From<AccessAccountCreatedV1> for AccessAccountProjectionDto {
    fn from(event: AccessAccountCreatedV1) -> Self {
        let user = UserProjectionDto::from(event.user().clone());

        Self {
            id: event.id().as_uuid(),
            user,
            roles: event.roles().clone(),
            permission: event.permission().clone(),
            status: event.status().clone(),
        }
    }
}

impl From<User> for UserProjectionDto {
    fn from(user: User) -> Self {
        Self {
            id: user.id().entity_id().as_uuid(),
            first_name: user.first_name().to_string(),
            last_name: user.last_name().to_string(),
            email: user.email().to_string(),
            active: user.active().to_owned(),
        }
    }
}
