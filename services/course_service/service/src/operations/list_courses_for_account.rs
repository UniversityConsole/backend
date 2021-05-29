use crate::Context;
use course_service_commons::controlplane::list_courses_for_account::{
    ListCoursesForAccountInput, ListCoursesForAccountOutput,
};
use lambda_http::Request;
use rusoto_dynamodb::{AttributeValue, QueryInput};
use serde::Deserialize;
use service_core::{EndpointError, GenericServiceError};
use std::collections::HashMap;
use std::convert::TryInto;
use uuid::Uuid;

#[derive(Deserialize)]
struct CourseIdItem {
    #[serde(rename = "CourseId")]
    pub course_id: Uuid,
}

async fn inner_handler(
    ctx: &Context,
    input: &ListCoursesForAccountInput,
) -> Result<ListCoursesForAccountOutput, EndpointError<GenericServiceError>> {
    let proj_expr = ["CourseId"].join(",");
    let mut attr_vals = HashMap::new();
    attr_vals.insert(
        ":account_id".to_string(),
        AttributeValue {
            s: Some(input.account_id.to_hyphenated().to_string()),
            ..Default::default()
        },
    );

    let output = ctx
        .dynamodb_client
        .query(QueryInput {
            table_name: ctx.course_enrollments_table.clone(),
            index_name: Some("UserAccountIdIndex".to_string()),
            key_condition_expression: Some("UserAccountId = :account_id".to_string()),
            expression_attribute_values: Some(attr_vals),
            projection_expression: Some(proj_expr),
            ..Default::default()
        })
        .await
        .map_err(|e| {
            log::error!("Failed DynamoDB query. Error: {:?}.", e);
            EndpointError::InternalError
        })?;

    let mut course_ids = vec![];
    for item in output.items.unwrap().into_iter() {
        let item: CourseIdItem = serde_dynamodb::from_hashmap(item).unwrap();
        course_ids.push(item.course_id);
    }

    Ok(ListCoursesForAccountOutput { course_ids })
}

pub(crate) async fn handler(
    req: &Request,
    ctx: &Context,
) -> Result<ListCoursesForAccountOutput, EndpointError<GenericServiceError>> {
    let input: ListCoursesForAccountInput = req
        .try_into()
        .map_err(|_| EndpointError::BadRequestError("Invalid request".to_string()))?;
    inner_handler(&ctx, &input).await
}
