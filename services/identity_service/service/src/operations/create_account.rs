use crate::Context;
use commons::{
    dataplane::UserAccount, CreateAccountError, CreateAccountInput, CreateAccountOutput,
};
use lambda_http::Request;
use rusoto_core::RusotoError;
use rusoto_dynamodb::{PutItemError, PutItemInput};
use std::convert::TryInto;
use utils::dynamodb_interop::Document;
use uuid::Uuid;

struct CreateAccountProcessor<'a> {
    ctx: &'a Context,
}

impl CreateAccountProcessor<'_> {
    pub async fn create_account(
        &self,
        input: &CreateAccountInput,
    ) -> Result<CreateAccountOutput, CreateAccountError> {
        let account_doc = UserAccount {
            account_id: Uuid::new_v4(),
            email: input.email.clone(),
            first_name: input.first_name.clone(),
            last_name: input.last_name.clone(),
            gov_id: input.gov_id.clone(),
            password: input.password.clone(),
        };

        self.ctx
            .dynamodb_client
            .put_item(PutItemInput {
                item: account_doc.document(),
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
            account_id: account_doc.account_id.to_hyphenated().to_string(),
        })
    }
}

pub async fn create_account(
    req: &Request,
    ctx: &Context,
) -> Result<CreateAccountOutput, CreateAccountError> {
    let input: CreateAccountInput = req.try_into().map_err(|_| CreateAccountError::BadRequest)?;
    let processor = CreateAccountProcessor { ctx };

    processor.create_account(&input).await
}
