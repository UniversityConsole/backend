use service_core::ddb::get_item::GetItem;
use service_core::ddb::query::Query;
use service_core::endpoint_error::EndpointError;
use service_core::operation_error::OperationError;
use thiserror::Error;
use uuid::Uuid;

use crate::svc::{AuthorizeInput, AuthorizeOutput};
use crate::utils::permissions::{get_permissions_from_ddb, GetPermissionsFromDdbError};
use crate::utils::validation::validate_resource_paths;
use crate::Context;

#[non_exhaustive]
#[derive(Error, Debug)]
pub enum AuthorizeError {
    #[error("Account not found.")]
    NotFound,

    #[error("Resource path {1} in statement {0} is invalid.")]
    InvalidResourcePath(usize, usize),
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
            GetPermissionsFromDdbError::AccountNotFound(_) => EndpointError::operation(AuthorizeError::NotFound),
            _ => EndpointError::internal(),
        })?;

    validate_resource_paths(&permissions_document.statements).map_err(|(stmt_idx, path_idx)| {
        EndpointError::operation(AuthorizeError::InvalidResourcePath(stmt_idx, path_idx))
    })?;

    Ok(AuthorizeOutput {
        permission_granted: false,
    })
}

impl OperationError for AuthorizeError {
    fn code(&self) -> tonic::Code {
        match self {
            Self::NotFound => tonic::Code::NotFound,
            Self::InvalidResourcePath(..) => tonic::Code::InvalidArgument,
        }
    }
}
