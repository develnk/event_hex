use std::sync::Arc;

use async_trait::async_trait;
use mongodb::bson::doc;
use mongodb::{Client, Collection};

use crate::adapters::persistence::mongo::projections::dto::identity_projection_dto::IdentityProjectionDto;
use crate::shared_kernel::errors::AppError;
use crate::{
    application::ports::projections::models::identity::IdentityProjection,
    core::identity_access_management::identity::ports::read_repository_ports::identity_projection_repository_port::IdentityReadProjectionRepository,
    shared_kernel::model::domain::EntityId,
};

#[derive(Debug)]
pub struct MongoIdentityReadProjectionAdapter {
    collection: Collection<IdentityProjectionDto>,
}

impl MongoIdentityReadProjectionAdapter {
    pub async fn new(client: Arc<Client>, db_name: &str) -> Self {
        Self {
            collection: client
                .database(db_name)
                .collection::<IdentityProjectionDto>("identity_projection"),
        }
    }
}

#[async_trait]
impl IdentityReadProjectionRepository for MongoIdentityReadProjectionAdapter {
    async fn get_projection(&self, id: &EntityId) -> Option<IdentityProjection> {
        let filter = doc! { "_id": id.as_uuid()};
        self.collection.find_one(filter).await.ok().flatten().map(|p| p.into())
    }

    async fn find_projection_by_external_id(
        &self, auth_provider_name: String, external_id: String,
    ) -> Result<Option<IdentityProjection>, AppError> {
        let filter =
            doc! {"all_external_ids": external_id, "linked_identities.type": auth_provider_name};
        let result =
            self.collection.find_one(filter).await.map_err(|e| AppError::MongoError(e.into()))?;
        Ok(result.map(|d| d.into()))
    }
}
