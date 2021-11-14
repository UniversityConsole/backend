use core::panic;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use service_core::{HttpError, HttpStatus};
use std::{error::Error, fmt::Display};

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "ErrorKind", content = "Message")]
pub enum ListAccountsError {}

pub fn default_page_size() -> u32 {
    32
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
