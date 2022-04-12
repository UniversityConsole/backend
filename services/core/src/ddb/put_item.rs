use super::adapter::Adapter;
use async_trait::async_trait;
use aws_sdk_dynamodb::error::PutItemError;
use aws_sdk_dynamodb::model::AttributeValue;
use aws_sdk_dynamodb::model::ReturnValue;
use aws_sdk_dynamodb::output::PutItemOutput;
use aws_sdk_dynamodb::types::SdkError;
use std::collections::HashMap;
use typed_builder::TypedBuilder;

#[derive(TypedBuilder)]
pub struct PutItemInput {
    #[builder(setter(into))]
    pub table_name: String,

    #[builder(setter(into))]
    pub item: HashMap<String, AttributeValue>,

    #[builder(default, setter(strip_option))]
    pub return_values: Option<ReturnValue>,

    #[builder(default, setter(strip_option, into))]
    pub condition_expression: Option<String>,

    #[builder(default, setter(strip_option))]
    pub expression_attribute_names: Option<HashMap<String, String>>,

    #[builder(default, setter(strip_option))]
    pub expression_attribute_values: Option<HashMap<String, AttributeValue>>,
}

#[async_trait]
pub trait PutItem {
    async fn put_item(&self, input: PutItemInput) -> Result<PutItemOutput, SdkError<PutItemError>>;
}

#[async_trait]
impl PutItem for Adapter {
    async fn put_item(&self, input: PutItemInput) -> Result<PutItemOutput, SdkError<PutItemError>> {
        self.raw
            .put_item()
            .table_name(input.table_name)
            .set_item(Some(input.item))
            .set_return_values(input.return_values)
            .set_condition_expression(input.condition_expression)
            .set_expression_attribute_names(input.expression_attribute_names)
            .set_expression_attribute_values(input.expression_attribute_values)
            .send()
            .await
    }
}
