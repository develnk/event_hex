use async_trait::async_trait;
use futures::FutureExt;
use std::any::type_name_of_val;
use std::pin::Pin;
use std::{any::TypeId, collections::HashMap, sync::Arc};
use std::sync::RwLock;
use crate::domain_event::{DomainEvent, DomainEventHandlerFactory};
use crate::errors::DomainEventHandlerError;
use crate::errors::DomainEventHandlerError::DomainEventHandlerNotRegistered;

#[async_trait]
pub trait EventBusPort: Send + Sync {
    async fn publish(&self, event: &dyn DomainEvent) -> Result<(), DomainEventHandlerError>;
}

type GenericDomainEventDispatcher =
    Box<dyn Send + Sync + Fn(&dyn DomainEvent) -> Pin<Box<dyn Future<Output = ()> + Send>>>;

#[derive(Default)]
pub struct EventBus {
    handlers: Arc<RwLock<HashMap<TypeId, Vec<GenericDomainEventDispatcher>>>>,
}

impl Clone for EventBus {
    fn clone(&self) -> Self {
        Self {
            handlers: Arc::clone(&self.handlers),
        }
    }
}

impl EventBus {
    pub fn new() -> Self {
        EventBus {
            handlers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn register_handler<E, F>(&self, factory: F)
    where
        E: DomainEvent + Clone + Send + Sync + 'static,
        F: DomainEventHandlerFactory<E> + 'static,
    {
        let type_id = TypeId::of::<E>();
        let factory_arc = Arc::new(factory);

        let boxed_dispatcher: GenericDomainEventDispatcher =
            Box::new(move |domain_event: &dyn DomainEvent| {
                let factory_clone = Arc::clone(&factory_arc);
                let event_opt = domain_event.as_any().downcast_ref::<E>().cloned();

                Box::pin(async move {
                    if let Some(event) = event_opt {
                        // Создаем обработчик
                        if let Ok(handler) = factory_clone.create().await {
                            // Вызываем обработку
                            handler.handle(&event).map(|_| ()).await;
                        }
                    }
                })
            });

        let mut handlers = self.handlers.write().unwrap();
        handlers.entry(type_id).or_insert_with(Vec::new).push(boxed_dispatcher);
    }

    pub async fn publish(&self, event: &dyn DomainEvent) -> Result<(), DomainEventHandlerError> {
        let type_id = event.event_type_id();
        let handlers_guard = self.handlers.read().unwrap();
        let dispatcher_funcs = handlers_guard
            .get(&type_id)
            .ok_or(DomainEventHandlerNotRegistered(type_name_of_val(&*event).to_string()))?;
        // Вызываем диспетчерскую функцию, которая создаст обработчик и выполнит его
        // По скольку на одно событие может быть зарегистрировано несколько событий, необходимо вызвать
        // каждый обработчик для события.
        for dispatcher in dispatcher_funcs {
            dispatcher(event).await
        }
        Ok(())
    }
}

#[async_trait]
impl EventBusPort for EventBus {
    async fn publish(&self, event: &dyn DomainEvent) -> Result<(), DomainEventHandlerError> {
        let type_id = event.event_type_id();
        let handlers_guard = self.handlers.read().unwrap();
        let dispatcher_funcs = handlers_guard
            .get(&type_id)
            .ok_or(DomainEventHandlerNotRegistered(type_name_of_val(&*event).to_string()))?;
        for dispatcher in dispatcher_funcs {
            dispatcher(event).await
        }
        Ok(())
    }
}
