use std::cmp::PartialEq;
use std::collections::btree_map::OccupiedError;
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

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, Copy)]
pub enum AccessKind {
    Query,
    Mutation,
}

#[derive(Debug, Clone, Default)]
pub struct PathSet {
    pub(crate) paths: BTreeMap<Segment, PathNode>,
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

    pub fn merge_path_set(&mut self, other: PathSet) {
        for other_path in other.into_paths() {
            self.merge_path_node(other_path);
        }
    }

    pub fn merge_path_node(&mut self, other: PathNode) {
        match self.paths.try_insert(other.segment.clone(), other) {
            Ok(_) => {}
            Err(OccupiedError { mut entry, value }) => entry.get_mut().merge(value),
        };
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

    pub fn merge(&mut self, mut other: PathNode) {
        if self.fields.contains_key(&Segment::Any) {
            // Merging other path on an any-match segment wouldn't change it's effect.
            return;
        }

        if other.fields.contains_key(&Segment::Any) && !self.fields.is_empty() {
            // If we merge an any-match into anything else, it will become an any-match.
            self.fields.clear();
            self.fields.append(&mut other.fields);
            return;
        }

        if self.fields.is_empty() {
            self.fields.append(&mut other.fields);
            return;
        }

        for a in self.fields.values_mut() {
            if let Some(b) = other.fields.remove(&a.segment) {
                a.merge(b)
            }
        }

        self.fields.append(&mut other.fields);
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

impl Argument {
    fn new(name: impl Into<String>, value: ArgumentValue) -> Self {
        Argument {
            name: name.into(),
            value,
        }
    }
}


pub trait Superset {
    fn is_superset_of(&self, other: &Self) -> bool;
}


impl Superset for PathSet {
    fn is_superset_of(&self, other: &Self) -> bool {
        if other.paths.len() > self.paths.len() {
            return false;
        }

        for l in self.paths.values() {
            if let Some(r) = other.paths.get(&l.segment) {
                if !l.is_superset_of(r) {
                    return false;
                }
            } else {
                return false;
            }
        }

        true
    }
}

impl Superset for PathNode {
    fn is_superset_of(&self, other: &Self) -> bool {
        if self.fields.contains_key(&Segment::Any) {
            // An any-match is superset of anything else.
            return true;
        }

        if other.fields.contains_key(&Segment::Any) {
            // Other node has an any-match while self doesn't.
            return false;
        }

        if !self.segment.is_superset_of(&other.segment) {
            return false;
        }

        for l in self.fields.values() {
            if let Some(r) = other.fields.get(&l.segment) {
                if !l.is_superset_of(r) {
                    println!("returning false in if");
                    return false;
                }
            }
        }

        true
    }
}

impl Superset for Segment {
    fn is_superset_of(&self, other: &Self) -> bool {
        println!("left = {:?} right = {:?}", &self, &other);

        match (self, other) {
            (Segment::Any, _) => true,
            (_, Segment::Any) => false,
            (Segment::Named(lname, largs), Segment::Named(rname, rargs)) => {
                if lname != rname {
                    false
                } else {
                    println!("largs = {:?}, rargs = {:?}", &largs, &rargs);

                    match (largs, rargs) {
                        (None, None) => true,
                        // Incompatible fields arguments are not comparable.
                        (Some(_), None) | (None, Some(_)) => false,
                        (Some(largs), Some(rargs)) => {
                            for larg in largs.values() {
                                match rargs.get(&larg.name) {
                                    // Incompatible fields arguments are not comparable.
                                    None => return false,
                                    Some(rarg) => {
                                        if !larg.is_superset_of(rarg) {
                                            return false;
                                        }
                                    }
                                }
                            }

                            true
                        }
                    }
                }
            }
        }
    }
}

impl Superset for Argument {
    fn is_superset_of(&self, other: &Self) -> bool {
        if self.name != other.name {
            panic!("called is_superset_of for arguments with different names");
        } else {
            self.value.is_superset_of(&other.value)
        }
    }
}

impl Superset for ArgumentValue {
    fn is_superset_of(&self, other: &Self) -> bool {
        match (self, other) {
            (ArgumentValue::Wildcard, _) => true,
            _ => self == other,
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

#[cfg(test)]
#[allow(unused_must_use)]
mod merge_tests {
    use super::*;
    use crate::resource_access::string_interop::compiler::from_string;

    #[test]
    fn merge_disjoint() {
        let mut a = from_string("a::b::c").unwrap();
        let b = from_string("d::e::f").unwrap();

        a.merge_path_set(b);

        assert_eq!(a.paths().len(), 2);

        let rendered_paths: Vec<String> = a.into_paths().into_iter().map(|v| v.to_string()).collect();
        let expected_paths: Vec<String> = vec!["a::b::c", "d::e::f"].into_iter().map(|v| v.to_string()).collect();
        assert_eq!(rendered_paths, expected_paths);
    }

    #[test]
    fn merge_common_root_disjoint() {
        let mut a = from_string("a::b::c").unwrap();
        let b = from_string("a::d::e").unwrap();

        a.merge_path_set(b);

        assert_eq!(a.paths().len(), 1);
        let Some(a) = a.into_paths().first() else { unreachable!() };

        assert_eq!(a.to_string(), "a::{b::c, d::e}".to_owned());
    }

    #[test]
    fn merge_common_root_into_any_match() {
        let mut a = from_string("a::*").unwrap();
        let b = from_string("a::b::c").unwrap();

        a.merge_path_set(b);

        assert_eq!(a.paths().len(), 1);
        let Some(a) = a.into_paths().first() else { unreachable!() };

        assert_eq!(a.to_string(), "a::*".to_owned());
    }

    #[test]
    fn merge_common_root_any_match_into_fields() {
        let mut a = from_string("a::{b, c}").unwrap();
        let b = from_string("a::*").unwrap();

        a.merge_path_set(b);

        assert_eq!(a.paths().len(), 1);
        let Some(a) = a.into_paths().first() else { unreachable!() };

        assert_eq!(a.to_string(), "a::*".to_owned());
    }

    #[test]
    fn merge_subset_of_children() {
        let mut a = from_string("a::{b, c}").unwrap();
        let b = from_string("a::c").unwrap();

        a.merge_path_set(b);

        assert_eq!(a.paths().len(), 1);
        let Some(a) = a.into_paths().first() else { unreachable!() };

        assert_eq!(a.to_string(), "a::{b, c}".to_owned());
    }

    #[test]
    fn merge_disjoint_children() {
        let mut a = from_string("a::{b, c}").unwrap();
        let b = from_string("a::d").unwrap();

        a.merge_path_set(b);

        assert_eq!(a.paths().len(), 1);
        let Some(a) = a.into_paths().first() else { unreachable!() };

        assert_eq!(a.to_string(), "a::{b, c, d}".to_owned());
    }

    #[test]
    fn merge_intersecting_children() {
        let mut a = from_string("a::{b, c}").unwrap();
        let b = from_string("a::{c, d::e}").unwrap();

        a.merge_path_set(b);

        assert_eq!(a.paths().len(), 1);
        let Some(a) = a.into_paths().first() else { unreachable!() };

        assert_eq!(a.to_string(), "a::{b, c, d::e}".to_owned());
    }

    #[test]
    fn merge_superset_of_children() {
        let mut a = from_string("a::{b, c}").unwrap();
        let b = from_string("a::{a, b, c, d}").unwrap();

        a.merge_path_set(b);

        assert_eq!(a.paths().len(), 1);
        let Some(a) = a.into_paths().first() else { unreachable!() };

        assert_eq!(a.to_string(), "a::{a, b, c, d}".to_owned());
    }
}

#[cfg(test)]
mod superset_tests {
    #![allow(unused_imports)]

    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case(ArgumentValue::Wildcard, ArgumentValue::Wildcard, true)]
    #[case(ArgumentValue::Wildcard, ArgumentValue::StringLiteral("abc".to_owned()), true)]
    #[case(ArgumentValue::Wildcard, ArgumentValue::IntegerLiteral(12), true)]
    #[case(ArgumentValue::Wildcard, ArgumentValue::BoolLiteral(true), true)]
    #[case(ArgumentValue::BoolLiteral(false), ArgumentValue::Wildcard, false)]
    #[case(ArgumentValue::BoolLiteral(false), ArgumentValue::BoolLiteral(true), false)]
    #[case(ArgumentValue::BoolLiteral(false), ArgumentValue::BoolLiteral(false), true)]
    fn argument_value(#[case] a: ArgumentValue, #[case] b: ArgumentValue, #[case] expected: bool) {
        assert_eq!(a.is_superset_of(&b), expected);
    }

    #[test]
    #[should_panic]
    fn arguments_with_different_names() {
        let a = Argument {
            name: "a".to_string(),
            value: ArgumentValue::Wildcard,
        };
        let b = Argument {
            name: "b".to_string(),
            value: ArgumentValue::Wildcard,
        };

        a.is_superset_of(&b);
    }

    #[test]
    fn argument_expected_true() {
        let a = Argument {
            name: "a".to_string(),
            value: ArgumentValue::Wildcard,
        };
        let b = Argument {
            name: "a".to_string(),
            value: ArgumentValue::Wildcard,
        };

        assert!(a.is_superset_of(&b));
    }

    #[rstest]
    #[case(Segment::Any, Segment::no_args("a"), true)]
    #[case(Segment::Any, Segment::Any, true)]
    #[case(Segment::no_args("a"), Segment::Any, false)]
    #[case(Segment::no_args("a"), Segment::no_args("b"), false)]
    #[case(Segment::no_args("a"), Segment::no_args("a"), true)]
    fn segment_simple(#[case] a: Segment, #[case] b: Segment, #[case] expected: bool) {
        assert_eq!(a.is_superset_of(&b), expected);
    }

    #[test]
    fn segments_with_same_arg_names() {
        let a = Segment::with_args(
            "a",
            [
                Argument::new("a", ArgumentValue::Wildcard),
                Argument::new("b", ArgumentValue::Wildcard),
            ],
        );
        let b = Segment::with_args(
            "a",
            [
                Argument::new("a", ArgumentValue::BoolLiteral(true)),
                Argument::new("b", ArgumentValue::IntegerLiteral(10)),
            ],
        );

        assert!(a.is_superset_of(&b));
    }

    #[test]
    fn segment_with_more_args() {
        let a = Segment::with_args(
            "a",
            [
                Argument::new("a", ArgumentValue::Wildcard),
                Argument::new("b", ArgumentValue::Wildcard),
                Argument::new("c", ArgumentValue::Wildcard),
            ],
        );
        let b = Segment::with_args(
            "a",
            [
                Argument::new("a", ArgumentValue::BoolLiteral(true)),
                Argument::new("b", ArgumentValue::IntegerLiteral(10)),
            ],
        );

        assert!(!a.is_superset_of(&b));
        assert!(!b.is_superset_of(&a));
    }

    #[test]
    fn segment_with_some_overlapping_args() {
        let a = Segment::with_args(
            "a",
            [
                Argument::new("a", ArgumentValue::Wildcard),
                Argument::new("b", ArgumentValue::Wildcard),
                Argument::new("c", ArgumentValue::Wildcard),
            ],
        );
        let b = Segment::with_args(
            "a",
            [
                Argument::new("b", ArgumentValue::IntegerLiteral(10)),
                Argument::new("c", ArgumentValue::BoolLiteral(true)),
                Argument::new("d", ArgumentValue::BoolLiteral(true)),
            ],
        );

        assert!(!a.is_superset_of(&b));
        assert!(!b.is_superset_of(&a));
    }

    #[test]
    fn segment_with_args_vs_no_args() {
        let a = Segment::with_args(
            "a",
            [
                Argument::new("a", ArgumentValue::Wildcard),
                Argument::new("b", ArgumentValue::Wildcard),
                Argument::new("c", ArgumentValue::Wildcard),
            ],
        );
        let b = Segment::no_args("a");

        assert!(!a.is_superset_of(&b));
        assert!(!b.is_superset_of(&a));
    }

    #[rstest]
    #[case("a::b", "a::b", true)]
    #[case("a::{b, c}", "a::b", true)]
    #[case("a::{b, c}", "a::{b, c}", true)]
    #[case("a::*", "a::b", true)]
    #[case("a::*", "a::{b, c}", true)]
    #[case("a::*", "a::*", true)]
    #[case("a::{b::*, c}", "a::*", false)]
    #[case("a::{b::*, c}", "a::c", true)]
    #[case("a::{b::*, c}", "a::b::d", true)]
    #[case("a::b(foo: *)", "a::b", false)]
    #[case("a::b(foo: *)", "a::b(foo: *)", true)]
    #[case("a::b(foo: *)", "a::b(foo: \"a\")", true)]
    #[case("a::b(foo: *)", "a::b(foo: true)", true)]
    #[case("a::b(foo: *)", "a::b(foo: 10)", true)]
    #[case("a::b(foo: 10)", "a::b(foo: 10)", true)]
    fn path_set_simple(#[case] a: &str, #[case] b: &str, #[case] expected: bool) {
        use crate::resource_access::string_interop::compiler::from_string;
        let a = from_string(a).expect("failed parsing a");
        let b = from_string(b).expect("failed parsing b");

        assert_eq!(a.is_superset_of(&b), expected);
    }
}
