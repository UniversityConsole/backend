use bytes::Bytes;
use commons::{CreateAccountError, CreateAccountInput, CreateAccountOutput};
use lambda_http::Request;
use rusoto_core::RusotoError;
use rusoto_dynamodb::{AttributeValue, DynamoDb, DynamoDbClient, PutItemError, PutItemInput};
use sha2::{Digest, Sha512};
use std::{collections::HashMap, convert::TryInto, env};
use uuid::Uuid;

const ACCOUNTS_DATASTORE_NAME_VAR: &str = "USER_ACCOUNTS_TABLE_NAME";

pub struct UserAccount {
    pub account_id: Uuid,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub gov_id: String,
    pub password: String,
}

impl UserAccount {
    pub fn from_input(input: CreateAccountInput) -> Self {
        UserAccount {
            account_id: Uuid::new_v4(),
            email: input.email,
            first_name: input.first_name,
            last_name: input.last_name,
            gov_id: input.gov_id,
            password: input.password,
        }
    }

    pub fn as_hashmap(&self) -> HashMap<String, AttributeValue> {
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

pub async fn create_account(req: &Request) -> Result<CreateAccountOutput, CreateAccountError> {
    let input: CreateAccountInput = req.try_into().map_err(|_| CreateAccountError::BadRequest)?;
    let accounts_datastore_name =
        env::var(ACCOUNTS_DATASTORE_NAME_VAR).map_err(|_| CreateAccountError::InternalError)?;
    let dynamodb_client = DynamoDbClient::new(rusoto_core::Region::EuWest1);

    let account_doc = UserAccount::from_input(input);

    dynamodb_client
        .put_item(PutItemInput {
            item: account_doc.as_hashmap(),
            table_name: accounts_datastore_name,
            condition_expression: Some("attribute_not_exists(Email)".to_string()),
            ..PutItemInput::default()
        })
        .await
        .map_err(|err| match err {
            RusotoError::Service(PutItemError::ConditionalCheckFailed(_)) => {
                CreateAccountError::DuplicateAccount
            }
            _ => {
                log::error!("Failed creating item in DynamoDB: {:?}", err);
                CreateAccountError::InternalError
            }
        })?;

    Ok(CreateAccountOutput {
        account_id: account_doc.account_id.to_hyphenated().to_string(),
    })
}
