use std::sync::Arc;

use crate::adapters::persistence::mongodb::projections::dto::access_account_projection_dto::AccessAccountProjectionDto;
use crate::domain::identity_access_management::identity::event::access_account_events::AccessAccountEvents;
use async_trait::async_trait;
use event_hex::domain::EntityId;
use event_hex::domain_event::DomainEvent;
use event_hex::errors::ProjectionError;
use event_hex::projection::{ProjectionDtoEventApplier, ProjectionRepository};
use mongodb::bson::doc;
use mongodb::options::ReplaceOptions;
use mongodb::{Client, Collection};

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
                    // Use replace_one with upsert=true to create if it doesn't exist
                    self.collection.replace_one(filter, new_doc).with_options(options).await?;
                }
                AccessAccountEvents::Updated(e) => {
                    let access_account = self.collection.find_one(filter.clone()).await?;
                    if let Some(mut account) = access_account {
                        // Apply the event to the aggregate projection DTO to update fields and then save to DB
                        account.apply_event_to_dto(event).await?;
                        self.collection.replace_one(filter, account).await?;
                    }
                }
            }
        }

        Ok(())
    }
    async fn rebuild(&mut self, id: &EntityId, stream: Vec<&dyn DomainEvent>) -> Result<(), ProjectionError> {
        // Delete the projection.
        let filter = doc! { "_id": id};
        self.collection.delete_one(filter).await?;

        // Replay all events
        for event in stream {
            self.apply_event(id, event).await?;
        }

        Ok(())
    }
}
