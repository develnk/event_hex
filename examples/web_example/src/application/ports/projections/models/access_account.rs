use crate::shared_kernel::model::common::{Permissions, Role};

use derive_getters::Getters;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct AccessAccountProjection {
    pub id: Uuid,
    pub user: UserProjection,
    pub roles: Vec<Role>,
    pub permission: Vec<Permissions>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Getters)]
pub struct UserProjection {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
}
