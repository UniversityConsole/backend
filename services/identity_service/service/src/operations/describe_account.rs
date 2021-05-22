use crate::Context;
use identity_service_commons::dataplane::UserAccount;
use identity_service_commons::{DescribeAccountError, DescribeAccountInput, DescribeAccountOutput};
use lambda_http::Request;
use rusoto_dynamodb::{AttributeValue, GetItemInput, QueryInput};
use serde::{Deserialize, Serialize};
use service_core::EndpointError;
use std::collections::HashMap;
use std::convert::TryInto;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
#[serde(deny_unknown_fields)]
struct AccountIdIndexProjection {
    account_id: uuid::Uuid,
    email: String,
}

struct DescribeAccountProcessor<'a> {
    ctx: &'a Context,
}

impl DescribeAccountProcessor<'_> {
    pub async fn describe_account(
        &self,
        input: &DescribeAccountInput,
    ) -> Result<DescribeAccountOutput, EndpointError<DescribeAccountError>> {
        let mut query_params = HashMap::new();
        query_params.insert(
            ":uuid".to_string(),
            AttributeValue {
                s: Some(input.account_id.to_hyphenated().to_string()),
                ..AttributeValue::default()
            },
        );

        let output = self
            .ctx
            .dynamodb_client
            .query(QueryInput {
                index_name: Some("AccountIdIndex".to_string()),
                table_name: self.ctx.datastore_name.clone(),
                key_condition_expression: Some("AccountId = :uuid".to_string()),
                select: Some("ALL_PROJECTED_ATTRIBUTES".to_string()),
                expression_attribute_values: Some(query_params),
                ..QueryInput::default()
            })
            .await
            .map_err(|e| {
                log::error!("Failed to query DynamoDB. Original error: {:?}.", e);
                EndpointError::InternalError
            })?;

        if output.count.unwrap() == 0 {
            return Err(EndpointError::Operation(
                DescribeAccountError::NotFoundError,
            ));
        }

        let items = output.items.unwrap();
        let item: AccountIdIndexProjection =
            serde_dynamodb::from_hashmap(items[0].clone()).unwrap();
        let projection_expression =
            ["AccountId", "Email", "FirstName", "LastName", "GovId"].join(",");
        let mut key = HashMap::new();
        key.insert(
            "Email".to_string(),
            AttributeValue {
                s: Some(item.email),
                ..AttributeValue::default()
            },
        );
        let output = self
            .ctx
            .dynamodb_client
            .get_item(GetItemInput {
                table_name: self.ctx.datastore_name.clone(),
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
                EndpointError::InternalError
            })?;

        match output.item {
            None => {
                log::warn!(
                    "Item found on Query, but not found on GetItem. Queried AccountId: {}",
                    input.account_id.to_hyphenated().to_string()
                );
                Err(EndpointError::Operation(
                    DescribeAccountError::NotFoundError,
                ))
            }
            Some(item) => {
                let user_account: UserAccount =
                    serde_dynamodb::from_hashmap(item).map_err(|e| {
                        log::error!("Invalid record in DynamoDB. Original error: {:?}.", e);
                        EndpointError::InternalError
                    })?;
                Ok(DescribeAccountOutput {
                    account: user_account,
                })
            }
        }
    }
}

pub async fn handler(
    req: &Request,
    ctx: &Context,
) -> Result<DescribeAccountOutput, EndpointError<DescribeAccountError>> {
    let input: DescribeAccountInput = req.try_into().map_err(|_| {
        EndpointError::BadRequestError("Could not parse request input.".to_string())
    })?;
    let processor = DescribeAccountProcessor { ctx };

    processor.describe_account(&input).await
}
