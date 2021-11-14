use serde::{Deserialize, Serialize, Serializer};
use sha2::{Digest, Sha512};
use uuid::Uuid;

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
#[serde(rename_all = "PascalCase")]
#[serde(deny_unknown_fields)]
pub struct UserAccount {
    #[serde(default = "Uuid::nil")]
    pub account_id: Uuid,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    #[serde(serialize_with = "serialize_password")]
    #[serde(default = "String::new")]
    pub password: String,
    pub discoverable: bool,
}

fn serialize_password<S>(val: &String, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut hasher = Sha512::new();
    hasher.update(&val);
    let hashed = hasher.finalize();
    serializer.serialize_bytes(&hashed.as_slice())
}

#[cfg(test)]
mod tests {
    #[test]
    fn deserializes_without_required_fields() {
        use super::*;
        use serde_json::json;
        use uuid::Uuid;

        let input = json!({
            "Email": "example@example.com",
            "FirstName": "John",
            "LastName": "Doe",
            "Password": "super_secret",
            "Discoverable": true
        })
        .to_string();

        let expected = UserAccount {
            account_id: Uuid::nil(),
            email: "example@example.com".to_string(),
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            password: "super_secret".to_string(),
            discoverable: true,
        };

        assert_eq!(expected, serde_json::from_str(&input.as_str()).unwrap());
    }

    #[test]
    fn deserializes_from_datastore_doc() {
        use super::*;
        use rusoto_dynamodb::AttributeValue;
        use std::collections::HashMap;
        use uuid::Uuid;

        let mut doc = HashMap::new();
        doc.insert(
            "AccountId".to_string(),
            AttributeValue {
                s: Some(Uuid::nil().to_hyphenated().to_string()),
                ..AttributeValue::default()
            },
        );
        doc.insert(
            "Email".to_string(),
            AttributeValue {
                s: Some("john.doe@example.com".to_string()),
                ..AttributeValue::default()
            },
        );
        doc.insert(
            "FirstName".to_string(),
            AttributeValue {
                s: Some("John".to_string()),
                ..AttributeValue::default()
            },
        );
        doc.insert(
            "LastName".to_string(),
            AttributeValue {
                s: Some("Doe".to_string()),
                ..AttributeValue::default()
            },
        );
        doc.insert(
            "Discoverable".to_string(),
            AttributeValue {
                bool: Some(true),
                ..AttributeValue::default()
            },
        );

        let expected = UserAccount {
            account_id: Uuid::nil(),
            email: "john.doe@example.com".to_string(),
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            password: "".to_string(),
            discoverable: true,
        };
        let actual = serde_dynamodb::from_hashmap::<UserAccount, _>(doc).unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn password_serializes_to_bytes() {
        use super::*;
        use uuid::Uuid;

        let account = UserAccount {
            account_id: Uuid::nil(),
            email: "john.doe@example.com".to_string(),
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            password: "super_secret".to_string(),
            discoverable: false,
        };
        let serialized = serde_dynamodb::to_hashmap(&account).unwrap();
        let serialized_password_attr = serialized.get(&"Password".to_string()).unwrap();
        assert_eq!(true, serialized_password_attr.b.is_some());
        assert_eq!(true, serialized_password_attr.s.is_none());
    }
}
