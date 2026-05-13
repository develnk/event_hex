use crate::domain::domain_event::{DomainEvent, DomainEventHandlerFactory};
use crate::shared_kernel::errors::DomainEventHandlerError;
use crate::shared_kernel::errors::DomainEventHandlerError::DomainEventHandlerNotRegistered;
use async_trait::async_trait;
use futures::FutureExt;
use std::any::type_name_of_val;
use std::pin::Pin;
use std::{any::TypeId, collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

#[async_trait]
pub trait EventBusPort: Send + Sync {
    async fn register_handler<E, F>(&self, factory: F)
    where
        E: DomainEvent + Clone + Send + Sync + 'static,
        F: DomainEventHandlerFactory<E> + 'static;
    async fn publish(&self, event: &dyn DomainEvent) -> Result<(), DomainEventHandlerError>;
}

type GenericDomainEventDispatcher = Arc<dyn Send + Sync + Fn(&dyn DomainEvent) -> Pin<Box<dyn Future<Output=()> + Send>>>;

#[derive(Default)]
pub struct EventBus {
    handlers: Arc<RwLock<HashMap<TypeId, Vec<GenericDomainEventDispatcher>>>>,
}

impl Clone for EventBus {
    fn clone(&self) -> Self {
        Self { handlers: Arc::clone(&self.handlers) }
    }
}

impl EventBus {
    pub fn new() -> Self {
        EventBus { handlers: Arc::new(RwLock::new(HashMap::new())) }
    }
}

#[async_trait]
impl EventBusPort for EventBus {
    async fn register_handler<E, F>(&self, factory: F)
    where
        E: DomainEvent + Clone + Send + Sync + 'static,
        F: DomainEventHandlerFactory<E> + 'static,
    {
        let type_id = TypeId::of::<E>();
        let factory_arc = Arc::new(factory);

        let dispatcher: GenericDomainEventDispatcher =
            Arc::new(move |domain_event: &dyn DomainEvent| {
                let factory_clone = Arc::clone(&factory_arc);
                let event_opt = domain_event.as_any().downcast_ref::<E>().cloned();

                Box::pin(async move {
                    if let Some(event) = event_opt {
                        if let Ok(handler) = factory_clone.create().await {
                            handler.handle(&event).map(|_| ()).await;
                        }
                    }
                })
            });

        let mut handlers = self.handlers.write().await;
        handlers.entry(type_id).or_insert_with(Vec::new).push(dispatcher);
    }

    async fn publish(&self, event: &dyn DomainEvent) -> Result<(), DomainEventHandlerError> {
        let type_id = event.event_type_id();
        let dispatchers = {
            let guard = self.handlers.read().await;
            guard.get(&type_id)
                .ok_or(DomainEventHandlerNotRegistered(type_name_of_val(&*event).to_string()))?
                .clone()
        };
        for dispatcher in &dispatchers {
            dispatcher(event).await
        }
        Ok(())
    }
}
