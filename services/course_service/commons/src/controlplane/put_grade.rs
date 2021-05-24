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
pub struct PutGradeInput {
    pub account_id: Uuid,
    pub course_id: Uuid,
    pub grade_component_id: Uuid,
    pub value: u8,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
#[serde(deny_unknown_fields)]
pub struct PutGradeOutput {}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
#[serde(deny_unknown_fields)]
#[serde(tag = "ErrorKind", content = "Message")]
pub enum PutGradeError {
    CourseNotFound,
    NotEnrolled,
    GradeComponentNotFound,
}

impl<'a> TryFrom<&'a Request> for PutGradeInput {
    type Error = SimpleError;

    fn try_from(req: &'a Request) -> Result<Self, Self::Error> {
        match req.body() {
            Body::Empty => Err(SimpleError::new("Unexpected empty request body.")),
            Body::Binary(_) => Err(SimpleError::new("Unexpected binary input.")),
            Body::Text(data) => serde_json::from_str(&data)
                .map_err(|_| SimpleError::new("Failed to parse PutGradeInput")),
        }
    }
}

impl IntoResponse for PutGradeOutput {
    fn into_response(self) -> Response<Body> {
        let body = serde_json::to_string(&self).unwrap();
        Response::builder()
            .status(200)
            .header("Content-Type", "application/json")
            .body(Body::Text(body))
            .unwrap()
    }
}

impl IntoResponse for PutGradeError {
    fn into_response(self) -> Response<Body> {
        let body = serde_json::to_string(&self).unwrap();
        Response::builder()
            .status(self.status_code())
            .header("Content-Type", "application/json")
            .body(Body::from(body))
            .unwrap()
    }
}

impl HttpStatus for PutGradeError {
    fn status_code(&self) -> StatusCode {
        match self {
            PutGradeError::CourseNotFound => StatusCode::NOT_FOUND,
            PutGradeError::NotEnrolled => StatusCode::NOT_FOUND,
            PutGradeError::GradeComponentNotFound => StatusCode::NOT_FOUND,
        }
    }
}

impl Display for PutGradeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            PutGradeError::CourseNotFound => "Course not found.",
            PutGradeError::NotEnrolled => "User is not enrolled or does not exist.",
            PutGradeError::GradeComponentNotFound => "Grade component not found.",
        };

        write!(f, "{}", msg)
    }
}

impl Error for PutGradeError {}
impl HttpError for PutGradeError {}
