use std::cmp::PartialEq;
use std::collections::BTreeMap;
use std::convert::TryFrom;
use std::fmt::{Display, Formatter, Result as FmtResult};

use async_graphql_parser::types::OperationType;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct AccessRequest {
    pub kind: AccessKind,
    pub paths: Vec<PathNode>,
}

#[derive(Debug, Serialize, Clone, PartialEq, Deserialize)]
pub struct PolicyStatement {
    pub kind: AccessKind,
    pub paths: Vec<PathNode>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum AccessKind {
    Query,
    Mutation,
}

#[derive(Debug, Clone, Default)]
pub struct PathSet {
    pub paths: BTreeMap<Segment, PathNode>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PathNode {
    pub segment: Segment,
    pub fields: BTreeMap<Segment, PathNode>,
}

#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub enum Segment {
    Named(String, Option<BTreeMap<String, Argument>>),
    Any,
}

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq, Hash, Clone)]
pub struct Argument {
    pub name: String,
    pub value: ArgumentValue,
}

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq, Clone, Hash)]
pub enum ArgumentValue {
    StringLiteral(String),
    IntegerLiteral(i64),
    BoolLiteral(bool),
    Wildcard,
}

#[derive(thiserror::Error, Debug)]
pub enum AppendNodeError {
    #[error("Node already has a wildcard match on subfields. Cannot append to that.")]
    CannotAppendToAny,
}

impl PathSet {
    pub fn extend(&mut self, path: impl IntoIterator<Item = Segment>) -> Result<(), AppendNodeError> {
        let mut it = path.into_iter();
        if let Some(first_segment) = it.next() {
            let mut parent = self
                .paths
                .entry(first_segment.clone())
                .or_insert_with(|| PathNode::new(first_segment));

            for segment in it {
                if matches!(segment, Segment::Any) && !parent.fields.is_empty() {
                    parent.fields.clear();
                }

                parent = parent.append(segment)?;
            }
        }

        Ok(())
    }

    pub fn paths(&self) -> Vec<&PathNode> {
        self.paths.values().collect()
    }

    pub fn into_paths(self) -> Vec<PathNode> {
        self.paths.into_values().collect()
    }
}

impl PathNode {
    pub fn new(segment: Segment) -> Self {
        PathNode {
            segment,
            fields: Default::default(),
        }
    }

    pub fn append(&mut self, segment: Segment) -> Result<&mut Self, AppendNodeError> {
        if matches!(self.segment, Segment::Any) {
            return Err(AppendNodeError::CannotAppendToAny);
        }

        if matches!(segment, Segment::Any) && !self.fields.is_empty() {
            self.fields.clear();
        }

        Ok(self
            .fields
            .try_insert(segment.clone(), PathNode::new(segment))
            .map_or_else(|e| e.entry.into_mut(), |v| v))
    }

    pub fn fields(&self) -> Vec<&PathNode> {
        self.fields.values().collect()
    }

    pub fn into_fields(self) -> Vec<PathNode> {
        self.fields.into_values().collect()
    }
}

impl Segment {
    pub fn no_args(name: impl Into<String>) -> Self {
        Segment::Named(name.into(), None)
    }

    pub fn with_args(name: impl Into<String>, args: impl IntoIterator<Item = Argument>) -> Self {
        Segment::Named(
            name.into(),
            Some(args.into_iter().map(|arg| (arg.name.clone(), arg)).collect()),
        )
    }
}

impl TryFrom<OperationType> for AccessKind {
    type Error = ();

    fn try_from(operation_type: OperationType) -> Result<Self, Self::Error> {
        match operation_type {
            OperationType::Query => Ok(AccessKind::Query),
            OperationType::Mutation => Ok(AccessKind::Mutation),
            _ => Err(()),
        }
    }
}

impl Display for PathNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let need_sep = !self.fields.is_empty();
        let mut joined_sub_fields = String::default();
        let mut it = self.fields.values().peekable();
        while let Some(sub_field) = it.next() {
            joined_sub_fields.push_str(&format!("{sub_field}{}", if it.peek().is_some() { ", " } else { "" }));
        }

        write!(
            f,
            "{}{}{}{}{}",
            self.segment,
            if need_sep { "::" } else { "" },
            if self.fields.len() > 1 { "{" } else { "" },
            joined_sub_fields,
            if self.fields.len() > 1 { "}" } else { "" }
        )?;

        Ok(())
    }
}

impl Display for Segment {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match &self {
            Segment::Any => write!(f, "*")?,
            Segment::Named(name, args) => {
                write!(f, "{}", name)?;

                if let Some(args) = &args {
                    let mut joined_args = String::default();
                    let mut it = args.values().peekable();
                    while let Some(arg) = it.next() {
                        joined_args.push_str(&format!("{arg}{}", if it.peek().is_some() { ", " } else { "" }));
                    }
                    write!(f, "({})", joined_args)?;
                }
            }
        }

        Ok(())
    }
}

impl Display for Argument {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}: {}", self.name, self.value)?;

        Ok(())
    }
}

impl Display for ArgumentValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            ArgumentValue::BoolLiteral(val) => write!(f, "{}", val)?,
            ArgumentValue::IntegerLiteral(val) => write!(f, "{}", val)?,
            // FIXME Properly escape characters.
            ArgumentValue::StringLiteral(val) => write!(f, "\"{}\"", val)?,
            ArgumentValue::Wildcard => write!(f, "*")?,
        };

        Ok(())
    }
}

#[cfg(test)]
mod path_tests {
    use super::*;

    #[test]
    fn extend() {
        let mut path = PathSet::default();
        path.extend([Segment::no_args("accounts"), Segment::no_args("id")])
            .unwrap();

        assert_eq!(path.paths.len(), 1);

        let accounts_segment = path
            .paths
            .get(&Segment::no_args("accounts"))
            .expect("accounts root does not exist");

        assert_eq!(accounts_segment.fields.len(), 1);

        accounts_segment
            .fields
            .get(&Segment::no_args("id"))
            .expect("no id field");
    }

    #[test]
    fn two_extends() {
        let mut path_set = PathSet::default();
        path_set
            .extend([Segment::no_args("accounts"), Segment::no_args("id")])
            .unwrap();
        path_set
            .extend([Segment::no_args("accounts"), Segment::no_args("firstName")])
            .unwrap();
        path_set
            .extend([Segment::no_args("accounts"), Segment::no_args("lastName")])
            .unwrap();

        assert_eq!(path_set.paths.len(), 1);

        let accounts_segment = path_set
            .paths
            .get(&Segment::no_args("accounts"))
            .expect("no accounts field");

        assert_eq!(accounts_segment.fields.len(), 3);

        for field in ["id", "firstName", "lastName"] {
            let node = accounts_segment
                .fields
                .get(&Segment::no_args(field))
                .unwrap_or_else(|| panic!("no {field} field"));

            assert!(node.fields.is_empty());
        }
    }
}

#[cfg(test)]
mod node_tests {
    use super::*;

    #[test]
    fn insert_new_segment() {
        let mut foo = PathNode::new(Segment::no_args("foo"));

        assert!(matches!(foo.segment, Segment::Named(..)));
        assert!(foo.fields.is_empty());

        let bar = foo.append(Segment::no_args("bar")).expect("foo.append failed");

        assert!(bar.fields.is_empty());
        assert_eq!(foo.fields.len(), 1);
        assert!(foo.fields.get(&Segment::no_args("bar")).is_some());
    }

    #[test]
    fn insert_existing() {
        let mut root = PathNode::new(Segment::no_args("root"));
        let foo = root.append(Segment::no_args("foo")).expect("append failed");
        foo.append(Segment::no_args("bar")).expect("append failed");

        let second_foo = root.append(Segment::no_args("foo")).expect("append failed");

        assert_eq!(second_foo.fields.len(), 1);
    }

    #[test]
    fn insert_after_any() {
        let mut root = PathNode::new(Segment::Any);
        assert!(matches!(
            root.append(Segment::no_args("foo")),
            Err(AppendNodeError::CannotAppendToAny)
        ));
    }

    #[test]
    fn insert_any_in_populated_selection() {
        let mut root = PathNode::new(Segment::no_args("foo"));
        assert!(root.append(Segment::no_args("bar")).is_ok());
        assert!(root.append(Segment::Any).is_ok());

        assert_eq!(root.fields.len(), 1);

        let mut fields = root.fields.into_iter();
        let (key, value) = fields.next().expect("should have one field");
        assert!(matches!(key, Segment::Any));
        assert!(matches!(value.segment, Segment::Any));
    }
}

#[cfg(test)]
#[allow(unused_must_use)]
mod string_tests {
    use super::*;

    #[test]
    fn single_root() {
        let mut path_set = PathSet::default();
        path_set.extend([Segment::no_args("accounts"), Segment::no_args("id")]);
        path_set.extend([Segment::no_args("accounts"), Segment::no_args("firstName")]);
        path_set.extend([Segment::no_args("accounts"), Segment::no_args("lastName")]);

        assert_eq!(path_set.paths.len(), 1);

        let paths = path_set.into_paths();
        let path = paths.first().unwrap();
        let expected_fmt = "accounts::{firstName, id, lastName}".to_owned();
        assert_eq!(path.to_string(), expected_fmt);
    }

    #[test]
    fn single_root_and_args() {
        let mut path_set = PathSet::default();
        let account_seg = Segment::with_args(
            "account",
            vec![Argument {
                name: "id".to_owned(),
                value: ArgumentValue::StringLiteral("foo".to_owned()),
            }],
        );
        path_set.extend([account_seg.clone(), Segment::no_args("id")]);
        path_set.extend([account_seg.clone(), Segment::no_args("firstName")]);
        path_set.extend([account_seg.clone(), Segment::no_args("lastName")]);

        let first = path_set
            .paths
            .get(&account_seg)
            .expect("there should be one path in the path set");
        let expected_fmt = "account(id: \"foo\")::{firstName, id, lastName}".to_owned();
        assert_eq!(first.to_string(), expected_fmt);
    }

    #[test]
    fn multi_root() {
        let mut path_set = PathSet::default();
        path_set.extend([Segment::no_args("foo"), Segment::no_args("a")]);
        path_set.extend([Segment::no_args("foo"), Segment::no_args("b")]);
        path_set.extend([Segment::no_args("foo"), Segment::no_args("c")]);
        path_set.extend([Segment::no_args("bar"), Segment::no_args("a")]);
        path_set.extend([Segment::no_args("bar"), Segment::no_args("b")]);

        let expected_fmt = ["bar::{a, b}", "foo::{a, b, c}"];

        for (path, expected) in std::iter::zip(path_set.paths(), expected_fmt) {
            assert_eq!(path.to_string(), expected.to_owned());
        }
    }
}
