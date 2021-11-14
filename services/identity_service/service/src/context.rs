use core::fmt;
use std::env;

use rusoto_dynamodb::DynamoDb;

pub(crate) enum ContextKey {
    DynamoDbEndpoint,
    AccountsTableName,
}

pub(crate) struct Context {
    pub dynamodb_client: Box<dyn DynamoDb + Send + Sync + 'static>,
    pub accounts_table_name: String,
}

impl fmt::Display for ContextKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Self::DynamoDbEndpoint => write!(f, "DYNAMODB_ENDPOINT"),
            &Self::AccountsTableName => write!(f, "ACCOUNTS_TABLE_NAME"),
        }
    }
}

impl Context {
    pub fn key(key: &ContextKey) -> String {
        env::var(key.to_string().to_owned()).unwrap()
    }
}
