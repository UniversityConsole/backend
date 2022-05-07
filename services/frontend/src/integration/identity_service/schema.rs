use async_graphql::{Object, SimpleObject, ID};
use thiserror::Error;

#[derive(Clone)]
pub struct UserAccount {
    pub account_id: ID,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
}

#[Object]
impl UserAccount {
    async fn account_id(&self) -> &ID {
        &self.account_id
    }

    async fn email(&self) -> &String {
        &self.email
    }

    async fn first_name(&self) -> &String {
        &self.first_name
    }

    async fn last_name(&self) -> &String {
        &self.last_name
    }
}

#[derive(Clone, SimpleObject)]
pub struct AuthenticationOutput {
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Clone, SimpleObject)]
pub struct GenerateAccessTokenOutput {
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Debug, Error)]
pub enum GraphQLError {
    #[error("Permission denied.")]
    PermissionDenied,

    #[error(transparent)]
    Operation(#[from] Box<dyn std::error::Error + Send + Sync>),
}
