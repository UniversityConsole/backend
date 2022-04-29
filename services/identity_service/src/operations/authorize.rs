use serde::{Deserialize, Serialize};
use service_core::ddb::get_item::GetItem;
use service_core::ddb::query::Query;
use service_core::endpoint_error::EndpointError;
use service_core::operation_error::OperationError;
use uuid::Uuid;

use crate::svc::{AuthorizeInput, AuthorizeOutput};
use crate::user_account::PermissionsDocument;
use crate::utils::validation::validate_resource_paths;
use crate::utils::{get_permissions_from_ddb, GetPermissionsFromDdbError};
use crate::Context;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
#[serde(deny_unknown_fields)]
struct AccountIdIndexProjection {
    account_id: uuid::Uuid,
    email: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "PascalCase")]
struct PermissionsDocumentItem {
    permissions_document: PermissionsDocument,
}

#[non_exhaustive]
#[derive(thiserror::Error, Debug)]
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
    let permissions_document = get_permissions_from_ddb(&account_id, &ctx.accounts_table_name, ddb)
        .await
        .map_err(|e| match e {
            GetPermissionsFromDdbError::Internal => EndpointError::internal(),
            GetPermissionsFromDdbError::NotFound => EndpointError::operation(AuthorizeError::NotFound),
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
