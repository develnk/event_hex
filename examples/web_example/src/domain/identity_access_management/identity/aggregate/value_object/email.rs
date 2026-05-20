use crate::domain::identity_access_management::identity::aggregate::access_account::AccessAccountAggregateRoot;
use event_hex::domain::AggregateRoot;
use event_hex::errors::DomainError;

#[derive(Debug, Clone, PartialEq)]
pub struct Email(String);
impl Email {
    pub fn inner(&self) -> &str {
        &self.0
    }

    pub fn into_inner(self) -> String {
        self.0
    }

    pub fn new(aggregate: &AccessAccountAggregateRoot, email: String, event_name: String) -> Result<Self, DomainError> {
        let mut is_err = false;
        let mut err_msg = String::new();

        if email.trim().is_empty() {
            is_err = true;
            err_msg = "Email cannot be whitespace only.".to_string();
        }

        if is_err {
            Err(DomainError::DomainValidationError {
                event_name,
                aggregate_id: aggregate.id().entity_id().as_uuid(),
                aggregate_type: AccessAccountAggregateRoot::aggregate_type().to_string(),
                actual: email,
                message: err_msg,
            })
        } else {
            Ok(Self(email.trim().to_string()))
        }
    }
}
