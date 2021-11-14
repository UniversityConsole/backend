use core::fmt;
use std::env;

use rusoto_core::Region;
use rusoto_dynamodb::DynamoDb;
use rusoto_dynamodb::DynamoDbClient;

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
    pub fn from_env() -> Self {
        let region = if let Some(endpoint) = Context::key(&ContextKey::DynamoDbEndpoint) {
            let custom_region = Region::Custom {
                name: "custom".to_string(),
                endpoint: endpoint.clone(),
            };
            log::info!("Using DynamoDB with endpoint: {}.", endpoint);
            custom_region
        } else {
            let default_region = Region::default();
            log::info!("Using DynamoDB in region: {}.", default_region.name());
            default_region
        };

        Context {
            dynamodb_client: Box::new(DynamoDbClient::new(region)),
            accounts_table_name: Context::key(&ContextKey::AccountsTableName).unwrap(),
        }
    }

    pub fn key(key: &ContextKey) -> Option<String> {
        env::var(key.to_string().to_owned()).ok()
    }
}
