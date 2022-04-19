use thiserror::Error;

use super::string_parser::{parse, types as parser_types, ParseError};
use super::types::{AppendNodeError, PathSet, Segment};

#[derive(Error, Debug)]
pub enum CompileError {
    #[error("Cannot match fields uner an any-match.")]
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

    use super::types::{Argument, ArgumentValue};

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
                                        FieldArgValue::Wildcard => ArgumentValue::Wildcard,
                                    },
                                },
                            )
                        })
                        .collect()
                }),
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
    fn playground() {
        println!(
            "{:#?}",
            from_string("accounts(includeNonDiscoverable: true)::{a(id: *), b, c}")
        );
    }
}
