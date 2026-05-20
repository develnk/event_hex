use std::sync::Arc;

use crate::adapters::persistence::mongodb::projections::dto::access_account_projection_dto::AccessAccountProjectionDto;
use crate::application::ports::projections::models::access_account::AccessAccountProjection;
use crate::domain::identity_access_management::identity::ports::read_repository_ports::access_account_projection::AccessAccountReadProjectionRepository;
use crate::shared_kernel::errors::AppError;
use async_trait::async_trait;
use event_hex::domain::EntityId;
use mongodb::bson::doc;
use mongodb::{Client, Collection};
use uuid::Uuid;

#[derive(Debug)]
pub struct MongoAccessAccountReadProjectionAdapter {
    collection: Collection<AccessAccountProjectionDto>,
}

impl MongoAccessAccountReadProjectionAdapter {
    pub async fn new(client: Arc<Client>, db_name: &str) -> Self {
        Self {
            collection: client
                .database(db_name)
                .collection::<AccessAccountProjectionDto>("access_account_projection"),
        }
    }
}

#[async_trait]
impl AccessAccountReadProjectionRepository for MongoAccessAccountReadProjectionAdapter {
    async fn get_projection(&self, id: &EntityId) -> Result<Option<AccessAccountProjection>, AppError> {
        let filter = doc! { "_id": id.as_uuid()};
        let account =
            self.collection.find_one(filter).await.map_err(|e| AppError::MongoError(e.into()))?;

        match account {
            Some(p) => Ok(Some(AccessAccountProjection::from(p))),
            None => Ok(None),
        }
    }

    async fn find_projection_by_user_id(&self, user_id: Uuid) -> Result<Option<AccessAccountProjection>, AppError> {
        let filter = doc! {"user.id": user_id};
        let account =
            self.collection.find_one(filter).await.map_err(|e| AppError::MongoError(e.into()))?;

        match account {
            Some(p) => Ok(Some(AccessAccountProjection::from(p))),
            None => Ok(None),
        }
    }
    async fn find_projection_by_email(&self, email: String) -> Result<Option<AccessAccountProjection>, AppError> {
        let filter = doc! {"user.email": email};
        let account =
            self.collection.find_one(filter).await.map_err(|e| AppError::MongoError(e.into()))?;

        match account {
            Some(p) => Ok(Some(AccessAccountProjection::from(p))),
            None => Ok(None),
        }
    }
}
