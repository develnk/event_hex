use async_trait::async_trait;
use std::any;
use std::any::{TypeId};
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use crate::cqrs::{Command, CommandHandlerFactory};
use crate::domain::EntityId;
use crate::domain_event::DomainEvent;
use crate::errors::CommandHandlerError;
use crate::errors::CommandHandlerError::{CommandHandlerDownCast, CommandHandlerNotRegistered};
use crate::transaction::TransactionContext;

#[async_trait]
pub trait CommandBusPort: Send + Sync {
    async fn dispatch(
        &self, command: Box<dyn Command>, ctx: Option<&mut dyn TransactionContext>,
    ) -> Result<(EntityId, Vec<Box<dyn DomainEvent>>), CommandHandlerError>;
}

type GenericCommandDispatcher = Box<
    dyn Send
        + Sync
        + for<'a> Fn(
            Box<dyn Command>,
            Option<&'a mut dyn TransactionContext>,
        ) -> Pin<Box<dyn Future<Output = Result<(EntityId, Vec<Box<dyn DomainEvent>>), CommandHandlerError>> + Send + 'a>>,
>;

pub struct CommandBus {
    handlers: Arc<Mutex<HashMap<TypeId, GenericCommandDispatcher>>>,
}

impl Clone for CommandBus {
    fn clone(&self) -> Self {
        Self {
            handlers: Arc::clone(&self.handlers),
        }
    }
}

impl Default for CommandBus {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandBus {
    pub fn new() -> Self {
        Self {
            handlers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn register<C, F>(&self, factory: F)
    where
        C: Command + 'static,
        F: CommandHandlerFactory<C> + 'static,
    {
        let type_id = TypeId::of::<C>();
        let factory_arc = Arc::new(factory); // Захватываем фабрику

        let boxed_dispatcher: GenericCommandDispatcher = Box::new(move |command_box: Box<dyn Command>, ctx: Option<&mut dyn TransactionContext>| {
            let factory_clone = Arc::clone(&factory_arc);

            Box::pin(async move {
                // 1. Извлекаем команду
                let command = command_box
                    .downcast::<C>()
                    .map_err(|_| CommandHandlerDownCast(String::from("Ошибка downcast команды")))?;

                // 2. Создаем обработчик с помощью фабрики
                let handler = factory_clone.create().await?;

                // 3. Вызываем метод handle у созданного обработчика
                // Используем .map(|_| ()) для преобразования Ok(Something) в Ok(())
                handler.handle(*command, ctx).await
            })
        });
        self.handlers.lock().unwrap().insert(type_id, boxed_dispatcher);
    }

    pub async fn dispatch<C>(
        &self, command: C, ctx: Option<&mut dyn TransactionContext>,
    ) -> Result<(EntityId, Vec<Box<dyn DomainEvent>>), CommandHandlerError>
    where
        C: Command + 'static,
    {
        let type_id = TypeId::of::<C>();
        let handlers_guard = self.handlers.lock().unwrap();
        let dispatcher_func = handlers_guard.get(&type_id).ok_or(CommandHandlerNotRegistered(any::type_name::<C>().to_string()))?;
        // Вызываем диспетчерскую функцию, которая создаст обработчик и выполнит его
        dispatcher_func(Box::new(command), ctx).await
    }
}

#[async_trait]
impl CommandBusPort for CommandBus {
    async fn dispatch(
        &self, command: Box<dyn Command>, ctx: Option<&mut dyn TransactionContext>,
    ) -> Result<(EntityId, Vec<Box<dyn DomainEvent>>), CommandHandlerError> {
        let type_id = (*command).type_id();
        let handlers_guard = self.handlers.lock().unwrap();
        let dispatcher_func = handlers_guard
            .get(&type_id)
            .ok_or(CommandHandlerNotRegistered(any::type_name_of_val(&*command).to_string()))?;
        dispatcher_func(command, ctx)
    }
}
