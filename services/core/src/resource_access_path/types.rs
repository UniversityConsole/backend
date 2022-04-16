use async_graphql_parser::types::OperationType;
use std::cmp::PartialEq;
use std::collections::BTreeMap;
use std::convert::TryFrom;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct AccessRequest {
    pub kind: AccessKind,
    pub paths: Vec<ResourceAccessPath>,
}

#[derive(Debug, PartialEq)]
pub enum AccessKind {
    Query,
    Mutation,
}

#[derive(Debug, Clone)]
pub struct ResourceAccessPath {
    pub segment_name: String,
    pub args: Option<Vec<FieldArgument>>,
    pub children: BTreeMap<String, ResourceAccessPath>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum FieldArgument {
    Unsupported(String),
}

impl ResourceAccessPath {
    pub fn new(
        segment_name: impl Into<String>,
        args: Option<Vec<FieldArgument>>,
        children: impl IntoIterator<Item = ResourceAccessPath>,
    ) -> Self {
        ResourceAccessPath {
            segment_name: segment_name.into(),
            args,
            children: children
                .into_iter()
                .map(|p| (p.segment_name.clone(), p))
                .collect(),
        }
    }

    #[cfg(test)]
    fn no_args(
        segment_name: impl Into<String>,
        children: impl IntoIterator<Item = ResourceAccessPath>,
    ) -> Self {
        ResourceAccessPath {
            segment_name: segment_name.into(),
            args: None,
            children: children
                .into_iter()
                .map(|p| (p.segment_name.clone(), p))
                .collect(),
        }
    }
}

impl PartialEq for ResourceAccessPath {
    fn eq(&self, other: &Self) -> bool {
        // Terribly inefficient solution until I figure out what we need exactly.

        if self.segment_name != other.segment_name || self.args != other.args {
            return false;
        }

        if self.children.len() != other.children.len() {
            return false;
        }

        std::iter::zip(self.children.values(), other.children.values())
            .map_while(|p| {
                let (a, b) = &p;
                if a == b {
                    Some(())
                } else {
                    None
                }
            })
            .count()
            == self.children.len()
    }
}

impl Display for ResourceAccessPath {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", &self.segment_name)?;

        if let Some(args) = &self.args {
            write!(f, "(")?;

            let mut it = args.iter().peekable();
            while let Some(arg) = it.next() {
                write!(f, "{}{}", &arg, if it.peek().is_some() { ", " } else { "" })?;
            }

            write!(f, ")")?;
        }

        if !self.children.is_empty() {
            write!(f, "::")?;
        }

        if self.children.len() > 1 {
            write!(f, "{{")?;
        }

        let mut it = self.children.values().peekable();
        while let Some(child) = it.next() {
            write!(
                f,
                "{}{}",
                &child,
                if it.peek().is_some() { ", " } else { "" }
            )?;
        }

        if self.children.len() > 1 {
            write!(f, "}}")?;
        }

        Ok(())
    }
}

impl Display for FieldArgument {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "UNSUPPORTED")?;

        Ok(())
    }
}

impl TryFrom<OperationType> for AccessKind {
    type Error = ();

    fn try_from(value: OperationType) -> Result<Self, Self::Error> {
        match value {
            OperationType::Query => Ok(AccessKind::Query),
            OperationType::Mutation => Ok(AccessKind::Mutation),
            _ => Err(()),
        }
    }
}

#[cfg(test)]
mod equality_tests {
    use super::*;

    #[test]
    fn single_segment_equal_paths() {
        let a = ResourceAccessPath::no_args("foo", vec![]);
        let b = ResourceAccessPath::no_args("foo", vec![]);

        assert!(a == b);
    }

    #[test]
    fn single_segment_not_equal_paths() {
        let a = ResourceAccessPath::no_args("foo", vec![]);
        let b = ResourceAccessPath::no_args("bar", vec![]);
        let c = ResourceAccessPath::new(
            "foo",
            Some(vec![FieldArgument::Unsupported("foo".to_owned())]),
            vec![],
        );

        assert_ne!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn one_single_segment_one_more() {
        let a = ResourceAccessPath::no_args("a", vec![]);
        let b = ResourceAccessPath::no_args("a", vec![ResourceAccessPath::no_args("bar", vec![])]);

        assert_ne!(a, b);
    }

    #[test]
    fn deep_equal_paths() {
        // a::{b::d, c}
        let a = ResourceAccessPath::no_args(
            "a",
            vec![
                ResourceAccessPath::no_args("b", vec![ResourceAccessPath::no_args("d", vec![])]),
                ResourceAccessPath::no_args("c", vec![]),
            ],
        );
        let b = ResourceAccessPath::no_args(
            "a",
            vec![
                ResourceAccessPath::no_args("c", vec![]),
                ResourceAccessPath::no_args("b", vec![ResourceAccessPath::no_args("d", vec![])]),
            ],
        );

        assert_eq!(a, b);
    }

    #[test]
    fn deep_not_equal_paths() {
        // a::{b::d, c} vs. a::{e::d, c}
        let a = ResourceAccessPath::no_args(
            "a",
            vec![
                ResourceAccessPath::no_args("b", vec![ResourceAccessPath::no_args("d", vec![])]),
                ResourceAccessPath::no_args("c", vec![]),
            ],
        );
        let b = ResourceAccessPath::no_args(
            "a",
            vec![
                ResourceAccessPath::no_args("c", vec![]),
                ResourceAccessPath::no_args("e", vec![ResourceAccessPath::no_args("d", vec![])]),
            ],
        );

        assert_ne!(a, b);
    }
}

#[cfg(test)]
mod display_test {
    use super::*;
    use std::string::ToString;

    #[test]
    fn single_segment() {
        let p = ResourceAccessPath::new("foo", None, vec![]);
        let s = "foo".to_owned();

        assert_eq!(s, p.to_string());
    }

    #[test]
    fn linear() {
        let p = ResourceAccessPath::new(
            "a",
            None,
            vec![ResourceAccessPath::new(
                "b",
                None,
                vec![ResourceAccessPath::new("c", None, vec![])],
            )],
        );
        let s = "a::b::c".to_owned();

        assert_eq!(s, p.to_string());
    }

    #[test]
    fn tree_like() {
        let p = ResourceAccessPath::new(
            "a",
            None,
            vec![
                ResourceAccessPath::new("z", None, vec![]),
                ResourceAccessPath::new(
                    "c",
                    None,
                    vec![
                        ResourceAccessPath::new("d", None, vec![]),
                        ResourceAccessPath::new("e", None, vec![]),
                    ],
                ),
            ],
        );
        let s = "a::{c::{d, e}, z}".to_owned();

        assert_eq!(s, p.to_string());
    }

    #[test]
    fn empty_args() {
        let p = ResourceAccessPath::new(
            "foo",
            Some(vec![]),
            vec![ResourceAccessPath::new("bar", None, vec![])],
        );
        let s = "foo()::bar".to_owned();

        assert_eq!(s, p.to_string());
    }
}
