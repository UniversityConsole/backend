mod operations;

extern crate log;
extern crate simple_logger;

use std::{collections::HashMap, env};

use lambda_http::lambda_runtime::{self, Context as LambdaRuntimeContext};
use lambda_http::{handler, http::Method, Body, IntoResponse, Request, Response};
use log::LevelFilter;
use rusoto_core::Region;
use rusoto_dynamodb::{DynamoDb, DynamoDbClient};
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

fn error_response<'a>(message: &'a str, status_code: u16) -> Response<Body> {
    let message_body = {
        let mut b = HashMap::new();
        b.insert("Message", message);
        b
    };

    Response::builder()
        .status(status_code)
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&message_body).unwrap()))
        .unwrap()
}

async fn process_request(
    request: Request,
    _: LambdaRuntimeContext,
) -> Result<impl IntoResponse, Error> {
    let method = request.method();
    if method != Method::POST {
        return Ok(error_response("Expected POST request.", 400));
    }

    let operation = &request.headers().get("X-Uc-Operation");
    if let None = operation {
        return Ok(error_response(
            "Expected operation in \"X-Uc-Operation\" header.",
            400,
        ));
    }
    let operation = operation.unwrap().to_str();
    if let Err(_) = operation {
        return Ok(error_response("Operation must be an ASCII string.", 400));
    }
    let operation = operation.unwrap();

    type Handler = dyn Fn(&Request, &Context) -> dyn std::future::Future<Output = dyn IntoResponse>;

    let executor: Option<Box<Test>> = match operation {
        "CreateAccount" => Some(Box::new(&crate::operations::create_account)),
        "ListAccounts" => Some(Box::new(&crate::operations::list_accounts)),
        _ => None,
    };

    if executor.is_none() {
        return Ok(error_response("Unknown operation.", 400));
    }

    let executor = executor.unwrap();
    let context = Context {
        dynamodb_client: Box::new(DynamoDbClient::new(Region::EuWest1)),
        datastore_name: Context::env_datastore_name(),
    };

    log::debug!("Using DynamoDB table \"{}\".", &context.datastore_name);

    let result = executor(&request, &context).await;
    match result {
        Ok(output) => Ok(output.into_response()),
        Err(err) => Ok(err.into_response()),
    }
}
