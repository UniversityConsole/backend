use bytes::Bytes;
use rusoto_dynamodb::AttributeValue;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha512};
use simple_error::SimpleError;
use std::collections::HashMap;
use std::convert::TryFrom;
use utils::dynamodb_interop::Document;
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
#[serde(deny_unknown_fields)]
pub struct Course {
    pub course_id: Uuid,
    pub title: String,
    pub description: String,
    #[serde(default = "Vec::new")]
    pub grading_rule: Vec<GradeComponent>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
#[serde(deny_unknown_fields)]
pub struct GradeComponent {
    pub grading_rule_id: Uuid,
    pub title: string,
    pub final_grade_percentage: f32,
}

impl Document for Course {
    fn document(&self) -> HashMap<String, AttributeValue> {
        let mut m = HashMap::new();
        m.insert(
            "CourseId".to_string(),
            AttributeValue {
                b: Some(Bytes::copy_from_slice(self.course_id.as_bytes())),
                ..AttributeValue::default()
            },
        );
        m.insert(
            "Title".to_string(),
            AttributeValue {
                s: Some(self.title.clone()),
                ..AttributeValue::default()
            },
        );
        m.insert(
            "Description".to_string(),
            AttributeValue {
                s: Some(self.description.clone()),
                ..AttributeValue::default()
            },
        );

        m
    }

}

impl Document for GradeComponent {

}
