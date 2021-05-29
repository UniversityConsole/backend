mod operations;

extern crate log;
extern crate simple_logger;

use lambda_http::lambda_runtime::{self, Context as LambdaRuntimeContext};
use lambda_http::{handler, http::Method, IntoResponse, Request};
use log::LevelFilter;
use rusoto_core::Region;
use rusoto_dynamodb::{DynamoDb, DynamoDbClient};
use service_core::{EndpointError, GenericServiceError};
use simple_logger::SimpleLogger;
use std::{env, str::FromStr};

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

pub(crate) struct Context {
    pub dynamodb_client: Box<dyn DynamoDb + Send + Sync + 'static>,
    pub courses_table: String,
    pub course_enrollments_table: String,
}

impl Context {
    pub fn from_env() -> Self {
        let aws_region = Region::from_str(Context::env("AWS_REGION").as_str()).unwrap();

        Context {
            dynamodb_client: Box::new(DynamoDbClient::new(aws_region)),
            courses_table: Context::env("COURSES_TABLE_NAME"),
            course_enrollments_table: Context::env("COURSE_ENROLLMENTS_TABLE_NAME"),
        }
    }

    fn env(name: &str) -> String {
        let value = env::var(name);

        if let Err(_) = value {
            panic!("Environment variable {} not set.", name);
        }

        value.unwrap()
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let debug_enabled = env::var("LAMBDA_DEBUG").is_ok();
    let log_level = if debug_enabled {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    };

    SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .with_module_level(module_path!(), log_level)
        .init()
        .unwrap();

    lambda_runtime::run(handler(process_request)).await?;
    Ok(())
}

async fn process_request(
    request: Request,
    _: LambdaRuntimeContext,
) -> Result<impl IntoResponse, Error> {
    let method = request.method();
    if method != Method::POST {
        return Ok(EndpointError::<GenericServiceError>::BadRequestError(
            "Expected POST request.".to_string(),
        )
        .into_response());
    }

    let operation = &request.headers().get("X-Uc-Operation");
    if let None = operation {
        return Ok(EndpointError::<GenericServiceError>::BadRequestError(
            "Expected operation in \"X-Uc-Operation\" header.".to_string(),
        )
        .into_response());
    }
    let operation = operation.unwrap().to_str();
    if let Err(_) = operation {
        return Ok(EndpointError::<GenericServiceError>::BadRequestError(
            "Operation must be an ANSI string.".to_string(),
        )
        .into_response());
    }
    let operation = operation.unwrap();
    let context = Context::from_env();

    Ok(match operation {
        "CreateCourse" => {
            match crate::operations::create_course::handler(&request, &context).await {
                Ok(r) => r.into_response(),
                Err(r) => r.into_response(),
            }
        }
        "ListCourses" => match crate::operations::list_courses::handler(&request, &context).await {
            Ok(r) => r.into_response(),
            Err(r) => r.into_response(),
        },
        "Enroll" => match crate::operations::enroll::handler(&request, &context).await {
            Ok(r) => r.into_response(),
            Err(r) => r.into_response(),
        },
        "PutGrade" => match crate::operations::put_grade::handler(&request, &context).await {
            Ok(r) => r.into_response(),
            Err(r) => r.into_response(),
        },
        "DescribeCourse" => {
            match crate::operations::describe_course::handler(&request, &context).await {
                Ok(r) => r.into_response(),
                Err(r) => r.into_response(),
            }
        }
        _ => EndpointError::<GenericServiceError>::BadRequestError("Unknown operation".to_string())
            .into_response(),
    })
}
