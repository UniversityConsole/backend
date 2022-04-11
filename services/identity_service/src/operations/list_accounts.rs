use crate::user_account::UserAccount;
use crate::Context;
use aws_sdk_dynamodb::model::AttributeValue;
use base64;
use common_macros::hash_map;
use serde_ddb::from_hashmap;
use service_core::ddb::scan::{Scan, ScanInput};
use service_core::endpoint_error::EndpointError;

pub(crate) async fn list_accounts(
    ctx: &Context,
    ddb: &impl Scan,
    input: &crate::svc::ListAccountsInput,
) -> Result<crate::svc::ListAccountsOutput, EndpointError<!>> {
    let page_size = if input.page_size > 0 {
        input.page_size
    } else {
        32
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
        let v = base64::decode(&v).map_err(|_| EndpointError::validation(PARSE_ERR_MSG))?;
        let v = String::from_utf8(v).map_err(|_| EndpointError::validation(PARSE_ERR_MSG))?;
        Some(hash_map! {
            "Email".to_owned() => AttributeValue::S(v),
        })
    } else {
        None
    };

    log::debug!(
        "page_start = {:?} projection_expression = {:?} page_size = {}",
        &page_start,
        &projection_expression,
        &page_size,
    );

    let scan_input = ScanInput::builder()
        .table_name(ctx.accounts_table_name.clone())
        .limit(page_size as i32)
        .projection_expression(projection_expression)
        .exclusive_start_key(page_start)
        .filter_expression(if input.include_non_discoverable == false {
            Some("Discoverable = :true".to_string())
        } else {
            None
        })
        .expression_attribute_values(if input.include_non_discoverable == false {
            Some(hash_map! {
                ":true".to_owned() => AttributeValue::Bool(true),
            })
        } else {
            None
        })
        .build();

    let scan_output = ddb.scan(scan_input).await.map_err(|e| {
        log::error!("scan failed, error: {:?}", e);
        EndpointError::internal()
    })?;

    let next_token = match scan_output.last_evaluated_key {
        None => None,
        Some(hm) => {
            let email_value = hm.get(&"Email".to_string()).unwrap();
            if let AttributeValue::S(email_value) = email_value {
                Some(base64::encode(email_value))
            } else {
                unreachable!();
            }
        }
    };

    let accounts = match scan_output.items {
        None => vec![],
        Some(items) => {
            let mut accounts = Vec::with_capacity(scan_output.count as usize);
            for item in items.into_iter() {
                let account: UserAccount = from_hashmap(item).map_err(|err| {
                    log::error!("Invalid record in DynamoDB: {:?}.", err);
                    EndpointError::internal()
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

    log::debug!("Got accounts: {:?}", &accounts);

    Ok(crate::svc::ListAccountsOutput {
        next_token,
        accounts,
    })
}
