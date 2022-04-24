use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

use super::string_interop::compiler::from_string;
use super::types::PathNode;


impl Serialize for PathNode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let rendered = self.to_string();
        serializer.serialize_str(rendered.as_str())
    }
}

impl<'de> Deserialize<'de> for PathNode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let path_set = from_string(s.as_str()).map_err(|_| de::Error::custom("invalid resource path"))?;

        let mut paths = path_set.into_paths();
        paths.pop().ok_or_else(|| de::Error::custom("empty path set"))
    }
}
