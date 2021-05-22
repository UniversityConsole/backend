use core::panic;
use lambda_http::{Body, IntoResponse, Response};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::{
    error::Error,
    fmt::{Debug, Display},
};
use strum::AsRefStr;

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

impl<E> HttpStatus for EndpointError<E>
where
    E: HttpError,
{
    fn status_code(&self) -> StatusCode {
        match self {
            EndpointError::BadRequestError(_) => StatusCode::BAD_REQUEST,
            EndpointError::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
            EndpointError::Operation(e) => e.status_code(),
        }
    }
}

impl<E> IntoResponse for EndpointError<E>
where
    E: HttpError,
{
    fn into_response(self) -> Response<Body> {
        let body = serde_json::to_string(&self).unwrap();
        Response::builder()
            .status(self.status_code())
            .header("Content-Type", "application/json")
            .body(Body::Text(body))
            .unwrap()
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
