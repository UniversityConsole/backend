use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
}
