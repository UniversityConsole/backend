use std::collections::HashMap;

use async_trait::async_trait;
use aws_sdk_dynamodb::error::ScanError;
use aws_sdk_dynamodb::model::{AttributeValue, Select};
use aws_sdk_dynamodb::output::ScanOutput;
use aws_sdk_dynamodb::types::SdkError;
use typed_builder::TypedBuilder;

use super::adapter::Adapter;

#[derive(Debug, TypedBuilder)]
pub struct ScanInput {
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
pub trait Scan {
    async fn scan(&self, input: ScanInput) -> Result<ScanOutput, SdkError<ScanError>>;
}

#[async_trait]
impl Scan for Adapter {
    async fn scan(&self, input: ScanInput) -> Result<ScanOutput, SdkError<ScanError>> {
        self.raw
            .scan()
            .set_table_name(input.table_name)
            .set_index_name(input.index_name)
            .limit(input.limit)
            .set_select(input.select)
            .set_exclusive_start_key(input.exclusive_start_key)
            .set_projection_expression(input.projection_expression)
            .set_filter_expression(input.filter_expression)
            .set_expression_attribute_names(input.expression_attribute_names)
            .set_expression_attribute_values(input.expression_attribute_values)
            .consistent_read(input.consistent_read)
            .send()
            .await
    }
}
