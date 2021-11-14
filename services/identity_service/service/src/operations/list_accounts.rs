use crate::Context;
use base64;
use identity_service_commons::dataplane::UserAccount;
use identity_service_commons::default_page_size;
use identity_service_commons::ListAccountsError;
use rusoto_dynamodb::{AttributeValue, ScanInput};
use serde_dynamodb::from_hashmap;
use service_core::EndpointError;
use std::collections::HashMap;

pub(crate) async fn list_accounts(
    ctx: &Context,
    input: &crate::svc::ListAccountsInput,
) -> Result<crate::svc::ListAccountsOutput, EndpointError<ListAccountsError>> {
    let _page_size = if input.page_size > 0 {
        input.page_size
    } else {
        default_page_size()
    };

    // TODO Find a way not to hard-code this.
    let projection_fields = [
        "AccountId",
        "Email",
        "FirstName",
        "LastName",
        "Discoverable",
    ];
    let projection_expression = projection_fields.join(",");
    let page_start = if let Some(v) = &input.starting_token {
        const PARSE_ERR_MSG: &str = "Could not parse StartingToken.";
        let v = base64::decode(&v)
            .map_err(|_| EndpointError::BadRequestError(PARSE_ERR_MSG.to_string()))?;
        let v = String::from_utf8(v)
            .map_err(|_| EndpointError::BadRequestError(PARSE_ERR_MSG.to_string()))?;
        let mut hm = HashMap::new();
        hm.insert(
            "Email".to_string(),
            AttributeValue {
                s: Some(v),
                ..AttributeValue::default()
            },
        );
        Some(hm)
    } else {
        None
    };
    let scan_output = ctx
        .dynamodb_client
        .scan(ScanInput {
            limit: Some(input.page_size.into()),
            projection_expression: Some(projection_expression),
            table_name: ctx.accounts_table_name.clone(),
            exclusive_start_key: page_start,
            filter_expression: if input.include_non_discoverable == false {
                Some("Discoverable = :true".to_string())
            } else {
                None
            },
            expression_attribute_values: if input.include_non_discoverable == false {
                let mut hm = HashMap::new();
                hm.insert(
                    ":true".to_string(),
                    AttributeValue {
                        bool: Some(true),
                        ..AttributeValue::default()
                    },
                );
                Some(hm)
            } else {
                None
            },
            ..ScanInput::default()
        })
        .await
        .map_err(|_| EndpointError::InternalError)?;

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
                let account: UserAccount = from_hashmap(item).map_err(|err| {
                    log::error!(
                        "Invalid record in DynamoDB: {}. Original item: {}",
                        err,
                        item_json
                    );
                    EndpointError::InternalError
                })?;
                accounts.push(account);
            }
            accounts
                .into_iter()
                .map(|data| crate::svc::Account {
                    account_id: data.account_id.to_hyphenated().to_string(),
                    email: data.email,
                    first_name: data.first_name,
                    last_name: data.last_name,
                    discoverable: data.discoverable,
                })
                .collect()
        }
    };

    Ok(crate::svc::ListAccountsOutput {
        next_token,
        accounts,
    })
}
