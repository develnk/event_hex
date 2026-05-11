use crate::adapters::persistence::mongo::projections::dto::profile_projection_dto::ProfileProjectionDto;
use crate::application::ports::projections::models::profile::ProfileProjection;
use crate::core::user_profile_context::profile::ports::read_repository_ports::profile_projection_port::ProfileReadProjectionRepository;
use crate::shared_kernel::errors::AppError;
use crate::shared_kernel::model::domain::EntityId;
use async_trait::async_trait;
use bson::doc;
use mongodb::{Client, Collection};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug)]
pub struct MongoProfileReadProjectionAdapter {
    collection: Collection<ProfileProjectionDto>,
}

impl MongoProfileReadProjectionAdapter {
    pub async fn new(client: Arc<Client>, db_name: &str) -> Self {
        Self {
            collection: client
                .database(db_name)
                .collection::<ProfileProjectionDto>("profile_projection"),
        }
    }
}

#[async_trait]
impl ProfileReadProjectionRepository for MongoProfileReadProjectionAdapter {
    async fn get_projection(&self, id: &EntityId) -> Result<Option<ProfileProjection>, AppError> {
        let filter = doc! { "_id": id.as_uuid()};
        let profile =
            self.collection.find_one(filter).await.map_err(|e| AppError::MongoError(e.into()))?;

        match profile {
            Some(p) => Ok(Some(ProfileProjection::from(p))),
            None => Ok(None),
        }
    }

    async fn find_projection_by_account_id(
        &self, account_id: Uuid,
    ) -> Result<Option<ProfileProjection>, AppError> {
        let filter = doc! {"account_id": account_id};
        let profile =
            self.collection.find_one(filter).await.map_err(|e| AppError::MongoError(e.into()))?;

        match profile {
            Some(p) => Ok(Some(ProfileProjection::from(p))),
            None => Ok(None),
        }
    }
}
