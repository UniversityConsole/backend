use identity_service::pb::{CreateAccountInput, CreateAccountOutput};
use service_core::endpoint_error::EndpointError;
use service_core::operation_error::OperationError;
use zeroize::Zeroize;

use crate::user_account::{hash_password, repository, UserAccount};
use crate::AccountsRepository;

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum CreateAccountError {
    #[error("An account with this email already exists.")]
    DuplicateAccount,
}

pub(crate) async fn create_account(
    accounts_repository: &impl AccountsRepository,
    mut input: CreateAccountInput,
) -> Result<CreateAccountOutput, EndpointError<CreateAccountError>> {
    let account_attributes = input
        .account_attributes
        .as_mut()
        .ok_or_else(|| EndpointError::validation("Account attributes missing."))?;

    let password = hash_password(&account_attributes.password).map_err(|e| {
        log::error!("Hashing password failed: {:?}", e);
        EndpointError::internal()
    })?;
    account_attributes.password.zeroize();

    let account = UserAccount::builder()
        .email(&account_attributes.email)
        .first_name(&account_attributes.first_name)
        .last_name(&account_attributes.last_name)
        .password(password)
        .discoverable(account_attributes.discoverable)
        .build();

    accounts_repository
        .create_account(&account)
        .await
        .map_err(|err| match err {
            repository::CreateAccountError::DuplicateAccount => {
                EndpointError::operation(CreateAccountError::DuplicateAccount)
            }
            _ => {
                log::error!("Create account failed: {:?}", err);
                EndpointError::Internal
            }
        })?;

    Ok(CreateAccountOutput {
        account_id: account.account_id.to_string(),
    })
}

impl OperationError for CreateAccountError {
    fn code(&self) -> tonic::Code {
        match self {
            CreateAccountError::DuplicateAccount => tonic::Code::AlreadyExists,
        }
    }
}
