use crate::dataplane::UserAccount;
use core::panic;
use lambda_http::{Body, IntoResponse, Request, Response};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use service_core::{HttpError, HttpStatus};
use simple_error::SimpleError;
use std::default::Default;
use std::{convert::TryFrom, error::Error, fmt::Display};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
#[serde(deny_unknown_fields)]
pub struct ListAccountsInput {
    pub starting_token: Option<String>,
    #[serde(default = "default_page_size")]
    pub page_size: i64,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
#[serde(deny_unknown_fields)]
pub struct ListAccountsOutput {
    pub accounts: Vec<UserAccount>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_token: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "ErrorKind", content = "Message")]
pub enum ListAccountsError {}

fn default_page_size() -> i64 {
    32
}

impl Default for ListAccountsInput {
    fn default() -> Self {
        ListAccountsInput {
            starting_token: None,
            page_size: default_page_size(),
        }
    }
}

impl<'a> TryFrom<&'a Request> for ListAccountsInput {
    type Error = SimpleError;

    fn try_from(req: &'a Request) -> Result<Self, Self::Error> {
        match req.body() {
            Body::Empty => Err(SimpleError::new("Unexpected empty request body.")),
            Body::Binary(_) => Err(SimpleError::new("Unexpected binary input.")),
            Body::Text(data) => serde_json::from_str(&data)
                .map_err(|_| SimpleError::new("Failed to parse ListAccountsInput")),
        }
    }
}

impl IntoResponse for ListAccountsOutput {
    fn into_response(self) -> Response<Body> {
        let body = serde_json::to_string(&self).unwrap();
        Response::builder()
            .status(200)
            .header("Content-Type", "application/json")
            .body(Body::Text(body))
            .unwrap()
    }
}

impl IntoResponse for ListAccountsError {
    fn into_response(self) -> Response<Body> {
        let body = serde_json::to_string(&self).unwrap();
        Response::builder()
            .status(self.status_code())
            .header("Content-Type", "application/json")
            .body(Body::from(body))
            .unwrap()
    }
}

impl HttpStatus for ListAccountsError {
    fn status_code(&self) -> StatusCode {
        panic!("Error should not be used.");
    }
}

impl Display for ListAccountsError {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        panic!("Error should not be used.");
    }
}

impl Error for ListAccountsError {}
impl HttpError for ListAccountsError {}
