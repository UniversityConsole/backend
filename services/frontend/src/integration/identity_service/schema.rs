use async_graphql::{Object, ID};
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
