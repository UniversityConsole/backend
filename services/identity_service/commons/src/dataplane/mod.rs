use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
#[serde(deny_unknown_fields)]
pub struct UserAccount {
    #[serde(skip_deserializing)]
    pub account_id: Uuid,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub gov_id: String,
    #[serde(skip_serializing)]
    pub password: String,
}
