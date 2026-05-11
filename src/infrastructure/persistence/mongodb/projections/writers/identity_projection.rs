use std::sync::Arc;

use async_trait::async_trait;
use mongodb::bson::doc;
use mongodb::options::ReplaceOptions;
use mongodb::{Client, Collection};

use crate::adapters::persistence::mongo::projections::dto::identity_projection_dto::IdentityProjectionDto;
use crate::application::ports::projections::projection::ProjectionDtoEventApplier;
use crate::core::identity_access_management::identity::event::identity_events::ProviderIdentityEvents;
use crate::shared_kernel::domain_event::DomainEvent;
use crate::{
    application::ports::projections::projection::ProjectionRepository,
    shared_kernel::{errors::ProjectionError, model::domain::EntityId},
};

#[derive(Debug)]
pub struct MongoIdentityProjectionAdapter {
    collection: Collection<IdentityProjectionDto>,
}

impl MongoIdentityProjectionAdapter {
    pub async fn new(client: Arc<Client>, db_name: &str) -> Result<Self, ProjectionError> {
        Ok(Self {
            collection: client
                .database(db_name)
                .collection::<IdentityProjectionDto>("identity_projection"),
        })
    }
}

#[async_trait]
impl ProjectionRepository for MongoIdentityProjectionAdapter {
    async fn apply_event(
        &mut self, aggregate_id: &EntityId, event: &dyn DomainEvent,
    ) -> Result<(), ProjectionError> {
        if let Some(converted_event) = event.as_any().downcast_ref::<ProviderIdentityEvents>() {
            let filter = doc! { "_id": aggregate_id.as_uuid() };

            match converted_event {
                ProviderIdentityEvents::Created(e) => {
                    let event = e.to_latest();
                    let new_doc = IdentityProjectionDto::from(event);
                    let options = ReplaceOptions::builder().upsert(true).build();
                    // Используем replace_one с upsert=true, чтобы создать, если не существует
                    self.collection.replace_one(filter, new_doc).with_options(options).await?;
                },
                ProviderIdentityEvents::ProviderIdentityMetadataUpdated(e) => {
                    let identity_projection = self.collection.find_one(filter.clone()).await?;
                    if let Some(mut projection) = identity_projection {
                        // Применить событие к DTO проекции агрегата, чтобы обновить поля и потом сохранить в БД
                        projection.apply_event_to_dto(event).await?;
                        self.collection.replace_one(filter, projection).await?;
                    }
                },
                ProviderIdentityEvents::AccessAccountLinked(e) => {
                    let identity_projection = self.collection.find_one(filter.clone()).await?;
                    if let Some(mut projection) = identity_projection {
                        projection.apply_event_to_dto(event).await?;
                        self.collection.replace_one(filter, projection).await?;
                    }
                },
            }
        }

        Ok(())
    }

    async fn rebuild(
        &mut self, aggregate_id: &EntityId, stream: Vec<&dyn DomainEvent>,
    ) -> Result<(), ProjectionError> {
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
