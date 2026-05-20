use crate::application::command::domain::identity::commands::AccessAccountCommand;
use crate::application::dto::user::UserDTO;
use crate::application::ports::projections::models::access_account::AccessAccountProjection;
use crate::domain::identity_access_management::identity::ports::read_repository_ports::access_account_projection::AccessAccountReadProjectionRepository;
use crate::shared_kernel::errors::AppError;
use event_hex::bus::in_memory::command_bus::{CommandBus, CommandBusPort};
use event_hex::bus::in_memory::event_bus::{EventBus, EventBusPort};
use event_hex::bus::in_memory::query_bus::QueryBus;
use event_hex::domain_event::DomainEvent;
use event_hex::errors::EventHexError;
use event_hex::persistence::transaction::EventTransactionManager;
use std::sync::Arc;
use uuid::Uuid;

pub struct IdentityApplicationService {
    query_bus: Arc<QueryBus>,
    command_bus: Arc<CommandBus>,
    event_bus: Arc<EventBus>,
    access_account_projection_repository: Arc<dyn AccessAccountReadProjectionRepository>,
    tx_manager: Arc<dyn EventTransactionManager>,
}

impl IdentityApplicationService {
    pub fn new(query_bus: Arc<QueryBus>, command_bus: Arc<CommandBus>, event_bus: Arc<EventBus>,
               access_account_projection_repository: Arc<dyn AccessAccountReadProjectionRepository>, tx_manager: Arc<dyn EventTransactionManager>) -> Self {
        Self {
            query_bus,
            command_bus,
            event_bus,
            access_account_projection_repository,
            tx_manager,
        }
    }

    pub async fn register_tg_identity_service(&self, user_dto: UserDTO) -> Result<Uuid, AppError> {
        // Clone all necessary data before calling the transaction to pass ownership into the 'static closure.
        let user = user_dto.clone();
        let event_bus = self.event_bus.clone();
        let mut all_events: Vec<Box<dyn DomainEvent>> = Vec::with_capacity(5);

        let access_account_id = if let Some(acc) = self.get_access_account(user.email().trim().to_string()).await? {
            acc.id
        } else {
            self.create_new_access_account(user, &mut all_events).await?
        };

        // Publish domain events using the Domain Event Publisher.
        for event in all_events {
            event_bus.publish(&*event).await.map_err(|e| AppError::from(EventHexError::from(e)))?;
        }

        Ok(access_account_id)
    }

    async fn get_access_account(&self, email: String) -> Result<Option<AccessAccountProjection>, AppError> {
        self.access_account_projection_repository.find_projection_by_email(email.to_owned()).await
    }

    async fn create_new_access_account(&self, new_user: UserDTO, events: &mut Vec<Box<dyn DomainEvent>>) -> Result<Uuid, AppError> {
        // Clone data for moving into async block
        let command_bus = self.command_bus.clone();
        let first_name = new_user.first_name().to_owned();
        let last_name = new_user.last_name().to_owned();
        let email = new_user.email().to_owned();

        let (account_id, new_events) = self
            .tx_manager
            .run(move |ctx| {
                // Clone variables for moving into the inner async block
                let cb = command_bus.clone();

                Box::pin(async move {
                    let create_account_command = AccessAccountCommand::CreateAccessAccountCommand {
                        first_name,
                        last_name,
                        email,
                    };

                    // Return the result as Ok
                    Ok(cb.dispatch(Box::new(create_account_command), Some(ctx)).await?)
                })
            })
            .await?;
        // Add events to the common list outside the transaction
        events.extend(new_events);

        Ok(account_id.as_uuid())
    }
}