
use thiserror::Error;

tonic::include_proto!("identity_service");

#[derive(Debug, Error)]
pub enum AccessRequestParseError {
    #[error("Path {0} is invalid.")]
    CompileError(usize),

    #[error("Path {0} has multiple roots.")]
    MultiRootPath(usize),
}

impl TryFrom<AccessRequest> for service_core::resource_access::AccessRequest {
    type Error = AccessRequestParseError;

    fn try_from(model: AccessRequest) -> Result<Self, Self::Error> {
        use policy_statement::AccessKind as AccessKindModel;
        use service_core::resource_access::string_interop::compiler::from_string;
        use service_core::resource_access::AccessKind;

        let mut paths = Vec::with_capacity(model.paths.len());
        for (idx, path) in model.paths.into_iter().enumerate() {
            let path_set = from_string(path.as_ref()).map_err(|_| AccessRequestParseError::CompileError(idx))?;
            let mut path_nodes = path_set.into_paths();
            if path_nodes.len() != 1 {
                return Err(AccessRequestParseError::MultiRootPath(idx));
            }

            paths.push(path_nodes.pop().unwrap())
        }

        Ok(service_core::resource_access::AccessRequest {
            paths,
            kind: if model.access_kind == AccessKindModel::Mutation as i32 {
                AccessKind::Mutation
            } else {
                AccessKind::Query
            },
        })
    }
}
