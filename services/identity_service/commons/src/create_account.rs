use lambda_http::{Body, IntoResponse, Request, Response};
use serde::{Deserialize, Serialize};
use serde_json::json;
use simple_error::SimpleError;
use std::convert::TryFrom;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
#[serde(deny_unknown_fields)]
pub struct CreateAccountInput {
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub gov_id: String,
    pub password: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
#[serde(deny_unknown_fields)]
pub struct CreateAccountOutput {
    pub account_id: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
#[serde(deny_unknown_fields)]
pub enum CreateAccountError {
    BadRequest,
    DuplicateAccount,
    InternalError,
}

impl<'a> TryFrom<&'a Request> for CreateAccountInput {
    type Error = SimpleError;

    fn try_from(req: &'a Request) -> Result<Self, Self::Error> {
        match req.body() {
            Body::Empty => Err(SimpleError::new("Unexpected empty request body.")),
            Body::Binary(_) => Err(SimpleError::new("Unexpected binary input.")),
            Body::Text(data) => serde_json::from_str(&data)
                .map_err(|_| SimpleError::new("Failed to parse CreateAccountInput")),
        }
    }
}

impl IntoResponse for CreateAccountOutput {
    fn into_response(self) -> Response<Body> {
        let body = serde_json::to_string(&self).unwrap();
        Response::builder()
            .status(200)
            .header("Content-Type", "application/json")
            .body(Body::Text(body))
            .unwrap()
    }
}

impl IntoResponse for CreateAccountError {
    fn into_response(self) -> Response<Body> {
        let body = json!({ "Message": self }).to_string();
        let status_code = match self {
            CreateAccountError::BadRequest => 400,
            CreateAccountError::DuplicateAccount => 400,
            CreateAccountError::InternalError => 500,
        };
        Response::builder()
            .status(status_code)
            .header("Content-Type", "application/json")
            .body(Body::from(body))
            .unwrap()
    }
}
