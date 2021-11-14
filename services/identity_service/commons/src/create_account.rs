use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use service_core::{HttpError, HttpStatus};
use std::{error::Error, fmt::Display};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
#[serde(deny_unknown_fields)]
#[serde(tag = "ErrorKind", content = "Message")]
pub enum CreateAccountError {
    DuplicateAccountError,
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
