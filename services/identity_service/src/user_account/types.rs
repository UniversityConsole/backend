use std::convert::From;
use std::fmt::{Display, Formatter};

use identity_service::pb::account::State as StateModel;
use identity_service::pb::PermissionsDocument as PermissionsDocumentModel;
use serde::{Deserialize, Serialize};
use service_core::resource_access::AccessKind;
use typed_builder::TypedBuilder;
use uuid::Uuid;

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug, Default, TypedBuilder)]
#[serde(rename_all = "PascalCase")]
#[serde(deny_unknown_fields)]
pub struct UserAccount {
    #[serde(default = "Uuid::nil")]
    #[builder(default = Uuid::new_v4())]
    pub account_id: Uuid,

    #[builder(setter(into))]
    pub email: String,

    #[builder(setter(into))]
    pub first_name: String,

    #[builder(setter(into))]
    pub last_name: String,

    #[serde(default)]
    #[builder(setter(into))]
    pub password: String,

    #[builder(default = true)]
    pub discoverable: bool,

    #[builder(default = AccountState::PendingActivation)]
    pub account_state: AccountState,

    #[serde(default)]
    #[builder(default)]
    pub permissions_document: PermissionsDocument,
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub enum AccountState {
    PendingActivation,
    Active,
    Deactivated,
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

// FIXME Generate this automatically from the UserAccount structure.
#[derive(Clone, Copy, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum AccountAttr {
    AccountId,
    Email,
    FirstName,
    LastName,
    Password,
    Discoverable,
    AccountState,
    PermissionsDocument,
}


impl Default for AccountState {
    fn default() -> Self {
        AccountState::PendingActivation
    }
}

impl Display for AccountState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", &self)
    }
}

impl From<AccountState> for StateModel {
    fn from(s: AccountState) -> Self {
        match s {
            AccountState::PendingActivation => StateModel::PendingActivation,
            AccountState::Active => StateModel::Active,
            AccountState::Deactivated => StateModel::Deactivated,
        }
    }
}

impl From<PermissionsDocument> for PermissionsDocumentModel {
    fn from(val: PermissionsDocument) -> PermissionsDocumentModel {
        use identity_service::pb::policy_statement::AccessKind as AccessKindModel;
        use identity_service::pb::PolicyStatement;

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
        use identity_service::pb::policy_statement::AccessKind as AccessKindModel;

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

impl Display for AccountAttr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
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
            "Discoverable": true,
            "AccountState": "PendingActivation"
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
        doc.insert("AccountState".to_string(), AttributeValue::S("Active".to_string()));

        let expected = UserAccount {
            account_id: Uuid::nil(),
            email: "john.doe@example.com".to_string(),
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            password: "".to_string(),
            discoverable: true,
            account_state: AccountState::Active,
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
        assert!(serialized_password_attr.is_s());
    }
}
