use service_core::resource_access::AccessKind as AccessKindModel;
use thiserror::Error;

use super::identity_service::access_request::AccessKind as AccessKindPb;
use super::identity_service::AccessRequest;


#[derive(Debug, Error)]
pub enum AccessRequestParseError {
    #[error("Path {0} is invalid: {1}.")]
    CompileError(usize, String),

    #[error("Path {0} has multiple roots: {1}.")]
    MultiRootPath(usize, String),
}

impl TryFrom<AccessRequest> for service_core::resource_access::AccessRequest {
    type Error = AccessRequestParseError;

    fn try_from(model: AccessRequest) -> Result<Self, Self::Error> {
        use service_core::resource_access::string_interop::compiler::from_string;
        use service_core::resource_access::AccessKind;

        let mut paths = Vec::with_capacity(model.paths.len());
        for (idx, path) in model.paths.into_iter().enumerate() {
            let path_set = from_string(path.as_ref()).map_err(|e| {
                log::error!("Failed to parse path: {}. Error: {:?}", &path, e);
                AccessRequestParseError::CompileError(idx, path.clone())
            })?;
            let mut path_nodes = path_set.into_paths();
            if path_nodes.len() != 1 {
                return Err(AccessRequestParseError::MultiRootPath(idx, path));
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

impl From<service_core::resource_access::AccessRequest> for AccessRequest {
    fn from(val: service_core::resource_access::AccessRequest) -> Self {
        AccessRequest {
            access_kind: match val.kind {
                AccessKindModel::Query => AccessKindPb::Query,
                AccessKindModel::Mutation => AccessKindPb::Mutation,
            } as i32,
            paths: val.paths.into_iter().map(|v| v.to_string()).collect(),
        }
    }
}
