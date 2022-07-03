use async_graphql_parser::types::{
    DocumentOperations, ExecutableDocument, OperationDefinition, OperationType, Selection,
};
use async_graphql_value::{ConstValue, Value, Variables};
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

    #[error("Variable {0} referenced by argument {1} is unknown.")]
    UnknownVariable(String, String),

    #[error("Cannot select subfields of a match-any.")]
    CannotAppendToAny,
}

pub fn from_document(document: &ExecutableDocument, variables: &Variables) -> Result<Vec<AccessRequest>, CompileError> {
    match &document.operations {
        DocumentOperations::Single(operation) => {
            let operation = &operation.node;
            let access_kind: AccessKind = operation
                .ty
                .try_into()
                .map_err(|_| CompileError::UnsupportedOperation(operation.ty))?;

            Ok(vec![AccessRequest {
                kind: access_kind,
                paths: from_operation(operation, &variables)?,
            }])
        }
        DocumentOperations::Multiple(_) => Err(CompileError::MultiOperationsNotSupported),
    }
}

fn from_operation(operation: &OperationDefinition, variables: &Variables) -> Result<Vec<PathNode>, CompileError> {
    let mut path_set = PathSet::default();
    operation
        .selection_set
        .node
        .items
        .iter()
        .map(|i| &i.node)
        .try_for_each(|node| append_path(&mut path_set, vec![], node, &variables))?;
    Ok(path_set.into_paths())
}

fn append_path(
    tree: &mut PathSet,
    mut stack: Vec<Segment>,
    selection: &Selection,
    variables: &Variables,
) -> Result<(), CompileError> {
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
                        let value = parse_argument_value(&name, &val.node, variables)?;

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
                append_path(tree, stack.clone(), &sub_field.node, &variables)?;
            }

            Ok(())
        }
        _ => Err(CompileError::unsupported_selection_kind(".")),
    }
}

/// Parses argument values from GraphQL Value to ArgumentValue type.
///
/// # Params
/// * `name` - the name of the argument to be parsed
/// * `value` - the value of the argument as received from the GraphQL parser
/// * `variables` - the variables known for this GraphQL request
///
/// # Returns
/// Returns a parsed argument value, or a compilation error.
///
/// # Limitations
/// * This parser only supports `i64` for numeric literals. It does not support `u64` or `f64`.
/// * Only string, number (with the above limitation) and boolean types are supported for arguments.
/// * If the argument references a variable, the parser will try to replace that. If the variable is
///   of an unsupported type, the resulting argument value will be a wildcard.
fn parse_argument_value(name: &str, value: &Value, variables: &Variables) -> Result<ArgumentValue, CompileError> {
    match value {
        Value::String(s) => Ok(ArgumentValue::StringLiteral(s.clone())),
        Value::Number(n) => Ok(ArgumentValue::IntegerLiteral(
            n.as_i64().ok_or(CompileError::UnsupportedNumericLiteral)?,
        )),
        Value::Boolean(b) => Ok(ArgumentValue::BoolLiteral(*b)),
        Value::Enum(name) => Ok(ArgumentValue::Enum(name.to_string())),
        Value::Variable(var_name) => {
            if let Some(var_value) = variables.get(var_name) {
                match var_value {
                    ConstValue::Number(_) | ConstValue::String(_) | ConstValue::Boolean(_) | ConstValue::Enum(_) => {
                        parse_argument_value(name, &var_value.clone().into_value(), variables)
                    }
                    _ => {
                        tracing::debug!(
                            arg_name = name,
                            "Converted argument value to a wildcard, since its type is not supported."
                        );
                        Ok(ArgumentValue::Wildcard)
                    }
                }
            } else {
                Err(CompileError::UnknownVariable(var_name.to_string(), name.to_string()))
            }
        }
        _ => Err(CompileError::UnsupportedArgument(name.to_string(), value.to_string())),
    }
}

impl CompileError {
    pub fn unsupported_selection_kind(kind: impl Into<String>) -> Self {
        CompileError::UnsupportedSelectionKind(kind.into())
    }
}

#[cfg(test)]
mod tests {
    use async_graphql_value::Name;
    use serde_json::json;

    use super::*;

    #[test]
    fn single_query() {
        use async_graphql_parser::parse_query;

        let document = parse_query("{ foo { bar baz } apiVersion }").expect("parse failed");
        let access_requests =
            from_document(&document, &Variables::default()).expect("failed compiling access requests");
        assert_eq!(access_requests.len(), 1);

        let request = access_requests.first().unwrap();
        assert_eq!(request.kind, AccessKind::Query);

        let expected_paths = ["apiVersion", "foo::{bar, baz}"];
        for (path, expected_path) in std::iter::zip(&request.paths, expected_paths) {
            assert_eq!(path.to_string(), expected_path.to_string());
        }
    }

    #[test]
    fn query_with_variables() {
        use async_graphql_parser::parse_query;

        let document = parse_query("{ account(id: $id) }").expect("parse failed");
        let variables = {
            let mut v = Variables::default();
            v.insert(Name::new("id"), ConstValue::String("foo".to_string()));
            v
        };
        let access_requests = from_document(&document, &variables).expect("failed compiling access requests");
        assert_eq!(access_requests.len(), 1);

        let request = access_requests.first().unwrap();
        assert_eq!(request.kind, AccessKind::Query);

        let expected_paths = ["account(id: \"foo\")"];
        for (path, expected_path) in std::iter::zip(&request.paths, expected_paths) {
            assert_eq!(path.to_string(), expected_path.to_string());
        }
    }

    #[test]
    fn query_with_input_object() {
        use async_graphql_parser::parse_query;
        use serde_json::json;

        let document = parse_query("{ createAccount(params: $params) { id } }").expect("parse failed");
        let variables = Variables::from_json(json!({
            "params": {
                "foo": "bar",
            }
        }));
        let access_requests = from_document(&document, &variables).expect("failed compiling access requests");
        assert_eq!(access_requests.len(), 1);

        let request = access_requests.first().unwrap();
        assert_eq!(request.kind, AccessKind::Query);

        let expected_paths = ["createAccount(params: *)::id"];
        for (path, expected_path) in std::iter::zip(&request.paths, expected_paths) {
            assert_eq!(path.to_string(), expected_path.to_string());
        }
    }

    #[test]
    fn query_with_enum_arg() {
        use async_graphql_parser::parse_query;

        let document = parse_query("{ foo(bar: BAZ) { doo } }").expect("parse failed");
        let access_requests =
            from_document(&document, &Variables::default()).expect("failed compiling access requests");
        assert_eq!(access_requests.len(), 1);

        let request = access_requests.first().unwrap();
        assert_eq!(request.kind, AccessKind::Query);

        let expected_paths = ["foo(bar: BAZ)::doo"];
        for (path, expected_path) in std::iter::zip(&request.paths, expected_paths) {
            assert_eq!(path.to_string(), expected_path.to_string());
        }
    }

    #[test]
    fn query_with_enum_arg_and_var() {
        use async_graphql_parser::parse_query;

        let document = parse_query("mutation($b: String!) { foo(b: $b, bar: BAZ) }").expect("parse failed");
        let variables = Variables::from_json(json!({
            "b": "var"
        }));
        let access_requests = from_document(&document, &variables).expect("failed compiling access requests");
        assert_eq!(access_requests.len(), 1);

        let request = access_requests.first().unwrap();
        assert_eq!(request.kind, AccessKind::Mutation);

        let expected_paths = ["foo(b: \"var\", bar: BAZ)"];
        for (path, expected_path) in std::iter::zip(&request.paths, expected_paths) {
            assert_eq!(path.to_string(), expected_path.to_string());
        }
    }
}
