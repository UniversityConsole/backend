use std::convert::TryFrom;

use crate::dataplane::UserAccount;
use lambda_http::{Body, IntoResponse, Request, Response};
use serde::{Deserialize, Serialize};
use serde_json::json;
use simple_error::SimpleError;

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
    InternalError,
    ValidationError(String),
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
        let body = json!({ "Message": self }).to_string();
        let status_code = match self {
            DescribeAccountError::ValidationError(_) => 400,
            DescribeAccountError::InternalError => 500,
            DescribeAccountError::NotFoundError => 404,
        };
        Response::builder()
            .status(status_code)
            .header("Content-Type", "application/json")
            .body(Body::from(body))
            .unwrap()
    }
}
