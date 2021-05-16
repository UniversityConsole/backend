use rusoto_dynamodb::AttributeValue;
use std::collections::HashMap;

pub trait Document {
    fn document(&self) -> HashMap<String, AttributeValue>;
}
