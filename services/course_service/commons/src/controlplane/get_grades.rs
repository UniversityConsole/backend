use crate::dataplane::Grade;
use lambda_http::{Body, IntoResponse, Request, Response};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use service_core::{HttpError, HttpStatus};
use simple_error::SimpleError;
use std::{collections::HashMap, convert::TryFrom, error::Error, fmt::Display};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
#[serde(deny_unknown_fields)]
pub struct GetGradesInput {
    pub course_id: Uuid,
    pub account_id: Uuid,
    #[serde(default = "default_calculate_final_grade")]
    pub calculate_final_grade: bool,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
#[serde(deny_unknown_fields)]
pub struct GetGradesOutput {
    pub grades: HashMap<Uuid, Vec<Grade>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub final_grade: Option<u8>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
#[serde(deny_unknown_fields)]
#[serde(tag = "ErrorKind", content = "Message")]
pub enum GetGradesError {
    NotEnrolled,
    CourseNotFound,
}

fn default_calculate_final_grade() -> bool {
    true
}

impl<'a> TryFrom<&'a Request> for GetGradesInput {
    type Error = SimpleError;

    fn try_from(req: &'a Request) -> Result<Self, Self::Error> {
        match req.body() {
            Body::Empty => Err(SimpleError::new("Unexpected empty request body.")),
            Body::Binary(_) => Err(SimpleError::new("Unexpected binary input.")),
            Body::Text(data) => serde_json::from_str(&data)
                .map_err(|_| SimpleError::new("Failed to parse GetGradesInput")),
        }
    }
}

impl IntoResponse for GetGradesOutput {
    fn into_response(self) -> Response<Body> {
        let body = serde_json::to_string(&self).unwrap();
        Response::builder()
            .status(200)
            .header("Content-Type", "application/json")
            .body(Body::Text(body))
            .unwrap()
    }
}

impl IntoResponse for GetGradesError {
    fn into_response(self) -> Response<Body> {
        let body = serde_json::to_string(&self).unwrap();
        Response::builder()
            .status(self.status_code())
            .header("Content-Type", "application/json")
            .body(Body::from(body))
            .unwrap()
    }
}

impl HttpStatus for GetGradesError {
    fn status_code(&self) -> StatusCode {
        match self {
            GetGradesError::NotEnrolled => StatusCode::NOT_FOUND,
            GetGradesError::CourseNotFound => StatusCode::NOT_FOUND,
        }
    }
}

impl Display for GetGradesError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            GetGradesError::NotEnrolled => "User is not enrolled to this course.",
            GetGradesError::CourseNotFound => "No such course exists.",
        };

        write!(f, "{}", msg)
    }
}

impl Error for GetGradesError {}
impl HttpError for GetGradesError {}
