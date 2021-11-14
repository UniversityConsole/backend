use crate::context::Context;
use crate::svc::CreateAccountInput;
use crate::svc::CreateAccountOutput;
use identity_service_commons::dataplane::UserAccount;
use identity_service_commons::CreateAccountError;
use rusoto_core::RusotoError;
use rusoto_dynamodb::{PutItemError, PutItemInput};
use service_core::EndpointError;
use uuid::Uuid;

pub(crate) async fn create_account(
    ctx: &Context,
    input: &CreateAccountInput,
) -> Result<CreateAccountOutput, EndpointError<CreateAccountError>> {
    log::debug!("CreateAccount input: {:?}", &input);

    let account_attributes =
        input
            .account_attributes
            .as_ref()
            .ok_or(EndpointError::BadRequestError(
                "Account attributes missing.".to_string(),
            ))?;

    if account_attributes.password.is_empty() {
        return Err(EndpointError::BadRequestError(String::from(
            "Password is required.",
        )));
    }

    let account = UserAccount {
        account_id: Uuid::new_v4(),
        email: account_attributes.email.clone(),
        first_name: account_attributes.first_name.clone(),
        last_name: account_attributes.last_name.clone(),
        password: account_attributes.password.clone(),
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
                EndpointError::InternalError
            }
        })?;

    Ok(CreateAccountOutput {
        account_id: account.account_id.to_string(),
    })
}
