use aws_sdk_dynamodb::error::{PutItemError, PutItemErrorKind};
use aws_sdk_dynamodb::types::SdkError;
use service_core::ddb::put_item::{PutItem, PutItemInput};
use service_core::endpoint_error::EndpointError;
use service_core::operation_error::OperationError;
use uuid::Uuid;

use crate::svc::{CreateAccountInput, CreateAccountOutput};
use crate::user_account::{PermissionsDocument, UserAccount};
use crate::Context;

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum CreateAccountError {
    #[error("An account with this email already exists.")]
    DuplicateAccountError,
}

pub(crate) async fn create_account(
    ctx: &Context,
    ddb: &impl PutItem,
    input: &CreateAccountInput,
) -> Result<CreateAccountOutput, EndpointError<CreateAccountError>> {
    let account_attributes = input
        .account_attributes
        .as_ref()
        .ok_or_else(|| EndpointError::validation("Account attributes missing."))?;

    if account_attributes.password.is_empty() {
        return Err(EndpointError::validation("Password is required."));
    }

    let account = UserAccount {
        account_id: Uuid::new_v4(),
        email: account_attributes.email.clone(),
        first_name: account_attributes.first_name.clone(),
        last_name: account_attributes.last_name.clone(),
        password: account_attributes.password.clone(),
        discoverable: account_attributes.discoverable,
        permissions_document: PermissionsDocument::default(),
    };

    let put_item_input = PutItemInput::builder()
        .table_name(ctx.accounts_table_name.clone())
        .item(serde_ddb::to_hashmap(&account).unwrap())
        .condition_expression("attribute_not_exists(Email)")
        .build();

    ddb.put_item(put_item_input).await.map_err(|err| match err {
        SdkError::ServiceError {
            err:
                PutItemError {
                    kind: PutItemErrorKind::ConditionalCheckFailedException(_),
                    ..
                },
            ..
        } => EndpointError::Operation(CreateAccountError::DuplicateAccountError),
        _ => {
            log::error!("Failed creating item in DynamoDB: {:?}", err);
            EndpointError::Internal
        }
    })?;

    Ok(CreateAccountOutput {
        account_id: account.account_id.to_string(),
    })
}

impl OperationError for CreateAccountError {
    fn code(&self) -> tonic::Code {
        match self {
            CreateAccountError::DuplicateAccountError => tonic::Code::AlreadyExists,
        }
    }
}
