use std::sync::Arc;

use crate::adapters::configurations::settings::get_app_settings;

use crate::{
    application::command::domain::identity::commands::AccessAccountCommand,
    domain::identity_access_management::identity::aggregate::access_account::AccessAccountAggregateRoot,
};
use async_trait::async_trait;
use event_hex::cqrs::{CommandHandler, CommandHandlerFactory};
use event_hex::domain::{AggregateContainer, EntityId};
use event_hex::domain_event::DomainEvent;
use event_hex::errors::{CommandHandlerError, EventHexError};
use event_hex::event_store::mongodb::event_store::MongoEventStoreStorage;
use event_hex::event_store::repository::MongoEventStoreRepository;
use event_hex::event_store_repository::EventStoreRepository;
use event_hex::persistence::transaction::EventTransactionContext;
use mongodb::Client;

pub struct AccessAccountHandler<R>
where
    R: EventStoreRepository<AccessAccountAggregateRoot> + ?Sized,
{
    event_repository: Arc<R>,
}

impl<R> AccessAccountHandler<R>
where
    R: EventStoreRepository<AccessAccountAggregateRoot> + ?Sized,
{
    pub fn new(event_repository: Arc<R>) -> Self {
        Self { event_repository }
    }
}

pub struct AccessAccountHandlerFactory {
    client: Arc<Client>,
}

impl AccessAccountHandlerFactory {
    pub fn new(client: Arc<Client>) -> Self {
        Self { client }
    }
}

#[async_trait]
impl<R> CommandHandler<AccessAccountCommand> for AccessAccountHandler<R>
where
    R: EventStoreRepository<AccessAccountAggregateRoot> + Send + Sync + 'static,
{
    async fn handle(&self, command: AccessAccountCommand, ctx: Option<&mut dyn EventTransactionContext>) -> Result<(EntityId, Vec<Box<dyn DomainEvent>>), CommandHandlerError> {
        match command {
            AccessAccountCommand::CreateAccessAccountCommand { first_name, last_name, email } => {
                let mut access_account = AccessAccountAggregateRoot::new();
                let new_event = access_account
                    .new_access_account(first_name, last_name, email)
                    .await
                    .map_err(CommandHandlerError::from)?;

                let mut container = AggregateContainer::new(access_account);
                // Apply the event to the aggregate and queue it for publishing
                container.push_event(new_event);

                let aggregate_id = container.aggregate.id().entity_id().to_owned();
                let events = container.get_erased_events();

                let result = self
                    .event_repository
                    .save_aggregate(ctx.ok_or_else(|| EventHexError::TransactionContextRequired())?, container)
                    .await;
                match result {
                    Ok(_) => Ok((aggregate_id, events)),
                    Err(e) => Err(CommandHandlerError::GenericCommandHandler(e.to_string())),
                }
            }
            AccessAccountCommand::UpdateAccessAccountCommand { id, roles, permission } => {
                let tx_ctx = ctx.ok_or_else(|| EventHexError::TransactionContextRequired())?;

                let mut container = self
                    .event_repository
                    .load_aggregate(&mut *tx_ctx, id.entity_id())
                    .await
                    .map_err(|e| CommandHandlerError::GenericCommandHandler(e.to_string()))?
                    .ok_or_else(|| CommandHandlerError::GenericCommandHandler("Aggregate not found".to_string()))?;

                let new_event = container
                    .aggregate
                    .update_access_account(roles.to_vec(), permission.to_vec())
                    .await
                    .map_err(CommandHandlerError::from)?;
                container.push_event(new_event);

                let aggregate_id = container.aggregate.id().entity_id().to_owned();
                let events = container.get_erased_events();

                let result = self.event_repository.save_aggregate(&mut *tx_ctx, container).await;
                match result {
                    Ok(_) => Ok((aggregate_id, events)),
                    Err(e) => Err(CommandHandlerError::GenericCommandHandler(e.to_string())),
                }
            }
        }
    }
}

#[async_trait]
impl CommandHandlerFactory<AccessAccountCommand> for AccessAccountHandlerFactory {
    async fn create(&self) -> Result<Box<dyn CommandHandler<AccessAccountCommand>>, CommandHandlerError> {
        let settings = get_app_settings();
        let storage = Arc::new(MongoEventStoreStorage::<AccessAccountAggregateRoot>::new(
            Arc::clone(&self.client),
            settings.database.dbname.as_str(),
        ));
        let event_repository = Arc::new(MongoEventStoreRepository::<AccessAccountAggregateRoot>::new(storage, 5));
        Ok(Box::new(AccessAccountHandler::new(event_repository)))
    }
}
