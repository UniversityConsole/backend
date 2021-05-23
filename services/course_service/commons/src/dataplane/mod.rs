use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "PascalCase")]
#[serde(deny_unknown_fields)]
pub struct Course {
    pub course_id: Uuid,
    pub title: String,
    pub description: String,
    #[serde(default = "Vec::new")]
    pub grading_rule: Vec<GradeComponent>,
    #[serde(default = "chrono::offset::Utc::now")]
    pub created_at: DateTime<Utc>,
    #[serde(default = "default_course_closed_at")]
    pub closed_at: Option<DateTime<Utc>>,
    pub owner_id: Uuid,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "PascalCase")]
#[serde(deny_unknown_fields)]
pub struct GradeComponent {
    pub grading_rule_id: Uuid,
    pub title: String,
    pub final_grade_percentage: f32,
}

fn default_course_closed_at() -> Option<DateTime<Utc>> {
    None
}
