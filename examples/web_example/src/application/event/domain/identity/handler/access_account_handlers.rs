use crate::adapters::configurations::db::get_initialized_mongodb_client;
use crate::adapters::persistence::mongodb::projections::writers::access_account_projection::MongoAccessAccountProjectionAdapter;
use crate::domain::identity_access_management::identity::event::access_account_events::AccessAccountEvents;
use async_trait::async_trait;
use event_hex::domain_event::{DomainEventHandler, DomainEventHandlerFactory};
use event_hex::domain_event_handlers::{ProjectionUpdaterEventHandler, ProjectionUpdaterEventHandlerFactory};
use event_hex::errors::DomainEventHandlerError;
use std::sync::Arc;
use tokio::sync::RwLock;

//-------------------------------------------------------------------------------------------------------
#[async_trait]
impl DomainEventHandler<AccessAccountEvents> for ProjectionUpdaterEventHandler {
    async fn handle(&self, event: &AccessAccountEvents) {
        let id = match event {
            AccessAccountEvents::Created(e) => e.id(),
            AccessAccountEvents::Updated(e) => e.id(),
        };

        // Use destructuring to access the repository.
        let mut repo_guard = self.repository.write().await;
        if let Err(e) = repo_guard.apply_event(id, event).await {
            //TODO logging error
        }
    }
}

#[async_trait]
impl DomainEventHandlerFactory<AccessAccountEvents> for ProjectionUpdaterEventHandlerFactory {
    async fn create(&self) -> Result<Box<dyn DomainEventHandler<AccessAccountEvents>>, DomainEventHandlerError> {
        let client = get_initialized_mongodb_client().await;
        let db_name = "example";
        let repository =
            Arc::new(RwLock::new(MongoAccessAccountProjectionAdapter::new(client, db_name).await?));
        Ok(Box::new(ProjectionUpdaterEventHandler::new(repository)))
    }
}
