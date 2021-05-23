mod operations;

extern crate log;
extern crate simple_logger;

use std::env;

use lambda_http::lambda_runtime::{self, Context as LambdaRuntimeContext};
use lambda_http::{handler, http::Method, IntoResponse, Request};
use log::LevelFilter;
use rusoto_core::Region;
use rusoto_dynamodb::{DynamoDb, DynamoDbClient};
use service_core::{EndpointError, GenericServiceError};
use simple_logger::SimpleLogger;

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

pub struct Context {
    pub dynamodb_client: Box<dyn DynamoDb + Send + Sync + 'static>,
    pub datastore_name: String,
}

impl Context {
    pub fn env_datastore_name() -> String {
        const VAR: &str = "USER_ACCOUNTS_TABLE_NAME";
        let name = env::var(VAR);

        if let Err(_) = name {
            panic!("Environment variable {} not set.", VAR);
        }

        name.unwrap()
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let debug_enabled = env::var("LAMBDA_DEBUG").is_ok();
    let log_level = if debug_enabled {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    };

    SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .with_module_level(module_path!(), log_level)
        .init()
        .unwrap();

    lambda_runtime::run(handler(process_request)).await?;
    Ok(())
}

async fn process_request(
    request: Request,
    _: LambdaRuntimeContext,
) -> Result<impl IntoResponse, Error> {
    let method = request.method();
    if method != Method::POST {
        return Ok(EndpointError::<GenericServiceError>::BadRequestError(
            "Expected POST request.".to_string(),
        )
        .into_response());
    }

    let operation = &request.headers().get("X-Uc-Operation");
    if let None = operation {
        return Ok(EndpointError::<GenericServiceError>::BadRequestError(
            "Expected operation in \"X-Uc-Operation\" header.".to_string(),
        )
        .into_response());
    }
    let operation = operation.unwrap().to_str();
    if let Err(_) = operation {
        return Ok(EndpointError::<GenericServiceError>::BadRequestError(
            "Operation must be an ANSI string.".to_string(),
        )
        .into_response());
    }
    let operation = operation.unwrap();
    let context = Context {
        dynamodb_client: Box::new(DynamoDbClient::new(Region::EuWest1)),
        datastore_name: Context::env_datastore_name(),
    };

    log::debug!("Using DynamoDB table \"{}\".", &context.datastore_name);

    Ok(match operation {
        "CreateAccount" => {
            match crate::operations::create_account::handler(&request, &context).await {
                Ok(r) => r.into_response(),
                Err(r) => r.into_response(),
            }
        }
        "ListAccounts" => {
            match crate::operations::list_accounts::handler(&request, &context).await {
                Ok(r) => r.into_response(),
                Err(r) => r.into_response(),
            }
        }
        "DescribeAccount" => {
            match crate::operations::describe_account::handler(&request, &context).await {
                Ok(r) => r.into_response(),
                Err(r) => r.into_response(),
            }
        }
        _ => EndpointError::<GenericServiceError>::BadRequestError("Unknown operation".to_string())
            .into_response(),
    })
}
