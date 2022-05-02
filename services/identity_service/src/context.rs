use core::fmt;
use std::env;
use std::str::FromStr;

use service_core::ddb::Adapter;

pub(crate) enum ContextKey {
    DynamoDbEndpoint,
    AccountsTableName,
    AccessTokenSecret,
    RefreshTokenSecret,
}

pub(crate) struct Context {
    pub dynamodb_adapter: Adapter,
    pub accounts_table_name: String,
    pub access_token_secret: String,
    pub refresh_token_secret: String,
}

impl fmt::Display for ContextKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::DynamoDbEndpoint => write!(f, "DYNAMODB_ENDPOINT"),
            Self::AccountsTableName => write!(f, "ACCOUNTS_TABLE_NAME"),
            Self::AccessTokenSecret => write!(f, "ACCESS_TOKEN_SECRET"),
            Self::RefreshTokenSecret => write!(f, "REFRESH_TOKEN_SECRET"),
        }
    }
}

impl Context {
    pub async fn from_env() -> Self {
        let shared_config = aws_config::load_from_env().await;
        let dynamodb_config = if let Some(endpoint) = Context::key(&ContextKey::DynamoDbEndpoint) {
            // TODO Handle the error properly.
            let uri = http::Uri::from_str(&endpoint).unwrap();
            log::info!("Using DynamoDB at {}.", &uri);
            aws_sdk_dynamodb::config::Builder::from(&shared_config)
                .endpoint_resolver(aws_sdk_dynamodb::Endpoint::immutable(uri))
                .build()
        } else {
            log::info!("Using default DynamoDB.");
            aws_sdk_dynamodb::config::Config::new(&shared_config)
        };

        let client = aws_sdk_dynamodb::Client::from_conf(dynamodb_config);
        Context {
            dynamodb_adapter: client.into(),
            accounts_table_name: Context::key(&ContextKey::AccountsTableName).unwrap(),
            access_token_secret: Context::key(&ContextKey::AccessTokenSecret).unwrap(),
            refresh_token_secret: Context::key(&ContextKey::RefreshTokenSecret).unwrap(),
        }
    }

    pub fn key(key: &ContextKey) -> Option<String> {
        env::var(&key.to_string()).ok()
    }
}
