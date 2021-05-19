use crate::Context;
use base64;
use identity_service_commons::{ListAccountsError, ListAccountsInput, ListAccountsOutput};
use lambda_http::Request;
use rusoto_dynamodb::{AttributeValue, ScanInput};
use serde_dynamodb::from_hashmap;
use std::collections::HashMap;
use std::convert::TryInto;

struct ListAccountsProcessor<'a> {
    ctx: &'a Context,
}

impl ListAccountsProcessor<'_> {
    pub async fn list_accounts(
        &self,
        input: &ListAccountsInput,
    ) -> Result<ListAccountsOutput, ListAccountsError> {
        // TODO Find a way not to hard-code this.
        let projection_expression =
            ["AccountId", "Email", "FirstName", "LastName", "GovId"].join(",");
        let page_start = match &input.starting_token {
            Some(v) => {
                const PARSE_ERR_MSG: &str = "Could not parse StartingToken.";
                let v = base64::decode(&v)
                    .map_err(|_| ListAccountsError::ValidationError(PARSE_ERR_MSG.to_string()))?;
                let v = String::from_utf8(v)
                    .map_err(|_| ListAccountsError::ValidationError(PARSE_ERR_MSG.to_string()))?;
                let mut hm = HashMap::new();
                hm.insert(
                    "Email".to_string(),
                    AttributeValue {
                        s: Some(v),
                        ..AttributeValue::default()
                    },
                );
                Some(hm)
            }
            None => None,
        };
        let scan_output = self
            .ctx
            .dynamodb_client
            .scan(ScanInput {
                limit: Some(input.page_size),
                projection_expression: Some(projection_expression),
                table_name: self.ctx.datastore_name.clone(),
                exclusive_start_key: page_start,
                ..ScanInput::default()
            })
            .await
            .map_err(|_| ListAccountsError::InternalError)?;

        let next_token = match scan_output.last_evaluated_key {
            None => None,
            Some(hm) => {
                let email_value = hm.get(&"Email".to_string()).unwrap();
                Some(base64::encode(email_value.s.as_ref().unwrap()))
            }
        };

        let accounts = match scan_output.items {
            None => vec![],
            Some(items) => {
                // TODO Allocate scan_output.items_count elements here.
                let mut accounts = vec![];
                for item in items.into_iter() {
                    let item_json = serde_json::json!(&item);
                    let account = from_hashmap(item).map_err(|err| {
                        log::error!(
                            "Invalid record in DynamoDB: {}. Original item: {}",
                            err,
                            item_json
                        );
                        ListAccountsError::InternalError
                    })?;
                    accounts.push(account);
                }
                accounts
            }
        };

        Ok(ListAccountsOutput {
            next_token,
            accounts,
        })
    }
}

pub async fn handler(
    req: &Request,
    ctx: &Context,
) -> Result<ListAccountsOutput, ListAccountsError> {
    let input: ListAccountsInput = req.try_into().map_err(|_| {
        ListAccountsError::ValidationError("Could not parse request input.".to_string())
    })?;
    let processor = ListAccountsProcessor { ctx };

    processor.list_accounts(&input).await
}
