use crate::adapters::configurations::db::get_initialized_mongodb_client;
use event_hex::shared_kernel::bus::in_memory::query_bus::QueryBus;
use std::sync::{Arc, OnceLock};

static QUERY_BUS: OnceLock<Arc<QueryBus>> = OnceLock::new();

pub async fn query_bus_init() {
    let _ = QUERY_BUS.set(Arc::new(QueryBus::new()));
    let query_bus = get_query_bus();
    let client = get_initialized_mongodb_client().await;

    //query_bus.register_handler::<GetCompanyQuery, _>(GetCompanyQueryHandlerFactory::new(client)).await;
}

pub fn get_query_bus() -> Arc<QueryBus> {
    Arc::clone(QUERY_BUS.get().unwrap())
}
