use super::adapter::Adapter;
use async_trait::async_trait;
use aws_sdk_dynamodb::error::QueryError;
use aws_sdk_dynamodb::model::{AttributeValue, Select};
use aws_sdk_dynamodb::output::QueryOutput;
use aws_sdk_dynamodb::types::SdkError;
use std::collections::HashMap;
use typed_builder::TypedBuilder;

#[derive(Debug, TypedBuilder)]
pub struct QueryInput {
    #[builder(default, setter(strip_option, into))]
    pub table_name: Option<String>,

    #[builder(default, setter(strip_option, into))]
    pub index_name: Option<String>,

    #[builder(setter(into))]
    pub limit: i32,

    #[builder(default, setter(strip_option))]
    pub select: Option<Select>,

    #[builder(default)]
    pub exclusive_start_key: Option<HashMap<String, AttributeValue>>,

    #[builder(default = true)]
    pub scan_index_forward: bool,

    #[builder(setter(into))]
    pub key_condition_expression: String,

    #[builder(default, setter(strip_option, into))]
    pub projection_expression: Option<String>,

    #[builder(default, setter(into))]
    pub filter_expression: Option<String>,

    #[builder(default)]
    pub expression_attribute_names: Option<HashMap<String, String>>,

    #[builder(default)]
    pub expression_attribute_values: Option<HashMap<String, AttributeValue>>,

    #[builder(default = false)]
    pub consistent_read: bool,
}

#[async_trait]
pub trait Query {
    async fn query(&self, input: QueryInput) -> Result<QueryOutput, SdkError<QueryError>>;
}

#[async_trait]
impl Query for Adapter {
    async fn query(&self, input: QueryInput) -> Result<QueryOutput, SdkError<QueryError>> {
        self.raw
            .query()
            .set_table_name(input.table_name)
            .set_index_name(input.index_name)
            .limit(input.limit)
            .set_select(input.select)
            .set_exclusive_start_key(input.exclusive_start_key)
            .set_projection_expression(input.projection_expression)
            .set_expression_attribute_names(input.expression_attribute_names)
            .set_expression_attribute_values(input.expression_attribute_values)
            .consistent_read(input.consistent_read)
            .scan_index_forward(input.scan_index_forward)
            .key_condition_expression(input.key_condition_expression)
            .send()
            .await
    }
}
