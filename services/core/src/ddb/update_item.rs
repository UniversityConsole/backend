use std::collections::HashMap;

use async_trait::async_trait;
use aws_sdk_dynamodb::error::UpdateItemError;
use aws_sdk_dynamodb::model::{AttributeValue, ReturnValue};
use aws_sdk_dynamodb::output::UpdateItemOutput;
use aws_sdk_dynamodb::types::SdkError;
use typed_builder::TypedBuilder;

use super::adapter::Adapter;

#[derive(TypedBuilder, Clone, Debug)]
pub struct UpdateItemInput {
    #[builder(setter(into))]
    pub table_name: String,

    #[builder(setter(into))]
    pub key: HashMap<String, AttributeValue>,

    #[builder(setter(into))]
    pub update_expression: String,

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
pub trait UpdateItem {
    async fn update_item(&self, input: UpdateItemInput) -> Result<UpdateItemOutput, SdkError<UpdateItemError>>;
}

#[async_trait]
impl UpdateItem for Adapter {
    async fn update_item(&self, input: UpdateItemInput) -> Result<UpdateItemOutput, SdkError<UpdateItemError>> {
        self.raw
            .update_item()
            .table_name(input.table_name)
            .set_key(Some(input.key))
            .update_expression(input.update_expression)
            .set_return_values(input.return_values)
            .set_condition_expression(input.condition_expression)
            .set_expression_attribute_names(input.expression_attribute_names)
            .set_expression_attribute_values(input.expression_attribute_values)
            .send()
            .await
    }
}
