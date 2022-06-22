use std::collections::HashMap;

use async_trait::async_trait;
use aws_sdk_dynamodb::error::{PutItemError, PutItemErrorKind};
use aws_sdk_dynamodb::model::{AttributeValue, Select};
use aws_sdk_dynamodb::types::SdkError;
use common_macros::hash_map;
use serde::{Deserialize, Serialize};
use service_core::ddb::get_item::{GetItem, GetItemInput};
use service_core::ddb::put_item::{PutItem, PutItemInput};
use service_core::ddb::query::{Query, QueryInput};
use service_core::ddb::scan::Scan;
use uuid::Uuid;
use validator::validate_email;

use crate::user_account::{
    AccountAttributes, AccountLookup, AccountsRepository, CreateAccountError, GetAccountError, UserAccount,
};


pub trait ThreadSafeDdbClient: PutItem + GetItem + Query + Scan + Send + Sync {}
impl<T: PutItem + GetItem + Query + Scan + Send + Sync> ThreadSafeDdbClient for T {}


pub struct DdbAccountsRepository<T: ThreadSafeDdbClient> {
    ddb: T,
    accounts_table_name: String,
}

impl<T: ThreadSafeDdbClient> DdbAccountsRepository<T> {
    pub fn new(ddb: T, accounts_table_name: impl Into<String>) -> Self {
        Self {
            ddb,
            accounts_table_name: accounts_table_name.into(),
        }
    }

    /// Given an account ID, create the correct DynamoDB key to interact with that item.
    async fn account_key_from_id(&self, account_id: &Uuid) -> Result<HashMap<String, AttributeValue>, GetAccountError> {
        let query_params = hash_map! {
            ":uuid".to_string() => AttributeValue::S(account_id.to_hyphenated().to_string()),
        };

        let query_input = QueryInput::builder()
            .index_name("AccountIdIndex")
            .table_name(self.accounts_table_name.as_str())
            .key_condition_expression("AccountId = :uuid")
            .select(Select::AllProjectedAttributes)
            .expression_attribute_values(Some(query_params))
            .limit(1)
            .build();
        let output = self
            .ddb
            .query(query_input)
            .await
            .map_err(|e| GetAccountError::Other(e.into()))?;

        let item = output
            .items
            .ok_or_else(|| GetAccountError::Other("Malformed reply: missing items".into()))?
            .pop()
            .ok_or(GetAccountError::NotFound)?;
        let projection: AccountIdIndexProjection =
            serde_ddb::from_hashmap(item).map_err(|e| GetAccountError::Serde(e))?;
        Ok(self.account_key_from_email(projection.email))
    }

    /// Given an email address, creates the map to be used as key to the User Accounts datastore.
    fn account_key_from_email(&self, email: String) -> HashMap<String, AttributeValue> {
        hash_map! {
            "Email".to_string() => AttributeValue::S(email),
        }
    }

    /// Retrieves an account from the DynamoDB table given its key.
    async fn account(
        &self,
        key: HashMap<String, AttributeValue>,
        attrs: &AccountAttributes,
    ) -> Result<UserAccount, GetAccountError> {
        let projection_expression = attrs.ddb_projection_expression();
        let get_item_input = GetItemInput::builder()
            .table_name(self.accounts_table_name.as_str())
            .projection_expression(projection_expression)
            .key(key)
            .build();
        let output = self
            .ddb
            .get_item(get_item_input)
            .await
            .map_err(|e| GetAccountError::Other(e.into()))?;

        match output.item {
            None => Err(GetAccountError::NotFound),
            Some(item) => {
                let user_account: UserAccount = serde_ddb::from_hashmap(item).map_err(|e| GetAccountError::Serde(e))?;
                Ok(user_account)
            }
        }
    }

    /// Generates the table key for the desired account ID, then retrieves the account from DynamoDB.
    ///
    /// # Notes
    ///
    /// The key generation can fail.
    async fn account_by_id(&self, id: &Uuid, attrs: &AccountAttributes) -> Result<UserAccount, GetAccountError> {
        let key = self.account_key_from_id(&id).await?;
        self.account(key, attrs).await
    }

    /// Generates the table key for the desired account email, then retrieves the account from DynamoDB.
    async fn account_by_email(&self, email: &str, attrs: &AccountAttributes) -> Result<UserAccount, GetAccountError> {
        let key = self.account_key_from_email(email.to_owned());
        self.account(key, attrs).await
    }
}

#[async_trait]
impl<T: ThreadSafeDdbClient> AccountsRepository for DdbAccountsRepository<T> {
    async fn create_account<'a>(&self, account: &'a UserAccount) -> Result<&'a Uuid, CreateAccountError> {
        if !validate_email(&account.email) {
            return Err(CreateAccountError::Validation("Email address is invalid."));
        }

        if account.password.is_empty() {
            return Err(CreateAccountError::Validation("Password is required."));
        }

        let put_item_input = PutItemInput::builder()
            .table_name(self.accounts_table_name.as_str())
            .item(serde_ddb::to_hashmap(&account).unwrap())
            .condition_expression("attribute_not_exists(Email)")
            .build();

        self.ddb.put_item(put_item_input).await.map_err(|err| match err {
            SdkError::ServiceError {
                err:
                    PutItemError {
                        kind: PutItemErrorKind::ConditionalCheckFailedException(_),
                        ..
                    },
                ..
            } => CreateAccountError::DuplicateAccount,
            e => CreateAccountError::Other(e.into()),
        })?;

        Ok(&account.account_id)
    }

    async fn get_account(
        &self,
        lookup: &AccountLookup,
        attrs: &AccountAttributes,
    ) -> Result<UserAccount, GetAccountError> {
        match lookup {
            AccountLookup::ByEmail(email) => self.account_by_email(email, attrs).await,
            AccountLookup::ById(id) => self.account_by_id(id, attrs).await,
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct AccountIdIndexProjection {
    account_id: Uuid,
    email: String,
}
