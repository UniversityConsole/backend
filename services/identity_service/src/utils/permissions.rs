use std::error::Error;

use serde::{Deserialize, Serialize};
use service_core::ddb::get_item::{GetItem, GetItemInput};
use service_core::ddb::query::Query;
use service_core::resource_access::string_interop::compiler::from_string;
use service_core::resource_access::types::PathSet;
use service_core::resource_access::{AccessKind, AccessRequest};
use thiserror::Error;
use uuid::Uuid;

use crate::user_account::PermissionsDocument;
use crate::utils::account::{account_key_from_id, AccountKeyFromIdError};

#[derive(Error, Debug)]
pub enum GetPermissionsFromDdbError {
    /// The account does not exist.
    #[error("Account not found.")]
    AccountNotFound,

    /// There was an error when communicating to the DynamoDB table.
    #[error("Underlying datastore error: {0}.")]
    Datastore(Box<dyn Error>),
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


/// Get the permissions document for the given account ID from the DynamoDB table.
pub async fn get_permissions_from_ddb(
    ddb: &(impl GetItem + Query),
    table_name: &str,
    account_id: &Uuid,
) -> Result<PermissionsDocument, GetPermissionsFromDdbError> {
    let key = account_key_from_id(ddb, table_name, &account_id)
        .await
        .map_err(|e| match e {
            AccountKeyFromIdError::AccountNotFound => GetPermissionsFromDdbError::AccountNotFound,
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
            Err(GetPermissionsFromDdbError::AccountNotFound)
        }
    }
}


/// Computes a single path set from the given permissions document. This function skips any statement
/// in the permissions document that does not match the desired access kind.
///
/// # Arguments
///
/// * `permissions_document` - the permissions document to be used.
/// * `access_kind` - the desired access kind. The statements in the permissions document will be
/// processed only if they match this.
///
/// # Returns
///
/// On success, returns the computed path set. On failure, returns a tuple of statement index and
/// path index (within that statement) indicating which path failed parsing.
pub fn get_access_path_set(
    permissions_document: &PermissionsDocument,
    access_kind: AccessKind,
) -> Result<PathSet, (&String, usize, usize)> {
    let mut path_set = PathSet::default();
    for (stmt_idx, stmt) in permissions_document.statements.iter().enumerate() {
        if stmt.access_kind != access_kind {
            continue;
        }

        for (path_idx, raw) in stmt.paths.iter().enumerate() {
            let curr_path_set = from_string(raw.as_ref()).map_err(|e| {
                log::error!("Invalid resource path in document: {}. Error: {:?}", &raw, e);
                (raw, stmt_idx, path_idx)
            })?;

            path_set.merge_path_set(curr_path_set);
        }
    }

    Ok(path_set)
}


/// Merges all resource paths in the given access request into a single path set.
///
/// # Arguments
///
/// * `access_request` - the access request as received by the __Authorize__ operation.
///
/// # Returns
///
/// If all resource paths are valid, returns the computed path set. Otherwise, returns a tuple
/// made of a single element: the index of the invalid path set.
pub fn merge_access_request_paths(access_request: AccessRequest) -> PathSet {
    let mut path_set = PathSet::default();

    for raw in access_request.paths.into_iter() {
        path_set.merge_path_node(raw);
    }

    path_set
}
