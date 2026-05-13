use derive_getters::Getters;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

#[derive(Debug, Clone, Serialize, Deserialize, Getters, TypedBuilder)]
pub struct UserDTO {
    first_name: String,
    last_name: String,
    email: String,
}
