use crate::Context;
use base64;
use commons::{ListAccountsError, ListAccountsInput, ListAccountsOutput};
use lambda_http::Request;
use rusoto_dynamodb::{ScanInput, AttributeValue};
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
                hm.insert("Email".to_string(), AttributeValue {
                    s: Some(v),
                    ..AttributeValue::default()
                });
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

        Err(ListAccountsError::InternalError)
    }
}

pub async fn list_accounts(
    req: &Request,
    ctx: &Context,
) -> Result<ListAccountsOutput, ListAccountsError> {
    let input: ListAccountsInput = req.try_into().map_err(|_| {
        ListAccountsError::ValidationError("Could not parse request input.".to_string())
    })?;
    let processor = ListAccountsProcessor { ctx };

    processor.list_accounts(&input).await
}
