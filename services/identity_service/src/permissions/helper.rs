use service_core::resource_access::string_interop::compiler::from_string;
use service_core::resource_access::{AccessKind, PolicyStatement};

pub fn compose_statement<const N: usize>(access_kind: AccessKind, paths: [&str; N]) -> PolicyStatement {
    PolicyStatement {
        kind: access_kind,
        paths: paths
            .iter()
            .map(|s| from_string(s).unwrap().into_paths().pop().unwrap())
            .collect(),
    }
}
