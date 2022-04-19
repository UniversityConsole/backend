use std::collections::HashMap;

use async_trait::async_trait;
use aws_sdk_dynamodb::error::GetItemError;
use aws_sdk_dynamodb::model::AttributeValue;
use aws_sdk_dynamodb::output::GetItemOutput;
use aws_sdk_dynamodb::types::SdkError;
use typed_builder::TypedBuilder;

use super::adapter::Adapter;

#[derive(TypedBuilder)]
pub struct GetItemInput {
    #[builder(setter(into))]
    pub table_name: String,

    pub key: HashMap<String, AttributeValue>,

    #[builder(default = false)]
    pub consistent_read: bool,

    #[builder(default, setter(strip_option, into))]
    pub projection_expression: Option<String>,

    #[builder(default, setter(strip_option))]
    pub expression_attribute_names: Option<HashMap<String, String>>,
}

#[async_trait]
pub trait GetItem {
    async fn get_item(&self, input: GetItemInput) -> Result<GetItemOutput, SdkError<GetItemError>>;
}

#[async_trait]
impl GetItem for Adapter {
    async fn get_item(&self, input: GetItemInput) -> Result<GetItemOutput, SdkError<GetItemError>> {
        self.raw
            .get_item()
            .table_name(input.table_name)
            .set_key(Some(input.key))
            .consistent_read(input.consistent_read)
            .set_projection_expression(input.projection_expression)
            .set_expression_attribute_names(input.expression_attribute_names)
            .send()
            .await
    }
}
