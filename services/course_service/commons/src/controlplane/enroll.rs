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
pub struct EnrollInput {
    pub account_id: Uuid,
    pub course_id: Uuid,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
#[serde(deny_unknown_fields)]
pub struct EnrollOutput {}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
#[serde(deny_unknown_fields)]
#[serde(tag = "ErrorKind", content = "Message")]
pub enum EnrollError {
    AccountNotFound,
    CourseNotFound,
    AlreadyEnrolled,
}

impl<'a> TryFrom<&'a Request> for EnrollInput {
    type Error = SimpleError;

    fn try_from(req: &'a Request) -> Result<Self, Self::Error> {
        match req.body() {
            Body::Empty => Err(SimpleError::new("Unexpected empty request body.")),
            Body::Binary(_) => Err(SimpleError::new("Unexpected binary input.")),
            Body::Text(data) => serde_json::from_str(&data)
                .map_err(|_| SimpleError::new("Failed to parse EnrollInput")),
        }
    }
}

impl IntoResponse for EnrollOutput {
    fn into_response(self) -> Response<Body> {
        let body = serde_json::to_string(&self).unwrap();
        Response::builder()
            .status(200)
            .header("Content-Type", "application/json")
            .body(Body::Text(body))
            .unwrap()
    }
}

impl IntoResponse for EnrollError {
    fn into_response(self) -> Response<Body> {
        let body = serde_json::to_string(&self).unwrap();
        Response::builder()
            .status(self.status_code())
            .header("Content-Type", "application/json")
            .body(Body::from(body))
            .unwrap()
    }
}

impl HttpStatus for EnrollError {
    fn status_code(&self) -> StatusCode {
        match self {
            EnrollError::AccountNotFound => StatusCode::NOT_FOUND,
            EnrollError::CourseNotFound => StatusCode::NOT_FOUND,
            EnrollError::AlreadyEnrolled => StatusCode::CONFLICT,
        }
    }
}

impl Display for EnrollError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            EnrollError::AccountNotFound => "No such account exists (AccountId).",
            EnrollError::CourseNotFound => "No such course exists (CourseId).",
            EnrollError::AlreadyEnrolled => "Account already enrolled.",
        };

        write!(f, "{}", msg)
    }
}

impl Error for EnrollError {}
impl HttpError for EnrollError {}
