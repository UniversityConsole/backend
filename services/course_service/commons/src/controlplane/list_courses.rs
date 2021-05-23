use crate::dataplane::Course;
use lambda_http::{Body, IntoResponse, Request, Response};
use serde::{Deserialize, Serialize};
use simple_error::SimpleError;
use std::convert::TryFrom;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
#[serde(deny_unknown_fields)]
pub struct ListCoursesInput {
    #[serde(default)]
    pub include_closed: bool,
    #[serde(default)]
    pub starting_token: Option<String>,
    #[serde(default = "default_page_size")]
    pub page_size: i64,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
#[serde(deny_unknown_fields)]
pub struct ListCoursesOutput {
    pub courses: Vec<Course>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_token: Option<String>,
}

fn default_page_size() -> i64 {
    32
}

impl<'a> TryFrom<&'a Request> for ListCoursesInput {
    type Error = SimpleError;

    fn try_from(req: &'a Request) -> Result<Self, Self::Error> {
        match req.body() {
            Body::Empty => Err(SimpleError::new("Unexpected empty request body.")),
            Body::Binary(_) => Err(SimpleError::new("Unexpected binary input.")),
            Body::Text(data) => serde_json::from_str(&data)
                .map_err(|_| SimpleError::new("Failed to parse ListCoursesInput")),
        }
    }
}

impl IntoResponse for ListCoursesOutput {
    fn into_response(self) -> Response<Body> {
        let body = serde_json::to_string(&self).unwrap();
        Response::builder()
            .status(200)
            .header("Content-Type", "application/json")
            .body(Body::Text(body))
            .unwrap()
    }
}
