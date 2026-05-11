use crate::adapters::persistence::mongo::projections::dto::profile_projection_dto::ProfileProjectionDto;
use crate::application::ports::projections::projection::{ProjectionDtoEventApplier, ProjectionRepository};
use crate::core::user_profile_context::profile::event::profile_events::ProfileEvent;
use crate::shared_kernel::domain_event::DomainEvent;
use crate::shared_kernel::errors::ProjectionError;
use crate::shared_kernel::model::domain::EntityId;
use async_trait::async_trait;
use bson::doc;
use icu::locale::LanguageIdentifier;
use mongodb::options::ReplaceOptions;
use mongodb::{Client, Collection};
use serde::Serialize;
use std::sync::Arc;

#[derive(Debug)]
pub struct MongoProfileProjectionAdapter {
    collection: Collection<ProfileProjectionDto>,
}

#[derive(Serialize)]
pub struct ProfileUpdate {
    email: String,
    full_name: String,
    reports_time_zone: String,
    language_code: LanguageIdentifier,
}

impl MongoProfileProjectionAdapter {
    pub async fn new(client: Arc<Client>, db_name: &str) -> Result<Self, ProjectionError> {
        Ok(Self {
            collection: client.database(db_name).collection::<ProfileProjectionDto>("profile_projection"),
        })
    }
}

#[async_trait]
impl ProjectionRepository for MongoProfileProjectionAdapter {
    async fn apply_event(&mut self, aggregate_id: &EntityId, event: &dyn DomainEvent) -> Result<(), ProjectionError> {
        if let Some(converted_event) = event.as_any().downcast_ref::<ProfileEvent>() {
            let filter = doc! { "_id": aggregate_id.as_uuid() };
            let profile = self.collection.find_one(filter.clone()).await?;

            match converted_event {
                ProfileEvent::Created(e) => {
                    let event = e.to_latest();
                    let new_doc = ProfileProjectionDto::from(event);
                    let options = ReplaceOptions::builder().upsert(true).build();
                    self.collection.replace_one(filter, new_doc).with_options(options).await?;
                },
                ProfileEvent::Updated(e) => {
                    if let Some(mut profile) = profile {
                        // Применить событие к DTO проекции агрегата, чтобы обновить поля и потом сохранить в БД
                        profile.apply_event_to_dto(event).await?;
                        self.collection.replace_one(filter, profile).await?;
                    }
                },
                ProfileEvent::SettingsUpdated(e) => {
                    todo!()
                },
                ProfileEvent::DateTimeSettingsUpdated(e) => {
                    if let Some(mut profile) = profile {
                        profile.apply_event_to_dto(event).await?;
                        self.collection.replace_one(filter, profile).await?;
                    }
                },
                ProfileEvent::FullNameUpdated(e) => {
                    if let Some(mut profile) = profile {
                        profile.apply_event_to_dto(event).await?;
                        self.collection.replace_one(filter, profile).await?;
                    }
                },
                ProfileEvent::EmailUpdated(e) => {
                    if let Some(mut profile) = profile {
                        profile.apply_event_to_dto(event).await?;
                        self.collection.replace_one(filter, profile).await?;
                    }
                },
                ProfileEvent::DateFormatUpdated(e) => {
                    if let Some(mut profile) = profile {
                        profile.apply_event_to_dto(event).await?;
                        self.collection.replace_one(filter, profile).await?;
                    }
                },
            }
        }

        Ok(())
    }

    async fn rebuild(&mut self, aggregate_id: &EntityId, stream: Vec<&dyn DomainEvent>) -> Result<(), ProjectionError> {
        // Удаляем проекцию.
        let filter = doc! { "_id": aggregate_id};
        self.collection.delete_one(filter).await?;

        // Проигрываем все события заново
        for event in stream {
            self.apply_event(aggregate_id, event).await?;
        }

        Ok(())
    }
}
