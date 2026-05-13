use std::sync::{Arc, OnceLock};

use crate::adapters::configurations::command_bus_config::get_command_bus;
use crate::adapters::configurations::event_bus_config::get_event_bus;
use crate::adapters::configurations::query_bus_config::get_query_bus;
use crate::adapters::configurations::{
    db::get_initialized_mongodb_client,
    settings::get_app_settings,
};
use crate::adapters::persistence::mongodb::projections::readers::access_account_projection::MongoAccessAccountReadProjectionAdapter;
use crate::application::service::identity_service::IdentityApplicationService;
use crate::domain::identity_access_management::identity::ports::read_repository_ports::access_account_projection::AccessAccountReadProjectionRepository;
use derive_getters::Getters;
use event_hex::infrastructure::persistence::mongodb::mongo_transaction::MongoTransactionManager;

static APP_STATE: OnceLock<Arc<AppState>> = OnceLock::new();

#[derive(Clone, Getters)]
pub struct AppState {
    // Services of all modules must be registered here
    // user_service: Arc<UserApplicationService>,
    identity_service: Arc<IdentityApplicationService>,
}

impl AppState {
    async fn new() -> Self {
        let command_bus = get_command_bus();
        let query_bus = get_query_bus();
        let event_bus = get_event_bus();
        let client = get_initialized_mongodb_client().await;
        let app_settings = get_app_settings();
        // Service configuration
        let access_account_read_projection_repository: Arc<dyn AccessAccountReadProjectionRepository, > = Arc::new(
            MongoAccessAccountReadProjectionAdapter::new(Arc::clone(&client), app_settings.database.dbname.as_str()).await,
        );
        let tx_manager = Arc::new(MongoTransactionManager::new(Arc::clone(&client)).await);

        AppState {
            identity_service: Arc::new(IdentityApplicationService::new(
                Arc::new((*query_bus).clone()),
                Arc::new((*command_bus).clone()),
                Arc::new((*event_bus).clone()),
                access_account_read_projection_repository,
                tx_manager,
            )),
        }
    }
}

pub async fn app_state_init() {
    let _ = APP_STATE.set(Arc::new(AppState::new().await));
}

pub fn get_app_state() -> Arc<AppState> {
    Arc::clone(APP_STATE.get().unwrap())
}
