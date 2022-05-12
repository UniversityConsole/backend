use identity_service::pb;
use identity_service::pb::{DescribeAccountInput, DescribeAccountOutput};
use serde::{Deserialize, Serialize};
use service_core::endpoint_error::EndpointError;
use service_core::operation_error::OperationError;
use uuid::Uuid;

use crate::user_account::{AccountAttributes, AccountLookup, GetAccountError};
use crate::AccountsRepository;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
#[serde(deny_unknown_fields)]
struct AccountIdIndexProjection {
    account_id: uuid::Uuid,
    email: String,
}

#[non_exhaustive]
#[derive(thiserror::Error, Debug)]
pub enum DescribeAccountError {
    #[error("Account not found.")]
    NotFound,
}

pub(crate) async fn describe_account(
    accounts_repository: &impl AccountsRepository,
    input: &DescribeAccountInput,
) -> Result<DescribeAccountOutput, EndpointError<DescribeAccountError>> {
    let account_id = Uuid::parse_str(input.account_id.clone().as_mut())
        .map_err(|_| EndpointError::validation("Invalid account ID provided."))?;
    let user_account = accounts_repository
        .get_account(&AccountLookup::ById(account_id), &AccountAttributes::Profile)
        .await
        .map_err(|e| match e {
            GetAccountError::NotFound => EndpointError::operation(DescribeAccountError::NotFound),
            _ => {
                log::error!("Failed retrieving account: {:?}.", e);
                EndpointError::internal()
            }
        })?;
    Ok(DescribeAccountOutput {
        account: Some(pb::Account {
            account_id: user_account.account_id.to_hyphenated().to_string(),
            email: user_account.email,
            first_name: user_account.first_name,
            last_name: user_account.last_name,
            discoverable: user_account.discoverable,
        }),
    })
}

impl OperationError for DescribeAccountError {
    fn code(&self) -> tonic::Code {
        match self {
            Self::NotFound => tonic::Code::NotFound,
        }
    }
}
