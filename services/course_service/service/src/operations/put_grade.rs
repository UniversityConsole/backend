use crate::Context;
use course_service_commons::{
    controlplane::put_grade::{PutGradeError, PutGradeInput, PutGradeOutput},
    dataplane::{Course, Grade, GradeComponent},
};
use lambda_http::Request;
use rusoto_dynamodb::{AttributeValue, GetItemInput, UpdateItemInput};
use serde::Serialize;
use service_core::EndpointError;
use std::{collections::HashMap, convert::TryInto};
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

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct CourseEnrollmentPk {
    pub course_id: Uuid,
    pub user_account_id: Uuid,
}

impl CourseEnrollmentPk {
    pub fn new(course_id: Uuid, user_account_id: Uuid) -> Self {
        CourseEnrollmentPk {
            course_id,
            user_account_id,
        }
    }

    pub fn as_key(self) -> HashMap<String, AttributeValue> {
        serde_dynamodb::to_hashmap(&self).unwrap()
    }
}

async fn inner_handler(
    ctx: &Context,
    input: &PutGradeInput,
) -> Result<PutGradeOutput, EndpointError<PutGradeError>> {
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
    if output.item.is_none() {
        return Err(EndpointError::Operation(PutGradeError::CourseNotFound));
    }

    let course: Course = serde_dynamodb::from_hashmap(output.item.unwrap()).map_err(|e| {
        log::error!("Failed deserializing DynamoDB item. Error: {:?}.", e);
        EndpointError::InternalError
    })?;

    let output = ctx
        .dynamodb_client
        .get_item(GetItemInput {
            table_name: ctx.course_enrollments_table.clone(),
            key: CourseEnrollmentPk::new(input.course_id.clone(), input.account_id.clone())
                .as_key(),
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
    if output.item.is_none() {
        return Err(EndpointError::Operation(PutGradeError::NotEnrolled));
    }

    let match_grade_component =
        |e: &GradeComponent| e.grade_component_id == input.grade_component_id;
    if !course.grading_rule.iter().any(match_grade_component) {
        return Err(EndpointError::Operation(
            PutGradeError::GradeComponentNotFound,
        ));
    }

    let update_expression =
        "SET list_append(if_not_exists(#grades.#component, :empty_list), :appender)";
    let mut expression_attribute_names = HashMap::new();
    expression_attribute_names.insert("#grades".to_string(), "Grades".to_string());
    expression_attribute_names.insert(
        "#component".to_string(),
        input.grade_component_id.clone().to_hyphenated().to_string(),
    );
    let mut expression_attribute_values = HashMap::new();
    expression_attribute_values.insert(
        ":empty_list".to_string(),
        AttributeValue {
            l: Some(Vec::new()),
            ..Default::default()
        },
    );
    expression_attribute_values.insert(
        ":appender".to_string(),
        AttributeValue {
            l: Some(vec![AttributeValue {
                m: Some(
                    serde_dynamodb::to_hashmap(&Grade {
                        timestamp: chrono::offset::Utc::now(),
                        value: input.value,
                    })
                    .unwrap(),
                ),
                ..Default::default()
            }]),
            ..Default::default()
        },
    );

    ctx.dynamodb_client
        .update_item(UpdateItemInput {
            table_name: ctx.course_enrollments_table.clone(),
            key: CourseEnrollmentPk::new(input.course_id.clone(), input.account_id.clone())
                .as_key(),
            update_expression: Some(update_expression.to_string()),
            expression_attribute_names: Some(expression_attribute_names.clone()),
            expression_attribute_values: Some(expression_attribute_values.clone()),
            ..Default::default()
        })
        .await
        .map_err(|e| {
            log::error!(
                "Failed to update item to DynamoDB. Original error: {:?}. \
                Update expression: {}. Attribute names: {:?}. Attribute values: {:?}.",
                e,
                update_expression,
                expression_attribute_names,
                expression_attribute_values
            );
            EndpointError::InternalError
        })?;

    Ok(PutGradeOutput {})
}

pub(crate) async fn handler(
    req: &Request,
    ctx: &Context,
) -> Result<PutGradeOutput, EndpointError<PutGradeError>> {
    let input: PutGradeInput = req
        .try_into()
        .map_err(|_| EndpointError::BadRequestError("Invalid request".to_string()))?;
    inner_handler(&ctx, &input).await
}
