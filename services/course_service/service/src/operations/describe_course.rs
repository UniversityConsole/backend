use crate::Context;
use course_service_commons::controlplane::describe_course::{
    DescribeCourseError, DescribeCourseInput, DescribeCourseOutput,
};
use lambda_http::Request;
use rusoto_dynamodb::{AttributeValue, GetItemInput};
use serde::Serialize;
use service_core::EndpointError;
use std::collections::HashMap;
use std::convert::TryInto;
use uuid::Uuid;

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct CourseIdPk {
    pub course_id: Uuid,
}

impl CourseIdPk {
    pub fn new(course_id: Uuid) -> Self {
        CourseIdPk { course_id }
    }

    pub fn as_key(self) -> HashMap<String, AttributeValue> {
        serde_dynamodb::to_hashmap(&self).unwrap()
    }
}

async fn inner_handler(
    ctx: &Context,
    input: &DescribeCourseInput,
) -> Result<DescribeCourseOutput, EndpointError<DescribeCourseError>> {
    let output = ctx
        .dynamodb_client
        .get_item(GetItemInput {
            table_name: ctx.courses_table.clone(),
            key: CourseIdPk::new(input.course_id.clone()).as_key(),
            ..GetItemInput::default()
        })
        .await
        .map_err(|e| {
            log::error!(
                "Failed to retrieve item from DynamoDB. Original error: {:?}.",
                e
            );
            EndpointError::InternalError
        })?;

    match output.item {
        None => Err(EndpointError::Operation(DescribeCourseError::NotFound)),
        Some(hm) => Ok(DescribeCourseOutput {
            course: serde_dynamodb::from_hashmap(hm).map_err(|e| {
                log::error!("Failed deserializing DynamoDB item. Error: {:?}.", e);
                EndpointError::InternalError
            })?,
        }),
    }
}

pub(crate) async fn handler(
    req: &Request,
    ctx: &Context,
) -> Result<DescribeCourseOutput, EndpointError<DescribeCourseError>> {
    let input: DescribeCourseInput = req
        .try_into()
        .map_err(|_| EndpointError::BadRequestError("Invalid request".to_string()))?;
    inner_handler(&ctx, &input).await
}
