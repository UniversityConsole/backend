use super::types::{AccessKind, AccessRequest, ResourceAccessPath};
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
                paths: from_operation(operation)?,
            }])
        }
        DocumentOperations::Multiple(_) => Err(CompileError::MultiOperationsNotSupported),
    }
}

fn from_operation(
    operation: &OperationDefinition,
) -> Result<Vec<ResourceAccessPath>, CompileError> {
    operation
        .selection_set
        .node
        .items
        .iter()
        .map(|i| &i.node)
        .map(compile_path)
        .collect()
}

fn compile_path(selection: &Selection) -> Result<ResourceAccessPath, CompileError> {
    match selection {
        Selection::Field(positioned_field) => {
            let field = &positioned_field.node;
            Ok(ResourceAccessPath::new(
                field.name.node.as_str(),
                None,
                field
                    .selection_set
                    .node
                    .items
                    .iter()
                    .map(|s| compile_path(&s.node))
                    .collect::<Result<Vec<ResourceAccessPath>, CompileError>>()?,
            ))
        }
        Selection::FragmentSpread(_) => {
            Err(CompileError::unsupported_selection_kind("FragmentSpread"))
        }
        Selection::InlineFragment(_) => {
            Err(CompileError::unsupported_selection_kind("InlineFragment"))
        }
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
        let mut actual_paths = request.paths.clone();
        actual_paths.sort_by_key(|p| p.to_string());

        assert_eq!(actual_paths.len(), expected_paths.len());
        assert_eq!(actual_paths[0].to_string(), expected_paths[0].to_string());
        assert_eq!(actual_paths[1].to_string(), expected_paths[1].to_string());
    }
}
