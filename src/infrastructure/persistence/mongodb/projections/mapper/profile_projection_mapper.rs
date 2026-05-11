use crate::adapters::persistence::mongo::projections::dto::profile_projection_dto::ProfileProjectionDto;
use crate::application::ports::projections::models::profile::ProfileProjection;
use crate::core::user_profile_context::profile::event::profile_events::ProfileCreatedV2;

impl From<ProfileProjection> for ProfileProjectionDto {
    fn from(projection: ProfileProjection) -> Self {
        Self {
            id: projection.id,
            account_id: projection.account_id,
            full_name: projection.full_name,
            language_code: projection.language_code,
            email: projection.email,
            email_is_approved: projection.email_is_approved,
            reports_time_zone: projection.reports_time_zone,
            date_time_settings: projection.date_time_settings,
        }
    }
}

impl From<ProfileProjectionDto> for ProfileProjection {
    fn from(dto: ProfileProjectionDto) -> Self {
        Self {
            id: dto.id,
            account_id: dto.account_id,
            full_name: dto.full_name,
            language_code: dto.language_code,
            email: dto.email,
            email_is_approved: dto.email_is_approved,
            reports_time_zone: dto.reports_time_zone,
            date_time_settings: dto.date_time_settings,
        }
    }
}

impl From<ProfileCreatedV2> for ProfileProjectionDto {
    fn from(event: ProfileCreatedV2) -> Self {
        Self {
            id: event.id().entity_id().as_uuid().into(),
            account_id: *event.account_id(),
            full_name: event.full_name().to_owned(),
            email: event.email().to_owned(),
            email_is_approved: false,
            reports_time_zone: event.reports_time_zone().to_string(),
            date_time_settings: event.date_time_settings().to_owned(),
            language_code: event.language_code().to_string(),
        }
    }
}
