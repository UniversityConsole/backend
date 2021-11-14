use crate::svc::DescribeAccountInput;
use crate::svc::DescribeAccountOutput;
use crate::user_account::UserAccount;
use crate::Context;
use rusoto_dynamodb::{AttributeValue, GetItemInput, QueryInput};
use serde::{Deserialize, Serialize};
use service_core::endpoint_error::EndpointError;
use service_core::operation_error::OperationError;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::Display;
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
#[serde(deny_unknown_fields)]
struct AccountIdIndexProjection {
    account_id: uuid::Uuid,
    email: String,
}

#[non_exhaustive]
#[derive(Debug)]
pub enum DescribeAccountError {
    NotFoundError,
}

pub(crate) async fn describe_account(
    ctx: &Context,
    input: &DescribeAccountInput,
) -> Result<DescribeAccountOutput, EndpointError<DescribeAccountError>> {
    let account_id = Uuid::parse_str(input.account_id.clone().as_mut())
        .map_err(|_| EndpointError::Validation("Invalid account ID provided.".to_string()))?;
    let mut query_params = HashMap::new();
    query_params.insert(
        ":uuid".to_string(),
        AttributeValue {
            s: Some(account_id.to_hyphenated().to_string()),
            ..AttributeValue::default()
        },
    );

    let output = ctx
        .dynamodb_client
        .query(QueryInput {
            index_name: Some("AccountIdIndex".to_string()),
            table_name: ctx.accounts_table_name.clone(),
            key_condition_expression: Some("AccountId = :uuid".to_string()),
            select: Some("ALL_PROJECTED_ATTRIBUTES".to_string()),
            expression_attribute_values: Some(query_params),
            ..QueryInput::default()
        })
        .await
        .map_err(|e| {
            log::error!("Failed to query DynamoDB. Original error: {:?}.", e);
            EndpointError::Internal
        })?;

    if output.count.unwrap() == 0 {
        return Err(EndpointError::Operation(
            DescribeAccountError::NotFoundError,
        ));
    }

    let items = output.items.unwrap();
    let item: AccountIdIndexProjection = serde_dynamodb::from_hashmap(items[0].clone()).unwrap();
    let projection_expression = [
        "AccountId",
        "Email",
        "FirstName",
        "LastName",
        "Discoverable",
    ]
    .join(",");
    let mut key = HashMap::new();
    key.insert(
        "Email".to_string(),
        AttributeValue {
            s: Some(item.email),
            ..AttributeValue::default()
        },
    );
    let output = ctx
        .dynamodb_client
        .get_item(GetItemInput {
            table_name: ctx.accounts_table_name.clone(),
            projection_expression: Some(projection_expression),
            key,
            ..GetItemInput::default()
        })
        .await
        .map_err(|e| {
            log::error!(
                "Failed to retrieve item from DynamoDB. Original error: {:?}.",
                e
            );
            EndpointError::Internal
        })?;

    match output.item {
        None => {
            log::warn!(
                "Item found on Query, but not found on GetItem. Queried AccountId: {}",
                account_id.to_hyphenated().to_string()
            );
            Err(EndpointError::Operation(
                DescribeAccountError::NotFoundError,
            ))
        }
        Some(item) => {
            let user_account: UserAccount = serde_dynamodb::from_hashmap(item).map_err(|e| {
                log::error!("Invalid record in DynamoDB. Original error: {:?}.", e);
                EndpointError::Internal
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

impl Display for DescribeAccountError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFoundError => write!(f, "Account not found."),
        }
    }
}

impl Error for DescribeAccountError {}
