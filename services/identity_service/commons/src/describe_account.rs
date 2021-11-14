use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use service_core::{HttpError, HttpStatus};
use std::fmt::Display;

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "ErrorKind", content = "Message")]
pub enum DescribeAccountError {
    NotFoundError,
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
