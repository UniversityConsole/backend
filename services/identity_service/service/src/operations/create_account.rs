use crate::Context;
use commons::{CreateAccountError, CreateAccountInput, CreateAccountOutput};
use lambda_http::Request;
use rusoto_core::RusotoError;
use rusoto_dynamodb::{PutItemError, PutItemInput};
use serde_dynamodb::to_hashmap;
use std::convert::TryInto;

struct CreateAccountProcessor<'a> {
    ctx: &'a Context,
}

impl CreateAccountProcessor<'_> {
    pub async fn create_account(
        &self,
        input: &CreateAccountInput,
    ) -> Result<CreateAccountOutput, CreateAccountError> {
        let account = &input.account;

        self.ctx
            .dynamodb_client
            .put_item(PutItemInput {
                item: to_hashmap(&account).unwrap(),
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
            account_id: account.account_id.to_hyphenated().to_string(),
        })
    }
}

pub async fn handler(
    req: &Request,
    ctx: &Context,
) -> Result<CreateAccountOutput, CreateAccountError> {
    let input: CreateAccountInput = req.try_into().map_err(|_| CreateAccountError::BadRequest)?;
    let processor = CreateAccountProcessor { ctx };

    processor.create_account(&input).await
}
