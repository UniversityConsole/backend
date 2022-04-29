use std::convert::From;

use serde::{Deserialize, Serialize, Serializer};
use service_core::resource_access::AccessKind;
use sha2::{Digest, Sha512};
use uuid::Uuid;

use crate::svc::PermissionsDocument as PermissionsDocumentModel;

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug, Default)]
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
    #[serde(default)]
    pub permissions_document: PermissionsDocument,
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug, Default)]
#[serde(rename_all = "PascalCase")]
pub struct PermissionsDocument {
    #[serde(default)]
    pub statements: Vec<RenderedPolicyStatement>,
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct RenderedPolicyStatement {
    pub access_kind: AccessKind,
    pub paths: Vec<String>,
}


fn serialize_password<S>(val: &String, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut hasher = Sha512::new();
    hasher.update(&val);
    let hashed = hasher.finalize();
    serializer.serialize_bytes(hashed.as_slice())
}

impl From<PermissionsDocument> for PermissionsDocumentModel {
    fn from(val: PermissionsDocument) -> PermissionsDocumentModel {
        use crate::svc::policy_statement::AccessKind as AccessKindModel;
        use crate::svc::PolicyStatement;

        PermissionsDocumentModel {
            statements: val
                .statements
                .into_iter()
                .map(|s| PolicyStatement {
                    access_kind: match s.access_kind {
                        AccessKind::Mutation => AccessKindModel::Mutation,
                        AccessKind::Query => AccessKindModel::Query,
                    } as i32,
                    paths: s.paths,
                })
                .collect(),
        }
    }
}

impl From<PermissionsDocumentModel> for PermissionsDocument {
    fn from(val: PermissionsDocumentModel) -> PermissionsDocument {
        use crate::svc::policy_statement::AccessKind as AccessKindModel;

        PermissionsDocument {
            statements: val
                .statements
                .into_iter()
                .map(|s| RenderedPolicyStatement {
                    access_kind: if s.access_kind == AccessKindModel::Mutation as i32 {
                        AccessKind::Mutation
                    } else {
                        AccessKind::Query
                    },
                    paths: s.paths,
                })
                .collect(),
        }
    }
}


#[cfg(test)]
mod tests {


    #[test]
    fn deserializes_without_required_fields() {
        use serde_json::json;
        use uuid::Uuid;

        use super::*;

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
            ..Default::default()
        };

        assert_eq!(expected, serde_json::from_str(input.as_str()).unwrap());
    }

    #[test]
    fn deserializes_from_datastore_doc() {
        use std::collections::HashMap;

        use aws_sdk_dynamodb::model::AttributeValue;
        use uuid::Uuid;

        use super::*;

        let mut doc = HashMap::new();
        doc.insert(
            "AccountId".to_string(),
            AttributeValue::S(Uuid::nil().to_hyphenated().to_string()),
        );
        doc.insert(
            "Email".to_string(),
            AttributeValue::S("john.doe@example.com".to_string()),
        );
        doc.insert("FirstName".to_string(), AttributeValue::S("John".to_string()));
        doc.insert("LastName".to_string(), AttributeValue::S("Doe".to_string()));
        doc.insert("Discoverable".to_string(), AttributeValue::Bool(true));

        let expected = UserAccount {
            account_id: Uuid::nil(),
            email: "john.doe@example.com".to_string(),
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            password: "".to_string(),
            discoverable: true,
            ..Default::default()
        };
        let actual = serde_ddb::from_hashmap::<UserAccount, _>(doc).unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn password_serializes_to_bytes() {
        use uuid::Uuid;

        use super::*;

        let account = UserAccount {
            account_id: Uuid::nil(),
            email: "john.doe@example.com".to_string(),
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            password: "super_secret".to_string(),
            discoverable: false,
            ..Default::default()
        };
        let serialized = serde_ddb::to_hashmap(&account).unwrap();
        let serialized_password_attr = serialized.get(&"Password".to_string()).unwrap();
        assert!(serialized_password_attr.is_bs());
        assert!(!serialized_password_attr.is_s());
    }
}
