use crate::application::ports::projections::projection::ProjectionRepository;
use std::sync::{Arc, RwLock};

#[derive(Clone, Debug)]
pub struct ProjectionUpdaterEventHandler {
    pub repository: Arc<RwLock<dyn ProjectionRepository>>,
}

impl ProjectionUpdaterEventHandler {
    pub fn new(repository: Arc<RwLock<dyn ProjectionRepository>>) -> Self {
        Self { repository }
    }
}

// Фабрика для обновления любых проекций.
// Если понадобится, в неё можно добавить поля, например context
pub struct ProjectionUpdaterEventHandlerFactory;

impl ProjectionUpdaterEventHandlerFactory {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ProjectionUpdaterEventHandlerFactory {
    fn default() -> Self {
        Self::new()
    }
}
