use std::error::Error;

use serde::{Deserialize, Serialize};
use service_core::ddb::get_item::{GetItem, GetItemInput};
use service_core::ddb::query::Query;
use thiserror::Error;
use uuid::Uuid;

use crate::user_account::PermissionsDocument;
use crate::utils::account::{account_key_from_id, AccountKeyFromIdError};

#[derive(Error, Debug)]
pub enum GetPermissionsFromDdbError<'a> {
    #[error("Account {0} not found.")]
    AccountNotFound(&'a Uuid),

    #[error("Underlying datastore error: {0}.")]
    Datastore(Box<dyn Error>),

    #[error("Unknown.")]
    Unknown,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
#[serde(deny_unknown_fields)]
struct AccountIdIndexProjection {
    account_id: Uuid,
    email: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "PascalCase")]
struct PermissionsDocumentItem {
    permissions_document: PermissionsDocument,
}


pub async fn get_permissions_from_ddb<'a>(
    ddb: &(impl GetItem + Query),
    table_name: &str,
    account_id: &'a Uuid,
) -> Result<PermissionsDocument, GetPermissionsFromDdbError<'a>> {
    let key = account_key_from_id(ddb, table_name, &account_id)
        .await
        .map_err(|e| match e {
            AccountKeyFromIdError::AccountNotFound => GetPermissionsFromDdbError::AccountNotFound(&account_id),
            _ => GetPermissionsFromDdbError::Datastore(e.into()),
        })?;
    let get_item_input = GetItemInput::builder()
        .table_name(table_name)
        .projection_expression("PermissionsDocument")
        .key(key)
        .build();
    let output = ddb.get_item(get_item_input).await.map_err(|e| {
        log::error!("Failed to get item from DynamoDB. Original error: {:?}.", &e);
        GetPermissionsFromDdbError::Datastore(e.into())
    })?;

    match output.item {
        Some(item) => {
            let item: PermissionsDocumentItem = serde_ddb::from_hashmap(item).map_err(|e| {
                log::error!("Invalid record in DynamoDB. Original error: {:?}.", &e);
                GetPermissionsFromDdbError::Datastore(e.into())
            })?;

            Ok(item.permissions_document)
        }
        None => {
            log::warn!(
                "Item found on Query, but not found on GetItem. Queried AccountId: {}",
                account_id.to_hyphenated().to_string()
            );
            Err(GetPermissionsFromDdbError::AccountNotFound(account_id))
        }
    }
}
