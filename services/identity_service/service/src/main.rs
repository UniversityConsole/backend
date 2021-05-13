extern crate simple_logger;
extern crate log;

use std::{collections::HashMap, env};

use lambda_http::{IntoResponse, Request, handler, http::Method, Response};
use lambda_http::lambda_runtime::{self, Context};
use simple_logger::SimpleLogger;
use log::LevelFilter;

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let debug_enabled = env::var("LAMBDA_DEBUG").is_ok();
    let log_level = if debug_enabled { LevelFilter::Debug } else { LevelFilter::Info };

    SimpleLogger::new()
        .with_module_level(module_path!(), log_level)
        .init()
        .unwrap();

    lambda_runtime::run(handler(process_request)).await?;
    Ok(())
}

fn error_response<'a>(message: &'a str, status_code: u16) -> Response<String> {
    let message_body = {
        let mut b = HashMap::new();
        b.insert("Message", message);
        b
    };

    Response::builder()
        .status(status_code)
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&message_body).unwrap())
        .unwrap()
}

async fn process_request(request: Request, _: Context) -> Result<impl IntoResponse, Error> {
    const URI_SCOPE: &str = "/identity-service";

    let method = request.method();
    let uri = &request.uri().path()[URI_SCOPE.len()..];

    log::debug!("Got raw URI: {}", &request.uri());
    log::debug!("Processed URI: {}", &uri);
    log::debug!("Got method: {}", &method);

    match (method, uri) {
        (&Method::GET, "/accounts") => list_accounts(&request).await,
        _ => Ok(error_response("Unknown operation.", 400))
    }
}

async fn list_accounts(_request: &Request) -> Result<Response<String>, Error> {
    let body = {
        let mut b = HashMap::new();
        b.insert("Operation", "ListAccounts");
        b
    };

    Ok(Response::builder()
        .status(200)
        .body(serde_json::to_string(&body).unwrap())
        .unwrap())
}
