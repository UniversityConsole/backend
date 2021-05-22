use crate::dataplane::UserAccount;
use lambda_http::{Body, IntoResponse, Request, Response};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use service_core::{HttpError, HttpStatus};
use simple_error::SimpleError;
use std::{convert::TryFrom, fmt::Display};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
#[serde(deny_unknown_fields)]
pub struct DescribeAccountInput {
    pub account_id: uuid::Uuid,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
#[serde(deny_unknown_fields)]
pub struct DescribeAccountOutput {
    pub account: UserAccount,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "ErrorKind", content = "Message")]
pub enum DescribeAccountError {
    NotFoundError,
}

impl<'a> TryFrom<&'a Request> for DescribeAccountInput {
    type Error = SimpleError;

    fn try_from(req: &'a Request) -> Result<Self, Self::Error> {
        match req.body() {
            Body::Empty => Err(SimpleError::new("Unexpected empty request body.")),
            Body::Binary(_) => Err(SimpleError::new("Unexpected binary input.")),
            Body::Text(data) => serde_json::from_str(&data)
                .map_err(|_| SimpleError::new("Failed to parse DescribeAccountInput")),
        }
    }
}

impl IntoResponse for DescribeAccountOutput {
    fn into_response(self) -> Response<Body> {
        let body = serde_json::to_string(&self).unwrap();
        Response::builder()
            .status(200)
            .header("Content-Type", "application/json")
            .body(Body::Text(body))
            .unwrap()
    }
}

impl IntoResponse for DescribeAccountError {
    fn into_response(self) -> Response<Body> {
        Response::builder()
            .status(self.status_code())
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_string(&self).unwrap()))
            .unwrap()
    }
}

impl HttpStatus for DescribeAccountError {
    fn status_code(&self) -> lambda_http::http::StatusCode {
        match self {
            DescribeAccountError::NotFoundError => StatusCode::NOT_FOUND,
        }
    }
}

impl Display for DescribeAccountError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            DescribeAccountError::NotFoundError => "No such account.",
        };

        write!(f, "{}", msg)
    }
}

impl std::error::Error for DescribeAccountError {}
impl HttpError for DescribeAccountError {}
