use lambda_http::{Body, IntoResponse, Request, Response};
use serde::{Deserialize, Serialize};
use simple_error::SimpleError;
use std::convert::TryFrom;
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
#[serde(deny_unknown_fields)]
pub struct ListCoursesForAccountInput {
    pub account_id: Uuid,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
#[serde(deny_unknown_fields)]
pub struct ListCoursesForAccountOutput {
    pub course_ids: Vec<Uuid>,
}

impl<'a> TryFrom<&'a Request> for ListCoursesForAccountInput {
    type Error = SimpleError;

    fn try_from(req: &'a Request) -> Result<Self, Self::Error> {
        match req.body() {
            Body::Empty => Err(SimpleError::new("Unexpected empty request body.")),
            Body::Binary(_) => Err(SimpleError::new("Unexpected binary input.")),
            Body::Text(data) => serde_json::from_str(&data)
                .map_err(|_| SimpleError::new("Failed to parse ListCoursesForAccountInput")),
        }
    }
}

impl IntoResponse for ListCoursesForAccountOutput {
    fn into_response(self) -> Response<Body> {
        let body = serde_json::to_string(&self).unwrap();
        Response::builder()
            .status(200)
            .header("Content-Type", "application/json")
            .body(Body::Text(body))
            .unwrap()
    }
}
