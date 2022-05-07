use std::fmt::{Display, Formatter};

use uuid::Uuid;

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub struct RequestId(Uuid);

impl Display for RequestId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.as_hyphenated())
    }
}

impl Default for RequestId {
    fn default() -> Self {
        Self(Uuid::new_v4())
    }
}
