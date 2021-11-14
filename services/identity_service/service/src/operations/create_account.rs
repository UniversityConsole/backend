use crate::svc::CreateAccountInput;
use crate::svc::CreateAccountOutput;
use crate::user_account::UserAccount;
use crate::Context;
use rusoto_core::RusotoError;
use rusoto_dynamodb::{PutItemError, PutItemInput};
use service_core::endpoint_error::EndpointError;
use service_core::operation_error::OperationError;
use std::error::Error;
use std::fmt::Display;
use uuid::Uuid;

#[non_exhaustive]
#[derive(Debug)]
pub enum CreateAccountError {
    DuplicateAccountError,
}

pub(crate) async fn create_account(
    ctx: &Context,
    input: &CreateAccountInput,
) -> Result<CreateAccountOutput, EndpointError<CreateAccountError>> {
    let account_attributes = input
        .account_attributes
        .as_ref()
        .ok_or(EndpointError::Validation(
            "Account attributes missing.".to_string(),
        ))?;

    if account_attributes.password.is_empty() {
        return Err(EndpointError::Validation(String::from(
            "Password is required.",
        )));
    }

    let account = UserAccount {
        account_id: Uuid::new_v4(),
        email: account_attributes.email.clone(),
        first_name: account_attributes.first_name.clone(),
        last_name: account_attributes.last_name.clone(),
        password: account_attributes.password.clone(),
        discoverable: account_attributes.discoverable,
    };

    ctx.dynamodb_client
        .put_item(PutItemInput {
            item: serde_dynamodb::to_hashmap(&account).unwrap(),
            table_name: ctx.accounts_table_name.clone(),
            condition_expression: Some("attribute_not_exists(Email)".to_string()),
            ..PutItemInput::default()
        })
        .await
        .map_err(|err| match err {
            RusotoError::Service(PutItemError::ConditionalCheckFailed(_)) => {
                EndpointError::Operation(CreateAccountError::DuplicateAccountError)
            }
            _ => {
                log::error!("Failed creating item in DynamoDB: {:?}", err);
                EndpointError::Internal
            }
        })?;

    Ok(CreateAccountOutput {
        account_id: account.account_id.to_string(),
    })
}

impl Display for CreateAccountError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CreateAccountError::DuplicateAccountError => {
                write!(f, "An account with this email address already exists.")
            }
        }
    }
}

impl Error for CreateAccountError {}

impl OperationError for CreateAccountError {
    fn code(&self) -> tonic::Code {
        match self {
            CreateAccountError::DuplicateAccountError => tonic::Code::AlreadyExists,
        }
    }
}
