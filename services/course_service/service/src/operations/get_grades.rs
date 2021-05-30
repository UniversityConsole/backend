use crate::Context;
use course_service_commons::{
    controlplane::get_grades::*,
    dataplane::{CourseEnrollment, Grade, GradeComponent},
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

struct Processor<'a> {
    pub ctx: &'a Context,
    input: GetGradesInput,
}

impl Processor<'_> {
    pub async fn inner_handler(&self) -> Result<GetGradesOutput, EndpointError<GetGradesError>> {
        let output = self
            .ctx
            .dynamodb_client
            .get_item(GetItemInput {
                table_name: self.ctx.course_enrollments_table.clone(),
                key: CourseEnrollmentPk::new(
                    self.input.course_id.clone(),
                    self.input.account_id.clone(),
                )
                .as_key(),
                ..Default::default()
            })
            .await
            .map_err(|e| {
                log::error!("Failed to retrieve item from DynamoDB. Error: {:?}.", e);
                EndpointError::InternalError
            })?;

        if output.item.is_none() {
            return Err(EndpointError::Operation(GetGradesError::NotEnrolled));
        }

        let enrollment: CourseEnrollment = serde_dynamodb::from_hashmap(output.item.unwrap())
            .map_err(|e| {
                log::error!("Failed deserializing item from DynamoDB. Error: {:?}.", e);
                EndpointError::InternalError
            })?;
        let final_grade = if self.input.calculate_final_grade {
            Some(self.calculate_final_grade(&enrollment.grades).await?)
        } else {
            None
        };

        Ok(GetGradesOutput {
            grades: enrollment.grades,
            final_grade,
        })
    }

    async fn calculate_final_grade(
        &self,
        grades: &HashMap<Uuid, Vec<Grade>>,
    ) -> Result<u8, EndpointError<GetGradesError>> {
        let proj_expr = "GradingRule".to_string();
        let output = self
            .ctx
            .dynamodb_client
            .get_item(GetItemInput {
                table_name: self.ctx.courses_table.clone(),
                key: CourseIdPk::new(self.input.course_id.clone()).as_key(),
                projection_expression: Some(proj_expr),
                ..Default::default()
            })
            .await
            .map_err(|e| {
                log::error!("Failed to retrieve item from DynamoDB. Error: {:?}.", e);
                EndpointError::InternalError
            })?;

        if output.item.is_none() {
            return Err(EndpointError::Operation(GetGradesError::CourseNotFound));
        }

        let item = output.item.unwrap();
        let grading_rule: Vec<GradeComponent> = serde_dynamodb::from_hashmap(item.clone())
            .map_err(|e| {
                log::error!("Failed deserializing item from DynamoDB. Error: {:?}. Item: {:?}.", e, item);
                EndpointError::InternalError
            })?;
        let grading_rule: HashMap<Uuid, f32> = grading_rule
            .into_iter()
            .map(|e| (e.grade_component_id, e.final_grade_percentage))
            .collect();

        let mut stats: HashMap<Uuid, f32> = HashMap::new();
        for (component, grades) in grades.iter() {
            let sum = grades.iter().fold(0, |s, i| s + i.value);
            let avg = (sum as f32) / (grades.len() as f32);
            stats.insert(component.clone(), avg);
        }

        Ok(stats.into_iter().fold(0 as u8, |g, (c, v)| {
            let percentage = grading_rule.get(&c).unwrap();
            g + (percentage * v).round() as u8
        }))
    }
}

pub(crate) async fn handler(
    req: &Request,
    ctx: &Context,
) -> Result<GetGradesOutput, EndpointError<GetGradesError>> {
    let input: GetGradesInput = req
        .try_into()
        .map_err(|_| EndpointError::BadRequestError("Invalid request".to_string()))?;
    let processor = Processor { ctx, input };
    processor.inner_handler().await
}
