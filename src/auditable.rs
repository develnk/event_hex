use bson::serde_helpers::uuid_1;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use uuid::Uuid;

#[serde_as]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Auditable {
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    #[serde_as(as = "uuid_1::AsBinary")]
    created_by: Uuid,
    #[serde_as(as = "uuid_1::AsBinary")]
    updated_by: Uuid,
    deleted: bool,
}

impl Auditable {
    pub fn build_new(current_user_id: Uuid) -> Self {
        let now = Utc::now();
        let created_at = now;
        let updated_at = now;
        let created_by = current_user_id;
        let updated_by = current_user_id;
        let deleted = false;
        Auditable {
            created_at,
            updated_at,
            created_by,
            updated_by,
            deleted,
        }
    }
}
