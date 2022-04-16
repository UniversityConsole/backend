use async_graphql_parser::types::{OperationDefinition, Selection};
use std::result::Result;
use thiserror::Error;

#[derive(Debug)]
pub struct Segment {
    pub name: String,
    pub args: Option<Vec<Argument>>,
    pub subsequent: Vec<Segment>,
}

#[derive(Debug)]
pub enum Argument {
    /// A string literal.
    StringLiteral(String),

    /// A context value expressed as a vec of segments, a path. Example: context.sender_id
    ContextValue(Vec<String>),

    /// Unsupported value with its string representation.
    Unsupported(String),
}

#[derive(Error, Debug)]
pub enum PathCompilationError {
    #[error("Unsupported selection kind: {0}")]
    UnsupportedSelectionKind(String),
}

/// Compiles all the possible resource access paths of the given operation.
pub fn compile_operation(
    operation: OperationDefinition,
) -> Result<Vec<Segment>, PathCompilationError> {
    let selection_set = operation.selection_set.into_inner();
    selection_set
        .items
        .into_iter()
        .map(|i| i.into_inner())
        .map(compile_path)
        .collect()
}

fn compile_path(selection: Selection) -> Result<Segment, PathCompilationError> {
    match selection {
        Selection::Field(positioned_field) => {
            let field = positioned_field.into_inner();
            Ok(Segment {
                name: field.name.into_inner().as_str().to_owned(),
                args: None,
                subsequent: field
                    .selection_set
                    .into_inner()
                    .items
                    .into_iter()
                    .map(|s| compile_path(s.into_inner()))
                    .collect::<Result<Vec<Segment>, PathCompilationError>>()?,
            })
        }
        Selection::FragmentSpread(_) => Err(PathCompilationError::unsupported_selection_kind(
            "FragmentSpread",
        )),
        Selection::InlineFragment(_) => Err(PathCompilationError::unsupported_selection_kind(
            "InlineFragment",
        )),
    }
}

impl PathCompilationError {
    pub fn unsupported_selection_kind(kind: impl Into<String>) -> Self {
        PathCompilationError::UnsupportedSelectionKind(kind.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_case() {
        use async_graphql_parser::parse_query;
        use async_graphql_parser::types::DocumentOperations;

        let document = parse_query("{ foo { bar baz } apiVersion }").expect("parse failed");
        if let DocumentOperations::Single(operation) = document.operations {
            let operation = operation.into_inner();

            let paths = compile_operation(operation);
            println!("{:#?}", paths);

            assert!(paths.is_ok());
        }
    }
}
