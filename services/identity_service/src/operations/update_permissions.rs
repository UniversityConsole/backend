use aws_sdk_dynamodb::model::AttributeValue;
use common_macros::hash_map;
use identity_service::pb::{UpdatePermissionsInput, UpdatePermissionsOutput};
use service_core::ddb::query::Query;
use service_core::ddb::update_item::{UpdateItem, UpdateItemInput};
use service_core::endpoint_error::EndpointError;
use service_core::operation_error::OperationError;
use thiserror::Error;
use uuid::Uuid;

use crate::user_account::PermissionsDocument;
use crate::utils::account::{account_key_from_id, AccountKeyFromIdError};
use crate::utils::validation::validate_resource_paths;
use crate::Context;

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum UpdatePermissionsError {
    #[error("Account not found.")]
    NotFound,

    #[error("Resource path {1} in statement {0} is invalid.")]
    InvalidResourcePath(usize, usize),
}

pub(crate) async fn update_permissions(
    ctx: &Context,
    ddb: &(impl Query + UpdateItem),
    input: &UpdatePermissionsInput,
) -> Result<UpdatePermissionsOutput, EndpointError<UpdatePermissionsError>> {
    let account_id = Uuid::parse_str(input.account_id.clone().as_mut())
        .map_err(|_| EndpointError::validation("Invalid account ID provided."))?;
    let permissions_document: PermissionsDocument = input
        .permissions_document
        .as_ref()
        .map(|s| s.clone())
        .ok_or_else(|| EndpointError::validation("missing permissions document"))?
        .into();

    validate_resource_paths(&permissions_document.statements).map_err(|(stmt_idx, path_idx)| {
        EndpointError::operation(UpdatePermissionsError::InvalidResourcePath(stmt_idx, path_idx))
    })?;

    let key = account_key_from_id(ddb, ctx.accounts_table_name.as_ref(), &account_id)
        .await
        .map_err(|e| match e {
            AccountKeyFromIdError::AccountNotFound => EndpointError::operation(UpdatePermissionsError::NotFound),
            _ => EndpointError::internal(),
        })?;
    let update_item_input = UpdateItemInput::builder()
        .table_name(ctx.accounts_table_name.clone())
        .key(key)
        .update_expression("SET PermissionsDocument = :permissions_document")
        .expression_attribute_values(hash_map! {
            ":permissions_document".to_owned() => AttributeValue::M(
                serde_ddb::to_hashmap(&permissions_document)
                    .expect("failed permissions document serialization")
            ),
        })
        .build();

    ddb.update_item(update_item_input).await.map_err(|e| {
        log::error!("Failed to update item in DynamoDB. Original error: {:?}.", e);
        EndpointError::internal()
    })?;

    Ok(UpdatePermissionsOutput {})
}

impl OperationError for UpdatePermissionsError {
    fn code(&self) -> tonic::Code {
        match self {
            Self::NotFound => tonic::Code::NotFound,
            Self::InvalidResourcePath(..) => tonic::Code::InvalidArgument,
        }
    }
}
