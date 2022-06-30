use std::collections::HashMap;
use std::error::Error;

use aws_sdk_dynamodb::model::{AttributeValue, Select};
use common_macros::hash_map;
use serde::{Deserialize, Serialize};
use service_core::ddb::query::{Query, QueryInput};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum AccountKeyFromIdError {
    #[error("Account not found.")]
    AccountNotFound,

    #[error("Underlying datastore error: {0}")]
    Datastore(Box<dyn Error>),
}


#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct AccountIdIndexProjection {
    account_id: Uuid,
    email: String,
}


pub async fn account_key_from_id(
    ddb: &impl Query,
    table_name: &str,
    account_id: &Uuid,
) -> Result<HashMap<String, AttributeValue>, AccountKeyFromIdError> {
    let query_params = hash_map! {
        ":uuid".to_string() => AttributeValue::S(account_id.to_hyphenated().to_string()),
    };

    let query_input = QueryInput::builder()
        .index_name("AccountIdIndex")
        .table_name(table_name)
        .key_condition_expression("AccountId = :uuid")
        .select(Select::AllProjectedAttributes)
        .expression_attribute_values(Some(query_params))
        .limit(1)
        .build();
    let output = ddb.query(query_input).await.map_err(|e| {
        log::error!("Failed to query DynamoDB. Original error: {:?}.", &e);
        AccountKeyFromIdError::Datastore(e.into())
    })?;

    let item = output
        .items
        .ok_or_else(|| AccountKeyFromIdError::Datastore("Malformed reply: missing items".into()))?
        .pop()
        .ok_or(AccountKeyFromIdError::AccountNotFound)?;
    let projection: AccountIdIndexProjection =
        serde_ddb::from_hashmap(item).map_err(|e| AccountKeyFromIdError::Datastore(e.into()))?;
    Ok(account_key_from_email(projection.email))
}

/// Given an email address, creates the map to be used as key to the User Accounts datastore.
pub fn account_key_from_email(email: String) -> HashMap<String, AttributeValue> {
    hash_map! {
        "Email".to_string() => AttributeValue::S(email),
    }
}
