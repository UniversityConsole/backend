use crate::Context;
use course_service_commons::controlplane::put_grade::{
    PutGradeError, PutGradeInput, PutGradeOutput,
};
use course_service_commons::dataplane::{Course, Grade, GradeComponent};
use lambda_http::Request;
use rusoto_core::RusotoError;
use rusoto_dynamodb::{AttributeValue, GetItemInput, UpdateItemError, UpdateItemInput};
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

type PutGradeResult = Result<PutGradeOutput, EndpointError<PutGradeError>>;

async fn inner_handler(ctx: &Context, input: &PutGradeInput) -> PutGradeResult {
    if input.value > 100 {
        return Err(EndpointError::BadRequestError(
            "Grade value must be between 0 and 100.".to_string(),
        ));
    }

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

    ensure_grade_component_exists(&ctx, &input).await?;
    append_grade(&ctx, &input).await?;

    Ok(PutGradeOutput {})
}

async fn ensure_grade_component_exists(
    ctx: &Context,
    input: &PutGradeInput,
) -> Result<(), EndpointError<PutGradeError>> {
    let update_expression = "SET Grades.#comp = :empty_list";
    let mut expression_attribute_names = HashMap::new();
    expression_attribute_names.insert(
        "#comp".to_string(),
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

    let output = ctx
        .dynamodb_client
        .update_item(UpdateItemInput {
            table_name: ctx.course_enrollments_table.clone(),
            key: CourseEnrollmentPk::new(input.course_id.clone(), input.account_id.clone())
                .as_key(),
            update_expression: Some(update_expression.to_string()),
            condition_expression: Some("attribute_not_exists(Grades.#comp)".to_string()),
            expression_attribute_names: Some(expression_attribute_names.clone()),
            expression_attribute_values: Some(expression_attribute_values.clone()),
            ..Default::default()
        })
        .await;

    match output {
        Ok(_) => Ok(()),
        Err(RusotoError::Service(UpdateItemError::ConditionalCheckFailed(_))) => Ok(()),
        e => {
            log::error!(
                "Failed to update item to DynamoDB. Original error: {:?}. \
                Update expression: {}. Attribute names: {:?}. Attribute values: {:?}.",
                e,
                update_expression,
                expression_attribute_names,
                expression_attribute_values
            );
            Err(EndpointError::InternalError)
        }
    }
}

async fn append_grade(
    ctx: &Context,
    input: &PutGradeInput,
) -> Result<(), EndpointError<PutGradeError>> {
    let update_expression = "SET Grades.#comp = list_append(Grades.#comp, :new_grade)";
    let mut expression_attribute_names = HashMap::new();
    expression_attribute_names.insert(
        "#comp".to_string(),
        input.grade_component_id.clone().to_hyphenated().to_string(),
    );

    let mut expression_attribute_values = HashMap::new();
    let grade = Grade {
        timestamp: chrono::offset::Utc::now(),
        value: input.value,
    };
    let grade_attr_val = AttributeValue {
        m: Some(serde_dynamodb::to_hashmap(&grade).unwrap()),
        ..Default::default()
    };
    expression_attribute_values.insert(
        ":new_grade".to_string(),
        AttributeValue {
            l: Some(vec![grade_attr_val]),
            ..Default::default()
        },
    );

    let output = ctx
        .dynamodb_client
        .update_item(UpdateItemInput {
            table_name: ctx.course_enrollments_table.clone(),
            key: CourseEnrollmentPk::new(input.course_id.clone(), input.account_id.clone())
                .as_key(),
            update_expression: Some(update_expression.to_string()),
            condition_expression: Some("attribute_exists(Grades.#comp)".to_string()),
            expression_attribute_names: Some(expression_attribute_names.clone()),
            expression_attribute_values: Some(expression_attribute_values.clone()),
            ..Default::default()
        })
        .await;

    match output {
        Ok(_) => Ok(()),
        e => {
            log::error!(
                "Failed to update item to DynamoDB. Original error: {:?}. \
                Update expression: {}. Attribute names: {:?}. Attribute values: {:?}.",
                e,
                update_expression,
                expression_attribute_names,
                expression_attribute_values
            );
            Err(EndpointError::InternalError)
        }
    }
}

pub(crate) async fn handler(req: &Request, ctx: &Context) -> PutGradeResult {
    let input: PutGradeInput = req
        .try_into()
        .map_err(|_| EndpointError::BadRequestError("Invalid request".to_string()))?;
    inner_handler(&ctx, &input).await
}
