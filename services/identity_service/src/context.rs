use core::fmt;
use std::env;

use rusoto_core::Region;
use rusoto_dynamodb::DynamoDb;
use rusoto_dynamodb::DynamoDbClient;
use service_core::ddb::Adapter;
use std::str::FromStr;

pub(crate) enum ContextKey {
    DynamoDbEndpoint,
    AccountsTableName,
}

pub(crate) struct Context {
    #[deprecated(note = "use dynamodb_adapter instead")]
    pub dynamodb_client: Box<dyn DynamoDb + Send + Sync + 'static>,
    pub dynamodb_adapter: Adapter,
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
    pub async fn from_env() -> Self {
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

        let shared_config = aws_config::load_from_env().await;

        let dynamodb_config = if let Some(endpoint) = Context::key(&ContextKey::DynamoDbEndpoint) {
            // TODO Handle the error properly.
            let uri = http::Uri::from_str(&endpoint).unwrap();
            aws_sdk_dynamodb::config::Builder::from(&shared_config)
                .endpoint_resolver(aws_sdk_dynamodb::Endpoint::immutable(uri))
                .build()
        } else {
            aws_sdk_dynamodb::config::Config::new(&shared_config)
        };

        let client = aws_sdk_dynamodb::Client::from_conf(dynamodb_config);
        Context {
            dynamodb_client: Box::new(DynamoDbClient::new(region)),
            dynamodb_adapter: client.into(),
            accounts_table_name: Context::key(&ContextKey::AccountsTableName).unwrap(),
        }
    }

    pub fn key(key: &ContextKey) -> Option<String> {
        env::var(key.to_string().to_owned()).ok()
    }
}
