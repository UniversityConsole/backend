use async_graphql_parser::types::OperationType;
use std::cmp::PartialEq;
use std::collections::BTreeMap;
use std::convert::TryFrom;
use std::fmt::{Display, Formatter, Result as FmtResult};

#[derive(Debug)]
pub struct AccessRequest {
    pub kind: AccessKind,
    pub path_set: PathSet,
}

#[derive(Debug, PartialEq)]
pub enum AccessKind {
    Query,
    Mutation,
}

#[derive(Debug, Clone, Default)]
pub struct PathSet {
    root: PathNode,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct PathNode {
    pub fields: Fields,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Fields {
    Explicit(BTreeMap<Segment, PathNode>),
    Any,
}

#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct Segment {
    pub name: String,
    pub args: Option<BTreeMap<String, Argument>>,
}

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq, Hash, Clone)]
pub struct Argument {
    pub name: String,
    pub value: ArgumentValue,
}

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq, Clone, Hash)]
pub enum ArgumentValue {
    Unsupported(String),
}

#[derive(thiserror::Error, Debug)]
pub enum AppendNodeError {
    #[error("Node already has a wildcard match on subfields. Cannot append to that.")]
    CannotAppendToAny,
}

impl PathSet {
    pub fn root(&self) -> &PathNode {
        &self.root
    }

    pub fn root_mut(&mut self) -> &mut PathNode {
        &mut self.root
    }

    pub fn extend(&mut self, path: impl IntoIterator<Item = Segment>) {
        let mut parent = &mut self.root;

        for segment in path.into_iter() {
            match &mut parent.fields {
                Fields::Explicit(fields) => {
                    parent = fields
                        .try_insert(segment, PathNode::default())
                        .map_or_else(|e| e.entry.into_mut(), |v| v);
                }
                Fields::Any => break,
            }
        }
    }
}

impl PathNode {
    pub fn explicit_match() -> Self {
        PathNode {
            fields: Fields::Explicit(BTreeMap::default()),
        }
    }

    pub fn any_match() -> Self {
        PathNode {
            fields: Fields::Any,
        }
    }

    pub fn append(&mut self, segment: Segment) -> Result<&mut Self, AppendNodeError> {
        match &mut self.fields {
            Fields::Any => Err(AppendNodeError::CannotAppendToAny),
            Fields::Explicit(fields) => Ok(fields
                .try_insert(segment, PathNode::default())
                .map_or_else(|e| e.entry.into_mut(), |v| v)),
        }
    }
}

impl Segment {
    pub fn no_args(name: impl Into<String>) -> Self {
        Segment {
            name: name.into(),
            args: None,
        }
    }

    pub fn with_args(name: impl Into<String>, args: impl IntoIterator<Item = Argument>) -> Self {
        Segment {
            name: name.into(),
            args: Some(
                args.into_iter()
                    .map(|arg| (arg.name.clone(), arg))
                    .collect(),
            ),
        }
    }
}

impl Default for Fields {
    fn default() -> Self {
        Fields::Explicit(BTreeMap::default())
    }
}
impl Fields {
    pub fn is_any(&self) -> bool {
        matches!(self, Fields::Any)
    }

    pub fn is_explicit(&self) -> bool {
        self.as_explicit().is_some()
    }

    pub fn as_explicit(&self) -> Option<&BTreeMap<Segment, PathNode>> {
        if let Fields::Explicit(fields) = self {
            Some(fields)
        } else {
            None
        }
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

impl Display for PathSet {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.root())?;
        Ok(())
    }
}

impl Display for PathNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let need_sep = match &self.fields {
            Fields::Any => true,
            Fields::Explicit(sub_fields) => !sub_fields.is_empty(),
        };
        write!(f, "{}{}", if need_sep { "::" } else { "" }, &self.fields)?;
        Ok(())
    }
}

impl Display for Fields {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Fields::Any => write!(f, "*")?,
            Fields::Explicit(sub_fields) => {
                let mut joined_sub_fields = String::default();
                let mut it = sub_fields.iter().peekable();
                while let Some((segment, sub_field)) = it.next() {
                    joined_sub_fields.push_str(&format!(
                        "{segment}{sub_field}{}",
                        if it.peek().is_some() { ", " } else { "" }
                    ));
                }

                write!(
                    f,
                    "{}{}{}",
                    if sub_fields.len() > 1 { "{" } else { "" },
                    joined_sub_fields,
                    if sub_fields.len() > 1 { "}" } else { "" }
                )?;
            }
        }
        Ok(())
    }
}

impl Display for Segment {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.name)?;

        if let Some(args) = &self.args {
            let mut joined_args = String::default();
            let mut it = args.values().peekable();
            while let Some(arg) = it.next() {
                joined_args.push_str(&format!(
                    "{arg}{}",
                    if it.peek().is_some() { ", " } else { "" }
                ));
            }
            write!(f, "({})", joined_args)?;
        }
        Ok(())
    }
}

impl Display for Argument {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}: unsupported", self.name)?;

        Ok(())
    }
}

#[cfg(test)]
mod path_tests {
    use super::*;

    #[test]
    fn extend() {
        let mut path = PathSet::default();
        path.extend([Segment::no_args("accounts"), Segment::no_args("id")]);

        assert_eq!(path.root().fields.as_explicit().unwrap().len(), 1);

        let accounts_segment = path
            .root()
            .fields
            .as_explicit()
            .unwrap()
            .get(&Segment::no_args("accounts"))
            .expect("no accounts field");

        assert_eq!(accounts_segment.fields.as_explicit().unwrap().len(), 1);

        accounts_segment
            .fields
            .as_explicit()
            .unwrap()
            .get(&Segment::no_args("id"))
            .expect("no id field");
    }

    #[test]
    fn two_extends() {
        let mut path = PathSet::default();
        path.extend([Segment::no_args("accounts"), Segment::no_args("id")]);
        path.extend([Segment::no_args("accounts"), Segment::no_args("firstName")]);
        path.extend([Segment::no_args("accounts"), Segment::no_args("lastName")]);

        assert_eq!(path.root().fields.as_explicit().unwrap().len(), 1);

        let accounts_segment = path
            .root()
            .fields
            .as_explicit()
            .unwrap()
            .get(&Segment::no_args("accounts"))
            .expect("no accounts field");

        assert_eq!(accounts_segment.fields.as_explicit().unwrap().len(), 3);

        for field in ["id", "firstName", "lastName"] {
            let node = accounts_segment
                .fields
                .as_explicit()
                .unwrap()
                .get(&Segment::no_args(field))
                .expect(format!("no {field} field").as_str());

            assert!(node.fields.as_explicit().unwrap().is_empty());
        }
    }
}

#[cfg(test)]
mod node_tests {
    use super::*;

    #[test]
    fn insert_new_segment() {
        let mut node = PathNode::default();

        assert!(node.fields.as_explicit().unwrap().is_empty());

        let child = node
            .append(Segment::no_args("foo"))
            .expect("node.append failed");

        assert!(child.fields.as_explicit().unwrap().is_empty());
        assert_eq!(node.fields.as_explicit().unwrap().len(), 1);
        assert!(node
            .fields
            .as_explicit()
            .unwrap()
            .get(&Segment::no_args("foo"))
            .is_some());
    }

    #[test]
    fn insert_existing() {
        let mut root = PathNode::default();
        let foo = root.append(Segment::no_args("foo")).expect("append failed");
        foo.append(Segment::no_args("bar")).expect("append failed");

        let second_foo = root.append(Segment::no_args("foo")).expect("append failed");

        assert_eq!(second_foo.fields.as_explicit().unwrap().len(), 1);
    }
}

#[cfg(test)]
mod string_tests {
    use super::*;

    #[test]
    fn single_root() {
        let mut path = PathSet::default();
        path.extend([Segment::no_args("accounts"), Segment::no_args("id")]);
        path.extend([Segment::no_args("accounts"), Segment::no_args("firstName")]);
        path.extend([Segment::no_args("accounts"), Segment::no_args("lastName")]);

        let expected_fmt = "::accounts::{firstName, id, lastName}".to_owned();
        assert_eq!(path.to_string(), expected_fmt);
    }

    #[test]
    fn single_root_and_args() {
        let mut path = PathSet::default();
        let account_seg = Segment::with_args(
            "account",
            vec![Argument {
                name: "id".to_owned(),
                value: ArgumentValue::Unsupported("foo".to_owned()),
            }],
        );
        path.extend([account_seg.clone(), Segment::no_args("id")]);
        path.extend([account_seg.clone(), Segment::no_args("firstName")]);
        path.extend([account_seg.clone(), Segment::no_args("lastName")]);

        let expected_fmt = "::account(id: unsupported)::{firstName, id, lastName}".to_owned();
        assert_eq!(path.to_string(), expected_fmt);
    }

    #[test]
    fn multi_root() {
        let mut path = PathSet::default();
        path.extend([Segment::no_args("accounts"), Segment::no_args("id")]);
        path.extend([Segment::no_args("accounts"), Segment::no_args("firstName")]);
        path.extend([Segment::no_args("accounts"), Segment::no_args("lastName")]);
        path.extend([Segment::no_args("courses"), Segment::no_args("id")]);
        path.extend([Segment::no_args("courses"), Segment::no_args("title")]);

        let expected_fmt =
            "::{accounts::{firstName, id, lastName}, courses::{id, title}}".to_owned();
        assert_eq!(path.to_string(), expected_fmt);
    }
}
