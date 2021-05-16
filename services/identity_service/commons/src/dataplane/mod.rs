use bytes::Bytes;
use rusoto_dynamodb::AttributeValue;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha512};
use std::collections::HashMap;
use utils::dynamodb_interop::Document;
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
#[serde(deny_unknown_fields)]
pub struct UserAccount {
    pub account_id: Uuid,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub gov_id: String,
    #[serde(skip_serializing)]
    pub password: String,
}

impl Document for UserAccount {
    fn document(&self) -> HashMap<String, AttributeValue> {
        let mut m = HashMap::new();
        m.insert(
            "AccountId".to_string(),
            AttributeValue {
                b: Some(Bytes::copy_from_slice(self.account_id.as_bytes())),
                ..AttributeValue::default()
            },
        );
        m.insert(
            "Email".to_string(),
            AttributeValue {
                s: Some(self.email.clone()),
                ..AttributeValue::default()
            },
        );
        m.insert(
            "FirstName".to_string(),
            AttributeValue {
                s: Some(self.first_name.clone()),
                ..AttributeValue::default()
            },
        );
        m.insert(
            "LastName".to_string(),
            AttributeValue {
                s: Some(self.last_name.clone()),
                ..AttributeValue::default()
            },
        );
        m.insert(
            "GovId".to_string(),
            AttributeValue {
                s: Some(self.gov_id.clone()),
                ..AttributeValue::default()
            },
        );

        let mut hasher = Sha512::new();
        hasher.update(&self.password);
        m.insert(
            "Password".to_string(),
            AttributeValue {
                b: Some(Bytes::copy_from_slice(hasher.finalize().as_slice())),
                ..AttributeValue::default()
            },
        );

        m
    }
}
