use std::collections::HashMap;

use rusoto_dynamodb::AttributeValue;

pub trait Document {
    fn document(&self) -> HashMap<String, AttributeValue>;
}
