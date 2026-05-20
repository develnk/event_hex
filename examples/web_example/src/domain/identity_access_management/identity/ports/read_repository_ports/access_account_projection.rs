use crate::application::ports::projections::models::access_account::AccessAccountProjection;
use crate::shared_kernel::errors::AppError;
use async_trait::async_trait;
use event_hex::domain::EntityId;
use uuid::Uuid;

#[async_trait]
pub trait AccessAccountReadProjectionRepository: Send + Sync {
    async fn get_projection(&self, id: &EntityId) -> Result<Option<AccessAccountProjection>, AppError>;
    async fn find_projection_by_user_id(&self, user_id: Uuid) -> Result<Option<AccessAccountProjection>, AppError>;
    async fn find_projection_by_email(&self, email: String) -> Result<Option<AccessAccountProjection>, AppError>;
}

//==============================================================================================
// Mock AccessAccountReadProjectionRepository
//==============================================================================================
pub struct MockAccessAccountProjectionRepo {
    _private: (),
}

impl MockAccessAccountProjectionRepo {
    pub fn new() -> Self {
        Self { _private: () }
    }
}

impl Default for MockAccessAccountProjectionRepo {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AccessAccountReadProjectionRepository for MockAccessAccountProjectionRepo {
    async fn get_projection(&self, _id: &EntityId) -> Result<Option<AccessAccountProjection>, AppError> {
        Ok(None)
    }

    async fn find_projection_by_user_id(&self, _user_id: Uuid) -> Result<Option<AccessAccountProjection>, AppError> {
        Ok(None)
    }

    async fn find_projection_by_email(&self, _email: String) -> Result<Option<AccessAccountProjection>, AppError> {
        Ok(None)
    }
}
