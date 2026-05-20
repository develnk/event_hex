use event_hex::bus::in_memory::event_bus::{EventBus, EventBusPort};
use event_hex::domain_event_handlers::ProjectionUpdaterEventHandlerFactory;
use std::sync::Arc;
use tokio::sync::OnceCell;

use crate::domain::identity_access_management::identity::event::access_account_events::AccessAccountEvents;


// Static storage (using lazy_static or once_cell)
static EVENT_BUS: OnceCell<Arc<EventBus>> = OnceCell::const_new();
pub async fn event_bus_init() {
    let event_bus = EVENT_BUS.get_or_init(|| async { Arc::new(EventBus::new()) }).await;

    // Register event handlers

    // To update aggregate projections, a projection update handler must be registered
    // for each domain event.
    event_bus.register_handler::<AccessAccountEvents, _>(ProjectionUpdaterEventHandlerFactory::new()).await;
}

pub fn get_event_bus() -> Arc<EventBus> {
    Arc::clone(EVENT_BUS.get().unwrap())
}
