use std::collections::HashMap;

use lambda_http::{IntoResponse, Request, handler, http::Method};
use lambda_http::lambda_runtime::{self, Context};
use serde::{Serialize, Deserialize};

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

#[tokio::main]
async fn main() -> Result<(), Error> {
    lambda_runtime::run(handler(process_request)).await?;
    Ok(())
}

#[derive(Serialize, Deserialize)]
struct IntegrationResponse {
    cookies: Vec<String>,
    #[serde(rename = "isBase64Encoded")]
    is_base64_encoded: bool,
    body: Option<String>,
    #[serde(rename = "statusCode")]
    status_code: u16,
    headers: HashMap<String, String>,
}

impl Default for IntegrationResponse {
    fn default() -> Self {
        IntegrationResponse {
            cookies: Vec::new(),
            is_base64_encoded: false,
            body: None,
            status_code: 200,
            headers: HashMap::new(),
        }
    }
}

fn error_response<'a>(message: &'a str, status_code: u16) -> String {
    let message_body = {
        let mut b = HashMap::new();
        b.insert("Message", message);
        b
    };

    serde_json::to_string(&IntegrationResponse {
        status_code,
        body: Some(serde_json::to_string(&message_body).unwrap()),
        ..IntegrationResponse::default()
    }).unwrap()
}

async fn process_request(request: Request, _: Context) -> Result<impl IntoResponse, Error> {
    const URI_SCOPE: &str = "/identity-service";

    let method = request.method();
    let uri = &request.uri().path()[URI_SCOPE.len()..];

    match (method, uri) {
        (&Method::GET, "/accounts") => list_accounts(&request).await,
        _ => Ok(error_response("Unknown operation.", 400))
    }
}

async fn list_accounts(_request: &Request) -> Result<String, Error> {
    Ok("ListAccounts".to_string())
}
