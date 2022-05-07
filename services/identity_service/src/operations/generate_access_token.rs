
use memcache::Client;
use service_core::ddb::get_item::{GetItem, GetItemInput};
use service_core::ddb::query::Query;
use service_core::endpoint_error::EndpointError;
use service_core::operation_error::OperationError;
use thiserror::Error;
use uuid::Uuid;



use crate::operations::authenticate::{create_access_token, create_refresh_token};
use crate::svc::{GenerateAccessTokenInput, GenerateAccessTokenOutput};
use crate::user_account::{UserAccount};
use crate::utils::account::{account_key_from_id, AccountKeyFromIdError};
use crate::{Context, MemcacheConnPool};

#[non_exhaustive]
#[derive(Error, Debug)]
pub enum GenerateAccessTokenError {
    #[error("Permission denied.")]
    PermissionDenied,

    #[error("Account not found.")]
    AccountNotFound,
}

pub(crate) async fn generate_access_token(
    ctx: &Context,
    ddb: &(impl GetItem + Query),
    refresh_token_cache: &MemcacheConnPool,
    input: &mut GenerateAccessTokenInput,
) -> Result<GenerateAccessTokenOutput, EndpointError<GenerateAccessTokenError>> {
    let account_id =
        Uuid::parse_str(input.account_id.as_ref()).map_err(|_| EndpointError::validation("Invalid account ID"))?;

    let client = Client::with_pool(refresh_token_cache.clone()).unwrap();
    let token_owner: Vec<u8> = client
        .get(input.refresh_token.as_ref())
        .map_err(|e| {
            log::error!("Memcache GET failed: {:?}", e);
            EndpointError::internal()
        })?
        .ok_or_else(|| EndpointError::operation(GenerateAccessTokenError::PermissionDenied))?;
    client.delete(input.refresh_token.as_ref()).map_err(|e| {
        log::error!("Memcache DELETE failed: {:?}", e);
        EndpointError::internal()
    })?;
    if token_owner.as_slice() != account_id.as_bytes().as_slice() {
        return Err(EndpointError::operation(GenerateAccessTokenError::PermissionDenied));
    }

    let fields = ["AccountId", "Email", "FirstName", "Discoverable", "LastName"];
    let key = account_key_from_id(ddb, ctx.accounts_table_name.as_ref(), &account_id)
        .await
        .map_err(|e| match e {
            AccountKeyFromIdError::AccountNotFound => {
                EndpointError::operation(GenerateAccessTokenError::AccountNotFound)
            }
            AccountKeyFromIdError::Datastore(_) => EndpointError::internal(),
        })?;
    let get_item_input = GetItemInput::builder()
        .table_name(&ctx.accounts_table_name)
        .key(key)
        .consistent_read(true)
        .projection_expression(fields.join(","))
        .build();
    let user_account = ddb
        .get_item(get_item_input)
        .await
        .map_err(|e| {
            log::error!("Failed to get item from DynamoDB. Original error: {:?}.", &e);
            EndpointError::internal()
        })?
        .item
        .ok_or_else(|| EndpointError::operation(GenerateAccessTokenError::AccountNotFound))?;

    let user_account: UserAccount = serde_ddb::from_hashmap(user_account).map_err(|parse_err| {
        log::error!("Decoding item from datastore failed: {:?}", parse_err);
        EndpointError::internal()
    })?;

    let refresh_token = create_refresh_token(refresh_token_cache, &user_account.account_id);
    let access_token = create_access_token(ctx, user_account).map_err(|e| {
        log::error!("Failed encoding the JWT access token: {:?}", e);
        EndpointError::internal()
    })?;


    Ok(GenerateAccessTokenOutput {
        access_token,
        refresh_token: refresh_token.to_hyphenated().to_string(),
    })
}

impl OperationError for GenerateAccessTokenError {
    fn code(&self) -> tonic::Code {
        match self {
            Self::PermissionDenied => tonic::Code::PermissionDenied,
            Self::AccountNotFound => tonic::Code::NotFound,
        }
    }
}
