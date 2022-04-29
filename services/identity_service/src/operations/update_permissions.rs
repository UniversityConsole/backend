use aws_sdk_dynamodb::model::{AttributeValue, Select};
use common_macros::hash_map;
use serde::{Deserialize, Serialize};
use service_core::ddb::query::{Query, QueryInput};
use service_core::ddb::update_item::{UpdateItem, UpdateItemInput};
use service_core::endpoint_error::EndpointError;
use service_core::operation_error::OperationError;
use uuid::Uuid;

use crate::svc::{UpdatePermissionsInput, UpdatePermissionsOutput};
use crate::user_account::PermissionsDocument;
use crate::utils::validation::validate_resource_paths;
use crate::Context;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
#[serde(deny_unknown_fields)]
struct AccountIdIndexProjection {
    account_id: uuid::Uuid,
    email: String,
}

#[non_exhaustive]
#[derive(thiserror::Error, Debug)]
pub enum UpdatePermissionsError {
    #[error("Account not found.")]
    NotFoundError,

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
    let query_params = hash_map! {
        ":uuid".to_string() => AttributeValue::S(account_id.to_hyphenated().to_string()),
    };

    let query_input = QueryInput::builder()
        .index_name("AccountIdIndex")
        .table_name(ctx.accounts_table_name.clone())
        .key_condition_expression("AccountId = :uuid")
        .select(Select::AllProjectedAttributes)
        .expression_attribute_values(Some(query_params))
        .limit(1)
        .build();
    let output = ddb.query(query_input).await.map_err(|e| {
        log::error!("Failed to query DynamoDB. Original error: {:?}.", e);
        EndpointError::internal()
    })?;

    if output.count == 0 {
        return Err(EndpointError::operation(UpdatePermissionsError::NotFoundError));
    }

    let items = output.items.unwrap();
    let item: AccountIdIndexProjection = serde_ddb::from_hashmap(items[0].clone()).unwrap();
    let key = hash_map! {
        "Email".to_string() => AttributeValue::S(item.email),
    };
    let permissions_document: PermissionsDocument = input
        .permissions_document
        .as_ref()
        .map(|s| s.clone())
        .ok_or_else(|| EndpointError::validation("missing permissions document"))?
        .into();

    validate_resource_paths(&permissions_document.statements).map_err(|(stmt_idx, path_idx)| {
        EndpointError::operation(UpdatePermissionsError::InvalidResourcePath(stmt_idx, path_idx))
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
            Self::NotFoundError => tonic::Code::NotFound,
            Self::InvalidResourcePath(..) => tonic::Code::InvalidArgument,
        }
    }
}
