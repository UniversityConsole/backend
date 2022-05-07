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

// impl Value for RequestId {
//     fn record(&self, key: &Field, visitor: &mut dyn Visit) {
//         visitor.record_value(key, display(self));
//     }
// }
