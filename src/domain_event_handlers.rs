use crate::projection::ProjectionRepository;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Updates projections in response to published domain events.
///
/// Wraps a [`ProjectionRepository`] behind an async read-write lock so that
/// projections can be shared and updated concurrently.
#[derive(Clone, Debug)]
pub struct ProjectionUpdaterEventHandler {
    /// The projection repository that receives the events.
    pub repository: Arc<RwLock<dyn ProjectionRepository>>,
}

impl ProjectionUpdaterEventHandler {
    /// Creates a new projection updater around the given repository.
    pub fn new(repository: Arc<RwLock<dyn ProjectionRepository>>) -> Self {
        Self { repository }
    }
}

/// Factory for updating any projections.
///
/// Additional fields (for example a context) can be added here if needed.
pub struct ProjectionUpdaterEventHandlerFactory;

impl ProjectionUpdaterEventHandlerFactory {
    /// Creates a new factory.
    pub fn new() -> Self {
        Self
    }
}

impl Default for ProjectionUpdaterEventHandlerFactory {
    fn default() -> Self {
        Self::new()
    }
}
