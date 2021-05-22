use crate::dataplane::UserAccount;
use lambda_http::{Body, IntoResponse, Request, Response};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use service_core::{HttpError, HttpStatus};
use simple_error::SimpleError;
use std::{convert::TryFrom, error::Error, fmt::Display};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
#[serde(deny_unknown_fields)]
pub struct CreateAccountInput {
    pub account: UserAccount,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
#[serde(deny_unknown_fields)]
pub struct CreateAccountOutput {
    pub account_id: Uuid,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
#[serde(deny_unknown_fields)]
#[serde(tag = "ErrorKind", content = "Message")]
pub enum CreateAccountError {
    DuplicateAccountError,
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
        let body = serde_json::to_string(&self).unwrap();
        Response::builder()
            .status(self.status_code())
            .header("Content-Type", "application/json")
            .body(Body::from(body))
            .unwrap()
    }
}

impl HttpStatus for CreateAccountError {
    fn status_code(&self) -> StatusCode {
        match self {
            CreateAccountError::DuplicateAccountError => StatusCode::BAD_REQUEST,
        }
    }
}

impl Display for CreateAccountError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            CreateAccountError::DuplicateAccountError => {
                "An account with this email already exists."
            }
        };

        write!(f, "{}", msg)
    }
}

impl Error for CreateAccountError {}
impl HttpError for CreateAccountError {}
