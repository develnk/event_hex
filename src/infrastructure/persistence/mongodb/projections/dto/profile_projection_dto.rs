use crate::application::ports::projections::projection::ProjectionDtoEventApplier;
use crate::core::user_profile_context::profile::aggregate::value_object::date_time::DateTimeSettings;
use crate::core::user_profile_context::profile::event::profile_events::{
    ProfileCreatedV2, ProfileDateFormatUpdatedV1, ProfileDateTimeSettingsUpdatedV1, ProfileEmailUpdatedV1, ProfileEvent, ProfileFullNameUpdatedV1,
    ProfileSettingsUpdatedV1, ProfileUpdatedV2,
};
use crate::shared_kernel::domain_event::DomainEvent;
use crate::shared_kernel::errors::ProjectionError;
use async_trait::async_trait;
use bson::serde_helpers::uuid_1;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use uuid::Uuid;

#[serde_as]
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ProfileProjectionDto {
    #[serde(rename = "_id")]
    #[serde_as(as = "uuid_1::AsBinary")]
    pub id: Uuid,
    #[serde_as(as = "uuid_1::AsBinary")]
    pub account_id: Uuid,
    pub full_name: String,
    #[serde(default = "default_language")]
    pub language_code: String,
    pub email: String,
    pub email_is_approved: bool,
    pub reports_time_zone: String,
    pub date_time_settings: DateTimeSettings,
}

fn default_language() -> String {
    "ru".to_string()
}

#[async_trait]
impl ProjectionDtoEventApplier for ProfileProjectionDto {
    async fn apply_event_to_dto(&mut self, event: &dyn DomainEvent) -> Result<(), ProjectionError> {
        if let Some(converted_event) = event.as_any().downcast_ref::<ProfileEvent>() {
            match converted_event {
                ProfileEvent::Created(e) => self.apply_profile_create(e.to_latest()),
                ProfileEvent::Updated(e) => self.apply_profile_update(e.to_latest()),
                ProfileEvent::SettingsUpdated(e) => self.apply_profile_settings_update(e.to_latest()),
                ProfileEvent::DateTimeSettingsUpdated(e) => self.apply_profile_date_time_settings_update(e.to_latest()),
                ProfileEvent::FullNameUpdated(e) => self.apply_profile_full_name_update(e.to_latest()),
                ProfileEvent::EmailUpdated(e) => self.apply_profile_email_update(e.to_latest()),
                ProfileEvent::DateFormatUpdated(e) => self.apply_profile_date_format_update(e.to_latest()),
            }
        }

        Ok(())
    }
}

impl ProfileProjectionDto {
    fn apply_profile_create(&mut self, event: ProfileCreatedV2) {
        self.id = event.id().entity_id().as_uuid();
        self.account_id = event.account_id().to_owned();
        self.email = event.email().to_owned();
        self.email_is_approved = false;
        self.full_name = event.full_name().to_owned();
        self.language_code = event.language_code().to_string();
        self.reports_time_zone = event.reports_time_zone().to_string();
        self.date_time_settings = event.date_time_settings().clone();
    }

    fn apply_profile_update(&mut self, event: ProfileUpdatedV2) {
        self.full_name = event.full_name().to_string();
        self.language_code = event.language_code().to_string();
        self.email = event.email().to_string();
        self.reports_time_zone = event.reports_time_zone().to_string();
    }

    fn apply_profile_settings_update(&mut self, event: ProfileSettingsUpdatedV1) {
        self.id = event.id().entity_id().as_uuid().into();
    }

    fn apply_profile_date_time_settings_update(&mut self, event: ProfileDateTimeSettingsUpdatedV1) {
        self.id = event.id().entity_id().as_uuid().into();
        self.date_time_settings = event.date_time_settings().clone();
    }

    fn apply_profile_full_name_update(&mut self, event: ProfileFullNameUpdatedV1) {
        self.id = event.id().entity_id().as_uuid().into();
        self.full_name = event.full_name().to_string();
    }

    fn apply_profile_email_update(&mut self, event: ProfileEmailUpdatedV1) {
        self.id = event.id().entity_id().as_uuid().into();
        self.email = event.email().to_string();
    }

    fn apply_profile_date_format_update(&mut self, event: ProfileDateFormatUpdatedV1) {
        self.id = event.id().entity_id().as_uuid().into();
        self.date_time_settings.date_format = event.date_format().to_owned();
    }
}
