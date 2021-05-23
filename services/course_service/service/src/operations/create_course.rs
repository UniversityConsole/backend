use crate::Context;
use course_service_commons::{
    controlplane::create_course::{
        CreateCourseError, CreateCourseInput, CreateCourseOutput, GradeComponentDetails,
    },
    dataplane::{Course, GradeComponent},
};
use identity_service_client::IdentityServiceClient;
use identity_service_commons::{DescribeAccountError, DescribeAccountInput};
use lambda_http::Request;
use rusoto_dynamodb::PutItemInput;
use service_client_runtime::OperationError;
use service_core::EndpointError;
use std::convert::TryInto;
use uuid::Uuid;

async fn inner_handler(
    ctx: &Context,
    input: &CreateCourseInput,
) -> Result<CreateCourseOutput, EndpointError<CreateCourseError>> {
    if input.course.title.is_empty() {
        return Err(EndpointError::BadRequestError(
            "Title is required.".to_string(),
        ));
    }
    if input.course.description.is_empty() {
        return Err(EndpointError::BadRequestError(
            "Description is required.".to_string(),
        ));
    }
    if input.course.owner_id.is_nil() {
        return Err(EndpointError::BadRequestError(
            "OwnerId is required.".to_string(),
        ));
    }

    validate_grading_rule(&input.course.grading_rule)?;

    let identity_service = IdentityServiceClient::from_env().map_err(|e| {
        log::error!("Service endpoint not set. Original error: {:?}.", e);
        EndpointError::InternalError
    })?;
    identity_service
        .describe_account(DescribeAccountInput {
            account_id: input.course.owner_id.clone(),
        })
        .await
        .map_err(|e| match e {
            OperationError::Endpoint(EndpointError::Operation(
                DescribeAccountError::NotFoundError,
            )) => EndpointError::Operation(CreateCourseError::AccountNotFound),
            _ => {
                log::error!("Failed to retrieve owner account. Original error: {}.", e);
                EndpointError::InternalError
            }
        })?;

    let course = Course {
        course_id: Uuid::new_v4(),
        title: input.course.title.clone(),
        description: input.course.description.clone(),
        owner_id: input.course.owner_id.clone(),
        created_at: chrono::offset::Utc::now(),
        closed_at: None,
        grading_rule: input
            .course
            .grading_rule
            .iter()
            .map(|c| GradeComponent {
                grading_rule_id: Uuid::new_v4(),
                title: c.title.clone(),
                final_grade_percentage: c.final_grade_percentage,
            })
            .collect(),
    };

    ctx.dynamodb_client
        .put_item(PutItemInput {
            item: serde_dynamodb::to_hashmap(&course).unwrap(),
            table_name: ctx.courses_table.clone(),
            condition_expression: Some("attribute_not_exists(CourseId)".to_string()),
            ..Default::default()
        })
        .await
        .map_err(|e| {
            log::error!(
                "Failed to write item to DynamoDB. Original item: {:?}. Original error: {:?}.",
                &course,
                e
            );
            EndpointError::InternalError
        })?;

    Ok(CreateCourseOutput {
        course_id: course.course_id,
    })
}

fn validate_grading_rule(
    components: &Vec<GradeComponentDetails>,
) -> Result<(), EndpointError<CreateCourseError>> {
    let total_percentage = components
        .iter()
        .fold(0 as f32, |acc, i| acc + i.final_grade_percentage);
    if (total_percentage - 1.0).abs() < f32::EPSILON {
        return Err(EndpointError::BadRequestError(
            "Grade components do not sum up to 100%.".to_string(),
        ));
    }

    Ok(())
}

pub(crate) async fn handler(
    req: &Request,
    ctx: &Context,
) -> Result<CreateCourseOutput, EndpointError<CreateCourseError>> {
    let input: CreateCourseInput = req
        .try_into()
        .map_err(|_| EndpointError::BadRequestError("Invalid request".to_string()))?;
    inner_handler(&ctx, &input).await
}
