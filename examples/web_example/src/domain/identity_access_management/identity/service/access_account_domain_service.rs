use std::sync::Arc;

use crate::domain::identity_access_management::identity::ports::read_repository_ports::access_account_projection::AccessAccountReadProjectionRepository;

pub struct AccessAccountDomainService {
    read_projection_repository: Arc<dyn AccessAccountReadProjectionRepository>,
}

impl AccessAccountDomainService {
    pub fn new(read_projection_repository: Arc<dyn AccessAccountReadProjectionRepository>) -> Self {
        Self {
            read_projection_repository,
        }
    }
}
