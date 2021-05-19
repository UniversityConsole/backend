use serde::{Deserialize, Serialize};
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
    pub gov_id: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    #[serde(default = "String::new")]
    pub password: String,
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
            "GovId": "JD",
            "Password": "super_secret"
        })
        .to_string();

        let expected = UserAccount {
            account_id: Uuid::nil(),
            email: "example@example.com".to_string(),
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            gov_id: "JD".to_string(),
            password: "super_secret".to_string(),
        };

        assert_eq!(expected, serde_json::from_str(&input.as_str()).unwrap());
    }

    #[test]
    fn deserializes_from_datastore_doc() {
        use super::*;
        use rusoto_dynamodb::AttributeValue;
        use serde_json::json;
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
            "GovId".to_string(),
            AttributeValue {
                s: Some("JD".to_string()),
                ..AttributeValue::default()
            },
        );

        let expected = UserAccount {
            account_id: Uuid::nil(),
            email: "john.doe@example.com".to_string(),
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            gov_id: "JD".to_string(),
            password: "".to_string(),
        };
        let actual = serde_dynamodb::from_hashmap::<UserAccount, _>(doc).unwrap();

        assert_eq!(expected, actual);
    }
}
