use crate::shared_kernel::errors::AppError;
use serde::Deserialize;
use std::sync::{Arc, OnceLock};

static GLOBAL_SETTINGS: OnceLock<Arc<AppSettings>> = OnceLock::new();

#[derive(Debug, Clone, Deserialize)]
pub struct DbCredentials {
    pub url: String,
    pub username: String,
    pub password: String,
    pub dbname: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppSettings {
    pub database: DbCredentials,
    pub logging_level: String,
}

impl AppSettings {
    pub fn init() -> Result<(), AppError> {
        let settings = AppSettings {
            database: DbCredentials {
                url: "mongodb://localhost:27018,localhost:27019,localhost:27020/test?replicaSet=rs0&authSource=admin&retryWrites=true".to_string(),
                username: "root".to_string(),
                password: "root".to_string(),
                dbname: "test".to_string(),
            },
            logging_level: "info".to_string(),
        };
        let _ = GLOBAL_SETTINGS.set(Arc::new(settings));
        Ok(())
    }
}

pub fn get_app_settings() -> Arc<AppSettings> {
    Arc::clone(GLOBAL_SETTINGS.get().unwrap())
}
