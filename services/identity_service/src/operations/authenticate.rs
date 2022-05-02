use memcache::Client;
use service_core::ddb::get_item::{GetItem, GetItemInput};
use service_core::ddb::query::Query;
use service_core::endpoint_error::EndpointError;
use service_core::operation_error::OperationError;
use thiserror::Error;
use uuid::Uuid;
use zeroize::Zeroize;

use crate::svc::{AuthenticateInput, AuthenticateOutput};
use crate::user_account::{verify_password, UserAccount};
use crate::utils::account::account_key_from_email;
use crate::{Context, MemcacheConnPool};

#[non_exhaustive]
#[derive(Error, Debug)]
pub enum AuthenticateError {
    #[error("Account not found.")]
    AccountNotFound,

    #[error("Provided credentials are invalid.")]
    InvalidCredentials,
}

pub(crate) async fn authenticate(
    ctx: &Context,
    ddb: &(impl GetItem + Query),
    refresh_token_cache: &MemcacheConnPool,
    input: &mut AuthenticateInput,
) -> Result<AuthenticateOutput, EndpointError<AuthenticateError>> {
    let fields = [
        "AccountId",
        "Email",
        "FirstName",
        "Discoverable",
        "LastName",
        "Password",
    ];
    let get_item_input = GetItemInput::builder()
        .table_name(&ctx.accounts_table_name)
        // FIXME Add validation on the email.
        .key(account_key_from_email(input.email.clone()))
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
        .ok_or_else(|| EndpointError::operation(AuthenticateError::AccountNotFound))?;

    let mut user_account: UserAccount = serde_ddb::from_hashmap(user_account).map_err(|parse_err| {
        log::error!("Decoding item from datastore failed: {:?}", parse_err);
        EndpointError::internal()
    })?;

    let pass_verify_result = verify_password(&input.password, &user_account.password);
    user_account.password.zeroize();

    use argon2::password_hash::Error::Password as PasswordErr;
    pass_verify_result.map_err(|e| match e {
        PasswordErr => EndpointError::operation(AuthenticateError::InvalidCredentials),
        _ => {
            log::error!("Password verification failed: {:?}", e);
            EndpointError::internal()
        }
    })?;

    let refresh_token = create_refresh_token(&ctx, &refresh_token_cache, &user_account.account_id);
    let access_token = create_access_token(&ctx, user_account).map_err(|e| {
        log::error!("Failed encoding the JWT access token: {:?}", e);
        EndpointError::internal()
    })?;

    Ok(AuthenticateOutput {
        access_token,
        refresh_token: refresh_token.to_hyphenated().to_string(),
    })
}

impl OperationError for AuthenticateError {
    fn code(&self) -> tonic::Code {
        match self {
            Self::AccountNotFound => tonic::Code::NotFound,
            Self::InvalidCredentials => tonic::Code::InvalidArgument,
        }
    }
}

fn create_access_token(ctx: &Context, user_account: UserAccount) -> jsonwebtoken::errors::Result<String> {
    use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
    use service_core::auth::jwt::Claims;

    let claims = Claims {
        sub: user_account.account_id.to_hyphenated().to_string(),
        email: user_account.email,
        first_name: user_account.first_name,
        last_name: user_account.last_name,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_base64_secret(ctx.access_token_secret.as_ref())?,
    )
}

fn create_refresh_token(ctx: &Context, refresh_token_cache: &MemcacheConnPool, account_id: &Uuid) -> Uuid {
    let client = Client::with_pool(refresh_token_cache.clone()).unwrap();
    let token = Uuid::new_v4();
    let ttl = chrono::Duration::hours(10).num_seconds();
    client
        .set(token.to_string().as_str(), account_id.as_bytes().as_slice(), ttl as u32)
        .unwrap();

    token
}
