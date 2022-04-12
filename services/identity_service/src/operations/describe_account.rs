use crate::svc::DescribeAccountInput;
use crate::svc::DescribeAccountOutput;
use crate::user_account::UserAccount;
use crate::Context;
use aws_sdk_dynamodb::model::AttributeValue;
use aws_sdk_dynamodb::model::Select;
use common_macros::hash_map;
use serde::{Deserialize, Serialize};
use service_core::ddb::get_item::{GetItem, GetItemInput};
use service_core::ddb::query::{Query, QueryInput};
use service_core::endpoint_error::EndpointError;
use service_core::operation_error::OperationError;
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
#[serde(deny_unknown_fields)]
struct AccountIdIndexProjection {
    account_id: uuid::Uuid,
    email: String,
}

#[non_exhaustive]
#[derive(thiserror::Error, Debug)]
pub enum DescribeAccountError {
    #[error("Account not found.")]
    NotFoundError,
}

pub(crate) async fn describe_account(
    ctx: &Context,
    ddb: &(impl GetItem + Query),
    input: &DescribeAccountInput,
) -> Result<DescribeAccountOutput, EndpointError<DescribeAccountError>> {
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
        return Err(EndpointError::operation(
            DescribeAccountError::NotFoundError,
        ));
    }

    let items = output.items.unwrap();
    let item: AccountIdIndexProjection = serde_ddb::from_hashmap(items[0].clone()).unwrap();
    let projection_expression = [
        "AccountId",
        "Email",
        "FirstName",
        "LastName",
        "Discoverable",
    ]
    .join(",");
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
            Err(EndpointError::operation(
                DescribeAccountError::NotFoundError,
            ))
        }
        Some(item) => {
            let user_account: UserAccount = serde_ddb::from_hashmap(item).map_err(|e| {
                log::error!("Invalid record in DynamoDB. Original error: {:?}.", e);
                EndpointError::internal()
            })?;
            Ok(DescribeAccountOutput {
                account: Some(crate::svc::Account {
                    account_id: user_account.account_id.to_hyphenated().to_string(),
                    email: user_account.email,
                    first_name: user_account.first_name,
                    last_name: user_account.last_name,
                    discoverable: user_account.discoverable,
                }),
            })
        }
    }
}

impl OperationError for DescribeAccountError {
    fn code(&self) -> tonic::Code {
        match self {
            Self::NotFoundError => tonic::Code::NotFound,
        }
    }
}
