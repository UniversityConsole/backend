pub mod graphql_interop;
pub mod serde;
pub mod string_interop;
pub mod types;

pub use graphql_interop::extension::{Authorizer, AuthorizerExtension};
pub use types::{AccessKind, AccessRequest, PolicyStatement};

/// Parse resource path sets from strings.
impl TryFrom<&str> for types::PathSet {
    type Error = string_interop::compiler::CompileError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        string_interop::compiler::from_string(s)
    }
}
