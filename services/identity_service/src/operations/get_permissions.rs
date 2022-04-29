use serde::{Deserialize, Serialize};
use service_core::ddb::get_item::GetItem;
use service_core::ddb::query::Query;
use service_core::endpoint_error::EndpointError;
use service_core::operation_error::OperationError;
use uuid::Uuid;

use crate::svc::{GetPermissionsInput, GetPermissionsOutput};
use crate::user_account::PermissionsDocument;
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
pub enum GetPermissionsError {
    #[error("Account not found.")]
    NotFoundError,
}

pub(crate) async fn get_permissions(
    ctx: &Context,
    ddb: &(impl GetItem + Query),
    input: &GetPermissionsInput,
) -> Result<GetPermissionsOutput, EndpointError<GetPermissionsError>> {
    let account_id = Uuid::parse_str(input.account_id.clone().as_mut())
        .map_err(|_| EndpointError::validation("Invalid account ID provided."))?;
    let permissions_document = get_permissions_from_ddb(&account_id, &ctx.accounts_table_name, ddb)
        .await
        .map_err(|e| match e {
            GetPermissionsFromDdbError::Internal => EndpointError::internal(),
            GetPermissionsFromDdbError::NotFound => EndpointError::operation(GetPermissionsError::NotFoundError),
        })?;
    Ok(GetPermissionsOutput {
        permissions_document: Some(permissions_document.into()),
    })
}

impl OperationError for GetPermissionsError {
    fn code(&self) -> tonic::Code {
        match self {
            Self::NotFoundError => tonic::Code::NotFound,
        }
    }
}
