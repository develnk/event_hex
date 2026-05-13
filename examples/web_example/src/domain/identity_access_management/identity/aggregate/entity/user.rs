use derive_getters::Getters;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use typed_builder::TypedBuilder;

use crate::shared_kernel::model::common::UserId;

#[serde_as]
#[derive(Clone, Debug, Default, Serialize, Deserialize, TypedBuilder, Getters)]
pub struct User {
    id: UserId,
    first_name: String,
    last_name: String,
    email: String,
    active: bool,
}

impl User {
    pub fn new(first_name: String, last_name: String, email: String) -> Self {
        Self {
            id: UserId::new(),
            first_name,
            last_name,
            email,
            active: true,
        }
    }
}
