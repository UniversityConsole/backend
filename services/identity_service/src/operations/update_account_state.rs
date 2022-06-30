use aws_sdk_dynamodb::model::AttributeValue;
use common_macros::hash_map;
use identity_service::pb::{UpdateAccountStateInput, UpdateAccountStateOutput};
use service_core::ddb::query::Query;
use service_core::ddb::update_item::{UpdateItem, UpdateItemInput};
use service_core::endpoint_error::EndpointError;
use service_core::operation_error::OperationError;
use thiserror::Error;
use uuid::Uuid;

use crate::user_account::types::AccountState;
use crate::utils::account::{account_key_from_id, AccountKeyFromIdError};
use crate::Context;

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum UpdateAccountStateError {
    #[error("Account not found.")]
    NotFound,
}

pub(crate) async fn update_account_state(
    ctx: &Context,
    ddb: &(impl Query + UpdateItem),
    mut input: UpdateAccountStateInput,
) -> Result<UpdateAccountStateOutput, EndpointError<UpdateAccountStateError>> {
    let account_id = Uuid::parse_str(input.account_id.as_mut())
        .map_err(|_| EndpointError::validation("Invalid account ID provided."))?;
    let account_state = AccountState::try_from(input.account_state)
        .map_err(|_| EndpointError::validation("Invalid account state provided."))?;

    let key = account_key_from_id(ddb, ctx.accounts_table_name.as_ref(), &account_id)
        .await
        .map_err(|e| match e {
            AccountKeyFromIdError::AccountNotFound => EndpointError::operation(UpdateAccountStateError::NotFound),
            _ => {
                log::error!("Failed to look up account by ID. Error: {:?}", e);
                EndpointError::internal()
            }
        })?;
    let update_item_input = UpdateItemInput::builder()
        .table_name(ctx.accounts_table_name.clone())
        .key(key)
        .update_expression("SET AccountState = :account_state")
        .expression_attribute_values(hash_map! {
            ":account_state".to_owned() => AttributeValue::M(
                serde_ddb::to_hashmap(&account_state)
                    .expect("failed permissions document serialization")
            ),
        })
        .build();

    ddb.update_item(update_item_input).await.map_err(|e| {
        log::error!("Failed to update item in DynamoDB. Original error: {:?}.", e);
        EndpointError::internal()
    })?;

    Ok(UpdateAccountStateOutput {})
}

impl OperationError for UpdateAccountStateError {
    fn code(&self) -> tonic::Code {
        match self {
            Self::NotFound => tonic::Code::NotFound,
        }
    }
}
