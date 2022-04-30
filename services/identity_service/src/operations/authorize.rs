use service_core::ddb::get_item::GetItem;
use service_core::ddb::query::Query;
use service_core::endpoint_error::EndpointError;
use service_core::operation_error::OperationError;
use service_core::resource_access::types::Superset;
use service_core::resource_access::AccessRequest;
use thiserror::Error;
use uuid::Uuid;

use crate::operations::authorize::AuthorizeError::InvalidResourcePath;
use crate::svc::{AccessRequestParseError, AuthorizeInput, AuthorizeOutput};
use crate::utils::permissions::{
    get_access_path_set, get_permissions_from_ddb, merge_access_request_paths, GetPermissionsFromDdbError,
};
use crate::Context;

#[non_exhaustive]
#[derive(Error, Debug)]
pub enum AuthorizeError {
    #[error("Account not found.")]
    NotFound,

    #[error("Resource path {0} is invalid.")]
    InvalidResourcePath(usize),
}

pub(crate) async fn authorize(
    ctx: &Context,
    ddb: &(impl GetItem + Query),
    input: &AuthorizeInput,
) -> Result<AuthorizeOutput, EndpointError<AuthorizeError>> {
    let account_id = Uuid::parse_str(input.account_id.clone().as_mut())
        .map_err(|_| EndpointError::validation("Invalid account ID provided."))?;
    let permissions_document = get_permissions_from_ddb(ddb, ctx.accounts_table_name.as_ref(), &account_id)
        .await
        .map_err(|e| match e {
            GetPermissionsFromDdbError::AccountNotFound => EndpointError::operation(AuthorizeError::NotFound),
            _ => EndpointError::internal(),
        })?;
    let access_request: AccessRequest = input.access_request.clone().unwrap().try_into().map_err(|e| match e {
        AccessRequestParseError::CompileError(idx) => EndpointError::operation(InvalidResourcePath(idx)),
        AccessRequestParseError::MultiRootPath(idx) => EndpointError::operation(InvalidResourcePath(idx)),
    })?;

    let allowed_paths = get_access_path_set(&permissions_document, access_request.kind).map_err(|err| {
        let (invalid_path, stmt_idx, path_idx) = err;
        log::error!(
            "Invalid path in permissions document for account {} (statement: {}, path: {}): {}.",
            account_id.to_hyphenated(),
            stmt_idx,
            path_idx,
            invalid_path,
        );
        EndpointError::internal()
    })?;
    let desired_paths = merge_access_request_paths(access_request);

    Ok(AuthorizeOutput {
        permission_granted: allowed_paths.is_superset_of(&desired_paths),
    })
}

impl OperationError for AuthorizeError {
    fn code(&self) -> tonic::Code {
        match self {
            Self::NotFound => tonic::Code::NotFound,
            InvalidResourcePath(..) => tonic::Code::InvalidArgument,
        }
    }
}
