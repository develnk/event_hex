use crate::adapters::configurations::db::get_initialized_mongodb_client;
use crate::application::command::domain::identity::commands::AccessAccountCommand;
use crate::application::command::domain::identity::handler::access_account_command_handler::AccessAccountHandlerFactory;
use event_hex::shared_kernel::bus::in_memory::command_bus::{CommandBus, CommandBusPort};
use std::sync::{Arc, OnceLock};

static COMMAND_BUS: OnceLock<Arc<CommandBus>> = OnceLock::new();

pub async fn command_bus_init() {
    let _ = COMMAND_BUS.set(Arc::new(CommandBus::new()));
    let command_bus = get_command_bus();
    let client = get_initialized_mongodb_client().await;
    // Register command handlers
    command_bus
        .register::<AccessAccountCommand, _>(AccessAccountHandlerFactory::new(client.clone()))
        .await;
}

pub fn get_command_bus() -> Arc<CommandBus> {
    Arc::clone(COMMAND_BUS.get().unwrap())
}
