use crate::bus::ports::query::QueryBusPort;
use crate::cqrs::{Query, QueryHandlerFactory};
use crate::errors::QueryHandlerError;
use crate::errors::QueryHandlerError::QueryHandlerNotRegistered;
use async_trait::async_trait;
use std::any;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::RwLock;

type GenericQueryDispatcher = Arc<dyn Send + Sync + Fn(Box<dyn Query>) -> Pin<Box<dyn Future<Output=Result<Box<dyn Any + Send + Sync + 'static>, QueryHandlerError>> + Send>>>;

pub struct QueryBus {
    handlers: Arc<RwLock<HashMap<TypeId, GenericQueryDispatcher>>>,
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
            handlers: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl QueryBusPort for QueryBus {
    async fn register_handler<Q, F>(&self, factory: F)
    where
        Q: Query + 'static,
        F: QueryHandlerFactory<Q> + 'static,
    {
        let type_id = TypeId::of::<Q>();
        let factory_arc = Arc::new(factory);
        let dispatcher: GenericQueryDispatcher = Arc::new(move |query_box: Box<dyn Query>| {
            let factory_clone = Arc::clone(&factory_arc);
            Box::pin(async move {
                let query = query_box.downcast::<Q>().map_err(QueryHandlerError::from)?;

                let handler = factory_clone.create().await?;

                handler.handle(*query).await
            })
        });
        self.handlers.write().await.insert(type_id, dispatcher);
    }

    async fn dispatch(&self, query: Box<dyn Query>) -> Result<Box<dyn Any + Send + Sync + 'static>, QueryHandlerError> {
        let type_id = (*query).type_id();
        let dispatcher = {
            let guard = self.handlers.read().await;
            guard.get(&type_id)
                .ok_or(QueryHandlerNotRegistered(any::type_name_of_val(&*query).to_string()))?
                .clone()
        };
        dispatcher(query).await
    }
}
