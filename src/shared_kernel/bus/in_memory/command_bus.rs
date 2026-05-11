use crate::application::ports::cqrs::{Command, CommandHandlerFactory};
use crate::application::ports::transaction::TransactionContext;
use crate::shared_kernel::domain::EntityId;
use crate::shared_kernel::domain_event::DomainEvent;
use crate::shared_kernel::errors::CommandHandlerError;
use crate::shared_kernel::errors::CommandHandlerError::{CommandHandlerDownCast, CommandHandlerNotRegistered};
use async_trait::async_trait;
use std::any;
use std::any::TypeId;
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::{Arc, RwLock};

#[async_trait]
pub trait CommandBusPort: Send + Sync {
    async fn register<C, F>(&self, factory: F)
    where
        C: Command + 'static,
        F: CommandHandlerFactory<C> + 'static;
    async fn dispatch(&self, command: Box<dyn Command>, ctx: Option<&mut dyn TransactionContext>) -> Result<(EntityId, Vec<Box<dyn DomainEvent>>), CommandHandlerError>;
}

type GenericCommandDispatcher = Arc<dyn Send + Sync + for<'a> Fn(Box<dyn Command>, Option<&'a mut dyn TransactionContext>) -> Pin<Box<dyn Future<Output=Result<(EntityId, Vec<Box<dyn DomainEvent>>), CommandHandlerError>> + Send + 'a>>>;

pub struct CommandBus {
    handlers: Arc<RwLock<HashMap<TypeId, GenericCommandDispatcher>>>,
}

impl Clone for CommandBus {
    fn clone(&self) -> Self {
        Self { handlers: Arc::clone(&self.handlers) }
    }
}

impl Default for CommandBus {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandBus {
    pub fn new() -> Self {
        Self { handlers: Arc::new(RwLock::new(HashMap::new())) }
    }
}

#[async_trait]
impl CommandBusPort for CommandBus {
    async fn register<C, F>(&self, factory: F)
    where
        C: Command + 'static,
        F: CommandHandlerFactory<C> + 'static,
    {
        let type_id = TypeId::of::<C>();
        let factory_arc = Arc::new(factory);

        let dispatcher: GenericCommandDispatcher = Arc::new(move |command_box: Box<dyn Command>, ctx: Option<&mut dyn TransactionContext>| {
            let factory_clone = Arc::clone(&factory_arc);

            Box::pin(async move {
                let command = command_box
                    .downcast::<C>()
                    .map_err(|_| CommandHandlerDownCast(String::from("Command downcast error")))?;

                let handler = factory_clone.create().await?;

                handler.handle(*command, ctx).await
            })
        });
        self.handlers.write().unwrap().insert(type_id, dispatcher);
    }

    async fn dispatch(
        &self, command: Box<dyn Command>, ctx: Option<&mut dyn TransactionContext>,
    ) -> Result<(EntityId, Vec<Box<dyn DomainEvent>>), CommandHandlerError> {
        let type_id = (*command).type_id();
        let dispatcher = {
            let guard = self.handlers.read().unwrap();
            guard.get(&type_id)
                .ok_or(CommandHandlerNotRegistered(any::type_name_of_val(&*command).to_string()))?
                .clone()
        };
        dispatcher(command, ctx).await
    }
}
