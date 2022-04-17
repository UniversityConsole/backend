use super::types::{AccessKind, AccessRequest, PathSet, Segment};
use async_graphql_parser::types::{
    DocumentOperations, ExecutableDocument, OperationDefinition, OperationType, Selection,
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CompileError {
    #[error("Unsupported selection kind: {0}")]
    UnsupportedSelectionKind(String),

    #[error("Document with multiple operations not supported.")]
    MultiOperationsNotSupported,

    #[error("Operation {0} not supported.")]
    UnsupportedOperation(OperationType),
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
                path_set: from_operation(operation)?,
            }])
        }
        DocumentOperations::Multiple(_) => Err(CompileError::MultiOperationsNotSupported),
    }
}

fn from_operation(operation: &OperationDefinition) -> Result<PathSet, CompileError> {
    let mut path_set = PathSet::default();
    operation
        .selection_set
        .node
        .items
        .iter()
        .map(|i| &i.node)
        .try_for_each(|node| append_path(&mut path_set, vec![], node))?;

    Ok(path_set)
}

fn append_path(
    tree: &mut PathSet,
    mut stack: Vec<Segment>,
    selection: &Selection,
) -> Result<(), CompileError> {
    match selection {
        Selection::Field(field) => {
            let field = &field.node;
            let segment = Segment::no_args(field.name.clone().into_inner().as_str());
            stack.push(segment);

            let sub_fields = &field.selection_set.node.items;
            if sub_fields.is_empty() {
                tree.extend(stack.clone());
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

        let expected_path_set = "::{apiVersion, foo::{bar, baz}}";
        assert_eq!(request.path_set.to_string(), expected_path_set.to_string());
    }
}
