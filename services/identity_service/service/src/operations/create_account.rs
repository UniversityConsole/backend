use crate::data_plane::UserAccount;
use commons::{CreateAccountError, CreateAccountInput, CreateAccountOutput};
use lambda_http::Request;
use rusoto_core::RusotoError;
use rusoto_dynamodb::{DynamoDb, DynamoDbClient, PutItemError, PutItemInput};
use std::{convert::TryInto, env};

const DATASTORE_NAME_VAR: &str = "USER_ACCOUNTS_TABLE_NAME";

struct CreateAccountProcessor<'a> {
    dynamodb_client: &'a (dyn DynamoDb + Sync + Send + 'a),
    datastore_name: String,
}

impl CreateAccountProcessor<'_> {
    pub async fn create_account(
        &self,
        input: &CreateAccountInput,
    ) -> Result<CreateAccountOutput, CreateAccountError> {
        let account_doc = UserAccount::from_input(input);

        self.dynamodb_client
            .put_item(PutItemInput {
                item: account_doc.as_hashmap(),
                table_name: self.datastore_name.clone(),
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
}

pub async fn create_account(req: &Request) -> Result<CreateAccountOutput, CreateAccountError> {
    let input: CreateAccountInput = req.try_into().map_err(|_| CreateAccountError::BadRequest)?;
    let datastore_name =
        env::var(DATASTORE_NAME_VAR).map_err(|_| CreateAccountError::InternalError)?;
    let dynamodb_client = DynamoDbClient::new(rusoto_core::Region::EuWest1);

    let processor = CreateAccountProcessor {
        dynamodb_client: &dynamodb_client,
        datastore_name,
    };

    processor.create_account(&input).await
}
