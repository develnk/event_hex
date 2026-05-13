use event_hex::application::ports::cqrs::Command;
use serde::{Deserialize, Serialize};

use crate::{
    domain::identity_access_management::identity::aggregate::value_object::account_vo::AccessAccountId,
    shared_kernel::model::common::{Permissions, Role},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccessAccountCommand {
    CreateAccessAccountCommand {
        first_name: String,
        last_name: String,
        email: String,
    },
    UpdateAccessAccountCommand {
        id: AccessAccountId,
        roles: Vec<Role>,
        permission: Vec<Permissions>,
    },
}
impl Command for AccessAccountCommand {}
