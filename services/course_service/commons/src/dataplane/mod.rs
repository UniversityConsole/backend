use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "PascalCase")]
#[serde(deny_unknown_fields)]
pub struct Course {
    pub course_id: Uuid,
    pub title: String,
    pub description: String,
    #[serde(default = "Vec::new")]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub grading_rule: Vec<GradeComponent>,
    #[serde(default = "chrono::offset::Utc::now")]
    pub created_at: DateTime<Utc>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub closed_at: Option<DateTime<Utc>>,
    pub owner_id: Uuid,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "PascalCase")]
#[serde(deny_unknown_fields)]
pub struct GradeComponent {
    pub grade_component_id: Uuid,
    pub title: String,
    pub final_grade_percentage: f32,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "PascalCase")]
#[serde(deny_unknown_fields)]
pub struct CourseEnrollment {
    pub course_id: Uuid,
    pub user_account_id: Uuid,
    pub enrolled_at: DateTime<Utc>,
    #[serde(default)]
    pub grades: HashMap<Uuid, Vec<Grade>>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "PascalCase")]
#[serde(deny_unknown_fields)]
pub struct Grade {
    pub value: u8,
    pub timestamp: DateTime<Utc>,
}
