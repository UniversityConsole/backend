use async_graphql_value::value;
use thiserror::Error;

use super::{parse, types as parser_types, ParseError};
use crate::resource_access::types::{AppendNodeError, Argument, ArgumentValue, PathSet, Segment};

#[derive(Error, Debug)]
pub enum CompileError {
    #[error("Cannot match fields under an any-match.")]
    CannotAppendToAny,

    #[error("Unknown.")]
    Unknown,
}

impl From<AppendNodeError> for CompileError {
    fn from(err: AppendNodeError) -> Self {
        match err {
            AppendNodeError::CannotAppendToAny => CompileError::CannotAppendToAny,
        }
    }
}


pub fn from_string(s: &str) -> Result<PathSet, CompileError> {
    let expression = parse(s).map_err(|e| match e {
        ParseError::Unknown => CompileError::Unknown,
    })?;

    let mut path_set = PathSet::default();
    tr_expression(&mut path_set, vec![], expression)?;

    Ok(path_set)
}

fn tr_expression(
    path_set: &mut PathSet,
    path: Vec<Segment>,
    expr: parser_types::Expression<'_>,
) -> Result<(), CompileError> {
    match expr {
        parser_types::Expression::SelectionSet(selection_set) => {
            tr_singular_selection_set(path_set, path, selection_set)
        }
    }
}

fn tr_selection_set(
    path_set: &mut PathSet,
    path: Vec<Segment>,
    selection_set: parser_types::SelectionSet<'_>,
) -> Result<(), CompileError> {
    match selection_set {
        parser_types::SelectionSet::Singular(singular) => tr_singular_selection_set(path_set, path, singular),
        parser_types::SelectionSet::Multi(list) => {
            for singular in list {
                tr_singular_selection_set(path_set, path.clone(), singular)?;
            }

            Ok(())
        }
    }
}

fn tr_singular_selection_set(
    path_set: &mut PathSet,
    mut path: Vec<Segment>,
    selection: parser_types::SingularSelectionSet<'_>,
) -> Result<(), CompileError> {
    use parser_types::{Field, FieldArgValue, SingularSelectionSet};

    match selection {
        SingularSelectionSet::Wildcard => {
            path.push(Segment::Any);
            path_set.extend(path)?;
        }
        SingularSelectionSet::Explicit(field, subsel) => {
            let Field { name, args } = field;
            path.push(Segment::Named(
                name.to_owned(),
                args.map(|args| {
                    args.into_iter()
                        .map(|arg| {
                            (
                                arg.name.to_owned(),
                                Argument {
                                    name: arg.name.to_owned(),
                                    value: match arg.value {
                                        FieldArgValue::BoolLiteral(value) => ArgumentValue::BoolLiteral(value),
                                        FieldArgValue::IntegerLiteral(value) => ArgumentValue::IntegerLiteral(value),
                                        FieldArgValue::StringLiteral(value) => ArgumentValue::StringLiteral(value),
                                        FieldArgValue::Enum(name) => ArgumentValue::Enum(name),
                                        FieldArgValue::Wildcard => ArgumentValue::Wildcard,
                                    },
                                },
                            )
                        })
                        .collect()
                })
                .unwrap_or_default(),
            ));

            match *subsel {
                Some(selection_set) => tr_selection_set(path_set, path, selection_set)?,
                None => path_set.extend(path)?,
            };
        }
    };

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_expression() {
        let raw = "accounts(includeNonDiscoverable: true)::{a(id: *), b}";
        let roots = from_string(raw).expect("parse failed").into_paths();
        let root = roots.first().expect("path_set empty");

        assert!(matches!(root.segment, Segment::Named(..)));

        let Segment::Named(field, args) = &root.segment else { panic!("segment is not named") };
        assert_eq!(field.as_str(), "accounts");
        assert_eq!(args.len(), 1);

        let Some(arg) = args.get(&"includeNonDiscoverable".to_owned()) else { panic!("no argS") };
        assert_eq!(arg.name.as_str(), "includeNonDiscoverable");
        assert!(matches!(arg.value, ArgumentValue::BoolLiteral(true)));

        let fields = root.fields();
        let Some(first_field) = fields.get(0) else { panic!("no field a") };
        let Segment::Named(field, args) = &first_field.segment else { panic!("segment not named") };
        assert_eq!(field.as_str(), "a");
        assert_eq!(args.len(), 1);

        let Some(first_field) = fields.get(1) else { panic!("no field b") };
        let Segment::Named(field, args) = &first_field.segment else { panic!("segment not named") };
        assert_eq!(field.as_str(), "b");
        assert!(args.is_empty());
    }
}
