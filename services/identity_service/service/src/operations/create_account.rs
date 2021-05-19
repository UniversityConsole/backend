use crate::Context;
use identity_service_commons::{CreateAccountError, CreateAccountInput, CreateAccountOutput};
use lambda_http::Request;
use rusoto_core::RusotoError;
use rusoto_dynamodb::{PutItemError, PutItemInput};
use std::convert::TryInto;
use uuid::Uuid;

struct CreateAccountProcessor<'a> {
    ctx: &'a Context,
}

impl CreateAccountProcessor<'_> {
    pub async fn create_account(
        &self,
        input: &CreateAccountInput,
    ) -> Result<CreateAccountOutput, CreateAccountError> {
        let mut account = input.account.clone();
        account.account_id = Uuid::new_v4();

        if account.password.is_empty() {
            return Err(CreateAccountError::Validation(String::from(
                "Password is required.",
            )));
        }

        self.ctx
            .dynamodb_client
            .put_item(PutItemInput {
                item: serde_dynamodb::to_hashmap(&account).unwrap(),
                table_name: self.ctx.datastore_name.clone(),
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
            account_id: account.account_id,
        })
    }
}

pub async fn handler(
    req: &Request,
    ctx: &Context,
) -> Result<CreateAccountOutput, CreateAccountError> {
    let input: CreateAccountInput = req
        .try_into()
        .map_err(|_| CreateAccountError::Validation("Invalid request".to_string()))?;
    let processor = CreateAccountProcessor { ctx };

    processor.create_account(&input).await
}
