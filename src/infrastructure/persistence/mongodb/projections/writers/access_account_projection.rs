use std::sync::Arc;

use async_trait::async_trait;
use mongodb::bson::doc;
use mongodb::options::ReplaceOptions;
use mongodb::{Client, Collection};

use crate::adapters::persistence::mongo::projections::dto::access_account_projection_dto::AccessAccountProjectionDto;
use crate::application::ports::projections::projection::ProjectionDtoEventApplier;
use crate::core::identity_access_management::identity::event::access_account_events::AccessAccountEvents;
use crate::shared_kernel::domain_event::DomainEvent;
use crate::{
    application::ports::projections::projection::ProjectionRepository,
    shared_kernel::{errors::ProjectionError, model::domain::EntityId},
};

#[derive(Debug)]
pub struct MongoAccessAccountProjectionAdapter {
    collection: Collection<AccessAccountProjectionDto>,
}

impl MongoAccessAccountProjectionAdapter {
    pub async fn new(client: Arc<Client>, db_name: &str) -> Result<Self, ProjectionError> {
        Ok(Self {
            collection: client
                .database(db_name)
                .collection::<AccessAccountProjectionDto>("access_account_projection"),
        })
    }
}

#[async_trait]
impl ProjectionRepository for MongoAccessAccountProjectionAdapter {
    async fn apply_event(
        &mut self, aggregate_id: &EntityId, event: &dyn DomainEvent,
    ) -> Result<(), ProjectionError> {
        if let Some(converted_event) = event.as_any().downcast_ref::<AccessAccountEvents>() {
            let filter = doc! { "_id": aggregate_id.as_uuid() };

            match converted_event {
                AccessAccountEvents::Created(e) => {
                    let event = e.to_latest();
                    let new_doc = AccessAccountProjectionDto::from(event);
                    let options = ReplaceOptions::builder().upsert(true).build();
                    // Используем replace_one с upsert=true, чтобы создать, если не существует
                    self.collection.replace_one(filter, new_doc).with_options(options).await?;
                },
                AccessAccountEvents::Updated(e) => {
                    let access_account = self.collection.find_one(filter.clone()).await?;
                    if let Some(mut account) = access_account {
                        // Применить событие к DTO проекции агрегата, чтобы обновить поля и потом сохранить в БД
                        account.apply_event_to_dto(event).await?;
                        self.collection.replace_one(filter, account).await?;
                    }
                },
                AccessAccountEvents::StatusUpdated(e) => {
                    let access_account = self.collection.find_one(filter.clone()).await?;
                    if let Some(mut account) = access_account {
                        account.apply_event_to_dto(event).await?;
                        self.collection.replace_one(filter, account).await?;
                    }
                },
            }
        }

        Ok(())
    }
    async fn rebuild(
        &mut self, id: &EntityId, stream: Vec<&dyn DomainEvent>,
    ) -> Result<(), ProjectionError> {
        // Удаляем проекцию.
        let filter = doc! { "_id": id};
        self.collection.delete_one(filter).await?;

        // Проигрываем все события заново
        for event in stream {
            self.apply_event(id, event).await?;
        }

        Ok(())
    }
}
