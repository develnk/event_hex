use async_trait::async_trait;
use std::any;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use crate::cqrs::{Query, QueryHandlerFactory};
use crate::errors::QueryHandlerError;
use crate::errors::QueryHandlerError::QueryHandlerNotRegistered;

#[async_trait]
pub trait QueryBusPort: Send + Sync {
    async fn dispatch(&self, query: Box<dyn Query>) -> Result<Box<dyn Any + Send + Sync + 'static>, QueryHandlerError>;
}

type GenericQueryDispatcher = Box<
    dyn Send + Sync + Fn(Box<dyn Query>) -> Pin<Box<dyn Future<Output = Result<Box<dyn Any + Send + Sync + 'static>, QueryHandlerError>> + Send>>,
>;

pub struct QueryBus {
    handlers: Arc<Mutex<HashMap<TypeId, GenericQueryDispatcher>>>,
}

impl Clone for QueryBus {
    fn clone(&self) -> Self {
        Self {
            handlers: Arc::clone(&self.handlers),
        }
    }
}

impl Default for QueryBus {
    fn default() -> Self {
        Self::new()
    }
}

impl QueryBus {
    pub fn new() -> Self {
        QueryBus {
            handlers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn register_handler<Q, F>(&self, factory: F)
    where
        Q: Query + 'static,
        F: QueryHandlerFactory<Q> + 'static,
    {
        let type_id = TypeId::of::<Q>();
        let factory_arc = Arc::new(factory);
        let boxed_dispatcher: GenericQueryDispatcher = Box::new(move |query_box: Box<dyn Query>| {
            let factory_clone = Arc::clone(&factory_arc);
            Box::pin(async move {
                // 1. Извлекаем команду
                let query = query_box.downcast::<Q>().map_err(QueryHandlerError::from)?;

                // 2. Создаем обработчик с помощью фабрики
                let handler = factory_clone.create().await?;

                // 3. Вызываем метод handle у созданного обработчика
                handler.handle(*query).await
            })
        });
        self.handlers.lock().unwrap().insert(type_id, boxed_dispatcher);
    }

    pub async fn dispatch<Q>(&self, query: Q) -> Result<Box<dyn Any + Send + Sync + 'static>, QueryHandlerError>
    where
        Q: Query + 'static,
    {
        let type_id = TypeId::of::<Q>();
        let handlers_guard = self.handlers.lock().unwrap();
        let dispatcher_func = handlers_guard.get(&type_id).ok_or(QueryHandlerNotRegistered(any::type_name::<Q>().to_string()))?;
        // Вызываем диспетчерскую функцию, которая создаст обработчик и выполнит его
        dispatcher_func(Box::new(query)).await
    }
}

#[async_trait]
impl QueryBusPort for QueryBus {
    async fn dispatch(&self, query: Box<dyn Query>) -> Result<Box<dyn Any + Send + Sync + 'static>, QueryHandlerError> {
        let type_id = (*query).type_id();
        let handlers_guard = self.handlers.lock().unwrap();
        let dispatcher_func = handlers_guard
            .get(&type_id)
            .ok_or(QueryHandlerNotRegistered(any::type_name_of_val(&*query).to_string()))?;
        dispatcher_func(query).await
    }
}
