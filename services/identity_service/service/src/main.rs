mod data_plane;
mod operations;

extern crate log;
extern crate simple_logger;

use std::{collections::HashMap, env};

use lambda_http::lambda_runtime::{self, Context};
use lambda_http::{handler, http::Method, Body, IntoResponse, Request, Response};
use log::LevelFilter;
use simple_logger::SimpleLogger;

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

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

async fn process_request(request: Request, _: Context) -> Result<impl IntoResponse, Error> {
    const URI_SCOPE: &str = "/identity-service";

    let method = request.method();
    let uri = &request.uri().path()[URI_SCOPE.len()..];

    let executor = match (method, uri) {
        (&Method::POST, "/accounts") => Some(&crate::operations::create_account),
        _ => None,
    };

    if executor.is_none() {
        return Ok(error_response("Unknown operation.", 400));
    }

    let executor = executor.unwrap();
    let result = executor(&request).await;
    match result {
        Ok(output) => Ok(output.into_response()),
        Err(err) => Ok(err.into_response()),
    }
}
