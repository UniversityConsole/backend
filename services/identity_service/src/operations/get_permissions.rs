use aws_sdk_dynamodb::model::{AttributeValue, Select};
use common_macros::hash_map;
use serde::{Deserialize, Serialize};
use service_core::ddb::get_item::{GetItem, GetItemInput};
use service_core::ddb::query::{Query, QueryInput};
use service_core::endpoint_error::EndpointError;
use service_core::operation_error::OperationError;
use uuid::Uuid;

use crate::svc::{GetPermissionsInput, GetPermissionsOutput};
use crate::user_account::PermissionsDocument;
use crate::Context;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
#[serde(deny_unknown_fields)]
struct AccountIdIndexProjection {
    account_id: uuid::Uuid,
    email: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "PascalCase")]
struct PermissionsDocumentItem {
    permissions_document: PermissionsDocument,
}

#[non_exhaustive]
#[derive(thiserror::Error, Debug)]
pub enum GetPermissionsError {
    #[error("Account not found.")]
    NotFoundError,
}

pub(crate) async fn get_permissions(
    ctx: &Context,
    ddb: &(impl GetItem + Query),
    input: &GetPermissionsInput,
) -> Result<GetPermissionsOutput, EndpointError<GetPermissionsError>> {
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
        return Err(EndpointError::operation(GetPermissionsError::NotFoundError));
    }

    let items = output.items.unwrap();
    let item: AccountIdIndexProjection = serde_ddb::from_hashmap(items[0].clone()).unwrap();
    let projection_expression = ["PermissionsDocument"].join(",");
    let key = hash_map! {
        "Email".to_string() => AttributeValue::S(item.email),
    };

    let get_item_input = GetItemInput::builder()
        .table_name(ctx.accounts_table_name.clone())
        .projection_expression(projection_expression)
        .key(key)
        .build();
    let output = ddb.get_item(get_item_input).await.map_err(|e| {
        log::error!("Failed to get item from DynamoDB. Original error: {:?}.", e);
        EndpointError::internal()
    })?;

    match output.item {
        None => {
            log::warn!(
                "Item found on Query, but not found on GetItem. Queried AccountId: {}",
                account_id.to_hyphenated().to_string()
            );
            Err(EndpointError::operation(GetPermissionsError::NotFoundError))
        }
        Some(item) => {
            let item: PermissionsDocumentItem = serde_ddb::from_hashmap(item).map_err(|e| {
                log::error!("Invalid record in DynamoDB. Original error: {:?}.", e);
                EndpointError::internal()
            })?;

            Ok(GetPermissionsOutput {
                permissions_document: Some(item.permissions_document.into()),
            })
        }
    }
}

impl OperationError for GetPermissionsError {
    fn code(&self) -> tonic::Code {
        match self {
            Self::NotFoundError => tonic::Code::NotFound,
        }
    }
}
