use core::panic;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::{
    error::Error,
    fmt::{Debug, Display},
};
use strum::AsRefStr;
use tonic::Code;
use tonic::Status;

pub trait HttpStatus {
    fn status_code(&self) -> StatusCode;
}

pub trait HttpError: Error + HttpStatus + Serialize {}

#[derive(Serialize, Deserialize, Debug, AsRefStr)]
#[serde(tag = "ErrorKind", content = "Content")]
pub enum EndpointError<E: HttpError> {
    BadRequestError(String),
    InternalError,
    Operation(E),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GenericServiceError;

impl<E> Display for EndpointError<E>
where
    E: HttpError,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let kind: &str = self.as_ref();
        let msg = match self {
            EndpointError::BadRequestError(msg) => msg.clone(),
            EndpointError::InternalError => String::from("Internal server error."),
            EndpointError::Operation(err) => err.to_string(),
        };

        write!(f, "{}: {}", kind, msg)
    }
}

impl<E> Into<Status> for EndpointError<E>
where
    E: HttpError,
{
    fn into(self) -> Status {
        Status::new(self.status_code(), self.message())
    }
}

impl<E> EndpointError<E>
where
    E: HttpError,
{
    fn status_code(&self) -> Code {
        match self {
            EndpointError::BadRequestError(_) => Code::InvalidArgument,
            EndpointError::InternalError => Code::Internal,
            EndpointError::Operation(_) => Code::Internal, // TODO Use something provided by the error.
        }
    }

    fn message(&self) -> String {
        match self {
            EndpointError::BadRequestError(msg) => msg.to_string(),
            EndpointError::InternalError => "Internal error".to_string(),
            EndpointError::Operation(e) => e.to_string(),
        }
    }
}

impl HttpStatus for GenericServiceError {
    fn status_code(&self) -> StatusCode {
        panic!("You shouldn't be using this.");
    }
}

impl Display for GenericServiceError {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        panic!("This type should not be used.");
    }
}

impl Error for GenericServiceError {}
impl HttpError for GenericServiceError {}
