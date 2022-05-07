use async_graphql_parser::types::{
    DocumentOperations, ExecutableDocument, OperationDefinition, OperationType, Selection,
};
use async_graphql_value::Value;
use thiserror::Error;

use crate::resource_access::types::{
    AccessKind, AccessRequest, AppendNodeError, Argument, ArgumentValue, PathNode, PathSet, Segment,
};

#[derive(Error, Debug)]
pub enum CompileError {
    #[error("Unsupported selection kind: {0}")]
    UnsupportedSelectionKind(String),

    #[error("Unsupported numeric literal. Only i64 is supported.")]
    UnsupportedNumericLiteral,

    #[error("Document with multiple operations not supported.")]
    MultiOperationsNotSupported,

    #[error("Operation {0} not supported.")]
    UnsupportedOperation(OperationType),

    #[error("Argument {0} is not supported: {1}.")]
    UnsupportedArgument(String, String),

    #[error("Cannot select subfields of a match-any.")]
    CannotAppendToAny,
}

pub fn from_document(document: &ExecutableDocument) -> Result<Vec<AccessRequest>, CompileError> {
    match &document.operations {
        DocumentOperations::Single(operation) => {
            let operation = &operation.node;
            let access_kind: AccessKind = operation
                .ty
                .try_into()
                .map_err(|_| CompileError::UnsupportedOperation(operation.ty))?;

            Ok(vec![AccessRequest {
                kind: access_kind,
                paths: from_operation(operation)?,
            }])
        }
        DocumentOperations::Multiple(_) => Err(CompileError::MultiOperationsNotSupported),
    }
}

fn from_operation(operation: &OperationDefinition) -> Result<Vec<PathNode>, CompileError> {
    let mut path_set = PathSet::default();
    operation
        .selection_set
        .node
        .items
        .iter()
        .map(|i| &i.node)
        .try_for_each(|node| append_path(&mut path_set, vec![], node))?;
    Ok(path_set.into_paths())
}

fn append_path(tree: &mut PathSet, mut stack: Vec<Segment>, selection: &Selection) -> Result<(), CompileError> {
    match selection {
        Selection::Field(field) => {
            let field = &field.node;
            let segment = Segment::with_args(
                field.name.clone().into_inner().as_str(),
                field
                    .arguments
                    .iter()
                    .map(|(name, val)| {
                        let name = name.node.to_string();
                        let value = match &val.node {
                            Value::String(s) => ArgumentValue::StringLiteral(s.clone()),
                            Value::Number(n) => ArgumentValue::IntegerLiteral(
                                n.as_i64().ok_or(CompileError::UnsupportedNumericLiteral)?,
                            ),
                            Value::Boolean(b) => ArgumentValue::BoolLiteral(*b),
                            _ => return Err(CompileError::UnsupportedArgument(name, val.to_string())),
                        };

                        Ok(Argument { name, value })
                    })
                    .collect::<Result<Vec<_>, CompileError>>()?,
            );
            stack.push(segment);

            let sub_fields = &field.selection_set.node.items;
            if sub_fields.is_empty() {
                tree.extend(stack.clone()).map_err(|e| match e {
                    AppendNodeError::CannotAppendToAny => CompileError::CannotAppendToAny,
                })?;
            }

            for sub_field in sub_fields {
                append_path(tree, stack.clone(), &sub_field.node)?;
            }

            Ok(())
        }
        _ => Err(CompileError::unsupported_selection_kind(".")),
    }
}

impl CompileError {
    pub fn unsupported_selection_kind(kind: impl Into<String>) -> Self {
        CompileError::UnsupportedSelectionKind(kind.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_query() {
        use async_graphql_parser::parse_query;

        let document = parse_query("{ foo { bar baz } apiVersion }").expect("parse failed");
        let access_requests = from_document(&document).expect("failed compiling access requests");
        assert_eq!(access_requests.len(), 1);

        let request = access_requests.first().unwrap();
        assert_eq!(request.kind, AccessKind::Query);

        let expected_paths = ["apiVersion", "foo::{bar, baz}"];
        for (path, expected_path) in std::iter::zip(&request.paths, expected_paths) {
            assert_eq!(path.to_string(), expected_path.to_string());
        }
    }
}
