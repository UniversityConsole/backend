use crate::Context;
use course_service_commons::controlplane::list_courses::{ListCoursesInput, ListCoursesOutput};
use lambda_http::Request;
use rusoto_dynamodb::{AttributeValue, ScanInput};
use service_core::{EndpointError, GenericServiceError};
use std::collections::HashMap;
use std::convert::TryInto;
use uuid::Uuid;

async fn inner_handler(
    ctx: &Context,
    input: &ListCoursesInput,
) -> Result<ListCoursesOutput, EndpointError<GenericServiceError>> {
    if input.page_size <= 0 {
        return Err(EndpointError::BadRequestError(
            "PageSize must be > 0.".to_string(),
        ));
    }

    let page_start = match &input.starting_token {
        Some(v) => {
            const PARSE_ERR_MSG: &str = "Could not parse StartingToken.";
            let v = base64::decode(&v)
                .map_err(|_| EndpointError::BadRequestError(PARSE_ERR_MSG.to_string()))?;
            let v = String::from_utf8(v)
                .map_err(|_| EndpointError::BadRequestError(PARSE_ERR_MSG.to_string()))?;
            let v = Uuid::parse_str(v.as_str())
                .map_err(|_| EndpointError::BadRequestError(PARSE_ERR_MSG.to_string()))?;
            let mut hm = HashMap::new();
            hm.insert(
                "CourseId".to_string(),
                AttributeValue {
                    s: Some(v.to_hyphenated().to_string()),
                    ..AttributeValue::default()
                },
            );
            Some(hm)
        }
        None => None,
    };
    let projection_expression = get_projection_expr(&input);
    let filter_expression = if input.include_closed {
        None
    } else {
        Some("ClosedAt = :closed_at".to_string())
    };
    let expression_attribute_values = if input.include_closed {
        None
    } else {
        let mut hm = HashMap::new();
        hm.insert(
            ":closed_at".to_string(),
            AttributeValue {
                null: Some(true),
                ..AttributeValue::default()
            },
        );
        Some(hm)
    };
    let output = ctx
        .dynamodb_client
        .scan(ScanInput {
            limit: Some(input.page_size),
            exclusive_start_key: page_start,
            projection_expression,
            table_name: ctx.courses_table.clone(),
            filter_expression,
attribute_values,
            ..ScanInput::default()
        })
        .await
        .map_err(|e| {
            log::error!("DynamoDB scan failed. Error: {:?}.", e);
            EndpointError::InternalError
        })?;

    let next_token = match output.last_evaluated_key {
        None => None,
        Some(hm) => {
            let course_id = hm.get(&"CourseId".to_string()).unwrap();
            Some(base64::encode(course_id.s.as_ref().unwrap()))
        }
    };
    let courses = match output.items {
        None => vec![],
        Some(items) => {
            let mut courses = vec![];
            courses.reserve(items.len());

            for item in items.into_iter() {
                let item_json = serde_json::json!(&item);
                let course = serde_dynamodb::from_hashmap(item).map_err(|err| {
                    log::error!(
                        "Invalid record in DynamoDB: {}. Original item: {}",
                        err,
                        item_json
                    );
                    EndpointError::InternalError
                })?;
                courses.push(course);
            }

            courses
        }
    };

    Ok(ListCoursesOutput {
        courses,
        next_token,
    })
}

fn get_projection_expr(input: &ListCoursesInput) -> Option<String> {
    let mut fields = vec!["CourseId", "Title", "Description", "OwnerId", "CreatedAt"];
    if input.include_closed {
        fields.push("ClosedAt");
    }

    Some(fields.join(","))
}

pub(crate) async fn handler(
    req: &Request,
    ctx: &Context,
) -> Result<ListCoursesOutput, EndpointError<GenericServiceError>> {
    let input: ListCoursesInput = req
        .try_into()
        .map_err(|_| EndpointError::BadRequestError("Invalid request".to_string()))?;
    inner_handler(&ctx, &input).await
}
