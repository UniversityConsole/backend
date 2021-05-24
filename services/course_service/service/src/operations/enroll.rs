use crate::Context;
use course_service_commons::{
    controlplane::enroll::{EnrollError, EnrollInput, EnrollOutput},
    dataplane::CourseEnrollment,
};
use identity_service_client::IdentityServiceClient;
use identity_service_commons::{DescribeAccountError, DescribeAccountInput};
use lambda_http::Request;
use rusoto_core::RusotoError;
use rusoto_dynamodb::{AttributeValue, GetItemInput, PutItemError, PutItemInput};
use serde::Serialize;
use service_client_runtime::OperationError;
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

async fn inner_handler(
    ctx: &Context,
    input: &EnrollInput,
) -> Result<EnrollOutput, EndpointError<EnrollError>> {
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
        return Err(EndpointError::Operation(EnrollError::CourseNotFound));
    }

    let identity_service = IdentityServiceClient::from_env().map_err(|e| {
        log::error!("Service endpoint not set. Original error: {:?}.", e);
        EndpointError::InternalError
    })?;
    identity_service
        .describe_account(DescribeAccountInput {
            account_id: input.account_id.clone(),
        })
        .await
        .map_err(|e| match e {
            OperationError::Endpoint(EndpointError::Operation(
                DescribeAccountError::NotFoundError,
            )) => EndpointError::Operation(EnrollError::AccountNotFound),
            _ => {
                log::error!("Failed to retrieve owner account. Original error: {}.", e);
                EndpointError::InternalError
            }
        })?;

    let enrollment = CourseEnrollment {
        course_id: input.course_id,
        user_account_id: input.account_id,
        enrolled_at: chrono::offset::Utc::now(),
        grades: HashMap::new(),
    };

    ctx.dynamodb_client
        .put_item(PutItemInput {
            item: serde_dynamodb::to_hashmap(&enrollment).unwrap(),
            table_name: ctx.course_enrollments_table.clone(),
            condition_expression: Some(
                "attribute_not_exists(CourseId) and attribute_not_exists(UserAccountId)"
                    .to_string(),
            ),
            ..Default::default()
        })
        .await
        .map_err(|e| match e {
            RusotoError::Service(PutItemError::ConditionalCheckFailed(_)) => {
                EndpointError::Operation(EnrollError::AlreadyEnrolled)
            }
            _ => {
                log::error!(
                    "Failed to write item to DynamoDB. Original item: {:?}. Original error: {:?}.",
                    &enrollment,
                    e
                );
                EndpointError::InternalError
            }
        })?;

    Ok(EnrollOutput {})
}

pub(crate) async fn handler(
    req: &Request,
    ctx: &Context,
) -> Result<EnrollOutput, EndpointError<EnrollError>> {
    let input: EnrollInput = req
        .try_into()
        .map_err(|_| EndpointError::BadRequestError("Invalid request".to_string()))?;
    inner_handler(&ctx, &input).await
}
