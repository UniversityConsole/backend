use identity_service::pb::conversion::AccessRequestParseError;
use identity_service::pb::{AuthorizeInput, AuthorizeOutput};
use service_core::ddb::get_item::GetItem;
use service_core::ddb::query::Query;
use service_core::endpoint_error::EndpointError;
use service_core::operation_error::OperationError;
use service_core::resource_access::types::Superset;
use service_core::resource_access::AccessRequest;
use thiserror::Error;
use uuid::Uuid;

use crate::operations::authorize::AuthorizeError::InvalidResourcePath;
use crate::user_account::PermissionsDocument;
use crate::utils::permissions::{
    get_access_path_set, get_permissions_from_ddb, merge_access_request_paths, GetPermissionsFromDdbError,
};
use crate::Context;

#[non_exhaustive]
#[derive(Error, Debug)]
pub enum AuthorizeError {
    #[error("Account not found.")]
    NotFound,

    #[error("Resource path at index {0} is invalid: {1}.")]
    InvalidResourcePath(usize, String),
}

pub(crate) async fn authorize(
    ctx: &Context,
    ddb: &(impl GetItem + Query),
    input: &AuthorizeInput,
) -> Result<AuthorizeOutput, EndpointError<AuthorizeError>> {
    let account_id = input
        .account_id
        .as_ref()
        .map(|account_id| Uuid::parse_str(account_id.clone().as_ref()))
        .transpose()
        .map_err(|_| EndpointError::validation("Invalid account ID provided."))?;
    let permissions_document = if let Some(account_id) = &account_id {
        get_permissions_from_ddb(ddb, ctx.accounts_table_name.as_ref(), &account_id)
            .await
            .map_err(|e| match e {
                GetPermissionsFromDdbError::AccountNotFound => EndpointError::operation(AuthorizeError::NotFound),
                _ => EndpointError::internal(),
            })?
    } else {
        PermissionsDocument::default()
    };

    let access_request: AccessRequest = input.access_request.clone().unwrap().try_into().map_err(|e| match e {
        AccessRequestParseError::CompileError(idx, path) => EndpointError::operation(InvalidResourcePath(idx, path)),
        AccessRequestParseError::MultiRootPath(idx, path) => EndpointError::operation(InvalidResourcePath(idx, path)),
    })?;

    let allowed_paths =
        get_access_path_set(&permissions_document, access_request.kind, account_id.is_some()).map_err(|err| {
            if let Some(account_id) = &account_id {
                let (invalid_path, stmt_idx, path_idx) = err;
                log::error!(
                    "Invalid path in permissions document for account {} (statement: {}, path: {}): {}.",
                    account_id.to_hyphenated(),
                    stmt_idx,
                    path_idx,
                    invalid_path,
                );
            }
            EndpointError::internal()
        })?;
    let desired_paths = merge_access_request_paths(access_request);

    log::info!(
        "Allowed paths: {:?}. Desired paths: {:?}.",
        &allowed_paths,
        &desired_paths
    );

    let permission_granted = allowed_paths.is_superset_of(&desired_paths);

    Ok(AuthorizeOutput { permission_granted })
}

impl OperationError for AuthorizeError {
    fn code(&self) -> tonic::Code {
        match self {
            Self::NotFound => tonic::Code::NotFound,
            InvalidResourcePath(..) => tonic::Code::InvalidArgument,
        }
    }
}
